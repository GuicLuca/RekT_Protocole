use crate::clients::client::{ConnectionId, Packet};
use crate::prelude::ClientSenderMap;
use crate::{CLIENT_MAP, TOPICS};
use dashmap::DashMap;
use quinn::SendStream;
use rand::Rng;
use rekt_lib::datagrams::topic_request::{DtgTopicRequest, DtgTopicRequestAck, DtgTopicRequestNack};
use rekt_lib::enums::topic_action::TopicAction;
use rekt_lib::enums::topic_response::TopicResponse;
use rekt_lib::libs::types::TopicId;
use std::cmp::PartialEq;
use std::sync::Arc;
use rekt_lib::datagrams::data_request::DtgData;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct Topic {
    pub id: TopicId,
    pub saved_data: RwLock<Vec<u8>>,
    pub subscribers: ClientSenderMap,
}

impl Topic {
    pub fn new(id: TopicId) -> Topic {
        Topic {
            id,
            saved_data: RwLock::new(Vec::new()),
            subscribers: ClientSenderMap::new(DashMap::new()),
        }
    }

    /**
     * This function convert a temporary topic id to a definitive one.
     * TopicId are u64 but the MSB (little-endian) must be 0 for a user generated topic
     * and one for a definitive server topic.
     */
    pub fn enforce_id(in_id: TopicId) -> crate::Result<TopicId> {
        // Ensure the MSB is not 0
        if (in_id & (1u64 << 63)) != 0 {
            return Err(crate::Error::InvalidTopicId(
                in_id,
                "Topic id must be a user generated topic".to_string(),
            ));
        }

        // set the MSB to 1
        let mut out_id = in_id | (1u64 << 63);

        let mut max_generations = 200;

        // Check if this is id is not already used
        while TOPICS.contains_key(&out_id) {
            // ID exists, generate a new one with a random value, and set the MSB to 1
            let mut new_id: TopicId = 0;
            let mut rng = rand::rng().random::<u64>();
            // put the new number in the first 63 bits and force the MSB to 1
            out_id = rng | (1u64 << 63);
            max_generations -= 1;
            if max_generations <= 0 {
                return Err(crate::Error::InvalidTopicId(
                    in_id,
                    "Too many generations attempts".to_string(),
                ));
            }
        }

        Ok(out_id)
    }
    
    /**
     * This function returns the topic id as a server topic id
     */
    pub fn get_topic_id_as_server(in_id: TopicId) -> TopicId {
        // Get the topic id as a server topic
        in_id | (1u64 << 63)
    }

    /**
     * Handle a topic request from a client.
     * This function must be called only by the job-system!
     */
    pub async fn handle_topic_request(packet: Packet) {
        let datagram = match DtgTopicRequest::try_from(packet.datagram.as_slice()) {
            Ok(dtg) => dtg,
            Err(e) => {
                error!("Error while parsing topic request: {}", e);
                // In this case we return but a good implementation would answer with a NACK message with the error.
                // + a security check to ensure the client has not tried to attack the server.
                return;
            }
        };

        // Get the client sender (we can use unwrap here because we know the client exists, checking already passed in the job-system)
        let client = CLIENT_MAP.get(&packet.source).unwrap();

        // Get the client corresponding sender BUT do not lock it yet
        let sender = match &client.read().await.sender {
            Some(sender) => sender.clone(),
            None => {
                error!(
                "Client {} has no sender stream, so handle_topic_request can't respond to the client.",
                packet.source
            );
                return;
            }
        };

        // check the request action
        match datagram.flag {
            TopicAction::Subscribe => {
                // Check if the topic exists
                let topic = TOPICS.get_mut(&Self::get_topic_id_as_server(datagram.topic_id));
                if topic.is_none() {
                    // Topic does not exist, create it
                    match Topic::enforce_id(datagram.topic_id) {
                        Ok(topic_id) => {
                            let mut topic = Topic::new(topic_id);
                            // Add the client to the topic and register the topic
                            topic.add_subscriber(&packet.source, sender.clone());
                            TOPICS.insert(topic_id, topic);

                            // Send a response to the client
                            let dtg = DtgTopicRequestAck::new(topic_id, TopicResponse::SubSuccess);

                            // Send the datagram to the client
                            {
                                sender.write().await.write_all(&dtg.as_bytes()).await;
                            }
                            info!(
                                "Client {} subscribed to topic {}.",
                                packet.source, topic_id
                            );
                        }
                        Err(e) => {
                            
                            // Send a NACK message to the client
                            let dtg = DtgTopicRequestNack::new(
                                TopicResponse::SubFailure,
                                e.to_string().as_str()
                            );

                            // Send the datagram to the client
                            {
                                sender.write().await.write_all(&dtg.as_bytes()).await;
                            }
                            
                            error!(
                                "Client {} tried to subscribe to topic {} but it does not exist and server failed to create it: {}.",
                                packet.source, datagram.topic_id, e
                            );
                        }
                    }
                } else {
                    // Add the client to the topic
                    let mut topic = topic.unwrap();
                    let topic = topic.value_mut();
                    topic.add_subscriber(&packet.source, sender.clone());
                    
                    // Send a response to the client
                    let dtg = DtgTopicRequestAck::new(topic.id, TopicResponse::SubSuccess);

                    // Send the datagram to the client
                    {
                        sender.write().await.write_all(&dtg.as_bytes()).await;
                    }
                    
                    // Send the saved data to the client
                    topic.send_saved_data(packet.source).await;
                    
                    info!(
                        "Client {} subscribed to topic {}. Topic was preexisting, so saved data were sent.",
                        packet.source, topic.id);
                }
            }
            TopicAction::Unsubscribe => {
                // Check if the topic exists
                let topic = TOPICS.get_mut(&datagram.topic_id);
                if topic.is_none() {
                    // nothing to do, the client is already unsubscribed
                    // Send a response to the client
                    let dtg = DtgTopicRequestNack::new(
                        TopicResponse::UnsubFailure,
                        "Topic does not exist".to_string().as_str(),
                    );
                    
                    // Send the datagram to the client
                    {
                        sender.write().await.write_all(&dtg.as_bytes()).await;
                    }
                    
                    warn!(
                        "Client {} tried to unsubscribe from topic {} but it does not exist.",
                        packet.source, datagram.topic_id
                    );
                }
                else {
                    // Remove the client from the topic
                    let remaining_subscribers ={
                        let mut topic = topic.unwrap();
                        let topic = topic.value_mut();
                        topic.remove_subscriber(&packet.source);
                        topic.get_subscribers_count()
                    };
                    

                    // Send a response to the client
                    let dtg = DtgTopicRequestAck::new(datagram.topic_id, TopicResponse::UnsubSuccess);

                    // Send the datagram to the client
                    {
                        sender.write().await.write_all(&dtg.as_bytes()).await;
                    }
                    
                    // Check if the topic is empty
                    if remaining_subscribers == 0 {
                        // Remove the topic from the map
                        TOPICS.remove(&datagram.topic_id);
                        info!("Client {} unsubscribed from topic {}. Topic is now empty and removed from the server.", packet.source, datagram.topic_id);
                    } else {
                        info!(
                            "Client {} unsubscribed from topic {}.",
                            packet.source, datagram.topic_id
                        );
                    }
            }
                    
            }
            TopicAction::Unknown => {
                error!("Unknown topic action: {:?}", datagram.flag);
                // In this case we return but a good implementation would answer with a NACK message with the error.
                // + a security check to ensure the client has not tried to attack the server.
            }
        }
    }

    pub fn add_subscriber(&mut self, client_id: &ConnectionId, sender: Arc<RwLock<SendStream>>) {
        self.subscribers.insert(*client_id, sender);
    }

    pub fn remove_subscriber(&mut self, client_id: &ConnectionId) {
        self.subscribers.remove(client_id);
    }

    pub fn get_subscribers_count(&self) -> usize {
        self.subscribers.len()
    }

    /**
     * Publish data to all subscribers of the topic.
     * If a connection_id is provided, it will be excluded from the publishing process.
     * This is useful for sending data to all clients except the one that sent the data.
     */
    pub async fn publish(&mut self, data: &Vec<u8>, excluded_connection: Option<ConnectionId>) {
        {
            let mut saved_data = self.saved_data.write().await;
            *saved_data = data.to_owned();
        }
        
        // Filter out the excluded connection if provided
        let subscribers = {
            if let Some(id) = excluded_connection {
                self.subscribers
                    .iter()
                    .filter(|row| row.key() != &id)
                    .collect::<Vec<_>>()
            } else {
                self.subscribers.iter().collect::<Vec<_>>()
            }
        };
        
        
        for subscriber in subscribers {
            let sender = subscriber.value();
            
            let dtg = DtgData::new(0, self.id, data.to_vec());
            {
                sender.write().await.write_all(&dtg.as_bytes()).await;
            }
        }
    }

    /**
     * Send saved data to a specific client.
     * The client MUST be already subscribed to the topic.
     */
    pub async fn send_saved_data(&self, client_id: ConnectionId) {
        if self.saved_data.read().await.is_empty() {
            return;
        }
        
        if let Some(sender) = self.subscribers.get(&client_id) 
        {
            let data ={
                 self.saved_data.read().await.to_vec()
            };
            
            let dtg = DtgData::new(5, self.id, data);
            
            {
                let mut s =sender.write().await;
                s.write_all(&dtg.as_bytes()).await;
                s.flush();
            }
        } else {
            error!(
                "Client {} is not subscribed to topic {}",
                client_id, self.id
            );
        }
    }
}

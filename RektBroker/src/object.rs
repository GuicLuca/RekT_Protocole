use crate::clients::client::{ConnectionId, Packet};
use crate::prelude::ClientSenderMap;
use crate::topics::Topic;
use crate::{increase_profiling_data, CLIENT_MAP, OBJECTS, PROFILING_DATA, TOPICS};
use dashmap::mapref::one::RefMut;
use dashmap::DashSet;
use rand::Rng;
use rekt_lib::datagrams::object_requests::{
    DtgObjectRequest, DtgObjectRequestACK, DtgObjectRequestNACK,
};
use rekt_lib::datagrams::topic_request::{
    DtgTopicRequest, DtgTopicRequestAck, DtgTopicRequestNack,
};
use rekt_lib::enums::object_request_action::ObjectRequestAction;
use rekt_lib::enums::topic_response::TopicResponse;
use rekt_lib::libs::types::{ObjectId, TopicId};
use std::collections::HashSet;
use std::fmt::format;
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct Object {
    pub id: ObjectId,
    pub topic_list: RwLock<HashSet<TopicId>>,
    // No need to store the client sender as in topic because we only need
    // to save subscribers. Only topics care about the sending data.
    pub subscribers: DashSet<ConnectionId>,
}

impl Object {
    pub fn new(id: ObjectId, topic_list: HashSet<TopicId>) -> Self {
        let mut final_topic_list: HashSet<TopicId> = HashSet::new();

        // Register all topics in the object
        for topic_id in topic_list {
            let topic = TOPICS.get_mut(&Topic::get_topic_id_as_server(topic_id));

            if topic.is_none() {
                // Topic does not exist, create it
                match Topic::enforce_id(topic_id) {
                    Ok(enforced_id) => {
                        let mut topic = Topic::new(enforced_id);
                        TOPICS.insert(enforced_id, topic);

                        // Add the topic id to the object
                        final_topic_list.insert(enforced_id);
                    }
                    Err(e) => {
                        error!(
                                "Tried to create the topic {} from the object {} but server failed to create it: {}.",
                                topic_id, id, e
                            );
                    }
                }
            } else {
                // Topic already exists, add it to the object
                let t = topic.unwrap();
                final_topic_list.insert(t.id);
            }
        }

        Self {
            id,
            topic_list: RwLock::new(final_topic_list),
            subscribers: DashSet::new(),
        }
    }

    /**
     * This function convert a temporary object id to a definitive one.
     * ObjectId are u64 but the MSB (little-endian) must be 0 for a user generated object id
     * and 1 for a definitive server object.
     */
    pub fn enforce_id(in_id: ObjectId) -> crate::Result<ObjectId> {
        // Ensure the MSB is not 0
        if (in_id & (1u64 << 63)) != 0 {
            return Err(crate::Error::InvalidObjectId(
                in_id,
                "Object id must be a user generated object".to_string(),
            ));
        }

        // set the MSB to 1
        let mut out_id = in_id | (1u64 << 63);

        let mut max_generations = 200;

        // Check if this is id is not already used
        while OBJECTS.contains_key(&out_id) {
            // ID exists, generate a new one with a random value, and set the MSB to 1
            let mut new_id: ObjectId = 0;
            let mut rng = rand::rng().random::<u64>();
            // put the new number in the first 63 bits and force the MSB to 1
            out_id = rng | (1u64 << 63);
            max_generations -= 1;
            if max_generations <= 0 {
                return Err(crate::Error::InvalidObjectId(
                    in_id,
                    "Too many generations attempts".to_string(),
                ));
            }
        }

        Ok(out_id)
    }

    /**
     * This function returns the object id as a server object id
     */
    pub fn get_object_id_as_server(in_id: ObjectId) -> ObjectId {
        // Get the topic id as a server topic
        in_id | (1u64 << 63)
    }

    pub async fn add_subscriber(&self, client_id: ConnectionId) {
        self.subscribers.insert(client_id);

        // Fetch the sender of the client ID
        let client = match CLIENT_MAP.get(&client_id) {
            None => {
                error!(
                    "Can't handle datagram because client {} is not in the client map.",
                    client_id
                );
                return;
            }
            Some(entry) => entry.value().clone(),
        };

        // Get the client corresponding sender BUT do not lock it yet
        let sender = match &client.read().await.sender {
            Some(sender) => sender.clone(),
            None => {
                error!(
                "Client {} has no sender stream, so handle_datagram can't respond to the client.",
                client_id
            );
                return;
            }
        };

        // For each topic in the object, we add the client to the sender map
        let topic_list = self.topic_list.read().await;
        for topic_id in topic_list.iter() {
            match TOPICS.get_mut(topic_id) {
                None => {
                    error!(
                        "Topic {} not found in topic map. Object {} is not well registered.",
                        topic_id, self.id
                    );
                    return;
                }
                Some(mut t) => {
                    t.value_mut().add_subscriber(&client_id, sender.clone());
                    // send the saved data to the client ( if any )
                    t.send_saved_data(client_id).await;
                }
            }
        }
    }

    pub async fn remove_subscriber(&self, client_id: ConnectionId) {
        self.subscribers.remove(&client_id);
        // For each topic in the object, we remove the client from the sender map
        let topic_list = self.topic_list.read().await;
        for topic_id in topic_list.iter() {
            match TOPICS.get_mut(topic_id) {
                None => {
                    error!(
                        "Topic {} not found in topic map. Object {} is not well registered.",
                        topic_id, self.id
                    );
                    return;
                }
                Some(mut t) => {
                    t.value_mut().remove_subscriber(&client_id);
                }
            }
        }
    }

    pub fn is_subscribed(&self, client_id: ConnectionId) -> bool {
        self.subscribers.contains(&client_id)
    }

    pub fn get_subscriber_amount(&self) -> usize {
        self.subscribers.len()
    }

    pub async fn handle_object_request(packet: Packet) {
        let datagram = match DtgObjectRequest::try_from(packet.datagram.as_slice()) {
            Ok(dtg) => dtg,
            Err(e) => {
                error!("Error while parsing object request: {}", e);
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

        // Switch on the request action
        match datagram.flag {
            ObjectRequestAction::Create => {
                // Check if the object already exists
                let object = OBJECTS.get(&Object::get_object_id_as_server(datagram.object_id));
                if object.is_some() {
                    warn!(
                        "Client {} tried to create an object {} that already exists. He has been subscribed instead.",
                        packet.source,
                        Object::get_object_id_as_server(datagram.object_id)
                    );

                    // Can't create the object, but we can add the client to the subscribers list
                    let object = object.unwrap();
                    if !object.is_subscribed(packet.source) {
                        object.add_subscriber(packet.source).await;
                    }

                    // Send an ACK message to the client
                    let dtg = DtgObjectRequestACK::new(
                        u8::from(datagram.flag),
                        datagram.object_id,
                        object.id,
                    );
                    {
                        sender.write().await.write_all(&dtg.as_bytes()).await;
                    }

                    increase_profiling_data("Object::Create");

                    return;
                }

                // Create the object
                match Object::enforce_id(datagram.object_id) {
                    Ok(enforced_id) => {
                        // Create the object
                        let object = Object::new(enforced_id, datagram.payload);
                        // Subscribe the client to the object
                        object.add_subscriber(packet.source).await;
                        // Insert the object in the object map
                        OBJECTS.insert(enforced_id, object);

                        // Send an ACK message to the client
                        let dtg = DtgObjectRequestACK::new(
                            u8::from(datagram.flag),
                            datagram.object_id,
                            enforced_id,
                        );

                        {
                            sender.write().await.write_all(&dtg.as_bytes()).await;
                        }

                        increase_profiling_data("Object::Create");
                    }
                    Err(e) => {
                        error!("Error while creating object: {}", e);
                        // Send a NACK message to the client
                        let dtg = DtgObjectRequestNACK::new(
                            u8::from(datagram.flag),
                            datagram.object_id,
                            format!("Error while creating object: {}", e).as_str(),
                        );
                        sender.write().await.write_all(&dtg.as_bytes()).await;
                    }
                }
            }
            ObjectRequestAction::Update => {
                unimplemented!(
                    "Object update is not required for our experiment and so, not implemented."
                );
            }
            ObjectRequestAction::Delete => {
                // Check if the object exists
                let object = OBJECTS.get(&datagram.object_id);
                if object.is_none() {
                    error!(
                        "Client {} tried to delete an object {} that does not exist.",
                        packet.source, datagram.object_id
                    );

                    // Send a NACK message to the client
                    let dtg = DtgObjectRequestNACK::new(
                        u8::from(datagram.flag),
                        datagram.object_id,
                        "Object does not exist.",
                    );
                    {
                        sender.write().await.write_all(&dtg.as_bytes()).await;
                    }

                    increase_profiling_data("Object::Delete");
                    return;
                }

                // To delete the object:
                // 1 - Remove the peer and send a delete message to the client who sent the request
                // 2 - Remove all remaining subscribers from the object : send an unsubscribe message to each client but the one who sent the delete request
                // 3 - Remove the object from the object map

                let object = object.unwrap();
                let mut subscribers = object.subscribers.clone();
                subscribers.remove(&packet.source);
                // Remove the client who sent the request from the list
                object.remove_subscriber(packet.source).await;
                // Send a delete message to the client who sent the request
                let dtg = DtgObjectRequestACK::new(
                    u8::from(datagram.flag),
                    datagram.object_id,
                    datagram.object_id,
                );
                {
                    sender.write().await.write_all(&dtg.as_bytes()).await;
                }

                increase_profiling_data("Object::Delete");

                // Unsubscribe all other clients from the object
                for client_id in subscribers.iter() {
                    object.remove_subscriber(*client_id).await;
                    // Send an unsubscribe message to the client
                    let dtg = DtgObjectRequestACK::new(
                        u8::from(ObjectRequestAction::Unsubscribe),
                        datagram.object_id,
                        datagram.object_id,
                    );
                    {
                        let client = CLIENT_MAP.get(&client_id).unwrap();
                        let sender = match &client.read().await.sender {
                            Some(sender) => sender.clone(),
                            None => {
                                error!(
                                        "Client {} has no sender stream, so handle_object_request can't respond to the client (in Object deletion process propagation).",
                                        *client_id
                                    );
                                continue;
                            }
                        };
                        {
                            sender.write().await.write_all(&dtg.as_bytes()).await;
                        }

                        increase_profiling_data("Object::Delete");
                    }
                }

                // Remove the object from the object map
                OBJECTS.remove(&datagram.object_id);
            }
            ObjectRequestAction::Subscribe => {
                // Check if the object exists
                let object = OBJECTS.get(&datagram.object_id);
                if object.is_none() {
                    error!(
                        "Client {} tried to subscribe to an object {} that does not exist.",
                        packet.source, datagram.object_id
                    );

                    // Send a NACK message to the client
                    let dtg = DtgObjectRequestNACK::new(
                        u8::from(datagram.flag),
                        datagram.object_id,
                        "Object does not exist.",
                    );
                    {
                        sender.write().await.write_all(&dtg.as_bytes()).await;
                    }

                    increase_profiling_data("Object::Subscribe");
                    return;
                }

                // Subscribe the client to the object
                let object = object.unwrap();
                if !object.is_subscribed(packet.source) {
                    object.add_subscriber(packet.source).await;
                }

                // Send an ACK message to the client
                let dtg = DtgObjectRequestACK::new(
                    u8::from(datagram.flag),
                    datagram.object_id,
                    datagram.object_id,
                );
                {
                    sender.write().await.write_all(&dtg.as_bytes()).await;
                }

                increase_profiling_data("Object::Subscribe");
            }
            ObjectRequestAction::Unsubscribe => {
                // Check if the object exists
                let object = OBJECTS.get(&datagram.object_id);
                if object.is_none() {
                    error!(
                        "Client {} tried to unsubscribe from an object {} that does not exist.",
                        packet.source, datagram.object_id
                    );

                    // Send a NACK message to the client
                    let dtg = DtgObjectRequestNACK::new(
                        u8::from(datagram.flag),
                        datagram.object_id,
                        "Object does not exist.",
                    );
                    {
                        sender.write().await.write_all(&dtg.as_bytes()).await;
                    }

                    increase_profiling_data("Object::Unsubscribe");
                    return;
                }

                // Unsubscribe the client from the object
                let object = object.unwrap();
                if object.is_subscribed(packet.source) {
                    object.remove_subscriber(packet.source).await;
                }

                // Send an ACK message to the client
                let dtg = DtgObjectRequestACK::new(
                    u8::from(datagram.flag),
                    datagram.object_id,
                    datagram.object_id,
                );
                {
                    sender.write().await.write_all(&dtg.as_bytes()).await;
                }

                increase_profiling_data("Object::Unsubscribe");
            }
            ObjectRequestAction::Unknown => {
                warn!(
                    "Unknown object request action: {:?} for object request from {}",
                    datagram.flag, packet.source
                );
            }
        }
    }
}

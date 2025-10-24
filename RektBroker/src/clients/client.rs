use crate::errors::Error;
use crate::errors::Error::InvalidDatagramType;
use crate::prelude::Result;
use crate::streams::streams::{RBiStream, RUnreliableStream};
use crate::{increase_profiling_data, CLIENT_MAP, CONFIG, PACKET_BUFFER, PROFILING_DATA, WORKER_CONDVAR};
use quinn::{Connection, RecvStream, SendStream};
use rand::random;
use rekt_lib::datagrams::connect_requests::DtgConnectAck;
use rekt_lib::datagrams::heartbeat_requests::DtgHeartbeat;
use rekt_lib::datagrams::latency_requests::{DtgPing, DtgPong};
use rekt_lib::datagrams::shutdown_request::DtgShutdown;
use rekt_lib::enums::connection_status::ConnectionStatus;
use rekt_lib::enums::datagram_type::DatagramType::{
    Connect, Data, Heartbeat, HeartbeatRequest, ObjectRequest, OpenStream, Ping, Pong,
    ServerStatus, Shutdown, TopicRequest,
};
use rekt_lib::enums::datagram_type::{display_datagram_type, DatagramType};
use rekt_lib::enums::end_connection_reason::EndConnexionReason;
use rekt_lib::libs::types::ClientId;
use rekt_lib::libs::utils::get_u16_at_pos;
use std::fmt::{Display, Formatter};
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use tokio::time::Instant;

#[derive(Debug)]
pub struct Packet {
    pub source: ConnectionId,
    pub datagram: Vec<u8>,
}

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub struct ConnectionId {
    pub ip_src: IpAddr,
    pub port_src: u16,
}

impl ConnectionId {
    pub fn new(ip_src: IpAddr, port_src: u16) -> ConnectionId {
        ConnectionId { ip_src, port_src }
    }
    pub fn from_connection(connection: &Connection) -> ConnectionId {
        ConnectionId {
            ip_src: connection.remote_address().ip(),
            port_src: connection.remote_address().port(),
        }
    }
}

impl Display for ConnectionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.ip_src, self.port_src)
    }
}

#[derive(Debug)]
pub struct Client {
    pub id: ClientId,
    pub connection_id: ConnectionId,
    pub connection: Connection,
    pub receiver: Option<Arc<RwLock<RecvStream>>>,
    pub sender: Option<Arc<RwLock<SendStream>>>,
    pub status: RwLock<ConnectionStatus>,
    pub life_signe: Arc<RwLock<Instant>>,
}

impl Client {
    pub fn new(connection_id: ConnectionId, connection: Connection) -> Client {
        Client {
            id: Client::get_new_id(),
            connection_id,
            connection,
            status: RwLock::new(ConnectionStatus::Connecting), // The client is connecting until he sends a CONNECT request
            receiver: None,
            sender: None,
            life_signe: Arc::new(RwLock::new(Instant::now())),
        }
    }
    /**
     * This method return a unique id for a new client.
     *
     * @return ClientId
     */
    fn get_new_id() -> ClientId {
        // Return the XOR operation between the current time and a random ClientId(u64)
        (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Failed to calculate duration since UNIX_EPOCH")
            .as_nanos() as ClientId)
            ^ random::<ClientId>()
    }

    /**
     * This method disconnect a client from the broker.
     * The client is removed from the global client map and his streams are closed.
     *
     * @return Result<()>
     */
    pub async fn disconnect(id: &ConnectionId, reason: EndConnexionReason) -> Result<()> {
        // 2 - Remove the client from the global client map to prevent further access
        if let Some((_, client)) = CLIENT_MAP.remove(id) {
            let read_client = client.read().await;
            // Check the connection status of the client to prevent multiple disconnections
            {
                if *read_client.status.read().await == ConnectionStatus::Disconnecting {
                    // The client is already disconnecting
                    return Ok(());
                }
            }
            // 1 - set the connection status to disconnected
            *read_client.status.write().await = ConnectionStatus::Disconnecting;

            // 3 - Send a shutdown datagram to the client
            if read_client.sender.is_none() {
                error!("Client {} has no sender stream in disconnect!", id);
            } else {
                let dtg = DtgShutdown::new(reason);
                {
                    let mut sender = read_client.sender.as_ref().unwrap().write().await;
                    sender.write_all(&dtg.as_bytes()).await?;
                    // Indicate that we will not send more data
                    sender.flush().await?;

                    increase_profiling_data("Client::Disconnect");

                    // Give a fair amount of time to the client to read the datagram
                    tokio::time::sleep(Duration::from_millis(5)).await;
                    sender.finish();
                }
            }

            // 4 - Close the connection
            read_client.connection.close(0u32.into(), b"done");

            info!("Client {} has been disconnected.", id);
        }
        // else the client is not in client map so we suppose he is already disconnected
        Ok(())
    }

    /**
     * This method update the life signe of the client.
     * The life signe is the last time the client sent a datagram to the broker.
     */
    async fn update_life_signe(&self) {
        let mut life_signe = self.life_signe.write().await;
        *life_signe = Instant::now();
    }

    /**
     * This method return the allowed DatagramType for the client based on his status.
     *
     * @param connection_status: &ConnectionStatus, the status of the client.
     *
     * @return Vec<DatagramType>, the allowed datagram types for the client.
     */
    async fn get_allowed_datagrams(&self) -> Vec<DatagramType> {
        // Return the allowed actions for the client based on his status
        match *self.status.read().await {
            ConnectionStatus::Connecting => {
                vec![Connect, ServerStatus]
            }
            ConnectionStatus::Connected | ConnectionStatus::Spurious => {
                vec![
                    Connect,
                    Shutdown,
                    OpenStream,
                    ServerStatus,
                    Heartbeat,
                    HeartbeatRequest,
                    Ping,
                    Pong,
                    TopicRequest,
                    ObjectRequest,
                    Data,
                ]
            }
            _ => {
                error!(
                    "Client {} has an unknown status. Got \"{:?}\" as status.",
                    self.connection_id, self.status
                );
                vec![]
            }
        }
    }

    /**
     * This method handle the bidirectional stream of a client.
     *
     * @return Result<()>, an empty result if the method succeed.
     */
    pub async fn handle_bi_stream(&self) -> Result<()> {
        // Handle the bidirectional stream of a client
        let mut client_buf: Vec<u8> = Vec::with_capacity(10 * 1500); // 15Kb = 10 RekT datagrams maximum

        'handling: loop {
            let mut network_buf: [u8; 1524] = [0; 1524]; // 1500 bytes + 24 bytes for the QUIC header

            {
                // Scope to release the lock on the receiver stream
                let arc_receiver = match &self.receiver {
                    Some(receiver) => receiver.clone(),
                    None => {
                        error!("Client {} has no receiver stream!", self.connection_id);
                        return Err(Error::ClientErr(format!(
                            "Client {} has no receiver stream!",
                            self.connection_id
                        )));
                    }
                };

                let mut receiver = arc_receiver.write().await;

                match receiver.read(&mut network_buf).await? {
                    Some(received) => {
                        // Handle the received bytes
                        trace!(
                            "Received {} bytes from client {}",
                            received,
                            self.connection_id
                        );
                        client_buf.extend_from_slice(&network_buf[..received]);
                    }
                    None => {
                        // The stream has been closed
                        info!(
                            "Client {} closed the bidirectional stream",
                            self.connection_id
                        );
                        return Ok(());
                    }
                }
            } // End of the lock on the receiver stream

            let dtg_type = DatagramType::from(client_buf[0]);

            // Check if the datagram type is valid :
            if dtg_type == DatagramType::Unknown {
                error!(
                    "Client {} sent an unknown datagram type. Got \"{}\"",
                    self.connection_id, client_buf[0]
                );
                return Err(InvalidDatagramType(client_buf[0]));
            }

            // Drain the whole buffer :
            let bytes_to_drain: usize = {
                if dtg_type.is_sized_datagram() {
                    // the "?" will never throw an error here
                    dtg_type.get_default_byte_size()
                        + get_u16_at_pos(&client_buf, 1).or_else(|_| Ok::<u16, Error>(0))? as usize
                } else {
                    dtg_type.get_default_byte_size()
                }
            };

            // Ensure that the buffer is big enough to drain the bytes
            if client_buf.len() < bytes_to_drain {
                // Not enough bytes received yet, continue the loop to collect more bytes
                continue 'handling;
            }

            // drain "bytes_to_drain" bytes from the client buffer
            let datagram_bytes: Vec<u8> = client_buf.drain(..bytes_to_drain).collect();

            // Check if datagram type is forbidden for the client
            if !self.get_allowed_datagrams().await.contains(&dtg_type) {
                error!(
                    "Client {} sent an unauthorized datagram type. Got \"{}\"",
                    self.connection_id,
                    display_datagram_type(dtg_type)
                );
                // The data type is not allowed for the client so drop it...
                // For now, we continue the loop but a strong implementation MAY track unauthorized client actions for security reasons
                continue;
            }

            // Direct respond to pong and heartbeat requests
            if [Ping, Pong, HeartbeatRequest, Heartbeat, Connect].contains(&dtg_type) {
                let fut = Client::direct_respond(self.connection_id, datagram_bytes.clone());
                tokio::spawn(async move {
                    fut.await;
                });
                continue 'handling;
            }

            if dtg_type == Shutdown {
                // Disconnect the client
                return Ok(());
            }

            // Update the life signe of the client because he sent a datagram (whatever the type)

            self.update_life_signe().await;

            // TODO: remove this line when the client is fully implemented
            info!(
                "Client {} sent a datagram of type \"{}\"",
                self.connection_id,
                display_datagram_type(dtg_type)
            );

            // From here, every command received MUST be handled by the broker
            let mut packet_queuing_retry = 0;
            'packet_queuing: loop {
                match PACKET_BUFFER.push(Packet {
                    source: self.connection_id,
                    datagram: datagram_bytes.clone(),
                }) {
                    Ok(_) => {
                        trace!("New packet enqueued for client {}", self.connection_id);
                        // Fetch the worker condvar to wake up a worker if the buffer was empty
                        let (_, ref cvar) = *WORKER_CONDVAR.as_ref();
                        cvar.notify_one();
                        break 'packet_queuing;
                    }
                    Err(packet) => {
                        if packet_queuing_retry < 1 {
                            warn!("The broker is congested! Global packet buffer is full!");
                            // pause the thread for a short time before retrying
                            tokio::time::sleep(Duration::from_millis(5)).await;
                            packet_queuing_retry += 1;
                            continue 'packet_queuing;
                        }
                        // drop the packet
                        error!(
                            "Packet congestion is too long, dropping a packet for client {}",
                            self.connection_id
                        );
                        continue 'handling;
                    }
                };
            }
        }
    }

    pub async fn direct_respond(connection_id: ConnectionId, datagram: Vec<u8>) -> Result<()> {
        // Get the client from the global client map
        let client = match CLIENT_MAP.get(&connection_id) {
            None => {
                // The client has been removed from the hashmap.
                return Err(Error::MissingClient(connection_id));
            }
            Some(entry) => entry.clone(),
        };

        // Directly respond to a client
        let arc_sender = {
            // internal scope to release the lock on the client after the sender is cloned
            let client_locked = client.read().await;

            match &client_locked.sender {
                Some(sender) => sender.clone(),
                None => {
                    error!("Client {} has no sender stream!", connection_id);
                    return Err(Error::ClientErr(format!(
                        "Client {} has no sender stream!",
                        connection_id
                    )));
                }
            }
        };

        // check if the datagram type:
        let dtg_type = DatagramType::from(datagram[0]);
        match dtg_type {
            Heartbeat | HeartbeatRequest => {
                let client_locked = client.read().await;
                // update the life signe of the client whatever the datagram type
                client_locked.update_life_signe().await;

                if dtg_type == HeartbeatRequest {
                    // if the datagram type is a heartbeat request, respond with a heartbeat
                    let heartbeat_datagram = DtgHeartbeat::new();
                    let mut sender = arc_sender.write().await;
                    sender.write_all(&heartbeat_datagram.as_bytes()).await?;

                    increase_profiling_data("Client::Heartbeat");
                } else {
                    // nothing to do here, the client is still alive
                    trace!("Heartbeat datagram received from client {}", connection_id);
                }
            }
            Pong | Ping => {
                if dtg_type == Ping {
                    let ping_datagram = DtgPing::try_from(&datagram[..])?;
                    // if the datagram type is a ping, respond with a pong
                    let pong_datagram = DtgPong::new(ping_datagram.ping_id);
                    {
                        let mut sender = arc_sender.write().await;
                        sender.write_all(&pong_datagram.as_bytes()).await?;

                        increase_profiling_data("Client::PingPong");
                    }
                } else {
                    // handle pong datagram locally
                    info!("Pong datagram received from client {}", connection_id);
                    // TODO: implement pong handling here
                    // latency measurement is not necessary for the main experiment so it is not implemented
                }
            }
            Connect => {
                // if the datagram type is a connect, answer with the heartbeat period + update his status
                let connect_ack_dtg = {
                    let client_read = client.read().await;
                    *client_read.status.write().await = ConnectionStatus::Connected;
                    DtgConnectAck::new(client_read.id, CONFIG.heart_beat_period)
                };

                let mut sender = arc_sender.write().await;
                sender.write_all(&connect_ack_dtg.as_bytes()).await?;
                info!("Client {} has now the status \"Connected\".", connection_id);

                increase_profiling_data("Client::Connect");
            }
            _ => {
                // if this case is reached, the datagram type is not a direct response type
                // so there is no need to respond to the client
                warn!(
                    "Methode direct_respond called with a non direct response datagram type. Got \"{}\"",
                    display_datagram_type(dtg_type)
                );
                return Ok(());
            }
        };

        Ok(())
    }
}

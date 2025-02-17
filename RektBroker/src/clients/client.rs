use std::fmt::{Display, Formatter};
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::errors::Error;
use crate::errors::Error::InvalidDatagramType;
use crate::prelude::Result;
use crate::streams::streams::{RBiStream, RUnreliableStream};
use crate::PACKET_BUFFER;
use quinn::{Connection, RecvStream, SendStream};
use rand::random;
use rekt_lib::enums::connection_status::ConnectionStatus;
use rekt_lib::enums::datagram_type::DatagramType::{
    Connect, Data, Heartbeat, HeartbeatRequest, ObjectRequest, OpenStream, Ping, Pong,
    ServerStatus, Shutdown, TopicRequest,
};
use rekt_lib::enums::datagram_type::{display_datagram_type, DatagramType};
use rekt_lib::libs::types::ClientId;
use rekt_lib::libs::utils::get_u16_at_pos;
use tokio::sync::RwLock;

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
    pub status: ConnectionStatus,
}

impl Client {
    pub fn new(connection_id: ConnectionId, connection: Connection) -> Client {
        Client {
            id: Client::get_new_id(),
            connection_id,
            connection,
            status: ConnectionStatus::Connecting, // The client is connecting until he sends a CONNECT request
            receiver: None,
            sender: None,
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
     * This method return the allowed DatagramType for the client based on his status.
     *
     * @param connection_status: &ConnectionStatus, the status of the client.
     *
     * @return Vec<DatagramType>, the allowed datagram types for the client.
     */
    fn get_allowed_datagrams(&self) -> Vec<DatagramType> {
        // Return the allowed actions for the client based on his status
        match self.status {
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
                        return Err(Error::ClientError(format!(
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
            if !self.get_allowed_datagrams().contains(&dtg_type) {
                error!(
                    "Client {} sent an unauthorized datagram type. Got \"{}\"",
                    self.connection_id,
                    display_datagram_type(dtg_type)
                );
                // The data type is not allowed for the client so drop it...
                // For now, we continue the loop but a strong implementation MAY track unauthorized client actions for security reasons
                continue;
            }
            
            // TODO: direct respond to pong and heartbeat requests

            // From here, every command received MUST be handled by the broker
            let mut packet_queuing_retry = 0;
            'packet_queuing: loop {
                match PACKET_BUFFER.push(Packet {
                    source: self.connection_id,
                    datagram: datagram_bytes.clone(),
                }) {
                    Ok(_) => {
                        trace!("New packet enqueued for client {}", self.connection_id);
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
}

#[derive(Debug)]
pub struct Packet {
    pub source: ConnectionId,
    pub datagram: Vec<u8>
}

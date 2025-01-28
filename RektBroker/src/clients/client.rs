use std::fmt::{Display, Formatter};
use std::net::IpAddr;
use std::time::{SystemTime, UNIX_EPOCH};

use quinn::Connection;
use rand::random;

use rekt_lib::libs::types::ClientId;
use crate::streams::streams::{RBiStream, RUnreliableStream};

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
    pub unreliable_stream: RUnreliableStream,
    pub bidirectional_stream: RBiStream,
}

impl Client {
    pub fn new(
        connection_id: ConnectionId,
        connection: Connection,
        bi_stream: RBiStream,
    ) -> Client {
        Client {
            id: Client::get_new_id(),
            connection_id,
            unreliable_stream: RUnreliableStream::from_connection(connection),
            bidirectional_stream: bi_stream,
        }
    }
    /**
     * This method return a unique id for a new client.
     *
     * @return ClientId
     */
    fn get_new_id() -> ClientId {
        // Return the XOR operation between the current time and a random ClientId(u64)
        return (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Failed to calculate duration since UNIX_EPOCH")
            .as_nanos() as ClientId)
            ^ random::<ClientId>();
    }
}

#[derive(Debug)]
pub struct Packet {
    pub source: ConnectionId,
    pub datagram: [u8; 1024],
}

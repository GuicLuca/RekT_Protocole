use std::fmt::{Display, Formatter};
use std::net::IpAddr;
use std::time::{SystemTime, UNIX_EPOCH};

use quinn::Connection;
use rand::random;

use crate::streams::streams::{RBiStream, RUnreliableStream};
use rekt_lib::libs::types::ClientId;
use crate::prelude::Result;

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
}

impl Client {
    pub fn new(connection_id: ConnectionId, connection: Connection) -> Client {
        Client {
            id: Client::get_new_id(),
            connection_id,
            connection,
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
    
    pub async fn handle_bi_stream(&self, mut stream: RBiStream) -> Result<()>{
        // Handle the bidirectional stream of a client
        loop {
            let mut bytes : [u8; 1500] = [0; 1500];
            
            let received = stream.receiver.read(&mut bytes).await?;
            match received {
                Some(received) => {
                    // Handle the received bytes
                    info!("Received {} bytes from client {}", received, self.connection_id);
                    info!("Received message: {:?}", String::from_utf8(bytes[..received].to_vec())?);
                    
                    // Send a response to the client
                    stream.sender.write_all(b"response").await?;
                },
                None => {
                    // The stream has been closed
                    info!("Client {} closed the bidirectional stream", self.connection_id);
                    return Ok(());
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Packet {
    pub source: ConnectionId,
    pub datagram: [u8; 1024],
}

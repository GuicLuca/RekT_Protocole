use quinn::{Connection, RecvStream, SendStream};

#[derive(Debug)]
pub enum UnidirectionalStreamOwner {
    Local,
    Remote,
}

#[derive(Debug)]
pub struct RBiStream {
    pub sender: SendStream,
    pub receiver: RecvStream,
}

#[derive(Debug)]
pub struct RUniStream {
    pub owner: UnidirectionalStreamOwner,
    pub sender: SendStream,
    pub receiver: RecvStream,
}

#[derive(Debug)]
pub struct RUnreliableStream {
    pub stream: Connection,
}

impl RUnreliableStream {
    pub fn from_connection(connection: Connection) -> RUnreliableStream
    {
        RUnreliableStream {
            stream: connection
        }
    }
}
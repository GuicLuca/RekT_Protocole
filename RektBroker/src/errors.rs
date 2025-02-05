use std::net::AddrParseError;
use std::string::FromUtf8Error;
use crate::clients::client::ConnectionId;

#[allow(dead_code)] // remove this line in production
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Generic {0}")]
    Generic(String),

    #[error("[InitializationError] - {0}")]
    Initialization(String),

    #[error(transparent)]
    Certificate(#[from] rcgen::Error),

    #[error(transparent)]
    RusTLS(#[from] rustls::Error),
    
    #[error(transparent)]
    QuinnTlsPme(#[from] quinn::rustls::pki_types::pem::Error),
    
    #[error(transparent)]
    QuinnTls(#[from] quinn::rustls::Error),

    #[error(transparent)]
    FromUtf8(#[from] FromUtf8Error),

    #[error(transparent)]
    ReadToEnd(#[from] quinn::ReadToEndError),
    
    #[error(transparent)]
    QuinnRead(#[from] quinn::ReadError),

    #[error(transparent)]
    WriteOnStream(#[from] quinn::WriteError),

    #[error(transparent)]
    QuinnConnection(#[from] quinn::ConnectionError),

    #[error(transparent)]
    AddrParse(#[from] AddrParseError),

    #[error(transparent)]
    SendDatagram(#[from] quinn::SendDatagramError),

    #[error(transparent)]
    IO(#[from] std::io::Error),
    
    #[error("[Missing Client] - Client of connection {0} not found in global client map.")]
    MissingClient(ConnectionId),
}

// Custom conversion from &ConnectionError to ConnectionError
impl From<&quinn::ConnectionError> for Error {
    fn from(err: &quinn::ConnectionError) -> Error {
        Error::QuinnConnection(err.clone())
    }
}
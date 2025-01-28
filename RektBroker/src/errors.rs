use std::net::AddrParseError;
use std::string::FromUtf8Error;

use quinn::{ConnectionError, ReadToEndError, SendDatagramError, WriteError};

#[allow(dead_code)] // remove this line in production
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Generic {0}")]
    Generic(String),

    #[error("[InitializationError] - {0}")]
    InitializationError(String),

    #[error(transparent)]
    CertificateError(#[from] rcgen::Error),

    #[error(transparent)]
    RustlsError(#[from] rustls::Error),
    
    #[error(transparent)]
    QuinnTlsPmeError(#[from] quinn::rustls::pki_types::pem::Error),
    
    #[error(transparent)]
    QuinnTlsError(#[from] quinn::rustls::Error),

    #[error(transparent)]
    Utf8Error(#[from] FromUtf8Error),

    #[error(transparent)]
    ReadToEndError(#[from] ReadToEndError),

    #[error(transparent)]
    WriteError(#[from] WriteError),

    #[error(transparent)]
    ConnectionError(#[from] ConnectionError),

    #[error(transparent)]
    AddrParseError(#[from] AddrParseError),

    #[error(transparent)]
    SendDatagramError(#[from] SendDatagramError),

    #[error(transparent)]
    IO(#[from] std::io::Error),
}

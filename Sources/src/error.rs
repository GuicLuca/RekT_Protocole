//! Main crate Error
//!
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("[Generic error] -> {0}.")]
    Generic(String),
    #[error(transparent)]
    IO (#[from] std::io::Error),
}
use prost::{DecodeError, EncodeError};
use redis::RedisError;
use thiserror::Error;
use tokio_tungstenite::tungstenite::Error as TungsteniteError;

#[derive(Error, Debug)]
pub enum BaseError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("tungstenite error: {0}")]
    Tungstenite(#[from] TungsteniteError),
    #[error("prost encode error: {0}")]
    ProstEncode(#[from] EncodeError),
    #[error("prost decode error: {0}")]
    ProstDecode(#[from] DecodeError),
    #[error("redis error: {0}")]
    Redis(#[from] RedisError),
    #[error("lock error")]
    Lock,
    #[error("channel error")]
    Channel,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

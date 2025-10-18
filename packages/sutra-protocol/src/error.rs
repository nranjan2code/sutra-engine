//! Error types for Sutra protocol

use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Connection timeout")]
    Timeout,
    
    #[error("Connection closed")]
    ConnectionClosed,
    
    #[error("Message too large: {0} bytes (max {1})")]
    MessageTooLarge(usize, usize),
    
    #[error("Protocol version mismatch: got {0}, expected {1}")]
    VersionMismatch(u32, u32),
    
    #[error("Server error: {0}")]
    ServerError(String),
    
    #[error("Client error: {0}")]
    ClientError(String),
}

pub type Result<T> = std::result::Result<T, ProtocolError>;

impl From<bincode::Error> for ProtocolError {
    fn from(e: bincode::Error) -> Self {
        ProtocolError::Serialization(e.to_string())
    }
}

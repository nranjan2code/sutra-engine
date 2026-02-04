use thiserror::Error;

#[derive(Error, Debug)]
pub enum TokenizerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Token not found in vocabulary: {0}")]
    TokenNotFound(String),

    #[error("Invalid vocabulary file: {0}")]
    InvalidVocab(String),

    #[error("Encoding error: {0}")]
    EncodingError(String),

    #[error("Decoding error: {0}")]
    DecodingError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Unsupported tokenizer type: {0}")]
    UnsupportedType(String),
}

pub type Result<T> = std::result::Result<T, TokenizerError>;

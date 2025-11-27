use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoaderError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Safetensors error: {0}")]
    Safetensors(#[from] safetensors::SafeTensorError),

    #[error("Download error: {0}")]
    Download(#[from] reqwest::Error),

    #[error("Tensor not found: {0}")]
    TensorNotFound(String),

    #[error("Invalid tensor shape: expected {expected:?}, got {actual:?}")]
    InvalidShape {
        expected: Vec<usize>,
        actual: Vec<usize>,
    },

    #[error("Invalid data type: expected {expected}, got {actual}")]
    InvalidDType { expected: String, actual: String },

    #[error("Checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    #[error("Model not found in registry: {0}")]
    ModelNotFound(String),

    #[error("Invalid model configuration: {0}")]
    InvalidConfig(String),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("Conversion error: {0}")]
    Conversion(String),

    #[error("Core error: {0}")]
    Core(#[from] sutra_core::SutraError),
}

pub type Result<T> = std::result::Result<T, LoaderError>;

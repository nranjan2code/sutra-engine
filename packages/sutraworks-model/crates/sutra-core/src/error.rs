use thiserror::Error;

#[derive(Error, Debug)]
pub enum SutraError {
    #[error("Invalid tensor shape: {0}")]
    InvalidShape(String),

    #[error("Dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },

    #[error("Model loading error: {0}")]
    ModelLoadError(String),

    #[error("Quantization error: {0}")]
    QuantizationError(String),

    #[error("Unsupported data type: {0}")]
    UnsupportedDType(String),

    #[error("Compute error: {0}")]
    ComputeError(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, SutraError>;

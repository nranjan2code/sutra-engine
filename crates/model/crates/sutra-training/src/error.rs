use thiserror::Error;

#[derive(Error, Debug)]
pub enum TrainingError {
    #[error("Core error: {0}")]
    Core(#[from] sutra_core::SutraError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Optimization error: {0}")]
    Optimization(String),

    #[error("Gradient error: {0}")]
    Gradient(String),

    #[error("Checkpoint error: {0}")]
    Checkpoint(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, TrainingError>;

pub mod error;
pub mod model;
pub mod ops;
/// Core types and utilities shared across all SutraWorks crates
pub mod tensor;

pub use error::{Result, SutraError};
pub use model::{ModelConfig, ModelWeights};
pub use tensor::{DType, Tensor, TensorView};

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::ops;
    pub use crate::{DType, ModelConfig, ModelWeights, Tensor, TensorView};
    pub use crate::{Result, SutraError};
}

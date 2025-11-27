pub mod error;
pub mod gradient;
pub mod loss;
/// Training infrastructure for SutraWorks models
///
/// Features:
/// - Optimizers (Adam, SGD, AdamW)
/// - Learning rate schedulers
/// - Gradient computation and backpropagation
/// - Loss functions
/// - Training loops with checkpointing
/// - Mixed precision training support
pub mod optimizer;
pub mod scheduler;
pub mod trainer;

pub use error::{Result, TrainingError};
pub use gradient::GradientAccumulator;
pub use loss::{CrossEntropyLoss, Loss, MSELoss};
pub use optimizer::{Adam, AdamConfig, AdamW, Optimizer, Sgd, SgdConfig};
pub use scheduler::{CosineScheduler, LRScheduler, LinearScheduler};
pub use trainer::{Trainer, TrainerConfig, TrainingState};

pub mod prelude {
    pub use crate::{Adam, AdamConfig, Sgd, SgdConfig};
    pub use crate::{CosineScheduler, LRScheduler, LinearScheduler};
    pub use crate::{CrossEntropyLoss, Loss, MSELoss};
    pub use crate::{Result, TrainingError};
    pub use crate::{Trainer, TrainerConfig, TrainingState};
}

//! Parameter-Efficient Fine-Tuning (PEFT) with LoRA and QLoRA
//!
//! Enables fine-tuning large models with minimal memory by:
//! - Training only small adapter matrices (LoRA)
//! - Working with quantized base models (QLoRA)

pub mod adapter;
pub mod lora;
pub mod qlora;
pub mod trainer;

pub use adapter::{Adapter, AdapterManager};
pub use lora::{LoraConfig, LoraLayer};
pub use qlora::{QLoraConfig, QLoraLayer};
pub use trainer::{PeftTrainer, TrainingConfig};

pub mod prelude {
    pub use crate::{
        Adapter, AdapterManager, LoraConfig, LoraLayer, PeftTrainer, QLoraConfig, QLoraLayer,
        TrainingConfig,
    };
}

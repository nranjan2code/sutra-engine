/// Model definitions for the training GUI
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainableModel {
    pub name: String,
    pub architecture: String,
    pub parameters: u64,
    pub memory_usage: f32,
    pub description: String,
}

impl TrainableModel {
    pub fn new_rwkv_small() -> Self {
        Self {
            name: "RWKV-Small".to_string(),
            architecture: "RWKV".to_string(),
            parameters: 500_000_000,
            memory_usage: 2.0,
            description: "Efficient RNN-style transformer with linear scaling".to_string(),
        }
    }

    pub fn new_mamba_medium() -> Self {
        Self {
            name: "Mamba-Medium".to_string(),
            architecture: "Mamba".to_string(),
            parameters: 1_000_000_000,
            memory_usage: 4.0,
            description: "State space model with selective attention".to_string(),
        }
    }
}
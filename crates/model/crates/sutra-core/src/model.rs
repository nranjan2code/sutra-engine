use crate::tensor::Tensor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Model configuration metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub name: String,
    pub version: String,
    pub hidden_size: usize,
    pub num_layers: usize,
    pub vocab_size: usize,
    pub max_seq_len: usize,
    pub metadata: HashMap<String, String>,
}

impl ModelConfig {
    pub fn new(
        name: impl Into<String>,
        hidden_size: usize,
        num_layers: usize,
        vocab_size: usize,
    ) -> Self {
        Self {
            name: name.into(),
            version: "0.1.0".to_string(),
            hidden_size,
            num_layers,
            vocab_size,
            max_seq_len: 2048,
            metadata: HashMap::new(),
        }
    }

    /// Estimate memory usage in bytes
    pub fn estimate_memory(&self) -> usize {
        // Rough estimate: hidden_size^2 * num_layers * 4 bytes (f32)
        let params = self.hidden_size * self.hidden_size * self.num_layers;
        params * 4
    }
}

/// Container for model weights
pub struct ModelWeights {
    pub tensors: HashMap<String, Tensor>,
    pub config: ModelConfig,
}

impl ModelWeights {
    pub fn new(config: ModelConfig) -> Self {
        Self {
            tensors: HashMap::new(),
            config,
        }
    }

    pub fn add_tensor(&mut self, name: impl Into<String>, tensor: Tensor) {
        self.tensors.insert(name.into(), tensor);
    }

    pub fn get_tensor(&self, name: &str) -> Option<&Tensor> {
        self.tensors.get(name)
    }

    pub fn total_parameters(&self) -> usize {
        self.tensors.values().map(|t| t.data().len()).sum()
    }

    pub fn total_memory(&self) -> usize {
        self.tensors.values().map(|t| t.memory_usage()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_config() {
        let config = ModelConfig::new("test-model", 512, 12, 50000);
        assert_eq!(config.hidden_size, 512);
        assert!(config.estimate_memory() > 0);
    }
}

use ndarray::{Array2, ArrayView2};
use rand_distr::{Distribution, Normal};
use serde::{Deserialize, Serialize};
use sutra_core::{Result, SutraError};

/// Configuration for LoRA (Low-Rank Adaptation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoraConfig {
    /// Rank of the low-rank decomposition
    pub rank: usize,
    /// Scaling factor (alpha / rank)
    pub alpha: f32,
    /// Dropout probability
    pub dropout: f32,
    /// Which layers to apply LoRA to
    pub target_modules: Vec<String>,
}

impl Default for LoraConfig {
    fn default() -> Self {
        Self {
            rank: 8,
            alpha: 16.0,
            dropout: 0.1,
            target_modules: vec!["q_proj".to_string(), "v_proj".to_string()],
        }
    }
}

impl LoraConfig {
    /// Compute the scaling factor
    pub fn scaling(&self) -> f32 {
        self.alpha / self.rank as f32
    }

    /// Create a config with custom rank
    pub fn with_rank(rank: usize) -> Self {
        Self {
            rank,
            alpha: rank as f32 * 2.0,
            ..Default::default()
        }
    }
}

/// LoRA layer implementing low-rank adaptation
///
/// Instead of fine-tuning weight W, we keep W frozen and train:
/// h = Wx + (BA)x * scaling
/// where B is (d × r) and A is (r × d), with r << d
pub struct LoraLayer {
    config: LoraConfig,
    /// Adapter matrix A (rank × in_features)
    lora_a: Array2<f32>,
    /// Adapter matrix B (out_features × rank)
    lora_b: Array2<f32>,
    /// Original frozen weights
    frozen_weight: Option<Array2<f32>>,
}

impl LoraLayer {
    /// Create a new LoRA layer
    pub fn new(in_features: usize, out_features: usize, config: LoraConfig) -> Result<Self> {
        if config.rank >= in_features.min(out_features) {
            return Err(SutraError::InvalidShape(format!(
                "Rank {} too large for dimensions {}×{}",
                config.rank, out_features, in_features
            )));
        }

        // Initialize A with random normal, B with zeros (standard practice)
        let mut rng = rand::thread_rng();
        let normal = Normal::new(0.0, 0.02).unwrap();

        let lora_a = Array2::from_shape_fn((config.rank, in_features), |_| normal.sample(&mut rng));

        let lora_b = Array2::zeros((out_features, config.rank));

        Ok(Self {
            config,
            lora_a,
            lora_b,
            frozen_weight: None,
        })
    }

    /// Attach frozen base weights
    pub fn with_frozen_weight(mut self, weight: Array2<f32>) -> Self {
        self.frozen_weight = Some(weight);
        self
    }

    /// Forward pass through LoRA layer
    /// output = frozen_weight @ input + (lora_b @ lora_a) @ input * scaling
    pub fn forward(&self, input: &ArrayView2<f32>) -> Result<Array2<f32>> {
        let scaling = self.config.scaling();

        // Base output from frozen weights
        let mut output = if let Some(ref weight) = self.frozen_weight {
            weight.dot(input)
        } else {
            Array2::zeros((self.lora_b.shape()[0], input.shape()[1]))
        };

        // LoRA adaptation: B @ A @ x * scaling
        let lora_out = self.lora_b.dot(&self.lora_a.dot(input)) * scaling;
        output = output + lora_out;

        Ok(output)
    }

    /// Get trainable parameters (only A and B matrices)
    pub fn trainable_parameters(&self) -> usize {
        self.lora_a.len() + self.lora_b.len()
    }

    /// Get memory usage for LoRA adapters
    pub fn adapter_memory(&self) -> usize {
        self.trainable_parameters() * std::mem::size_of::<f32>()
    }

    /// Merge LoRA weights into base weights
    /// merged_weight = frozen_weight + lora_b @ lora_a * scaling
    pub fn merge_weights(&self) -> Result<Array2<f32>> {
        let scaling = self.config.scaling();
        let lora_weight = self.lora_b.dot(&self.lora_a) * scaling;

        if let Some(ref base) = self.frozen_weight {
            Ok(base + &lora_weight)
        } else {
            Ok(lora_weight)
        }
    }

    /// Get reference to adapter matrices
    pub fn adapters(&self) -> (&Array2<f32>, &Array2<f32>) {
        (&self.lora_a, &self.lora_b)
    }

    /// Get mutable reference to adapter matrices (for training)
    pub fn adapters_mut(&mut self) -> (&mut Array2<f32>, &mut Array2<f32>) {
        (&mut self.lora_a, &mut self.lora_b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_lora_creation() {
        let config = LoraConfig::with_rank(4);
        let layer = LoraLayer::new(512, 512, config).unwrap();

        // LoRA should dramatically reduce trainable params
        let base_params = 512 * 512;
        let lora_params = layer.trainable_parameters();

        assert!(lora_params < base_params / 10);
    }

    #[test]
    fn test_lora_forward() {
        let config = LoraConfig::with_rank(4);
        let layer = LoraLayer::new(128, 128, config).unwrap();

        let input = Array2::ones((128, 16)); // batch_size=16
        let output = layer.forward(&input.view()).unwrap();

        assert_eq!(output.shape(), &[128, 16]);
    }

    #[test]
    fn test_parameter_efficiency() {
        let in_dim = 4096;
        let out_dim = 4096;
        let rank = 8;

        let config = LoraConfig::with_rank(rank);
        let layer = LoraLayer::new(in_dim, out_dim, config).unwrap();

        let full_params = in_dim * out_dim;
        let lora_params = layer.trainable_parameters();
        let reduction = full_params as f32 / lora_params as f32;

        println!("Parameter reduction: {:.2}x", reduction);
        assert!(reduction > 100.0); // Should be >100x reduction
    }
}

use crate::lora::{LoraConfig, LoraLayer};
use serde::{Deserialize, Serialize};
use sutra_core::Result;
use sutra_quantize::QuantizedTensor;

/// Configuration for QLoRA (Quantized LoRA)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QLoraConfig {
    /// Base LoRA configuration
    pub lora: LoraConfig,
    /// Quantization bits for base model
    pub quant_bits: u8,
    /// Whether to use double quantization
    pub double_quant: bool,
}

impl Default for QLoraConfig {
    fn default() -> Self {
        Self {
            lora: LoraConfig::default(),
            quant_bits: 4,
            double_quant: true,
        }
    }
}

/// QLoRA layer: LoRA on top of a quantized base model
///
/// QLoRA enables fine-tuning massive models on consumer hardware by:
/// 1. Loading base model in 4-bit quantized format
/// 2. Training only small LoRA adapters in full precision
/// 3. Using paged optimizers to manage memory spikes
pub struct QLoraLayer {
    #[allow(dead_code)]
    config: QLoraConfig,
    /// LoRA adapters (trained)
    lora_layer: LoraLayer,
    /// Quantized base weights (frozen)
    quantized_base: Option<QuantizedTensor>,
}

impl QLoraLayer {
    /// Create a new QLoRA layer
    pub fn new(in_features: usize, out_features: usize, config: QLoraConfig) -> Result<Self> {
        let lora_layer = LoraLayer::new(in_features, out_features, config.lora.clone())?;

        Ok(Self {
            config,
            lora_layer,
            quantized_base: None,
        })
    }

    /// Attach quantized base weights
    pub fn with_quantized_base(mut self, base: QuantizedTensor) -> Self {
        self.quantized_base = Some(base);
        self
    }

    /// Get memory usage
    /// QLoRA only stores:
    /// - Quantized base (4-bit)
    /// - LoRA adapters (full precision, but tiny)
    pub fn memory_usage(&self) -> usize {
        let base_mem = self
            .quantized_base
            .as_ref()
            .map(|q| q.memory_usage())
            .unwrap_or(0);

        let adapter_mem = self.lora_layer.adapter_memory();

        base_mem + adapter_mem
    }

    /// Get trainable parameter count
    pub fn trainable_parameters(&self) -> usize {
        self.lora_layer.trainable_parameters()
    }

    /// Get reference to LoRA layer for training
    pub fn lora_layer(&self) -> &LoraLayer {
        &self.lora_layer
    }

    /// Get mutable reference to LoRA layer for training
    pub fn lora_layer_mut(&mut self) -> &mut LoraLayer {
        &mut self.lora_layer
    }
}

/// Helper to estimate QLoRA memory requirements
pub struct QLoraMemoryEstimator;

impl QLoraMemoryEstimator {
    /// Estimate memory for fine-tuning with QLoRA
    ///
    /// # Arguments
    /// * `model_params` - Total parameters in base model
    /// * `rank` - LoRA rank
    /// * `trainable_layers` - Number of layers with LoRA adapters
    pub fn estimate(model_params: usize, rank: usize, trainable_layers: usize) -> MemoryEstimate {
        // Base model in 4-bit
        let base_mem = (model_params / 2) as f64; // 4 bits = 0.5 bytes per param

        // LoRA adapters (estimate 2 * hidden_dim * rank per layer)
        let hidden_dim = (model_params as f64 / trainable_layers as f64).sqrt();
        let adapter_params = (2.0 * hidden_dim * rank as f64 * trainable_layers as f64) as usize;
        let adapter_mem = adapter_params * 4; // f32

        // Optimizer states (Adam needs 2x params)
        let optimizer_mem = adapter_params * 4 * 2;

        // Gradients
        let gradient_mem = adapter_params * 4;

        // Activations (rough estimate)
        let activation_mem = 1_000_000; // 1MB buffer

        let total_bytes =
            base_mem as usize + adapter_mem + optimizer_mem + gradient_mem + activation_mem;

        MemoryEstimate {
            base_model: base_mem as usize,
            adapters: adapter_mem,
            optimizer_states: optimizer_mem,
            gradients: gradient_mem,
            activations: activation_mem,
            total: total_bytes,
        }
    }
}

/// Memory usage breakdown for QLoRA
#[derive(Debug, Clone)]
pub struct MemoryEstimate {
    pub base_model: usize,
    pub adapters: usize,
    pub optimizer_states: usize,
    pub gradients: usize,
    pub activations: usize,
    pub total: usize,
}

impl MemoryEstimate {
    pub fn total_gb(&self) -> f64 {
        self.total as f64 / 1_073_741_824.0
    }

    pub fn fits_in(&self, available_gb: usize) -> bool {
        self.total_gb() <= available_gb as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qlora_memory_estimate() {
        // Simulate a 3B parameter model
        let model_params = 3_000_000_000;
        let rank = 8;
        let layers = 32;

        let estimate = QLoraMemoryEstimator::estimate(model_params, rank, layers);

        println!("QLoRA Memory Estimate:");
        println!(
            "  Base model (4-bit): {:.2} GB",
            estimate.base_model as f64 / 1e9
        );
        println!("  Adapters: {:.2} GB", estimate.adapters as f64 / 1e9);
        println!(
            "  Optimizer: {:.2} GB",
            estimate.optimizer_states as f64 / 1e9
        );
        println!("  Total: {:.2} GB", estimate.total_gb());

        // Should fit in 16GB for 3B model
        assert!(estimate.fits_in(16));
    }

    #[test]
    fn test_qlora_creation() {
        let config = QLoraConfig::default();
        let layer = QLoraLayer::new(512, 512, config).unwrap();

        assert_eq!(
            layer.trainable_parameters(),
            layer.lora_layer().trainable_parameters()
        );
    }
}

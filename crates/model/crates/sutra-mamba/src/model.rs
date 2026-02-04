use crate::layer::{MambaLayer, MambaState};
use serde::{Deserialize, Serialize};
use sutra_core::Result;
use ndarray::{Array1, Array2};

/// Mamba model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MambaConfig {
    pub num_layers: usize,
    pub hidden_size: usize,
    pub vocab_size: usize,
    pub state_size: usize,
    pub expand_factor: usize,
    pub conv_kernel: usize,
    pub max_seq_len: usize,
}

impl MambaConfig {
    pub fn new(num_layers: usize, hidden_size: usize, vocab_size: usize) -> Self {
        Self {
            num_layers,
            hidden_size,
            vocab_size,
            state_size: 16,   // SSM state dimension
            expand_factor: 2, // Internal expansion
            conv_kernel: 4,   // Convolution kernel size
            max_seq_len: 2048,
        }
    }

    /// Mamba-3B configuration
    pub fn mamba_3b() -> Self {
        Self::new(48, 2560, 50000)
    }

    /// Estimate memory usage
    pub fn estimate_memory(&self) -> usize {
        // Mamba has linear memory complexity
        let params_per_layer = self.hidden_size * self.hidden_size * self.expand_factor * 4;
        let total_params = params_per_layer * self.num_layers;
        total_params * std::mem::size_of::<f32>()
    }

    /// Estimate throughput advantage over Transformer
    pub fn throughput_multiplier(&self, seq_len: usize) -> f32 {
        // Mamba is O(n) while Transformer is O(n²)
        // For typical sequences, Mamba is ~5x faster
        let transformer_ops = seq_len * seq_len;
        let mamba_ops = seq_len;
        transformer_ops as f32 / mamba_ops as f32
    }
}

/// Mamba model for efficient sequence modeling
#[allow(dead_code)]
pub struct MambaModel {
    config: MambaConfig,
    layers: Vec<MambaLayer>,
    // Embedding layer
    token_embedding: Array2<f32>,
    // Final layer norm
    final_norm_weight: Array1<f32>,
    final_norm_bias: Array1<f32>,
    // Output projection
    output_weight: Array2<f32>,
}

impl MambaModel {
    pub fn new(config: MambaConfig) -> Result<Self> {
        let mut layers = Vec::with_capacity(config.num_layers);

        for _ in 0..config.num_layers {
            layers.push(MambaLayer::new(&config)?);
        }

        // Initialize with Xavier/Glorot uniform  
        let scale = (6.0 / (config.vocab_size as f32 + config.hidden_size as f32)).sqrt();
        
        // Token embedding matrix
        let token_embedding = Array2::from_shape_fn(
            (config.vocab_size, config.hidden_size),
            |_| (Self::rand() * 2.0 - 1.0) * scale
        );

        // Final layer norm
        let final_norm_weight = Array1::ones(config.hidden_size);
        let final_norm_bias = Array1::zeros(config.hidden_size);

        // Output projection (tied weights with input embedding)
        let output_weight = token_embedding.t().to_owned();

        Ok(Self { 
            config, 
            layers,
            token_embedding,
            final_norm_weight,
            final_norm_bias,
            output_weight,
        })
    }

    /// Forward pass through Mamba
    ///
    /// # Arguments
    /// * `input` - Input tokens [batch_size, seq_len]
    ///
    /// # Returns
    /// * Logits [batch_size, seq_len, vocab_size]
    pub fn forward(&self, input: &[usize]) -> Result<Vec<f32>> {
        if input.is_empty() {
            return Ok(vec![0.0; self.config.vocab_size]);
        }

        // Process only the last token for autoregressive generation
        let token_id = input[input.len() - 1];

        // 1. Embed the token
        let mut x = self.embed_token(token_id);

        // 2. Initialize layer states
        let mut layer_states: Vec<MambaState> = (0..self.config.num_layers)
            .map(|_| MambaState::new(
                self.config.hidden_size,
                self.config.state_size,
                self.config.expand_factor,
                self.config.conv_kernel,
            ))
            .collect();

        // 3. Process through Mamba layers sequentially
        for (i, layer) in self.layers.iter().enumerate() {
            x = layer.forward(&x, &mut layer_states[i])?;
        }

        // 4. Apply final layer norm
        x = self.final_layer_norm(&x);

        // 5. Project to vocabulary
        let logits = self.output_projection(&x);

        Ok(logits)
    }

    /// Generate text with Mamba
    #[allow(unused_variables)]
    pub fn generate(
        &self,
        prompt: &[usize],
        max_tokens: usize,
        temperature: f32,
    ) -> Result<Vec<usize>> {
        let mut tokens = prompt.to_vec();

        for _ in 0..max_tokens {
            let logits = self.forward(&tokens)?;
            let next_token = self.sample_token(&logits, temperature);
            tokens.push(next_token);

            if next_token == 0 {
                break;
            }
        }

        Ok(tokens)
    }

    fn sample_token(&self, logits: &[f32], _temperature: f32) -> usize {
        logits
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    }

    pub fn config(&self) -> &MambaConfig {
        &self.config
    }

    /// Embed a single token
    fn embed_token(&self, token_id: usize) -> Array1<f32> {
        if token_id >= self.config.vocab_size {
            // Return zero vector for out-of-vocab tokens
            return Array1::zeros(self.config.hidden_size);
        }
        self.token_embedding.row(token_id).to_owned()
    }

    /// Apply final layer normalization
    fn final_layer_norm(&self, x: &Array1<f32>) -> Array1<f32> {
        let mean = x.mean().unwrap_or(0.0);
        let var = x.iter().map(|&v| (v - mean).powi(2)).sum::<f32>() / x.len() as f32;
        let std = (var + 1e-5).sqrt();
        
        x.iter()
            .zip(self.final_norm_weight.iter())
            .zip(self.final_norm_bias.iter())
            .map(|((&val, &w), &b)| ((val - mean) / std) * w + b)
            .collect()
    }

    /// Project hidden state to vocabulary logits
    fn output_projection(&self, x: &Array1<f32>) -> Vec<f32> {
        // Matrix multiply with embedding weights (vocab_size x hidden_size) * (hidden_size,) = (vocab_size,)
        // Use the original embedding matrix, not the transposed one
        let mut logits = vec![0.0; self.config.vocab_size];
        for (i, row) in self.token_embedding.rows().into_iter().enumerate() {
            logits[i] = x.dot(&row);
        }
        logits
    }

    /// Load model weights from checkpoint
    pub fn load_weights(
        &mut self,
        token_embedding: Array2<f32>,
        final_norm_weight: Array1<f32>,
        final_norm_bias: Array1<f32>,
    ) {
        self.token_embedding = token_embedding;
        self.final_norm_weight = final_norm_weight;
        self.final_norm_bias = final_norm_bias;
        // Update output weights (tied)
        self.output_weight = self.token_embedding.t().to_owned();
    }

    /// Get mutable access to layers for loading weights
    pub fn layers_mut(&mut self) -> &mut [MambaLayer] {
        &mut self.layers
    }

    /// Simple PRNG for initialization
    fn rand() -> f32 {
        use std::cell::Cell;
        thread_local! {
            static SEED: Cell<u64> = const { Cell::new(0xfedcba0987654321) };
        }
        SEED.with(|seed| {
            let mut s = seed.get();
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            seed.set(s);
            (s as f64 / u64::MAX as f64) as f32
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mamba_config() {
        let config = MambaConfig::mamba_3b();
        assert_eq!(config.num_layers, 48);
        assert_eq!(config.hidden_size, 2560);
    }

    #[test]
    fn test_complexity_advantage() {
        let config = MambaConfig::new(24, 1024, 50000);

        // For 2048 token sequence
        let speedup = config.throughput_multiplier(2048);
        println!("Mamba throughput advantage: {:.1}x", speedup);

        // Should be ~2048x faster (linear vs quadratic)
        assert!(speedup > 1000.0);
    }

    #[test]
    fn test_mamba_model() {
        let config = MambaConfig::new(12, 768, 50000);
        let model = MambaModel::new(config).unwrap();
        assert_eq!(model.layers.len(), 12);
    }
}

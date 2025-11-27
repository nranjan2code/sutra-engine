use crate::layer::RwkvLayer;
use crate::state::RwkvState;
use serde::{Deserialize, Serialize};
use sutra_core::Result;
use ndarray::Array1;

/// RWKV model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RwkvConfig {
    pub num_layers: usize,
    pub hidden_size: usize,
    pub vocab_size: usize,
    pub max_seq_len: usize,
    /// Layer normalization epsilon
    pub layer_norm_eps: f32,
}

impl RwkvConfig {
    pub fn new(num_layers: usize, hidden_size: usize, vocab_size: usize) -> Self {
        Self {
            num_layers,
            hidden_size,
            vocab_size,
            max_seq_len: 2048,
            layer_norm_eps: 1e-5,
        }
    }

    /// Estimate memory usage for inference
    pub fn estimate_memory(&self) -> usize {
        // RWKV has constant memory complexity
        // State: hidden_size * num_layers * 2 (for att and ffn states)
        let state_mem = self.hidden_size * self.num_layers * 2 * std::mem::size_of::<f32>();

        // Weights: rough estimate
        let weight_mem = self.hidden_size * self.hidden_size * self.num_layers * 4 * 4;

        state_mem + weight_mem
    }
}

/// RWKV model for efficient inference
#[allow(dead_code)]
pub struct RwkvModel {
    config: RwkvConfig,
    layers: Vec<RwkvLayer>,
    // Embedding layer
    token_embedding: ndarray::Array2<f32>,
    // Final layer norm
    final_ln_weight: ndarray::Array1<f32>,
    final_ln_bias: ndarray::Array1<f32>,
    // Output projection (often tied with input embedding)
    output_weight: ndarray::Array2<f32>,
}

impl RwkvModel {
    pub fn new(config: RwkvConfig) -> Result<Self> {
        let mut layers = Vec::with_capacity(config.num_layers);

        for layer_idx in 0..config.num_layers {
            layers.push(RwkvLayer::new(config.hidden_size, layer_idx)?);
        }

        // Initialize with Xavier/Glorot uniform
        let scale = (6.0 / (config.vocab_size as f32 + config.hidden_size as f32)).sqrt();
        
        // Token embedding matrix
        let token_embedding = ndarray::Array2::from_shape_fn(
            (config.vocab_size, config.hidden_size),
            |_| (rand::random::<f32>() * 2.0 - 1.0) * scale
        );

        // Final layer norm
        let final_ln_weight = ndarray::Array1::ones(config.hidden_size);
        let final_ln_bias = ndarray::Array1::zeros(config.hidden_size);

        // Output projection (tied weights with input embedding)
        let output_weight = token_embedding.t().to_owned();

        Ok(Self { 
            config, 
            layers,
            token_embedding,
            final_ln_weight,
            final_ln_bias,
            output_weight,
        })
    }

    /// Forward pass through the model
    ///
    /// # Arguments
    /// * `input` - Input token IDs [batch_size, seq_len]
    /// * `state` - Optional previous state for sequential generation
    ///
    /// # Returns
    /// * Logits [batch_size, seq_len, vocab_size]
    /// * Updated state for next step
    pub fn forward(
        &self,
        input: &[usize],
        state: Option<RwkvState>,
    ) -> Result<(Vec<f32>, RwkvState)> {
        if input.is_empty() {
            return Ok((vec![0.0; self.config.vocab_size], state.unwrap_or_else(|| RwkvState::new(&self.config))));
        }

        let mut state = state.unwrap_or_else(|| RwkvState::new(&self.config));

        // Process only the last token for autoregressive generation
        let token_id = input[input.len() - 1];
        
        // 1. Embed the token
        let mut x = self.embed_token(token_id);

        // 2. Process through RWKV layers sequentially
        for (i, layer) in self.layers.iter().enumerate() {
            x = layer.forward(&x, &mut state.layers[i])?;
        }

        // 3. Apply final layer norm
        x = self.final_layer_norm(&x);

        // 4. Project to vocabulary (using input embedding weights transposed as is common)
        let logits = self.output_projection(&x);

        Ok((logits, state))
    }

    /// Generate text autoregressively
    #[allow(unused_variables)]
    pub fn generate(
        &self,
        prompt: &[usize],
        max_tokens: usize,
        temperature: f32,
    ) -> Result<Vec<usize>> {
        let mut tokens = prompt.to_vec();
        let mut state = RwkvState::new(&self.config);

        for _ in 0..max_tokens {
            let (logits, new_state) = self.forward(&tokens, Some(state))?;
            state = new_state;

            // Sample next token (placeholder)
            let next_token = self.sample_token(&logits, temperature);
            tokens.push(next_token);

            // Check for EOS token
            if next_token == 0 {
                break;
            }
        }

        Ok(tokens)
    }

    fn sample_token(&self, logits: &[f32], _temperature: f32) -> usize {
        // Placeholder: return token with highest logit
        logits
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    }

    pub fn config(&self) -> &RwkvConfig {
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
            .zip(self.final_ln_weight.iter())
            .zip(self.final_ln_bias.iter())
            .map(|((&val, &w), &b)| ((val - mean) / std) * w + b)
            .collect()
    }

    /// Project hidden state to vocabulary logits
    fn output_projection(&self, x: &Array1<f32>) -> Vec<f32> {
        // Matrix multiply with output weights (hidden_size x vocab_size) * (hidden_size,) = (vocab_size,)
        // Note: output_weight is transposed embedding, so we need x.dot(output_weight)
        let mut logits = vec![0.0; self.config.vocab_size];
        for (i, row) in self.token_embedding.rows().into_iter().enumerate() {
            logits[i] = x.dot(&row);
        }
        logits
    }

    /// Load model weights from checkpoint
    pub fn load_weights(
        &mut self,
        token_embedding: ndarray::Array2<f32>,
        final_ln_weight: ndarray::Array1<f32>,
        final_ln_bias: ndarray::Array1<f32>,
    ) {
        self.token_embedding = token_embedding;
        self.final_ln_weight = final_ln_weight;
        self.final_ln_bias = final_ln_bias;
        // Update output weights (tied)
        self.output_weight = self.token_embedding.t().to_owned();
    }

    /// Get mutable access to layers for loading weights
    pub fn layers_mut(&mut self) -> &mut [RwkvLayer] {
        &mut self.layers
    }
}

// Simple PRNG for initialization
mod rand {
    use std::cell::Cell;
    thread_local! {
        static SEED: Cell<u64> = const { Cell::new(0x1234567890abcdef) };
    }
    
    pub fn random<T>() -> T 
    where
        T: From<f32>
    {
        SEED.with(|seed| {
            let mut s = seed.get();
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            seed.set(s);
            T::from(((s as f64) / (u64::MAX as f64)) as f32)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;    #[test]
    fn test_rwkv_model_creation() {
        let config = RwkvConfig::new(12, 768, 50000);
        let model = RwkvModel::new(config).unwrap();
        assert_eq!(model.layers.len(), 12);
    }

    #[test]
    fn test_memory_efficiency() {
        let config = RwkvConfig::new(24, 1024, 50000);
        let memory = config.estimate_memory();
        let memory_gb = memory as f64 / 1_073_741_824.0;

        println!("RWKV-24L-1024D estimated memory: {:.2} GB", memory_gb);

        // RWKV should fit comfortably in 16GB
        assert!(memory_gb < 10.0);
    }
}

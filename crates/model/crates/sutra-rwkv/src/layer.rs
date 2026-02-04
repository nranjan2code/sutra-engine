use crate::attention::{RwkvAttention, WkvState};
use crate::ffn::RwkvFfn;
use ndarray::Array1;
use sutra_core::Result;

/// RWKV layer combining time-mixing and channel-mixing
///
/// Architecture:
/// x -> LayerNorm -> Time-Mixing (Attention) -> Residual
///   -> LayerNorm -> Channel-Mixing (FFN) -> Residual -> output
pub struct RwkvLayer {
    #[allow(dead_code)]
    hidden_size: usize,
    #[allow(dead_code)]
    layer_idx: usize,
    // Layer components
    attention: RwkvAttention,
    ffn: RwkvFfn,
    // Layer normalization parameters
    ln1_weight: Array1<f32>,
    ln1_bias: Array1<f32>,
    ln2_weight: Array1<f32>,
    ln2_bias: Array1<f32>,
}

impl RwkvLayer {
    pub fn new(hidden_size: usize, layer_idx: usize) -> Result<Self> {
        Ok(Self {
            hidden_size,
            layer_idx,
            attention: RwkvAttention::new(hidden_size),
            ffn: RwkvFfn::new(hidden_size),
            ln1_weight: Array1::ones(hidden_size),
            ln1_bias: Array1::zeros(hidden_size),
            ln2_weight: Array1::ones(hidden_size),
            ln2_bias: Array1::zeros(hidden_size),
        })
    }
    
    /// Load layer weights from checkpoint
    pub fn load_layer_norm(&mut self, 
        ln1_weight: Array1<f32>,
        ln1_bias: Array1<f32>,
        ln2_weight: Array1<f32>,
        ln2_bias: Array1<f32>,
    ) {
        self.ln1_weight = ln1_weight;
        self.ln1_bias = ln1_bias;
        self.ln2_weight = ln2_weight;
        self.ln2_bias = ln2_bias;
    }
    
    pub fn attention_mut(&mut self) -> &mut RwkvAttention {
        &mut self.attention
    }
    
    pub fn ffn_mut(&mut self) -> &mut RwkvFfn {
        &mut self.ffn
    }

    /// Forward pass through RWKV layer
    ///
    /// RWKV layer implements:
    /// 1. LayerNorm + Time-mixing (O(n) attention-like mechanism)
    /// 2. Residual connection
    /// 3. LayerNorm + Channel-mixing (FFN-like mechanism)  
    /// 4. Residual connection
    pub fn forward(&self, x: &Array1<f32>, state: &mut LayerState) -> Result<Array1<f32>> {
        // Time-mixing block with residual
        let ln1_out = self.layer_norm(x, &self.ln1_weight, &self.ln1_bias);
        let att_out = self.attention.forward(&ln1_out, &mut state.att_state)?;
        let x = x + &att_out;
        
        // Channel-mixing block with residual
        let ln2_out = self.layer_norm(&x, &self.ln2_weight, &self.ln2_bias);
        let ffn_out = self.ffn.forward(&ln2_out, &mut state.ffn_prev_x)?;
        let x = x + &ffn_out;
        
        Ok(x)
    }
    
    /// Layer normalization
    fn layer_norm(&self, x: &Array1<f32>, weight: &Array1<f32>, bias: &Array1<f32>) -> Array1<f32> {
        let mean = x.mean().unwrap_or(0.0);
        let var = x.iter().map(|&v| (v - mean).powi(2)).sum::<f32>() / x.len() as f32;
        let std = (var + 1e-5).sqrt();
        
        x.iter()
            .zip(weight.iter())
            .zip(bias.iter())
            .map(|((&val, &w), &b)| ((val - mean) / std) * w + b)
            .collect()
    }
}

/// State for a single RWKV layer
#[derive(Debug, Clone)]
pub struct LayerState {
    /// Time-mixing (attention) state
    pub att_state: WkvState,
    /// Channel-mixing (FFN) previous input
    pub ffn_prev_x: Array1<f32>,
}

impl LayerState {
    pub fn new(hidden_size: usize) -> Self {
        Self {
            att_state: WkvState::new(hidden_size),
            ffn_prev_x: Array1::zeros(hidden_size),
        }
    }
}

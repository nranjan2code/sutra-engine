use crate::layer::LayerState;
use crate::model::RwkvConfig;

/// RWKV model state for sequential inference
///
/// Unlike Transformers that require full KV cache (O(n)),
/// RWKV maintains constant-size state (O(1))
#[derive(Debug, Clone)]
pub struct RwkvState {
    pub layers: Vec<LayerState>,
}

impl RwkvState {
    pub fn new(config: &RwkvConfig) -> Self {
        let layers = (0..config.num_layers)
            .map(|_| LayerState::new(config.hidden_size))
            .collect();

        Self { layers }
    }

    /// Reset state to initial values
    pub fn reset(&mut self) {
        for layer in &mut self.layers {
            layer.att_state.aa.fill(0.0);
            layer.att_state.bb.fill(0.0);
            layer.att_state.pp.fill(-1e30);
            layer.att_state.prev_x.fill(0.0);
            layer.ffn_prev_x.fill(0.0);
        }
    }

    /// Get memory usage
    pub fn memory_usage(&self) -> usize {
        self.layers
            .iter()
            .map(|l| {
                l.att_state.aa.len() + 
                l.att_state.bb.len() + 
                l.att_state.pp.len() + 
                l.att_state.prev_x.len() +
                l.ffn_prev_x.len()
            })
            .sum::<usize>()
            * std::mem::size_of::<f32>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_memory() {
        let config = RwkvConfig::new(24, 2048, 50000);
        let state = RwkvState::new(&config);

        let mem_bytes = state.memory_usage();
        let mem_kb = mem_bytes as f64 / 1024.0;

        println!("RWKV state memory: {:.2} KB", mem_kb);

        // State should be tiny compared to Transformer KV cache
        assert!(mem_kb < 10000.0); // Less than 10MB (allowing for larger WKV state)
    }
}

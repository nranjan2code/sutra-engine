use ndarray::{Array1, Array2};
use sutra_core::Result;

/// RWKV channel-mixing (FFN) mechanism
/// 
/// Similar to standard FFN but with time-mixing for temporal coherence
pub struct RwkvFfn {
    #[allow(dead_code)]
    hidden_size: usize,
    #[allow(dead_code)]
    ffn_size: usize,
    // Time-mixing weight
    time_mix_k: Array1<f32>,
    time_mix_r: Array1<f32>,
    // FFN weights
    key: Array2<f32>,
    value: Array2<f32>,
    receptance: Array2<f32>,
}

impl RwkvFfn {
    pub fn new(hidden_size: usize) -> Self {
        let ffn_size = hidden_size * 4; // Standard 4x expansion
        let scale = (6.0 / (hidden_size as f32 * 2.0)).sqrt();
        
        Self {
            hidden_size,
            ffn_size,
            time_mix_k: Array1::from_elem(hidden_size, 0.5),
            time_mix_r: Array1::from_elem(hidden_size, 0.5),
            key: Array2::from_shape_fn((ffn_size, hidden_size), |_| 
                (rand::random::<f32>() * 2.0 - 1.0) * scale),
            value: Array2::from_shape_fn((hidden_size, ffn_size), |_| 
                (rand::random::<f32>() * 2.0 - 1.0) * scale),
            receptance: Array2::from_shape_fn((hidden_size, hidden_size), |_| 
                (rand::random::<f32>() * 2.0 - 1.0) * scale),
        }
    }
    
    /// Load weights from checkpoint
    pub fn load_weights(
        &mut self,
        time_mix_k: Array1<f32>,
        time_mix_r: Array1<f32>,
        key: Array2<f32>,
        value: Array2<f32>,
        receptance: Array2<f32>,
    ) {
        self.time_mix_k = time_mix_k;
        self.time_mix_r = time_mix_r;
        self.key = key;
        self.value = value;
        self.receptance = receptance;
    }

    /// Channel-mixing forward pass
    ///
    /// 1. Time-mix current and previous inputs
    /// 2. Compute K and R projections
    /// 3. Apply squared ReLU activation to K
    /// 4. Project through V
    /// 5. Gate with sigmoid(R)
    pub fn forward(&self, x: &Array1<f32>, prev_x: &mut Array1<f32>) -> Result<Array1<f32>> {
        // Time-mixing
        let xk = prev_x.clone() * &self.time_mix_k + x * (1.0 - &self.time_mix_k);
        let xr = prev_x.clone() * &self.time_mix_r + x * (1.0 - &self.time_mix_r);
        
        // Update state
        *prev_x = x.clone();
        
        // Project and activate
        let k = self.key.dot(&xk);
        let r = self.receptance.dot(&xr);
        
        // Squared ReLU activation on K
        let k_activated = k.mapv(|x| {
            let relu = x.max(0.0);
            relu * relu
        });
        
        // Project through V
        let v = self.value.dot(&k_activated);
        
        // Gate with sigmoid(R)
        let r_activated = r.mapv(|x| 1.0 / (1.0 + (-x).exp()));
        let output = &v * &r_activated;
        
        Ok(output)
    }
}

// Simple PRNG for initialization
mod rand {
    use std::cell::Cell;
    thread_local! {
        static SEED: Cell<u64> = const { Cell::new(0x987654321fedcba0) };
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

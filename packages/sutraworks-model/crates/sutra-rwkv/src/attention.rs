use ndarray::{Array1, Array2};
use sutra_core::Result;

/// RWKV time-mixing (attention) mechanism
///
/// Unlike standard attention with O(n²) complexity,
/// RWKV attention has O(1) memory per step using WKV recurrence
///
/// WKV formula: wkv_t = (α_t · state_{t-1} + exp(k_t + w_t) · v_t) / (β_t · state_{t-1} + exp(k_t + w_t))
pub struct RwkvAttention {
    hidden_size: usize,
    // Time-mixing weights
    time_mix_k: Array1<f32>,
    time_mix_v: Array1<f32>,
    time_mix_r: Array1<f32>,
    // Projection weights
    key: Array2<f32>,
    value: Array2<f32>,
    receptance: Array2<f32>,
    output: Array2<f32>,
    // Time decay and bonus
    time_decay: Array1<f32>,
    time_first: Array1<f32>,
}

impl RwkvAttention {
    pub fn new(hidden_size: usize) -> Self {
        // Initialize with Xavier/Glorot uniform initialization
        let scale = (6.0 / (hidden_size as f32 * 2.0)).sqrt();
        
        Self {
            hidden_size,
            time_mix_k: Array1::from_elem(hidden_size, 0.5),
            time_mix_v: Array1::from_elem(hidden_size, 0.5),
            time_mix_r: Array1::from_elem(hidden_size, 0.5),
            key: Array2::from_shape_fn((hidden_size, hidden_size), |_| 
                (rand::random::<f32>() * 2.0 - 1.0) * scale),
            value: Array2::from_shape_fn((hidden_size, hidden_size), |_| 
                (rand::random::<f32>() * 2.0 - 1.0) * scale),
            receptance: Array2::from_shape_fn((hidden_size, hidden_size), |_| 
                (rand::random::<f32>() * 2.0 - 1.0) * scale),
            output: Array2::from_shape_fn((hidden_size, hidden_size), |_| 
                (rand::random::<f32>() * 2.0 - 1.0) * scale),
            time_decay: Array1::from_elem(hidden_size, -5.0),
            time_first: Array1::from_elem(hidden_size, 0.0),
        }
    }
    
    /// Load weights from checkpoint
    #[allow(clippy::too_many_arguments)]
    pub fn load_weights(
        &mut self,
        time_mix_k: Array1<f32>,
        time_mix_v: Array1<f32>,
        time_mix_r: Array1<f32>,
        key_weight: Array2<f32>,
        value_weight: Array2<f32>,
        receptance_weight: Array2<f32>,
        output_weight: Array2<f32>,
        time_decay: Array1<f32>,
        time_first: Array1<f32>,
    ) {
        self.time_mix_k = time_mix_k;
        self.time_mix_v = time_mix_v;
        self.time_mix_r = time_mix_r;
        self.key = key_weight;
        self.value = value_weight;
        self.receptance = receptance_weight;
        self.output = output_weight;
        self.time_decay = time_decay;
        self.time_first = time_first;
    }

    /// Time-mixing forward pass with WKV kernel
    ///
    /// RWKV-4 formula:
    /// 1. Mix current and previous inputs
    /// 2. Compute K, V, R projections  
    /// 3. Apply WKV recurrence (O(n) complexity)
    /// 4. Gate with receptance and project output
    pub fn forward(&self, x: &Array1<f32>, state: &mut WkvState) -> Result<Array1<f32>> {
        // Time-mixing: interpolate between current and previous input
        let xk = &state.prev_x * &self.time_mix_k + x * (1.0 - &self.time_mix_k);
        let xv = &state.prev_x * &self.time_mix_v + x * (1.0 - &self.time_mix_v);
        let xr = &state.prev_x * &self.time_mix_r + x * (1.0 - &self.time_mix_r);
        
        // Update state for next step
        state.prev_x = x.clone();
        
        // Project to K, V, R
        let k = self.key.dot(&xk);
        let v = self.value.dot(&xv);
        let r = self.receptance.dot(&xr);
        
        // Apply WKV kernel (this is the O(n) magic)
        let wkv = self.compute_wkv(&k, &v, &mut state.aa, &mut state.bb, &mut state.pp);
        
        // Gate with sigmoid(receptance) and project output
        let r_activated = r.mapv(|x| 1.0 / (1.0 + (-x).exp())); // sigmoid
        let gated = &wkv * &r_activated;
        let output = self.output.dot(&gated);
        
        Ok(output)
    }
    
    /// WKV (Weighted Key-Value) kernel - the core RWKV innovation
    ///
    /// This achieves O(n) complexity instead of O(n²) attention
    /// by maintaining running statistics in the state
    fn compute_wkv(
        &self,
        k: &Array1<f32>,
        v: &Array1<f32>,
        aa: &mut Array1<f32>,
        bb: &mut Array1<f32>,
        pp: &mut Array1<f32>,
    ) -> Array1<f32> {
        let mut wkv = Array1::zeros(self.hidden_size);
        
        for i in 0..self.hidden_size {
            let k_i = k[i];
            let v_i = v[i];
            let w_i = self.time_decay[i];
            let u_i = self.time_first[i];
            
            // RWKV-4 WKV computation
            // Numerically stable version using log-sum-exp trick
            let p = pp[i].max(u_i + k_i).max(w_i + pp[i]);
            
            let e1 = (u_i + k_i - p).exp();
            let e2 = (w_i + pp[i] - p).exp();
            
            let a = e1 * v_i + e2 * aa[i];
            let b = e1 + e2 * bb[i];
            
            wkv[i] = a / (b + 1e-8); // Add epsilon for numerical stability
            
            // Update state for next token
            let q = pp[i].max(w_i + k_i);
            let e3 = (pp[i] - q).exp();
            let e4 = (w_i + k_i - q).exp();
            
            aa[i] = e3 * aa[i] + e4 * v_i;
            bb[i] = e3 * bb[i] + e4;
            pp[i] = q;
        }
        
        wkv
    }
}

/// State for WKV kernel computation
#[derive(Debug, Clone)]
pub struct WkvState {
    pub prev_x: Array1<f32>,
    pub aa: Array1<f32>,  // Numerator accumulator
    pub bb: Array1<f32>,  // Denominator accumulator  
    pub pp: Array1<f32>,  // Log-space maximum for stability
}

impl WkvState {
    pub fn new(hidden_size: usize) -> Self {
        Self {
            prev_x: Array1::zeros(hidden_size),
            aa: Array1::zeros(hidden_size),
            bb: Array1::zeros(hidden_size),
            pp: Array1::from_elem(hidden_size, -1e30),
        }
    }
}

// Simple PRNG for initialization (avoid external rand dependency issues)
mod rand {
    use std::cell::Cell;
    thread_local! {
        static SEED: Cell<u64> = const { Cell::new(0x123456789abcdef0) };
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

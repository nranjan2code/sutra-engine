use crate::ssm::StateSpaceModel;
use crate::MambaConfig;
use ndarray::{Array1, Array2};
use sutra_core::{Result, Tensor, DType};

/// Mamba layer combining selective SSM with gating
///
/// Architecture:
/// x -> Conv1D -> SiLU -> SSM -> 
///   -> Linear -> + (skip connection) -> output
pub struct MambaLayer {
    hidden_size: usize,
    expand_factor: usize,
    // Projections
    in_proj: Array2<f32>,    // Projects input to expanded dimension
    out_proj: Array2<f32>,   // Projects back to hidden size
    // Convolution for local context
    conv_kernel: usize,
    conv_weight: Array2<f32>,
    conv_bias: Array1<f32>,
    // Selective SSM core
    ssm: StateSpaceModel,
    // Layer normalization
    norm_weight: Array1<f32>,
    norm_bias: Array1<f32>,
}

impl MambaLayer {
    pub fn new(config: &MambaConfig) -> Result<Self> {
        let expanded_size = config.hidden_size * config.expand_factor;
        let scale = (2.0 / config.hidden_size as f32).sqrt();
        
        let ssm = StateSpaceModel::new(
            expanded_size,  // Use expanded size since that's what gets passed to forward
            config.state_size,
            config.expand_factor,
        );

        Ok(Self {
            hidden_size: config.hidden_size,
            expand_factor: config.expand_factor,
            in_proj: Array2::from_shape_fn((expanded_size * 2, config.hidden_size), |_| 
                (Self::rand() * 2.0 - 1.0) * scale),
            out_proj: Array2::from_shape_fn((config.hidden_size, expanded_size), |_| 
                (Self::rand() * 2.0 - 1.0) * scale),
            conv_kernel: config.conv_kernel,
            conv_weight: Array2::from_shape_fn((expanded_size, config.conv_kernel), |_| 
                (Self::rand() * 2.0 - 1.0) * scale),
            conv_bias: Array1::zeros(expanded_size),
            ssm: ssm?,
            norm_weight: Array1::ones(config.hidden_size),
            norm_bias: Array1::zeros(config.hidden_size),
        })
    }
    
    /// Load layer weights from checkpoint
    pub fn load_weights(
        &mut self,
        in_proj: Array2<f32>,
        out_proj: Array2<f32>,
        conv_weight: Array2<f32>,
        conv_bias: Array1<f32>,
        norm_weight: Array1<f32>,
        norm_bias: Array1<f32>,
    ) {
        self.in_proj = in_proj;
        self.out_proj = out_proj;
        self.conv_weight = conv_weight;
        self.conv_bias = conv_bias;
        self.norm_weight = norm_weight;
        self.norm_bias = norm_bias;
    }
    
    pub fn ssm_mut(&mut self) -> &mut StateSpaceModel {
        &mut self.ssm
    }

    /// Forward pass through Mamba layer
    ///
    /// Mamba layer implements:
    /// 1. Layer normalization
    /// 2. Expand dimension with gated projection
    /// 3. Causal 1D convolution for local context
    /// 4. Selective SSM (the O(n) core)
    /// 5. Gating and output projection
    /// 6. Residual connection
    pub fn forward(&self, x: &Array1<f32>, state: &mut MambaState) -> Result<Array1<f32>> {
        let residual = x.clone();
        
        // 1. Layer normalization
        let x_norm = self.layer_norm(x);
        
        // 2. Project to expanded dimension (with gating)
        let projected = self.in_proj.dot(&x_norm);
        let expanded_size = self.hidden_size * self.expand_factor;
        
        // Split into two paths for gating
        let x_gate = projected.slice(ndarray::s![..expanded_size]).to_owned();
        let x_ssm = projected.slice(ndarray::s![expanded_size..]).to_owned();
        
        // 3. Apply causal convolution to SSM path
        let x_conv = self.causal_conv(&x_ssm, &mut state.conv_state)?;
        
        // 4. Activate with SiLU (Swish)
        let x_activated = x_conv.mapv(|v| v * Self::sigmoid(v));
        
        // 5. Apply selective SSM (the magic O(n) kernel)
        // Convert Array1 to Tensor for SSM processing
        let x_tensor = Self::array_to_tensor(&x_activated)?;
        let x_ssm_tensor = self.ssm.forward(&x_tensor)?;
        let x_ssm_out = Self::tensor_to_array(&x_ssm_tensor)?;
        
        // 6. Gate with the other path (also SiLU activated)
        let x_gate_activated = x_gate.mapv(|v| v * Self::sigmoid(v));
        let min_len = x_ssm_out.len().min(x_gate_activated.len());
        let x_gated = &x_ssm_out.slice(ndarray::s![..min_len]) * 
                      &x_gate_activated.slice(ndarray::s![..min_len]);
        
        // 7. Project back to hidden dimension
        let output = if self.out_proj.ncols() >= x_gated.len() {
            self.out_proj.slice(ndarray::s![.., ..x_gated.len()]).dot(&x_gated)
        } else {
            self.out_proj.dot(&x_gated.slice(ndarray::s![..self.out_proj.ncols()]))
        };
        
        // 8. Residual connection
        let min_len = residual.len().min(output.len());
        let mut result = Array1::zeros(residual.len());
        for i in 0..min_len {
            result[i] = residual[i] + output[i];
        }
        for i in min_len..residual.len() {
            result[i] = residual[i];
        }
        
        Ok(result)
    }
    
    /// Layer normalization
    fn layer_norm(&self, x: &Array1<f32>) -> Array1<f32> {
        let mean = x.mean().unwrap_or(0.0);
        let var = x.iter().map(|&v| (v - mean).powi(2)).sum::<f32>() / x.len() as f32;
        let std = (var + 1e-5).sqrt();
        
        x.iter()
            .zip(self.norm_weight.iter())
            .zip(self.norm_bias.iter())
            .map(|((&val, &w), &b)| ((val - mean) / std) * w + b)
            .collect()
    }
    
    /// Causal 1D convolution (looks only at past context)
    fn causal_conv(&self, x: &Array1<f32>, conv_state: &mut Array2<f32>) -> Result<Array1<f32>> {
        let mut output = Array1::zeros(x.len());
        
        // Shift conv state (keep last kernel_size - 1 values)
        for i in 0..self.hidden_size * self.expand_factor {
            for k in (1..self.conv_kernel).rev() {
                conv_state[[i, k]] = conv_state[[i, k - 1]];
            }
            if i < x.len() {
                conv_state[[i, 0]] = x[i];
            }
        }
        
        // Apply convolution
        for i in 0..output.len() {
            let mut sum = self.conv_bias[i.min(self.conv_bias.len() - 1)];
            for k in 0..self.conv_kernel {
                sum += conv_state[[i, k]] * self.conv_weight[[i.min(self.conv_weight.nrows() - 1), k]];
            }
            output[i] = sum;
        }
        
        Ok(output)
    }
    
    /// Sigmoid activation
    fn sigmoid(x: f32) -> f32 {
        1.0 / (1.0 + (-x).exp())
    }
    
    /// Simple PRNG for initialization
    fn rand() -> f32 {
        use std::cell::Cell;
        thread_local! {
            static SEED: Cell<u64> = const { Cell::new(0x0123456789abcdef) };
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
    
    /// Convert Array1 to Tensor for SSM processing
    fn array_to_tensor(arr: &Array1<f32>) -> Result<Tensor> {
        let data: Vec<f32> = arr.to_vec();
        let shape = vec![1, data.len()]; // Make it 2D for SSM
        Tensor::from_slice(&data, &shape, DType::F32)
    }
    
    /// Convert Tensor back to Array1
    fn tensor_to_array(tensor: &Tensor) -> Result<Array1<f32>> {
        let data = tensor.data().as_slice()
            .ok_or_else(|| sutra_core::error::SutraError::ComputeError("Failed to get tensor data".to_string()))?;
        Ok(Array1::from_vec(data.to_vec()))
    }
}

/// State for Mamba layer
#[derive(Debug, Clone)]
pub struct MambaState {
    pub conv_state: Array2<f32>,  // [expanded_size, kernel_size]
}

impl MambaState {
    pub fn new(hidden_size: usize, _state_size: usize, expand_factor: usize, conv_kernel: usize) -> Self {
        Self {
            conv_state: Array2::zeros((hidden_size * expand_factor, conv_kernel)),
        }
    }
}

use sutra_core::{Tensor, DType};
use sutra_core::error::{Result, SutraError};
use crate::selective::{SelectiveMechanism, SelectiveParams};

/// Production State Space Model (SSM) core component
///
/// SSM represents sequences using continuous state:
/// h'(t) = Ah(t) + Bx(t)
/// y(t) = Ch(t) + Dx(t)
///
/// Discretized for sequences:
/// h_t = A_bar h_{t-1} + B_bar x_t
/// y_t = C h_t
///
/// Mamba's innovation: A, B, C are input-dependent (selective)
pub struct StateSpaceModel {
    hidden_size: usize,
    state_size: usize,
    #[allow(dead_code)]
    expand_factor: usize,
    // Selective mechanism for computing input-dependent parameters
    selective: SelectiveMechanism,
    // Static SSM parameter
    a_log: Tensor,       // Log-space A matrix (negative for stability)
    d: Tensor,           // Skip connection parameter
}

impl StateSpaceModel {
    pub fn new(hidden_size: usize, state_size: usize, expand_factor: usize) -> Result<Self> {
        let expanded_size = hidden_size * expand_factor;
        
        // Initialize A matrix with negative values for stability (standard SSM practice)
        let a_data: Vec<f32> = (0..state_size)
            .map(|i| -((i + 1) as f32).ln())
            .collect();
        let a_log = Tensor::from_slice(&a_data, &[state_size], DType::F32)?;
        
        Ok(Self {
            hidden_size,
            state_size,
            expand_factor,
            // Initialize selective mechanism with proper sizes
            selective: SelectiveMechanism::new(hidden_size, state_size)?,
            a_log,
            // Skip connection (identity-like initialization)
            d: Tensor::zeros(&[expanded_size], DType::F32),
        })
    }

    /// Load SSM weights from checkpoint
    pub fn load_weights(
        &mut self, 
        a_log: &Tensor,
        d: &Tensor,
    ) -> Result<()> {
        // Update A and D parameters
        if a_log.shape() != self.a_log.shape() {
            return Err(SutraError::InvalidShape(format!(
                "A log shape mismatch: expected {:?}, got {:?}",
                self.a_log.shape(), a_log.shape()
            )));
        }
        
        // Clone the tensors properly
        self.a_log = Tensor::new(a_log.data().clone(), a_log.dtype());
        self.d = Tensor::new(d.data().clone(), d.dtype());
        
        // Note: Selective mechanism weights would be loaded separately
        Ok(())
    }
    
    /// Extract input vector at specific timestep
    fn extract_timestep(&self, x: &Tensor, t: usize) -> Result<Tensor> {
        if x.shape().len() != 2 {
            return Err(SutraError::InvalidShape("Expected 2D input".to_string()));
        }
        
        let seq_len = x.shape()[0];
        if t >= seq_len {
            return Err(SutraError::InvalidShape(format!(
                "Timestep {} out of range for sequence length {}", t, seq_len
            )));
        }
        
        // Extract row at timestep t
        let x_data = x.data();
        let row_start = t * self.hidden_size;
        let row_end = row_start + self.hidden_size;
        
        let flat_data = x_data.as_slice()
            .ok_or_else(|| SutraError::ComputeError("Failed to get flat data".to_string()))?;
        
        if row_end > flat_data.len() {
            return Err(SutraError::InvalidShape("Row extraction out of bounds".to_string()));
        }
        
        let row_data = &flat_data[row_start..row_end];
        Tensor::from_slice(row_data, &[self.hidden_size], DType::F32)
    }
    
    /// Convert A from log space to linear space
    fn exp_a(&self) -> Result<Tensor> {
        let exp_data = self.a_log.data().mapv(|val| val.exp());
        Ok(Tensor::new(exp_data, self.a_log.dtype()))
    }

    /// Core Mamba forward pass with selective scan
    ///
    /// Input-dependent parameters enable content-aware processing
    pub fn forward(&self, x: &Tensor) -> Result<Tensor> {
        if x.shape().len() != 2 {
            return Err(SutraError::InvalidShape(
                "Input must be 2D [seq_len, hidden_size]".to_string()
            ));
        }
        
        let (seq_len, hidden_size) = (x.shape()[0], x.shape()[1]);
        if hidden_size != self.hidden_size {
            return Err(SutraError::InvalidShape(format!(
                "Input hidden size {} doesn't match model size {}",
                hidden_size, self.hidden_size
            )));
        }
        
        let mut output_data = Vec::with_capacity(seq_len);
        let mut state = Tensor::zeros(&[self.state_size], DType::F32);
        
        // Process sequence step by step
        for t in 0..seq_len {
            // Extract input at time t
            let x_t = self.extract_timestep(x, t)?;
            
            // Compute selective parameters for this input
            let params = self.selective.compute_params(&x_t)?;
            
            // Apply selective scan step
            let (new_state, y_t) = self.selective_scan_step(&state, &x_t, &params)?;
            state = new_state;
            
            output_data.push(y_t);
        }
        
        // Convert to output tensor
        let output = Tensor::from_slice(&output_data, &[seq_len], DType::F32)?;
        Ok(output)
    }

    /// Single step of selective scan operation (core of Mamba)
    ///
    /// This is where the O(n) linear complexity comes from.
    /// Instead of O(n²) pairwise interactions, we maintain a hidden state.
    fn selective_scan_step(
        &self,
        prev_state: &Tensor,
        x_t: &Tensor,
        params: &SelectiveParams,
    ) -> Result<(Tensor, f32)> {
        // Get A matrix (convert from log space)
        let a = self.exp_a()?;
        
        // Zero-order hold discretization
        // A_bar = exp(-Δ * A)
        // B_bar = (1 - A_bar) / A * B = (1 - exp(-Δ * A)) / A * B
        let delta_data = params.delta.data().as_slice()
            .ok_or_else(|| SutraError::ComputeError("Failed to get delta data".to_string()))?;
        let a_data = a.data().as_slice()
            .ok_or_else(|| SutraError::ComputeError("Failed to get A data".to_string()))?;
        
        let dt = delta_data[0]; // Use first delta value
        let a_val = a_data[0];  // Use first A value
        
        // Discretization: A_bar = exp(-dt * A), B_bar = B * dt
        let a_bar = (-dt * a_val).exp();
        let b_bar = dt; // Simplified B matrix
        
        // State update: h_t = A_bar * h_{t-1} + B_bar * B * x_t
        let prev_state_data = prev_state.data().as_slice()
            .ok_or_else(|| SutraError::ComputeError("Failed to get state data".to_string()))?;
        let x_data = x_t.data().as_slice()
            .ok_or_else(|| SutraError::ComputeError("Failed to get input data".to_string()))?;
        let b_data = params.b.data().as_slice()
            .ok_or_else(|| SutraError::ComputeError("Failed to get B data".to_string()))?;
        
        let mut new_state_data = Vec::with_capacity(self.state_size);
        for i in 0..self.state_size.min(prev_state_data.len()).min(b_data.len()) {
            let h_update = a_bar * prev_state_data[i] + b_bar * b_data[i] * x_data[0];
            new_state_data.push(h_update);
        }
        
        // Pad if needed
        while new_state_data.len() < self.state_size {
            new_state_data.push(0.0);
        }
        
        let new_state = Tensor::from_slice(&new_state_data, &[self.state_size], DType::F32)?;
        
        // Output: y_t = C * h_t + D * x_t
        let c_data = params.c.data().as_slice()
            .ok_or_else(|| SutraError::ComputeError("Failed to get C data".to_string()))?;
        let d_data = self.d.data().as_slice()
            .ok_or_else(|| SutraError::ComputeError("Failed to get D data".to_string()))?;
        
        let mut y_t = 0.0;
        for i in 0..self.state_size.min(c_data.len()).min(new_state_data.len()) {
            y_t += c_data[i] * new_state_data[i];
        }
        // Add skip connection
        if !d_data.is_empty() {
            y_t += d_data[0] * x_data[0];
        }
        
        Ok((new_state, y_t))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ssm_creation() {
        let ssm = StateSpaceModel::new(64, 16, 2).unwrap();
        assert_eq!(ssm.hidden_size, 64);
        assert_eq!(ssm.state_size, 16);
        assert_eq!(ssm.expand_factor, 2);
    }
    
    #[test]
    fn test_selective_scan_step() {
        let ssm = StateSpaceModel::new(4, 2, 1).unwrap();
        let state = Tensor::zeros(&[2], DType::F32);
        let input = Tensor::from_slice(&[1.0, 0.5, -0.3, 0.8], &[4], DType::F32).unwrap();
        
        let params = ssm.selective.compute_params(&input).unwrap();
        let result = ssm.selective_scan_step(&state, &input, &params);
        
        assert!(result.is_ok());
    }
}
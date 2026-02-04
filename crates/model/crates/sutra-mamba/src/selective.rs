use sutra_core::{Tensor, DType, ops};
use sutra_core::error::{Result, SutraError};

/// Production-grade selective mechanism for Mamba
///
/// The "selection" in Mamba means the SSM parameters (Δ, B, C)
/// are computed as functions of the input using learned linear projections.
/// This enables content-aware state space processing.
pub struct SelectiveMechanism {
    hidden_size: usize,
    // Learned projection weights
    delta_proj: Tensor,  // [hidden_size, hidden_size]
    b_proj: Tensor,      // [hidden_size, state_size] 
    c_proj: Tensor,      // [hidden_size, state_size]
    // Biases
    delta_bias: Tensor,  // [hidden_size]
    b_bias: Tensor,      // [state_size]
    c_bias: Tensor,      // [state_size]
    state_size: usize,
}

impl SelectiveMechanism {
    /// Create new selective mechanism with initialized weights
    pub fn new(hidden_size: usize, state_size: usize) -> Result<Self> {
        // Xavier/Glorot initialization for projection weights
        let std = (2.0 / (hidden_size + state_size) as f32).sqrt();
        
        Ok(Self {
            hidden_size,
            state_size,
            // Initialize projection matrices with proper scaling
            delta_proj: Tensor::randn(&[hidden_size, hidden_size], DType::F32)?
                .scale(std)?,
            b_proj: Tensor::randn(&[hidden_size, state_size], DType::F32)?
                .scale(std)?,
            c_proj: Tensor::randn(&[hidden_size, state_size], DType::F32)?
                .scale(std)?,
            // Initialize biases
            delta_bias: Tensor::zeros(&[hidden_size], DType::F32),
            b_bias: Tensor::zeros(&[state_size], DType::F32),
            c_bias: Tensor::zeros(&[state_size], DType::F32),
        })
    }

    /// Compute input-dependent selective parameters
    ///
    /// Δ = softplus(x @ W_Δ + b_Δ)  - time scale (positive)
    /// B = x @ W_B + b_B            - input matrix  
    /// C = x @ W_C + b_C            - output matrix
    pub fn compute_params(&self, x: &Tensor) -> Result<SelectiveParams> {
        if x.shape().len() != 1 || x.shape()[0] != self.hidden_size {
            return Err(SutraError::InvalidShape(format!(
                "Input must be 1D with size {}, got shape {:?}",
                self.hidden_size, x.shape()
            )));
        }

        // Reshape input for matrix multiplication [1, hidden_size]
        let x_2d = x.reshape(&[1, self.hidden_size])?;
        
        // Δ = softplus(x @ W_Δ + b_Δ)
        let delta_linear = ops::matmul(&x_2d, &self.delta_proj)?;
        let delta_with_bias = ops::add(&delta_linear, &self.delta_bias.reshape(&[1, self.hidden_size])?)?;
        let delta = self.softplus(&delta_with_bias)?;
        
        // B = x @ W_B + b_B  
        let b_linear = ops::matmul(&x_2d, &self.b_proj)?;
        let b = ops::add(&b_linear, &self.b_bias.reshape(&[1, self.state_size])?)?;
        
        // C = x @ W_C + b_C
        let c_linear = ops::matmul(&x_2d, &self.c_proj)?;
        let c = ops::add(&c_linear, &self.c_bias.reshape(&[1, self.state_size])?)?;
        
        Ok(SelectiveParams {
            delta: delta.squeeze(0)?,
            b: b.squeeze(0)?,
            c: c.squeeze(0)?,
        })
    }
    
    /// Softplus activation: ln(1 + exp(x))
    /// More numerically stable than naive implementation
    fn softplus(&self, x: &Tensor) -> Result<Tensor> {
        // For numerical stability: softplus(x) = max(0, x) + ln(1 + exp(-|x|))
        let result = x.data().mapv(|v| {
            let abs_v = v.abs();
            abs_v.max(0.0) + (1.0 + (-abs_v).exp()).ln()
        });
        Ok(Tensor::new(result, x.dtype()))
    }
}

/// Input-dependent SSM parameters computed by selective mechanism
pub struct SelectiveParams {
    /// Time scale parameter (Δ) - controls discretization step size
    /// Must be positive, computed via softplus activation
    pub delta: Tensor,
    /// Input-to-state matrix (B) - projects input to state space
    pub b: Tensor,
    /// State-to-output matrix (C) - projects state to output
    pub c: Tensor,
}

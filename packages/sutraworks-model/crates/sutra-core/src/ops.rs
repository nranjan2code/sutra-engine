use crate::error::{Result, SutraError};
use crate::tensor::Tensor;
/// Core tensor operations for neural network computation
use ndarray::Axis;

/// Matrix multiplication: C = A @ B
pub fn matmul(a: &Tensor, b: &Tensor) -> Result<Tensor> {
    let a_shape = a.shape();
    let b_shape = b.shape();

    if a_shape.len() != 2 || b_shape.len() != 2 {
        return Err(SutraError::InvalidShape(
            "Matrix multiplication requires 2D tensors".to_string(),
        ));
    }

    if a_shape[1] != b_shape[0] {
        return Err(SutraError::InvalidShape(format!(
            "Incompatible shapes for matmul: {:?} @ {:?}",
            a_shape, b_shape
        )));
    }

    let a_data = a
        .data()
        .view()
        .into_dimensionality::<ndarray::Ix2>()
        .map_err(|e| SutraError::InvalidShape(e.to_string()))?;
    let b_data = b
        .data()
        .view()
        .into_dimensionality::<ndarray::Ix2>()
        .map_err(|e| SutraError::InvalidShape(e.to_string()))?;

    let result = a_data.dot(&b_data);
    let result_dyn = result.into_dyn();

    Ok(Tensor::new(result_dyn, a.dtype()))
}

/// Element-wise addition
pub fn add(a: &Tensor, b: &Tensor) -> Result<Tensor> {
    if a.shape() != b.shape() {
        return Err(SutraError::InvalidShape(format!(
            "Shapes must match for addition: {:?} + {:?}",
            a.shape(),
            b.shape()
        )));
    }

    let result = a.data() + b.data();
    Ok(Tensor::new(result, a.dtype()))
}

/// Element-wise multiplication
pub fn mul(a: &Tensor, b: &Tensor) -> Result<Tensor> {
    if a.shape() != b.shape() {
        return Err(SutraError::InvalidShape(format!(
            "Shapes must match for multiplication: {:?} * {:?}",
            a.shape(),
            b.shape()
        )));
    }

    let result = a.data() * b.data();
    Ok(Tensor::new(result, a.dtype()))
}

/// Activation functions
pub mod activations {
    use super::*;

    /// ReLU activation: max(0, x)
    pub fn relu(x: &Tensor) -> Tensor {
        let result = x.data().mapv(|v| v.max(0.0));
        Tensor::new(result, x.dtype())
    }

    /// GELU activation (Gaussian Error Linear Unit)
    /// Approximation: 0.5 * x * (1 + tanh(sqrt(2/π) * (x + 0.044715 * x^3)))
    pub fn gelu(x: &Tensor) -> Tensor {
        let sqrt_2_over_pi = (2.0_f32 / std::f32::consts::PI).sqrt();
        let result = x.data().mapv(|v| {
            let inner = sqrt_2_over_pi * (v + 0.044715 * v.powi(3));
            0.5 * v * (1.0 + inner.tanh())
        });
        Tensor::new(result, x.dtype())
    }

    /// Sigmoid activation: 1 / (1 + exp(-x))
    pub fn sigmoid(x: &Tensor) -> Tensor {
        let result = x.data().mapv(|v| 1.0 / (1.0 + (-v).exp()));
        Tensor::new(result, x.dtype())
    }

    /// Tanh activation
    pub fn tanh(x: &Tensor) -> Tensor {
        let result = x.data().mapv(|v| v.tanh());
        Tensor::new(result, x.dtype())
    }

    /// SiLU/Swish activation: x * sigmoid(x)
    pub fn silu(x: &Tensor) -> Tensor {
        let result = x.data().mapv(|v| v / (1.0 + (-v).exp()));
        Tensor::new(result, x.dtype())
    }

    /// Softmax activation along last dimension
    pub fn softmax(x: &Tensor) -> Result<Tensor> {
        let data = x.data();
        let shape = data.shape();

        if shape.is_empty() {
            return Err(SutraError::InvalidShape("Empty tensor".to_string()));
        }

        // Compute softmax along last axis
        let mut result = data.clone();
        let last_axis = shape.len() - 1;

        for mut lane in result.axis_iter_mut(Axis(last_axis)) {
            // Subtract max for numerical stability
            let max_val = lane.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            lane.mapv_inplace(|v| (v - max_val).exp());
            let sum: f32 = lane.sum();
            lane.mapv_inplace(|v| v / sum);
        }

        Ok(Tensor::new(result, x.dtype()))
    }
}

/// Layer normalization
pub fn layer_norm(x: &Tensor, eps: f32) -> Result<Tensor> {
    let data = x.data();
    let shape = data.shape();

    if shape.is_empty() {
        return Err(SutraError::InvalidShape("Empty tensor".to_string()));
    }

    let mut result = data.clone();
    let last_axis = shape.len() - 1;

    // Normalize along last dimension
    for mut lane in result.axis_iter_mut(Axis(last_axis)) {
        let mean: f32 = lane.mean().unwrap_or(0.0);
        let variance: f32 =
            lane.iter().map(|&v| (v - mean).powi(2)).sum::<f32>() / lane.len() as f32;
        let std = (variance + eps).sqrt();

        lane.mapv_inplace(|v| (v - mean) / std);
    }

    Ok(Tensor::new(result, x.dtype()))
}

/// RMS (Root Mean Square) normalization - used in modern LLMs
pub fn rms_norm(x: &Tensor, eps: f32) -> Result<Tensor> {
    let data = x.data();
    let shape = data.shape();

    if shape.is_empty() {
        return Err(SutraError::InvalidShape("Empty tensor".to_string()));
    }

    let mut result = data.clone();
    let last_axis = shape.len() - 1;

    // Normalize along last dimension
    for mut lane in result.axis_iter_mut(Axis(last_axis)) {
        let rms: f32 = (lane.iter().map(|&v| v * v).sum::<f32>() / lane.len() as f32).sqrt();
        lane.mapv_inplace(|v| v / (rms + eps));
    }

    Ok(Tensor::new(result, x.dtype()))
}

/// Embedding lookup
pub fn embedding(indices: &[usize], weights: &Tensor) -> Result<Tensor> {
    let weight_shape = weights.shape();

    if weight_shape.len() != 2 {
        return Err(SutraError::InvalidShape(
            "Embedding weights must be 2D [vocab_size, embedding_dim]".to_string(),
        ));
    }

    let (vocab_size, embed_dim) = (weight_shape[0], weight_shape[1]);
    let weight_data = weights
        .data()
        .view()
        .into_dimensionality::<ndarray::Ix2>()
        .map_err(|e| SutraError::InvalidShape(e.to_string()))?;

    // Verify indices are in range
    for &idx in indices {
        if idx >= vocab_size {
            return Err(SutraError::InvalidShape(format!(
                "Index {} out of range for vocab size {}",
                idx, vocab_size
            )));
        }
    }

    // Gather embeddings
    let mut result_data = Vec::with_capacity(indices.len() * embed_dim);
    for &idx in indices {
        let row = weight_data.row(idx);
        result_data.extend_from_slice(row.as_slice().unwrap());
    }

    let result =
        ndarray::Array::from_shape_vec(ndarray::IxDyn(&[indices.len(), embed_dim]), result_data)
            .map_err(|e| SutraError::InvalidShape(e.to_string()))?;

    Ok(Tensor::new(result, weights.dtype()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DType;

    #[test]
    fn test_matmul() {
        let a = Tensor::from_slice(&[1.0, 2.0, 3.0, 4.0], &[2, 2], DType::F32).unwrap();
        let b = Tensor::from_slice(&[5.0, 6.0, 7.0, 8.0], &[2, 2], DType::F32).unwrap();

        let c = matmul(&a, &b).unwrap();
        assert_eq!(c.shape(), &[2, 2]);

        // [1 2]   [5 6]   [19 22]
        // [3 4] @ [7 8] = [43 50]
        let data = c.data().as_slice().unwrap();
        assert_eq!(data[0], 19.0);
        assert_eq!(data[1], 22.0);
        assert_eq!(data[2], 43.0);
        assert_eq!(data[3], 50.0);
    }

    #[test]
    fn test_relu() {
        let x = Tensor::from_slice(&[-1.0, 0.0, 1.0, 2.0], &[4], DType::F32).unwrap();
        let y = activations::relu(&x);

        let data = y.data().as_slice().unwrap();
        assert_eq!(data, &[0.0, 0.0, 1.0, 2.0]);
    }

    #[test]
    fn test_layer_norm() {
        let x = Tensor::from_slice(&[1.0, 2.0, 3.0, 4.0], &[2, 2], DType::F32).unwrap();
        let y = layer_norm(&x, 1e-5).unwrap();

        // After normalization, each row should have mean≈0, std≈1
        assert_eq!(y.shape(), &[2, 2]);
    }

    #[test]
    fn test_embedding() {
        // Vocab size 5, embedding dim 3
        let weights = Tensor::from_slice(
            &[
                1.0, 2.0, 3.0, // word 0
                4.0, 5.0, 6.0, // word 1
                7.0, 8.0, 9.0, // word 2
                10.0, 11.0, 12.0, // word 3
                13.0, 14.0, 15.0, // word 4
            ],
            &[5, 3],
            DType::F32,
        )
        .unwrap();

        let indices = vec![0, 2, 1];
        let embedded = embedding(&indices, &weights).unwrap();

        assert_eq!(embedded.shape(), &[3, 3]);
        let data = embedded.data().as_slice().unwrap();
        assert_eq!(&data[0..3], &[1.0, 2.0, 3.0]); // word 0
        assert_eq!(&data[3..6], &[7.0, 8.0, 9.0]); // word 2
        assert_eq!(&data[6..9], &[4.0, 5.0, 6.0]); // word 1
    }
}

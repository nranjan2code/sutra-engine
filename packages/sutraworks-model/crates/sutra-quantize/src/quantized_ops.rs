//! Quantized operations for efficient inference
//!
//! These operations work directly on quantized tensors,
//! dequantizing on-the-fly for computation
use crate::awq::QuantizedWeights;
use ndarray::{Array1, Array2};
use sutra_core::{Result, SutraError, Tensor};

/// Quantized matrix multiplication
/// 
/// Performs A @ B where B is quantized.
/// Dequantizes B on-the-fly in a memory-efficient manner.
pub fn quantized_matmul(a: &Tensor, b_quantized: &QuantizedWeights) -> Result<Tensor> {
    let a_shape = a.shape();
    
    if a_shape.len() != 2 {
        return Err(SutraError::InvalidShape(
            "Input must be 2D for quantized matmul".to_string(),
        ));
    }
    
    let (m, k) = (a_shape[0], a_shape[1]);
    let (k_b, n) = (b_quantized.shape[0], b_quantized.shape[1]);
    
    if k != k_b {
        return Err(SutraError::InvalidShape(format!(
            "Incompatible shapes for matmul: [{}, {}] @ [{}, {}]",
            m, k, k_b, n
        )));
    }
    
    // Get input as 2D array
    let a_data = a.data()
        .view()
        .into_dimensionality::<ndarray::Ix2>()
        .map_err(|e| SutraError::InvalidShape(e.to_string()))?;
    
    // Allocate output
    let mut output = Array2::zeros((m, n));
    
    // Dequantize B column-wise and compute
    // This is more memory-efficient than dequantizing the entire matrix
    let group_size = b_quantized.group_size;
    let n_groups = n.div_ceil(group_size);
    
    for col_idx in 0..n {
        // Dequantize one column of B
        let mut b_col = Vec::with_capacity(k);
        
        for row_idx in 0..k {
            // Groups are along the column dimension (in_features)
            let group_idx = col_idx / group_size;
            let scale_idx = row_idx * n_groups + group_idx;
            let scale = b_quantized.scales[scale_idx.min(b_quantized.scales.len() - 1)];
            let zero = b_quantized.zeros
                .as_ref()
                .map(|z| z[scale_idx.min(z.len() - 1)] as i8 as f32) // Interpret u8 as i8
                .unwrap_or(0.0);
            
            // Unpack quantized value (row-major packing: row0[all_cols], row1[all_cols], ...)
            let value_idx = row_idx * n + col_idx;
            let byte_idx = value_idx / 2;
            let is_high_nibble = value_idx % 2 == 1;
            
            let qval = if byte_idx < b_quantized.qweights.len() {
                if is_high_nibble {
                    (b_quantized.qweights[byte_idx] >> 4) & 0x0F
                } else {
                    b_quantized.qweights[byte_idx] & 0x0F
                }
            } else {
                0
            };
            
            let dequant = (qval as f32 - zero) * scale;
            b_col.push(dequant);
        }
        
        // Compute A @ b_col and store in output column
        for i in 0..m {
            let mut sum = 0.0;
            for j in 0..k {
                sum += a_data[[i, j]] * b_col[j];
            }
            output[[i, col_idx]] = sum;
        }
    }
    
    Ok(Tensor::new(output.into_dyn(), a.dtype()))
}

/// Quantized linear layer: y = Wx + b
/// 
/// Where W is quantized for memory efficiency
pub fn quantized_linear(
    x: &Tensor,
    weight_quantized: &QuantizedWeights,
    bias: Option<&Array1<f32>>,
) -> Result<Tensor> {
    // Compute Wx
    let mut output = quantized_matmul(x, weight_quantized)?;
    
    // Add bias if present
    if let Some(b) = bias {
        let output_data = output.data_mut();
        let mut output_2d = output_data
            .view_mut()
            .into_dimensionality::<ndarray::Ix2>()
            .map_err(|e| SutraError::InvalidShape(e.to_string()))?;
        
        for mut row in output_2d.axis_iter_mut(ndarray::Axis(0)) {
            for (i, val) in row.iter_mut().enumerate() {
                if i < b.len() {
                    *val += b[i];
                }
            }
        }
    }
    
    Ok(output)
}

/// Calibrate quantization using activation statistics
/// 
/// This is the "activation-aware" part of AWQ
pub fn calibrate_quantization(
    weights: &Tensor,
    activations: &[Tensor],
) -> Result<Array1<f32>> {
    let weight_shape = weights.shape();
    
    if weight_shape.len() != 2 {
        return Err(SutraError::InvalidShape(
            "Weight matrix must be 2D".to_string(),
        ));
    }
    
    let in_features = weight_shape[1];
    let mut salience = Array1::zeros(in_features);
    
    // Compute activation magnitudes across calibration samples
    for act in activations {
        let act_data = act.data();
        let act_flat = act_data.view().into_dimensionality::<ndarray::Ix1>()
            .map_err(|e| SutraError::InvalidShape(e.to_string()))?;
        
        for i in 0..in_features.min(act_flat.len()) {
            salience[i] += act_flat[i].abs();
        }
    }
    
    // Normalize
    let n_samples = activations.len() as f32;
    if n_samples > 0.0 {
        salience.mapv_inplace(|x| x / n_samples);
    }
    
    Ok(salience)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::awq::{AwqConfig, AwqQuantizer};
    use ndarray::{Array, IxDyn};
    use sutra_core::DType;
    
    #[test]
    fn test_quantized_matmul() {
        // Create test matrices
        let a_data: Vec<f32> = (0..32).map(|i| i as f32 * 0.1).collect();
        let a_arr = Array::from_shape_vec(IxDyn(&[4, 8]), a_data).unwrap();
        let a = Tensor::new(a_arr, DType::F32);
        
        let b_data: Vec<f32> = (0..64).map(|i| (i as f32) * 0.05).collect();
        let b_arr = Array::from_shape_vec(IxDyn(&[8, 8]), b_data).unwrap();
        let b = Tensor::new(b_arr, DType::F32);
        
        // Quantize B
        let quantizer = AwqQuantizer::new(AwqConfig::default());
        let b_quantized = quantizer.quantize(&b, None).unwrap();
        
        // Perform quantized matmul
        let result = quantized_matmul(&a, &b_quantized).unwrap();
        
        assert_eq!(result.shape(), &[4, 8]);
    }
    
    #[test]
    fn test_calibrate_quantization() {
        let weight_data: Vec<f32> = (0..256).map(|i| i as f32 * 0.01).collect();
        let weight_arr = Array::from_shape_vec(IxDyn(&[16, 16]), weight_data).unwrap();
        let weights = Tensor::new(weight_arr, DType::F32);
        
        let act1_data: Vec<f32> = (0..16).map(|i| (i as f32) * 0.1).collect();
        let act1_arr = Array::from_shape_vec(IxDyn(&[16]), act1_data).unwrap();
        let act1 = Tensor::new(act1_arr, DType::F32);
        
        let salience = calibrate_quantization(&weights, &[act1]).unwrap();
        
        assert_eq!(salience.len(), 16);
        assert!(salience.iter().all(|&x| x >= 0.0));
    }
    
    #[test]
    fn test_quantized_matmul_accuracy() {
        use sutra_core::ops;
        
        // Create test matrices with known values
        let a_data: Vec<f32> = vec![
            1.0, 2.0, 3.0, 4.0,
            5.0, 6.0, 7.0, 8.0,
        ];
        let a_arr = Array::from_shape_vec(IxDyn(&[2, 4]), a_data).unwrap();
        let a = Tensor::new(a_arr, DType::F32);
        
        let b_data: Vec<f32> = vec![
            0.5, 1.5,
            2.0, 2.5,
            1.0, 0.5,
            3.0, 1.0,
        ];
        let b_arr = Array::from_shape_vec(IxDyn(&[4, 2]), b_data).unwrap();
        let b = Tensor::new(b_arr.clone(), DType::F32);
        
        // Compute f32 baseline
        let baseline = ops::matmul(&a, &b).unwrap();
        
        // Quantize B without salience weighting for testing
        let quantizer = AwqQuantizer::new(AwqConfig {
            bits: 4,
            group_size: 4, // Small group size for this test
            n_samples: 512,
            zero_point: true,
        });
        let b_quantized = quantizer.quantize(&b, None).unwrap();
        
        // Perform quantized matmul
        let result = quantized_matmul(&a, &b_quantized).unwrap();
        
        // Check shapes match
        assert_eq!(result.shape(), baseline.shape());
        
        // Check values are reasonably close (accounting for quantization error)
        let baseline_data = baseline.data().as_slice().unwrap();
        let result_data = result.data().as_slice().unwrap();
        
        for (i, (&expected, &actual)) in baseline_data.iter().zip(result_data.iter()).enumerate() {
            let error = (expected - actual).abs();
            let relative_error = if expected.abs() > 1e-5 {
                error / expected.abs()
            } else {
                error
            };
            
            // Allow up to 25% relative error due to 4-bit quantization with AWQ salience scaling
            assert!(
                relative_error < 0.25 || error < 2.0,
                "Value {} mismatch at index {}: expected {}, got {} (error: {}, relative: {})",
                i, i, expected, actual, error, relative_error
            );
        }
    }
}

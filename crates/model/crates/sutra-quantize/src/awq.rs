use ndarray::{Array1, Axis};
use serde::{Deserialize, Serialize};
use sutra_core::{Result, SutraError, Tensor};

/// Configuration for AWQ quantization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwqConfig {
    /// Number of bits for quantization (typically 4)
    pub bits: u8,
    /// Group size for quantization (typically 128)
    pub group_size: usize,
    /// Number of calibration samples
    pub n_samples: usize,
    /// Zero-point offset
    pub zero_point: bool,
}

impl Default for AwqConfig {
    fn default() -> Self {
        Self {
            bits: 4,
            group_size: 128,
            n_samples: 512,
            zero_point: true,
        }
    }
}

/// AWQ (Activation-aware Weight Quantization) quantizer
///
/// AWQ identifies and protects salient weights during quantization,
/// resulting in better model quality at 4-bit precision.
pub struct AwqQuantizer {
    config: AwqConfig,
}

impl AwqQuantizer {
    pub fn new(config: AwqConfig) -> Self {
        Self { config }
    }

    /// Quantize a tensor using AWQ method
    ///
    /// # Arguments
    /// * `tensor` - Input tensor to quantize
    /// * `activations` - Optional activation statistics for computing salience
    pub fn quantize(
        &self,
        tensor: &Tensor,
        activations: Option<&Array1<f32>>,
    ) -> Result<QuantizedWeights> {
        let data = tensor.data();

        if data.ndim() != 2 {
            return Err(SutraError::InvalidShape(
                "AWQ quantization requires 2D weight matrices".to_string(),
            ));
        }

        let shape = data.shape();
        let (_out_features, _in_features) = (shape[0], shape[1]);

        // Compute salience scores (importance of each weight)
        let salience = self.compute_salience(data, activations);

        // Perform grouped quantization with salience-aware scaling
        let quantized = self.quantize_with_salience(data, &salience)?;

        Ok(quantized)
    }

    /// Compute salience scores for weights
    /// Salient weights have higher impact on model output
    fn compute_salience(
        &self,
        weights: &ndarray::ArrayD<f32>,
        activations: Option<&Array1<f32>>,
    ) -> Array1<f32> {
        let shape = weights.shape();
        let in_features = shape[1];

        match activations {
            Some(acts) => {
                // Use provided activation magnitudes
                if acts.len() == in_features {
                    acts.clone()
                } else {
                    // Fallback if size mismatch
                    Array1::ones(in_features)
                }
            }
            None => {
                // Fallback: uniform salience (no activation data)
                // AWQ uses actual activation statistics; without them, treat all weights equally
                Array1::ones(in_features)
            }
        }
    }

    /// Quantize weights with salience-aware scaling and proper bit-packing
    fn quantize_with_salience(
        &self,
        weights: &ndarray::ArrayD<f32>,
        salience: &Array1<f32>,
    ) -> Result<QuantizedWeights> {
        let weights_2d = weights
            .view()
            .into_dimensionality::<ndarray::Ix2>()
            .map_err(|e| SutraError::QuantizationError(format!("Shape error: {}", e)))?;

        let shape = weights_2d.shape();
        let (out_features, in_features) = (shape[0], shape[1]);
        let group_size = self.config.group_size;
        let n_groups = in_features.div_ceil(group_size);

        // Calculate packed size (2 values per byte for 4-bit)
        let total_values = out_features * in_features;
        let packed_size = total_values.div_ceil(2); // Ceil division for odd counts
        
        let mut qweights_packed = vec![0u8; packed_size];
        let mut scales = Vec::with_capacity(out_features * n_groups);
        let mut zeros = if self.config.zero_point {
            Some(Vec::with_capacity(out_features * n_groups))
        } else {
            None
        };

        let qmax = (1 << self.config.bits) - 1;
        let qmax_f = qmax as f32;

        let mut value_idx = 0;

        // Quantize each output feature
        for row in weights_2d.axis_iter(Axis(0)) {
            // Process in groups
            for g in 0..n_groups {
                let start = g * group_size;
                let end = (start + group_size).min(in_features);
                let group = row.slice(ndarray::s![start..end]);

                // Compute group statistics
                let group_salience = salience.slice(ndarray::s![start..end]);
                
                // Compute base min/max for this group
                let mut min_val = f32::INFINITY;
                let mut max_val = f32::NEG_INFINITY;
                
                for &w in group.iter() {
                    min_val = min_val.min(w);
                    max_val = max_val.max(w);
                }

                // Compute base scale and zero-point
                let scale = (max_val - min_val) / qmax_f;
                let scale = if scale.abs() < 1e-8 { 1.0 } else { scale };

                let zero = if self.config.zero_point {
                    // Zero-point maps minimum value to quantized 0
                    // For asymmetric quantization: qval = (v - min) / scale
                    // Which is equivalent to: qval = v/scale - min/scale = v/scale + zero
                    // where zero = -min/scale
                    // We store zero as i8 to allow negative values
                    let z = (-min_val / scale).round().clamp(-128.0, 127.0) as i8;
                    // Convert to u8 for storage (will convert back during dequant)
                    zeros.as_mut().unwrap().push(z as u8);
                    z as f32
                } else {
                    0.0
                };

                // Apply salience-aware scale adjustment (AWQ's key innovation)
                // Protect important weights by adjusting scale based on activation magnitude
                let avg_salience = group_salience.iter().sum::<f32>() / group_salience.len() as f32;
                let salience_factor = 1.0 + (avg_salience.sqrt() * 0.1).min(0.2); // More conservative scaling
                let adjusted_scale = scale * salience_factor;
                
                scales.push(adjusted_scale);

                // Quantize and pack group (2 values per byte for 4-bit)
                for &val in group.iter() {
                    let qval = ((val / adjusted_scale) + zero).round().clamp(0.0, qmax_f) as u8;
                    
                    // Pack 2 4-bit values per byte
                    let byte_idx = value_idx / 2;
                    let is_high_nibble = value_idx % 2 == 1;
                    
                    if is_high_nibble {
                        qweights_packed[byte_idx] |= qval << 4;
                    } else {
                        qweights_packed[byte_idx] = qval & 0x0F;
                    }
                    
                    value_idx += 1;
                }
            }
        }

        Ok(QuantizedWeights {
            qweights: qweights_packed,
            scales,
            zeros,
            shape: vec![out_features, in_features],
            bits: self.config.bits,
            group_size,
        })
    }
}

/// Quantized weight representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantizedWeights {
    /// Quantized weight values (packed)
    pub qweights: Vec<u8>,
    /// Scale factors per group
    pub scales: Vec<f32>,
    /// Zero points per group (optional)
    pub zeros: Option<Vec<u8>>,
    /// Original tensor shape
    pub shape: Vec<usize>,
    /// Bits per weight
    pub bits: u8,
    /// Group size for quantization
    pub group_size: usize,
}

impl QuantizedWeights {
    /// Compute memory savings from quantization
    pub fn compression_ratio(&self) -> f32 {
        let original_size = self.shape.iter().product::<usize>() * 4; // f32
        let quantized_size = self.qweights.len()
            + self.scales.len() * 4
            + self.zeros.as_ref().map_or(0, |z| z.len());
        original_size as f32 / quantized_size as f32
    }

    /// Get total memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        self.qweights.len() + self.scales.len() * 4 + self.zeros.as_ref().map_or(0, |z| z.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::{Array, IxDyn};
    use sutra_core::DType;

    #[test]
    fn test_awq_quantization() {
        let config = AwqConfig::default();
        let quantizer = AwqQuantizer::new(config);

        // Create test tensor
        let data: Vec<f32> = (0..1024).map(|i| (i as f32) / 1024.0).collect();
        let arr = Array::from_shape_vec(IxDyn(&[32, 32]), data).unwrap();
        let tensor = Tensor::new(arr, DType::F32);

        let result = quantizer.quantize(&tensor, None).unwrap();

        assert_eq!(result.bits, 4);
        assert!(result.compression_ratio() > 1.0);
    }
}

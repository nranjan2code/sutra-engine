use crate::awq::QuantizedWeights;
use crate::quantizer::QuantizedTensor;
use ndarray::{Array, IxDyn};
use sutra_core::{DType, Result, SutraError, Tensor};

/// Dequantizer for converting quantized tensors back to full precision
pub struct Dequantizer;

impl Dequantizer {
    /// Dequantize a QuantizedTensor back to f32
    pub fn dequantize(&self, qtensor: &QuantizedTensor) -> Result<Tensor> {
        let total_elements: usize = qtensor.shape.iter().product();
        let mut output = Vec::with_capacity(total_elements);

        let _qmax = (1u32 << qtensor.bits) - 1;

        for (i, &qval) in qtensor.data.iter().enumerate() {
            let scale_idx = i / (total_elements / qtensor.scales.len().max(1));
            let scale = qtensor.scales[scale_idx.min(qtensor.scales.len() - 1)];

            let zero_point = qtensor
                .zero_points
                .as_ref()
                .map(|zp| zp[scale_idx.min(zp.len() - 1)] as f32)
                .unwrap_or(0.0);

            let dequant = (qval as f32 - zero_point) * scale;
            output.push(dequant);
        }

        let arr = Array::from_shape_vec(IxDyn(&qtensor.shape), output)
            .map_err(|e| SutraError::InvalidShape(e.to_string()))?;

        Ok(Tensor::new(arr, DType::F32))
    }

    /// Dequantize AWQ quantized weights with proper bit-unpacking
    pub fn dequantize_awq(&self, qweights: &QuantizedWeights) -> Result<Tensor> {
        let (out_features, in_features) = (qweights.shape[0], qweights.shape[1]);
        let group_size = qweights.group_size;
        let n_groups = in_features.div_ceil(group_size);

        let mut output = Vec::with_capacity(out_features * in_features);
        let mut value_idx = 0;

        for row_idx in 0..out_features {
            for g in 0..n_groups {
                let start = g * group_size;
                let end = (start + group_size).min(in_features);
                let group_len = end - start;

                let scale_idx = row_idx * n_groups + g;
                let scale = qweights.scales[scale_idx];
                let zero = qweights
                    .zeros
                    .as_ref()
                    .map(|z| z[scale_idx] as f32)
                    .unwrap_or(0.0);

                // Dequantize group with bit-unpacking
                for _ in 0..group_len {
                    // Unpack 4-bit value from packed representation
                    let byte_idx = value_idx / 2;
                    let is_high_nibble = value_idx % 2 == 1;
                    
                    let qval = if is_high_nibble {
                        (qweights.qweights[byte_idx] >> 4) & 0x0F
                    } else {
                        qweights.qweights[byte_idx] & 0x0F
                    };
                    
                    let dequant = (qval as f32 - zero) * scale;
                    output.push(dequant);
                    value_idx += 1;
                }
            }
        }

        let arr = Array::from_shape_vec(IxDyn(&qweights.shape), output)
            .map_err(|e| SutraError::InvalidShape(e.to_string()))?;

        Ok(Tensor::new(arr, DType::F32))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::awq::{AwqConfig, AwqQuantizer};

    #[test]
    fn test_dequantization_round_trip() {
        let config = AwqConfig::default();
        let quantizer = AwqQuantizer::new(config);
        let dequantizer = Dequantizer;

        // Create test tensor
        let data: Vec<f32> = (0..256).map(|i| (i as f32) / 256.0).collect();
        let arr = Array::from_shape_vec(IxDyn(&[16, 16]), data.clone()).unwrap();
        let tensor = Tensor::new(arr, DType::F32);

        // Quantize and dequantize
        let quantized = quantizer.quantize(&tensor, None).unwrap();
        let restored = dequantizer.dequantize_awq(&quantized).unwrap();

        // Check shape preservation
        assert_eq!(restored.shape(), tensor.shape());
    }
}

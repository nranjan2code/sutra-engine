use serde::{Deserialize, Serialize};
use sutra_core::{DType, Result, Tensor};

/// Generic quantizer trait
pub trait Quantizer {
    fn quantize(&self, tensor: &Tensor) -> Result<QuantizedTensor>;
}

/// Container for quantized tensor data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantizedTensor {
    /// Quantized values
    pub data: Vec<u8>,
    /// Scale factors
    pub scales: Vec<f32>,
    /// Zero points (optional)
    pub zero_points: Option<Vec<i32>>,
    /// Original shape
    pub shape: Vec<usize>,
    /// Bit width
    pub bits: u8,
    /// Original dtype
    pub original_dtype: DType,
}

impl QuantizedTensor {
    pub fn new(
        data: Vec<u8>,
        scales: Vec<f32>,
        zero_points: Option<Vec<i32>>,
        shape: Vec<usize>,
        bits: u8,
    ) -> Self {
        Self {
            data,
            scales,
            zero_points,
            shape,
            bits,
            original_dtype: DType::F32,
        }
    }

    /// Get memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        self.data.len()
            + self.scales.len() * std::mem::size_of::<f32>()
            + self
                .zero_points
                .as_ref()
                .map_or(0, |zp| zp.len() * std::mem::size_of::<i32>())
    }

    /// Get compression ratio compared to original f32 tensor
    pub fn compression_ratio(&self) -> f32 {
        let original_size = self.shape.iter().product::<usize>() * std::mem::size_of::<f32>();
        original_size as f32 / self.memory_usage() as f32
    }
}

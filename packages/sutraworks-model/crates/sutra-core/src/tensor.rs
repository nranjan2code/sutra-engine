use crate::error::{Result, SutraError};
use ndarray::{Array, ArrayD, IxDyn};
use serde::{Deserialize, Serialize};

/// Supported data types for tensors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DType {
    F32,
    F16,
    I32,
    I8,
    U8,
    I4, // 4-bit integer for quantization
}

impl DType {
    pub fn size_bytes(&self) -> usize {
        match self {
            DType::F32 | DType::I32 => 4,
            DType::F16 => 2,
            DType::I8 | DType::U8 => 1,
            DType::I4 => 1, // Packed, but accounting at byte level
        }
    }
}

/// Multi-dimensional tensor with dynamic shape
#[derive(Clone)]
pub struct Tensor {
    data: ArrayD<f32>,
    dtype: DType,
    name: Option<String>,
}

impl Tensor {
    pub fn new(data: ArrayD<f32>, dtype: DType) -> Self {
        Self {
            data,
            dtype,
            name: None,
        }
    }

    pub fn zeros(shape: &[usize], dtype: DType) -> Self {
        let data = ArrayD::zeros(IxDyn(shape));
        Self::new(data, dtype)
    }

    pub fn randn(shape: &[usize], dtype: DType) -> Result<Self> {
        use rand_distr::{Normal, Distribution};
        
        let total_size: usize = shape.iter().product();
        let mut rng = rand::thread_rng();
        let normal = Normal::new(0.0, 1.0).map_err(|e| SutraError::ComputeError(e.to_string()))?;
        
        let data: Vec<f32> = (0..total_size)
            .map(|_| normal.sample(&mut rng))
            .collect();
            
        let arr = Array::from_shape_vec(IxDyn(shape), data)
            .map_err(|e| SutraError::InvalidShape(e.to_string()))?;
            
        Ok(Self::new(arr, dtype))
    }

    pub fn from_slice(data: &[f32], shape: &[usize], dtype: DType) -> Result<Self> {
        let total_size: usize = shape.iter().product();
        if data.len() != total_size {
            return Err(SutraError::InvalidShape(format!(
                "Data length {} doesn't match shape {:?}",
                data.len(),
                shape
            )));
        }

        let arr = Array::from_shape_vec(IxDyn(shape), data.to_vec())
            .map_err(|e| SutraError::InvalidShape(e.to_string()))?;

        Ok(Self::new(arr, dtype))
    }

    pub fn shape(&self) -> &[usize] {
        self.data.shape()
    }

    pub fn dtype(&self) -> DType {
        self.dtype
    }

    pub fn data(&self) -> &ArrayD<f32> {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut ArrayD<f32> {
        &mut self.data
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        self.data.len() * self.dtype.size_bytes()
    }

    /// Reshape tensor to new dimensions
    pub fn reshape(&self, new_shape: &[usize]) -> Result<Self> {
        let total_size: usize = self.data.len();
        let new_total: usize = new_shape.iter().product();
        
        if total_size != new_total {
            return Err(SutraError::InvalidShape(format!(
                "Cannot reshape tensor of size {} to shape {:?} (size {})",
                total_size, new_shape, new_total
            )));
        }
        
        let reshaped = self.data.clone().into_shape_with_order(IxDyn(new_shape))
            .map_err(|e| SutraError::InvalidShape(e.to_string()))?;
            
        Ok(Tensor::new(reshaped, self.dtype))
    }
    
    /// Scale tensor by constant factor
    pub fn scale(&self, factor: f32) -> Result<Self> {
        let scaled_data = &self.data * factor;
        Ok(Tensor::new(scaled_data, self.dtype))
    }
    
    /// Remove dimension of size 1 at specified axis
    pub fn squeeze(&self, axis: usize) -> Result<Self> {
        let shape = self.shape();
        if axis >= shape.len() {
            return Err(SutraError::InvalidShape(format!(
                "Axis {} out of range for tensor with {} dimensions", 
                axis, shape.len()
            )));
        }
        
        if shape[axis] != 1 {
            return Err(SutraError::InvalidShape(format!(
                "Cannot squeeze axis {} with size {}", 
                axis, shape[axis]
            )));
        }
        
        let mut new_shape = Vec::new();
        for (i, &dim) in shape.iter().enumerate() {
            if i != axis {
                new_shape.push(dim);
            }
        }
        
        self.reshape(&new_shape)
    }
}

/// View into a tensor for zero-copy operations
pub struct TensorView<'a> {
    data: &'a ArrayD<f32>,
    dtype: DType,
}

impl<'a> TensorView<'a> {
    pub fn new(data: &'a ArrayD<f32>, dtype: DType) -> Self {
        Self { data, dtype }
    }

    pub fn shape(&self) -> &[usize] {
        self.data.shape()
    }

    pub fn dtype(&self) -> DType {
        self.dtype
    }

    pub fn data(&self) -> &ArrayD<f32> {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tensor_creation() {
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let tensor = Tensor::from_slice(&data, &[2, 2], DType::F32).unwrap();
        assert_eq!(tensor.shape(), &[2, 2]);
        assert_eq!(tensor.dtype(), DType::F32);
    }

    #[test]
    fn test_tensor_memory() {
        let tensor = Tensor::zeros(&[100, 100], DType::F32);
        assert_eq!(tensor.memory_usage(), 10000 * 4); // 40KB
    }
}

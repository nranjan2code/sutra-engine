use crate::error::{LoaderError, Result};
use memmap2::Mmap;
use safetensors::SafeTensors;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use sutra_core::{DType, Tensor};

/// Information about a tensor in a safetensors file
#[derive(Debug, Clone)]
pub struct TensorInfo {
    pub name: String,
    pub shape: Vec<usize>,
    pub dtype: DType,
    pub offset: usize,
    pub size_bytes: usize,
}

/// Loader for safetensors format model weights
///
/// Safetensors is a fast and safe format for storing tensors.
/// Features:
/// - Memory-mapped I/O for efficient loading
/// - Zero-copy deserialization
/// - Built-in integrity checking
///
/// # Example
/// ```no_run
/// use sutra_loader::SafetensorsLoader;
///
/// let loader = SafetensorsLoader::new("model.safetensors")?;
/// let tensor = loader.load_tensor("model.layers.0.weight")?;
/// println!("Loaded tensor with shape: {:?}", tensor.shape());
/// # Ok::<(), sutra_loader::LoaderError>(())
/// ```
pub struct SafetensorsLoader {
    #[allow(dead_code)]
    path: String,
    mmap: Option<Mmap>,
    metadata: HashMap<String, TensorInfo>,
}

impl SafetensorsLoader {
    /// Create a new safetensors loader for the given file path
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();

        // Open file and create memory map
        let file = File::open(&path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        // Parse safetensors header
        let tensors = SafeTensors::deserialize(&mmap)?;

        // Extract metadata
        let mut metadata = HashMap::new();
        for name in tensors.names() {
            let view = tensors.tensor(name)?;
            let shape = view.shape().to_vec();
            let dtype = Self::parse_dtype(view.dtype())?;

            metadata.insert(
                name.to_string(),
                TensorInfo {
                    name: name.to_string(),
                    shape,
                    dtype,
                    offset: view.data().as_ptr() as usize - mmap.as_ptr() as usize,
                    size_bytes: view.data().len(),
                },
            );
        }

        Ok(Self {
            path: path_str,
            mmap: Some(mmap),
            metadata,
        })
    }

    /// List all tensor names in the file
    pub fn list_tensors(&self) -> Vec<String> {
        self.metadata.keys().cloned().collect()
    }

    /// Get metadata for a specific tensor
    pub fn tensor_info(&self, name: &str) -> Option<&TensorInfo> {
        self.metadata.get(name)
    }

    /// Load a tensor by name
    pub fn load_tensor(&self, name: &str) -> Result<Tensor> {
        let mmap = self
            .mmap
            .as_ref()
            .ok_or_else(|| LoaderError::Conversion("Memory map not available".to_string()))?;

        let tensors = SafeTensors::deserialize(mmap)?;
        let view = tensors
            .tensor(name)
            .map_err(|_| LoaderError::TensorNotFound(name.to_string()))?;

        let shape = view.shape();
        let dtype = Self::parse_dtype(view.dtype())?;
        let data = view.data();

        // Convert bytes to appropriate dtype
        let tensor = match dtype {
            DType::F32 => {
                let values: Vec<f32> = Self::bytes_to_vec(data);
                Tensor::from_slice(&values, shape, DType::F32)?
            }
            DType::F16 => {
                // For now, convert f16 to f32
                let values: Vec<f32> = Self::bytes_to_vec_f16(data);
                Tensor::from_slice(&values, shape, DType::F16)?
            }
            DType::I32 => {
                // Convert i32 bytes to f32 for computation
                let values: Vec<f32> = Self::bytes_to_vec_i32(data);
                Tensor::from_slice(&values, shape, DType::I32)?
            }
            DType::I8 | DType::U8 => {
                let float_values: Vec<f32> = data.iter().map(|&x| x as f32).collect();
                Tensor::from_slice(&float_values, shape, DType::U8)?
            }
            DType::I4 => {
                // Handle 4-bit quantized data
                let float_values: Vec<f32> = data.iter().map(|&x| x as f32).collect();
                Tensor::from_slice(&float_values, shape, DType::I4)?
            }
        };

        Ok(tensor)
    }

    /// Load multiple tensors at once
    pub fn load_tensors(&self, names: &[&str]) -> Result<HashMap<String, Tensor>> {
        let mut tensors = HashMap::new();
        for name in names {
            tensors.insert(name.to_string(), self.load_tensor(name)?);
        }
        Ok(tensors)
    }

    /// Load all tensors from the file
    pub fn load_all(&self) -> Result<HashMap<String, Tensor>> {
        let names: Vec<&str> = self.metadata.keys().map(|s| s.as_str()).collect();
        self.load_tensors(&names)
    }

    /// Get total size of all tensors in bytes
    pub fn total_size(&self) -> usize {
        self.metadata.values().map(|info| info.size_bytes).sum()
    }

    /// Parse safetensors dtype to our DType
    fn parse_dtype(dtype: safetensors::Dtype) -> Result<DType> {
        match dtype {
            safetensors::Dtype::F32 => Ok(DType::F32),
            safetensors::Dtype::F16 => Ok(DType::F16),
            safetensors::Dtype::I32 => Ok(DType::I32),
            safetensors::Dtype::I8 => Ok(DType::I8),
            safetensors::Dtype::U8 => Ok(DType::U8),
            _ => Err(LoaderError::UnsupportedFormat(format!(
                "Unsupported dtype: {:?}",
                dtype
            ))),
        }
    }

    /// Convert bytes to Vec<f32> with alignment-safe conversion
    fn bytes_to_vec<T>(bytes: &[u8]) -> Vec<T> 
    where
        T: Clone + From<f32>,
    {
        // For f32 specifically, use from_le_bytes to avoid alignment UB
        let count = bytes.len() / 4;
        let mut vec = Vec::with_capacity(count);
        
        for i in 0..count {
            let f32_val = f32::from_le_bytes([
                bytes[i * 4],
                bytes[i * 4 + 1],
                bytes[i * 4 + 2],
                bytes[i * 4 + 3],
            ]);
            vec.push(T::from(f32_val));
        }
        
        vec
    }

    /// Convert f16 bytes to f32 vec
    fn bytes_to_vec_f16(bytes: &[u8]) -> Vec<f32> {
        // Simple f16 to f32 conversion (can be optimized with half crate)
        let count = bytes.len() / 2;
        let mut result = Vec::with_capacity(count);

        for i in 0..count {
            let f16_bits = u16::from_le_bytes([bytes[i * 2], bytes[i * 2 + 1]]);
            // Basic f16 to f32 conversion
            let sign = (f16_bits >> 15) & 0x1;
            let exp = (f16_bits >> 10) & 0x1f;
            let frac = f16_bits & 0x3ff;

            let f32_bits = if exp == 0 {
                if frac == 0 {
                    (sign as u32) << 31
                } else {
                    // Subnormal
                    let leading_zeros = frac.leading_zeros() - 16;
                    let exp_adjusted = 127 - 15 - leading_zeros;
                    let frac_adjusted = (frac << (leading_zeros + 1)) & 0x3ff;
                    ((sign as u32) << 31) | (exp_adjusted << 23) | ((frac_adjusted as u32) << 13)
                }
            } else if exp == 31 {
                // Inf or NaN
                ((sign as u32) << 31) | (0xff << 23) | ((frac as u32) << 13)
            } else {
                // Normal
                let exp_adjusted = exp as u32 + (127 - 15);
                ((sign as u32) << 31) | (exp_adjusted << 23) | ((frac as u32) << 13)
            };

            result.push(f32::from_bits(f32_bits));
        }

        result
    }

    /// Convert i32 bytes to f32 vec for computation
    fn bytes_to_vec_i32(bytes: &[u8]) -> Vec<f32> {
        let count = bytes.len() / 4;
        let mut result = Vec::with_capacity(count);

        for i in 0..count {
            let i32_val = i32::from_le_bytes([
                bytes[i * 4],
                bytes[i * 4 + 1],
                bytes[i * 4 + 2],
                bytes[i * 4 + 3],
            ]);
            result.push(i32_val as f32);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_loader_creation() {
        // Create a minimal safetensors file
        let mut file = NamedTempFile::new().unwrap();

        // Safetensors format: [8 byte header size][header JSON][tensor data]
        let header = r#"{"test":{"dtype":"F32","shape":[2,2],"data_offsets":[0,16]}}"#;
        let header_size = header.len() as u64;

        file.write_all(&header_size.to_le_bytes()).unwrap();
        file.write_all(header.as_bytes()).unwrap();

        // Write tensor data (2x2 f32 matrix)
        let data: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
        for &val in &data {
            file.write_all(&val.to_le_bytes()).unwrap();
        }

        file.flush().unwrap();

        // Test loader
        let result = SafetensorsLoader::new(file.path());
        assert!(
            result.is_ok(),
            "Failed to create loader: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_dtype_conversion() {
        use safetensors::Dtype;

        assert!(matches!(
            SafetensorsLoader::parse_dtype(Dtype::F32),
            Ok(DType::F32)
        ));
        assert!(matches!(
            SafetensorsLoader::parse_dtype(Dtype::F16),
            Ok(DType::F16)
        ));
        assert!(matches!(
            SafetensorsLoader::parse_dtype(Dtype::I32),
            Ok(DType::I32)
        ));
    }
}

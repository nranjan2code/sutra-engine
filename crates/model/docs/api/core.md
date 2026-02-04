# Core API Reference

The `sutra-core` crate provides the foundational building blocks for the SutraWorks Model framework.

## Tensor Operations

### Tensor Type

The core tensor abstraction supporting multiple data types:

```rust
pub struct Tensor<T> {
    shape: Shape,
    data: Vec<T>,
    dtype: DType,
}

impl<T> Tensor<T> {
    pub fn new(data: Vec<T>, shape: Shape, dtype: DType) -> Self;
    pub fn shape(&self) -> &Shape;
    pub fn dtype(&self) -> DType;
    pub fn data(&self) -> &[T];
    pub fn memory_usage(&self) -> usize;
}
```

### Supported Data Types

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DType {
    F32,    // 32-bit float
    F16,    // 16-bit float
    I32,    // 32-bit signed integer
    I8,     // 8-bit signed integer
    U8,     // 8-bit unsigned integer
    I4,     // 4-bit signed integer (quantized)
}
```

### Creation Methods

```rust
impl Tensor<f32> {
    // Create tensor filled with zeros
    pub fn zeros(shape: Shape, dtype: DType) -> Result<Self>;
    
    // Create tensor filled with ones
    pub fn ones(shape: Shape, dtype: DType) -> Result<Self>;
    
    // Create tensor with random normal distribution
    pub fn randn(shape: Shape, dtype: DType) -> Result<Self>;
    
    // Create from raw data
    pub fn from_vec(data: Vec<f32>, shape: Shape) -> Result<Self>;
}
```

### Example Usage

```rust
use sutra_core::{Tensor, DType, Shape};

// Create a 2D tensor
let shape = Shape::new(vec![3, 4]);
let tensor = Tensor::zeros(shape, DType::F32)?;

// Create from data
let data = vec![1.0, 2.0, 3.0, 4.0];
let tensor = Tensor::from_vec(data, Shape::new(vec![2, 2]))?;

// Check properties
println!("Shape: {:?}", tensor.shape());
println!("Memory usage: {} bytes", tensor.memory_usage());
```

## Mathematical Operations

### Matrix Operations

```rust
pub mod ops {
    use crate::{Tensor, Result};

    /// Matrix multiplication: C = A * B
    pub fn matmul(a: &Tensor<f32>, b: &Tensor<f32>) -> Result<Tensor<f32>>;

    /// Element-wise addition: C = A + B
    pub fn add(a: &Tensor<f32>, b: &Tensor<f32>) -> Result<Tensor<f32>>;

    /// Element-wise multiplication: C = A * B
    pub fn mul(a: &Tensor<f32>, b: &Tensor<f32>) -> Result<Tensor<f32>>;

    /// Transpose matrix
    pub fn transpose(a: &Tensor<f32>) -> Result<Tensor<f32>>;
}
```

### Activation Functions

```rust
pub mod activations {
    use crate::{Tensor, Result};

    /// ReLU activation: max(0, x)
    pub fn relu(x: &Tensor<f32>) -> Tensor<f32>;

    /// GELU activation: x * Φ(x)
    pub fn gelu(x: &Tensor<f32>) -> Tensor<f32>;

    /// Sigmoid activation: 1 / (1 + exp(-x))
    pub fn sigmoid(x: &Tensor<f32>) -> Tensor<f32>;

    /// Tanh activation: tanh(x)
    pub fn tanh(x: &Tensor<f32>) -> Tensor<f32>;

    /// SiLU (Swish) activation: x * sigmoid(x)
    pub fn silu(x: &Tensor<f32>) -> Tensor<f32>;

    /// Softmax activation
    pub fn softmax(x: &Tensor<f32>, dim: i32) -> Result<Tensor<f32>>;
}
```

### Normalization

```rust
/// Layer normalization
pub fn layer_norm(
    x: &Tensor<f32>, 
    eps: f32
) -> Result<Tensor<f32>>;

/// RMS normalization
pub fn rms_norm(
    x: &Tensor<f32>, 
    weight: &Tensor<f32>, 
    eps: f32
) -> Result<Tensor<f32>>;
```

### Example Usage

```rust
use sutra_core::ops::{matmul, activations};

// Matrix multiplication
let a = Tensor::randn(vec![64, 128], DType::F32)?;
let b = Tensor::randn(vec![128, 256], DType::F32)?;
let c = matmul(&a, &b)?;

// Apply activations
let activated = activations::gelu(&c);
let normalized = layer_norm(&activated, 1e-5)?;
```

## Model Traits

### Model Interface

```rust
pub trait Model {
    type Input;
    type Output;
    type Error;

    /// Run forward pass
    fn forward(&self, input: &Self::Input) -> Result<Self::Output, Self::Error>;

    /// Get model configuration
    fn config(&self) -> &ModelConfig;

    /// Get memory usage in bytes
    fn memory_usage(&self) -> usize;
}
```

### ModelConfig

```rust
#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub hidden_size: usize,
    pub intermediate_size: usize,
    pub num_layers: usize,
    pub vocab_size: usize,
    pub max_sequence_length: usize,
}

impl ModelConfig {
    pub fn new(
        hidden_size: usize,
        intermediate_size: usize,
        num_layers: usize,
        vocab_size: usize,
    ) -> Self;
    
    pub fn parameter_count(&self) -> usize;
    pub fn memory_estimate(&self) -> usize;
}
```

### Example Implementation

```rust
use sutra_core::{Model, ModelConfig, Tensor};

struct MyModel {
    config: ModelConfig,
    weights: HashMap<String, Tensor<f32>>,
}

impl Model for MyModel {
    type Input = Tensor<f32>;
    type Output = Tensor<f32>;
    type Error = SutraError;

    fn forward(&self, input: &Self::Input) -> Result<Self::Output> {
        // Model implementation
        let output = process_input(input, &self.weights)?;
        Ok(output)
    }

    fn config(&self) -> &ModelConfig {
        &self.config
    }

    fn memory_usage(&self) -> usize {
        self.weights.values()
            .map(|t| t.memory_usage())
            .sum()
    }
}
```

## Error Handling

### SutraError

```rust
#[derive(Debug, thiserror::Error)]
pub enum SutraError {
    #[error("Shape mismatch: expected {expected:?}, got {actual:?}")]
    ShapeMismatch { expected: Shape, actual: Shape },

    #[error("Invalid data type: expected {expected:?}, got {actual:?}")]
    DTypeMismatch { expected: DType, actual: DType },

    #[error("Index out of bounds: index {index}, length {length}")]
    IndexOutOfBounds { index: usize, length: usize },

    #[error("Memory allocation failed: {size} bytes")]
    OutOfMemory { size: usize },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Computation error: {message}")]
    Computation { message: String },
}
```

### Result Type

```rust
pub type Result<T> = std::result::Result<T, SutraError>;
```

### Example Error Handling

```rust
use sutra_core::{Result, SutraError};

fn safe_matmul(a: &Tensor<f32>, b: &Tensor<f32>) -> Result<Tensor<f32>> {
    // Check dimensions
    if a.shape().dims()[1] != b.shape().dims()[0] {
        return Err(SutraError::ShapeMismatch {
            expected: Shape::new(vec![a.shape().dims()[0], b.shape().dims()[1]]),
            actual: b.shape().clone(),
        });
    }
    
    // Perform operation
    ops::matmul(a, b)
}
```

## Performance Utilities

### Memory Tracking

```rust
pub struct MemoryTracker {
    allocations: HashMap<String, usize>,
    peak_usage: usize,
}

impl MemoryTracker {
    pub fn new() -> Self;
    pub fn track_allocation(&mut self, name: &str, size: usize);
    pub fn current_usage(&self) -> usize;
    pub fn peak_usage(&self) -> usize;
    pub fn report(&self) -> MemoryReport;
}
```

### Performance Profiler

```rust
pub struct Profiler {
    timings: HashMap<String, Vec<Duration>>,
}

impl Profiler {
    pub fn new() -> Self;
    pub fn start_timer(&mut self, name: &str);
    pub fn stop_timer(&mut self, name: &str);
    pub fn average_time(&self, name: &str) -> Option<Duration>;
    pub fn report(&self) -> ProfileReport;
}
```

### Example Usage

```rust
use sutra_core::{MemoryTracker, Profiler};

let mut memory = MemoryTracker::new();
let mut profiler = Profiler::new();

// Track memory allocation
let tensor = Tensor::zeros(vec![1024, 1024], DType::F32)?;
memory.track_allocation("weights", tensor.memory_usage());

// Profile operation
profiler.start_timer("matmul");
let result = ops::matmul(&a, &b)?;
profiler.stop_timer("matmul");

// Generate reports
println!("Memory report: {:?}", memory.report());
println!("Performance report: {:?}", profiler.report());
```

## Configuration

### Global Configuration

```rust
pub struct GlobalConfig {
    pub max_memory: usize,
    pub num_threads: usize,
    pub enable_profiling: bool,
    pub log_level: LogLevel,
}

impl GlobalConfig {
    pub fn default() -> Self;
    pub fn set_max_memory(&mut self, bytes: usize);
    pub fn set_num_threads(&mut self, threads: usize);
}
```

### Environment Variables

```bash
# Set maximum memory usage
export SUTRA_MAX_MEMORY=16GB

# Set number of threads
export SUTRA_NUM_THREADS=8

# Enable profiling
export SUTRA_ENABLE_PROFILING=true

# Set log level
export SUTRA_LOG_LEVEL=info
```

### Example Configuration

```rust
use sutra_core::GlobalConfig;

let mut config = GlobalConfig::default();
config.set_max_memory(16 * 1024 * 1024 * 1024); // 16GB
config.set_num_threads(8);
config.enable_profiling = true;

// Apply configuration
sutra_core::set_global_config(config)?;
```

## Testing Utilities

### Test Helpers

```rust
pub mod test_utils {
    use crate::{Tensor, DType, Shape};

    /// Create tensor with known test data
    pub fn create_test_tensor(shape: Shape) -> Tensor<f32>;

    /// Assert tensors are approximately equal
    pub fn assert_tensor_eq(a: &Tensor<f32>, b: &Tensor<f32>, tolerance: f32);

    /// Generate random test data
    pub fn random_tensor(shape: Shape, range: (f32, f32)) -> Tensor<f32>;
}
```

### Example Test

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use sutra_core::test_utils::*;

    #[test]
    fn test_matmul() {
        let a = create_test_tensor(Shape::new(vec![2, 3]));
        let b = create_test_tensor(Shape::new(vec![3, 4]));
        
        let result = ops::matmul(&a, &b).unwrap();
        
        assert_eq!(result.shape().dims(), &[2, 4]);
    }
}
```

## Best Practices

### Memory Management

```rust
// ✅ Good: Check memory before allocation
if estimated_size > available_memory() {
    return Err(SutraError::OutOfMemory { size: estimated_size });
}

// ✅ Good: Use memory tracking
let mut tracker = MemoryTracker::new();
tracker.track_allocation("model", model_size);

// ❌ Avoid: Large intermediate allocations
let huge_temp = Tensor::zeros(vec![10000, 10000], DType::F32)?; // Bad
```

### Error Handling

```rust
// ✅ Good: Validate inputs
fn validate_shape(tensor: &Tensor<f32>, expected: &[usize]) -> Result<()> {
    if tensor.shape().dims() != expected {
        return Err(SutraError::ShapeMismatch {
            expected: Shape::new(expected.to_vec()),
            actual: tensor.shape().clone(),
        });
    }
    Ok(())
}

// ✅ Good: Use Result types consistently
fn safe_operation(input: &Tensor<f32>) -> Result<Tensor<f32>> {
    validate_shape(input, &[128, 256])?;
    ops::activate(input)
}
```

### Performance

```rust
// ✅ Good: Reuse allocations when possible
let mut buffer = Tensor::zeros(output_shape, DType::F32)?;
for batch in batches {
    process_batch(&batch, &mut buffer)?;
}

// ✅ Good: Use appropriate data types
let quantized = tensor.to_dtype(DType::I8)?; // Save memory

// ❌ Avoid: Unnecessary copies
let copy = tensor.clone(); // Expensive for large tensors
```

---

This API reference covers the core functionality. See the [other API references](../api/) for specialized modules.
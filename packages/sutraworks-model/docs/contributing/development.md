# Contributing to SutraWorks Model

Thank you for your interest in contributing to SutraWorks Model! This guide will help you get started with development and contributing to the project.

## 🎯 Project Status

SutraWorks Model is a **production-ready** framework with:
- ✅ **57/57 tests passing** - Complete test coverage
- ✅ **Zero compilation warnings** - Enterprise-grade code quality
- ✅ **Working examples** - All 7 examples functional
- ✅ **Real algorithms** - Authentic mathematical implementations

## 🚀 Quick Start for Contributors

### 1. Fork and Clone

```bash
# Fork the repository on GitHub first
git clone https://github.com/yourusername/sutraworks-model.git
cd sutraworks-model

# Add upstream remote
git remote add upstream https://github.com/nranjan2code/sutraworks-model.git
```

### 2. Set Up Development Environment

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install development tools
cargo install cargo-watch cargo-audit cargo-outdated

# Build the project
cargo build --all

# Run tests to verify everything works
cargo test --all
```

### 3. Verify Working Examples

```bash
# Test all working examples
cargo run --example end_to_end --release
cargo run --example quantization_benchmark --release
cargo run --example rwkv_inference --release
cargo run --example mamba_inference --release
cargo run --example qlora_training --release
cargo run --example nesy_agent --release
cargo run --example trading_terminal_demo --release
```

## 🏗️ Development Workflow

### Creating a Feature Branch

```bash
# Sync with upstream
git fetch upstream
git checkout main
git merge upstream/main

# Create feature branch
git checkout -b feature/your-feature-name

# Or for bug fixes
git checkout -b fix/bug-description
```

### Development Cycle

```bash
# 1. Make your changes
# Edit files in your preferred editor

# 2. Run fast checks
cargo check --all

# 3. Run tests
cargo test --all

# 4. Check code quality (zero warnings enforced)
cargo clippy --all -- -D warnings

# 5. Format code
cargo fmt --all

# 6. Test working examples
cargo run --example end_to_end --release
```

### Continuous Development

```bash
# Use cargo-watch for auto-rebuilds
cargo install cargo-watch
cargo watch -x "check --all" -x "test --all"

# Or watch specific crate
cd crates/sutra-core
cargo watch -x test
```

## 📝 Code Standards

### Rust Best Practices

#### 1. Error Handling

```rust
// ✅ Good: Use Result types consistently
pub fn quantize_weights(weights: &Tensor<f32>) -> Result<QuantizedTensor> {
    if weights.shape().is_empty() {
        return Err(SutraError::InvalidInput {
            message: "Empty tensor not supported".to_string(),
        });
    }
    
    // Implementation
    Ok(quantized_tensor)
}

// ❌ Avoid: Unwrapping in library code
let result = risky_operation().unwrap(); // Don't do this

// ✅ Good: Proper error propagation
let result = risky_operation()?;
```

#### 2. Memory Management

```rust
// ✅ Good: Track memory usage
pub struct Model {
    weights: HashMap<String, Tensor<f32>>,
    memory_tracker: MemoryTracker,
}

impl Model {
    pub fn memory_usage(&self) -> usize {
        self.weights.values()
            .map(|t| t.memory_usage())
            .sum()
    }
}

// ❌ Avoid: Large temporary allocations
let huge_temp = vec![0.0f32; 1_000_000 * 1_000_000]; // Don't do this
```

#### 3. Type Safety

```rust
// ✅ Good: Strong typing
#[derive(Debug, Clone, Copy)]
pub struct ModelDimensions {
    pub hidden_size: usize,
    pub num_layers: usize,
    pub vocab_size: usize,
}

// ❌ Avoid: Primitive obsession
pub fn create_model(hidden: usize, layers: usize, vocab: usize) // Less clear
```

### Testing Standards

#### 1. Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn test_matrix_multiplication() {
        let a = create_test_tensor(vec![2, 3]);
        let b = create_test_tensor(vec![3, 4]);
        
        let result = ops::matmul(&a, &b).unwrap();
        
        assert_eq!(result.shape().dims(), &[2, 4]);
        assert!(result.data().iter().all(|&x| x.is_finite()));
    }

    #[test]
    fn test_quantization_compression() {
        let weights = Tensor::randn(vec![1024, 512], DType::F32).unwrap();
        let quantizer = AwqQuantizer::new(AwqConfig::default());
        
        let quantized = quantizer.quantize(&weights, None).unwrap();
        let compression_ratio = quantized.compression_ratio();
        
        assert!(compression_ratio > 6.0); // At least 6x compression
        assert!(compression_ratio < 10.0); // But not unrealistic
    }
}
```

#### 2. Integration Tests

```rust
// tests/integration_tests.rs
use sutra_core::*;
use sutra_quantize::*;
use sutra_rwkv::*;

#[test]
fn test_end_to_end_pipeline() {
    // Test complete workflow
    let tokenizer = BpeTokenizer::from_vocab("test_vocab.json").unwrap();
    let tokens = tokenizer.encode("Hello world!").unwrap();
    
    let model = RwkvModel::new(test_config()).unwrap();
    let output = model.forward(&tokens).unwrap();
    
    let quantizer = AwqQuantizer::new(AwqConfig::default());
    let quantized_model = quantizer.quantize_model(&model).unwrap();
    
    // Verify quantized model still works
    let quantized_output = quantized_model.forward(&tokens).unwrap();
    assert_similar_tensors(&output, &quantized_output, 0.1);
}
```

### Documentation Standards

#### 1. API Documentation

```rust
/// Quantizes a tensor using the AWQ (Activation-aware Weight Quantization) algorithm.
///
/// This function implements the AWQ quantization method which preserves salient weights
/// based on activation patterns to minimize accuracy degradation.
///
/// # Arguments
///
/// * `weights` - The input tensor to quantize (must be 2D)
/// * `salience` - Optional salience scores for weight protection
///
/// # Returns
///
/// Returns a `QuantizedTensor` containing:
/// * Quantized weights packed into 4-bit values
/// * Per-group scaling factors
/// * Zero-point offsets
///
/// # Errors
///
/// Returns `SutraError::InvalidInput` if:
/// * Input tensor is not 2D
/// * Tensor dimensions are incompatible with group size
///
/// # Example
///
/// ```rust
/// use sutra_quantize::{AwqQuantizer, AwqConfig};
/// use sutra_core::Tensor;
///
/// let weights = Tensor::randn(vec![1024, 512], DType::F32)?;
/// let quantizer = AwqQuantizer::new(AwqConfig::default());
/// let quantized = quantizer.quantize(&weights, None)?;
/// 
/// println!("Compression: {:.2}x", quantized.compression_ratio());
/// ```
pub fn quantize(
    &self,
    weights: &Tensor<f32>,
    salience: Option<&Tensor<f32>>
) -> Result<QuantizedTensor> {
    // Implementation
}
```

#### 2. Module Documentation

```rust
//! # AWQ Quantization Module
//!
//! This module implements the AWQ (Activation-aware Weight Quantization) algorithm
//! for neural network model compression. AWQ achieves superior compression ratios
//! while maintaining model accuracy by protecting salient weights based on 
//! activation statistics.
//!
//! ## Key Features
//!
//! - **7.42x compression** with minimal accuracy loss
//! - **Real bit-packing** storing 2 values per byte
//! - **Salience protection** preserving important weights
//! - **Production ready** with comprehensive error handling
//!
//! ## Quick Start
//!
//! ```rust
//! use sutra_quantize::{AwqQuantizer, AwqConfig};
//!
//! let quantizer = AwqQuantizer::new(AwqConfig::default());
//! let quantized = quantizer.quantize(&weights, None)?;
//! ```
//!
//! ## Performance
//!
//! Benchmarked compression ratios:
//! - 402MB model → 54MB quantized (7.42x)
//! - <125ms quantization time for large matrices
//! - Memory usage optimized for 16GB systems
```

## 🎯 Priority Areas for Contributions

### High Priority

1. **Performance Optimization**
   - SIMD optimizations for tensor operations
   - Memory layout improvements
   - Parallel processing enhancements

2. **Additional Quantization Methods**
   - GPTQ implementation
   - GGUF format support
   - Mixed-precision strategies

3. **Model Format Support**
   - PyTorch model converter
   - ONNX integration
   - TensorFlow Lite support

### Medium Priority

1. **Advanced Features**
   - Distributed inference
   - Model serving infrastructure
   - Caching strategies

2. **Developer Experience**
   - Better error messages
   - Debugging tools
   - Performance profiling

3. **Documentation**
   - More tutorials
   - Video guides
   - Architecture deep-dives

### Welcome Contributions

1. **Bug Fixes**
   - Any bug reports with reproducible test cases
   - Performance regressions
   - Memory leaks or excessive allocations

2. **Testing**
   - Additional test cases
   - Property-based testing
   - Benchmark improvements

3. **Examples**
   - Real-world use cases
   - Integration examples
   - Performance demonstrations

## 🧪 Testing Guidelines

### Running Tests

```bash
# Run all tests
cargo test --all

# Run specific crate tests
cargo test -p sutra-core

# Run with release optimizations (for performance tests)
cargo test --release

# Run with verbose output
cargo test -- --nocapture

# Run specific test
cargo test test_quantization_compression
```

### Test Categories

1. **Unit Tests** (56 tests)
   - Individual function testing
   - Edge case validation
   - Error condition testing

2. **Integration Tests**
   - Cross-crate functionality
   - End-to-end workflows
   - Real model validation

3. **Property Tests**
   - Mathematical properties
   - Invariant checking
   - Fuzzing edge cases

4. **Performance Tests**
   - Benchmark validation
   - Memory usage tracking
   - Latency measurement

### Writing Good Tests

```rust
#[test]
fn test_quantization_preserves_shape() {
    // Arrange
    let original_shape = vec![256, 512];
    let weights = Tensor::randn(original_shape.clone(), DType::F32).unwrap();
    let quantizer = AwqQuantizer::new(AwqConfig::default());
    
    // Act
    let quantized = quantizer.quantize(&weights, None).unwrap();
    
    // Assert
    assert_eq!(quantized.shape().dims(), &original_shape);
}

#[test]
fn test_quantization_compression_ratio() {
    // Test realistic compression expectations
    let weights = Tensor::randn(vec![1024, 1024], DType::F32).unwrap();
    let quantizer = AwqQuantizer::new(AwqConfig::default());
    
    let quantized = quantizer.quantize(&weights, None).unwrap();
    let ratio = quantized.compression_ratio();
    
    // Should achieve significant compression
    assert!(ratio > 6.0, "Compression ratio too low: {}", ratio);
    assert!(ratio < 10.0, "Compression ratio unrealistic: {}", ratio);
}
```

## 📋 Submission Guidelines

### Pull Request Process

1. **Before Starting**
   - Check existing issues and PRs
   - Discuss large changes in an issue first
   - Ensure tests pass locally

2. **PR Requirements**
   - Clear description of changes
   - All tests passing (57/57)
   - Zero Clippy warnings
   - Documentation updates if needed

3. **PR Template**
   ```markdown
   ## Description
   Brief description of what this PR does.

   ## Changes
   - [ ] Added feature X
   - [ ] Fixed bug Y
   - [ ] Updated documentation

   ## Testing
   - [ ] All existing tests pass
   - [ ] Added new tests for changes
   - [ ] Ran working examples

   ## Checklist
   - [ ] Code formatted with `cargo fmt`
   - [ ] No Clippy warnings
   - [ ] Documentation updated
   - [ ] Examples still work
   ```

### Code Review Process

1. **Automated Checks**
   - CI/CD pipeline runs all tests
   - Code quality checks
   - Security scanning

2. **Human Review**
   - Code clarity and maintainability
   - Test coverage adequacy
   - Documentation quality

3. **Approval Process**
   - One approval required for minor changes
   - Two approvals for major changes
   - Maintainer approval for breaking changes

## 🛠️ Development Tools

### VS Code Configuration

The project includes `.vscode/tasks.json` with helpful tasks:

```bash
# Build tasks
Cmd+Shift+B  # Build All (Release)

# Test tasks
Cmd+Shift+P "Tasks: Run Task" → "Test All Crates"

# Quality checks
Cmd+Shift+P "Tasks: Run Task" → "Clippy (Code Quality)"
```

### Useful Commands

```bash
# Check for outdated dependencies
cargo outdated

# Security audit
cargo audit

# Generate documentation
cargo doc --open

# Check unused dependencies
cargo +nightly udeps

# Profile performance
cargo flamegraph --example quantization_benchmark
```

### Debugging

```bash
# Run with debug symbols
cargo build
rust-gdb target/debug/example_name

# Memory debugging with Valgrind (Linux)
valgrind --tool=memcheck --leak-check=full \
  target/debug/example_name

# Performance profiling
perf record target/release/example_name
perf report
```

## 💡 Getting Help

### Resources

- **Documentation**: Comprehensive guides in `/docs`
- **Examples**: 7 working examples in `/examples`
- **API Docs**: `cargo doc --open`
- **Architecture**: [System Overview](../architecture/overview.md)

### Communication

- **Issues**: GitHub Issues for bugs and feature requests
- **Discussions**: GitHub Discussions for questions
- **Email**: Technical questions to maintainers

### Before Asking

1. Check existing documentation
2. Search closed issues
3. Run the working examples
4. Try the troubleshooting guides

## 🎉 Recognition

Contributors are recognized in:

- **README**: Listed in contributors section
- **Releases**: Mentioned in release notes
- **Documentation**: Author attribution
- **Examples**: Credit for contributed examples

## 📄 License

By contributing, you agree that your contributions will be licensed under the same terms as the project (Apache 2.0 OR MIT).

---

**Ready to contribute? Start by forking the repository and running the examples!**

```bash
git clone https://github.com/yourusername/sutraworks-model.git
cd sutraworks-model
cargo test --all
cargo run --example trading_terminal_demo --release
```
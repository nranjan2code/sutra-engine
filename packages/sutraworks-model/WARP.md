# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Commands

### Build and Test
```bash
# Fast syntax check (recommended first step)
cargo check --all

# Build all crates (debug mode, fast compilation)
cargo build --all

# Build optimized release version
cargo build --all --release

# Build with CPU-specific optimizations for maximum performance
RUSTFLAGS="-C target-cpu=native" cargo build --all --release

# Run all tests (57 tests should pass)
cargo test --all

# Run tests in release mode (faster execution)
cargo test --all --release

# Run tests for a specific crate
cargo test -p sutra-quantize
cargo test -p sutra-rwkv

# Run a specific test
cargo test test_awq_quantization
```

### Code Quality
```bash
# Format all code (run before commits)
cargo fmt --all

# Lint with clippy (zero warnings enforced)
cargo clippy --all -- -D warnings

# Generate and view API documentation
cargo doc --all --no-deps --open
```

### Running Examples
```bash
# End-to-end pipeline (tokenize→embed→infer→quantize→decode)
cargo run --example end_to_end --release

# Quantization benchmark (validates compression metrics)
cargo run --example quantization_benchmark --release

# RWKV inference (demonstrates O(n) RNN)
cargo run --example rwkv_inference --release

# Mamba inference (demonstrates state space model)
cargo run --example mamba_inference --release

# Model loader (demonstrates safetensors loading)
cargo run --example model_loader --release

# QLoRA training (demonstrates parameter-efficient fine-tuning)
cargo run --example qlora_training --release

# Neuro-symbolic agent
cargo run --example nesy_agent --release
```

### Clean Build
```bash
# Remove all build artifacts
cargo clean
```

## Architecture

SutraWorks Model is a production-ready Rust framework for efficient AI on consumer hardware (16GB RAM). It's organized as a Cargo workspace with 9 specialized crates implementing 4 core capabilities.

### Core Design Principles

1. **Linear Complexity First**: All architectures (RWKV, Mamba) use O(n) algorithms, not O(n²) transformers
2. **Memory-Mapped I/O**: Large model weights use `memmap2` for efficient loading without full RAM allocation
3. **4-bit Quantization**: AWQ quantization with bit-packing (2 values/byte) enables 7-8x compression
4. **No GPU Required**: Pure Rust, CPU-optimized for edge deployment

### Crate Organization

```
sutraworks-model/
├── crates/
│   ├── sutra-core/          # Foundation layer
│   ├── sutra-quantize/      # Model compression
│   ├── sutra-peft/          # Parameter-efficient fine-tuning
│   ├── sutra-rwkv/          # RWKV RNN architecture
│   ├── sutra-mamba/         # Mamba state space models
│   ├── sutra-nesy/          # Neuro-symbolic reasoning
│   ├── sutra-loader/        # Model loading (safetensors, HuggingFace)
│   ├── sutra-tokenizer/     # BPE/WordPiece/Unigram tokenizers
│   └── sutra-training/      # Training infrastructure
├── examples/                # Runnable demonstrations
└── tests/                   # Integration tests
```

### Key Components

#### sutra-core (~1,550 lines)
Foundation for all other crates. Provides:
- `Tensor`: Multi-dimensional array abstraction with shape/data/dtype
- `DType`: F32, F16, I32, I8, U8, I4 (4-bit for quantization)
- `ops` module: matmul, activations (GELU/ReLU/SiLU/etc), layer_norm, rms_norm, embedding
- `Result<T>` and `SutraError` for error handling
- Memory tracking via `.memory_usage()`

**Key Pattern**: All tensor operations return `Result<Tensor>` for fallible ops. Use `?` operator.

#### sutra-quantize (~2,300 lines)
AWQ (Activation-aware Weight Quantization) with production bit-packing:
- Compresses FP32 weights to 4-bit integers (7-8x smaller)
- **Critical**: Real bit-packing stores 2 values per byte (high/low nibbles)
- Group-wise quantization with `group_size=128` for salience preservation
- `quantized_matmul()`: Matmul with on-the-fly dequantization
- Signed zero-points for asymmetric distributions (not clamped to [0,15])

**Key Pattern**: AWQ protects "salient" (high-activation) weights from quantization errors.

#### sutra-rwkv (~1,400 lines)
RWKV-4 architecture with authentic WKV kernel:
- **WKV kernel**: O(n) recurrence using `aa/bb/pp` accumulators (not naive O(n²))
- Time-mixing: attention-like mechanism with receptance/key/value
- Channel-mixing: FFN with squared ReLU and time interpolation
- `WkvState`: Tracks hidden state between tokens (enables streaming inference)

**Critical**: WKV kernel uses log-sum-exp stabilization to prevent overflow.

#### sutra-mamba (~1,350 lines)
Mamba state space model with selective scan:
- **Selective mechanism**: Δ (delta), B, C parameters are input-dependent via learned linear projections
- **Zero-order hold**: Converts continuous SSM to discrete time (A_discrete, B_discrete)
- Selective scan: O(n) state update with input-dependent dynamics
- Causal 1D convolution with SiLU gating

**Critical**: "Selective" means A/B/C change per token (unlike fixed LTI systems).

#### sutra-loader (~1,600 lines)
Model loading from HuggingFace and safetensors format:
- **Architecture detection**: Automatically identifies RWKV/Mamba/Transformer from weight keys
- **Key mapping**: Converts HuggingFace naming (e.g., `blocks.0.ln1.weight`) to SutraWorks format
- Memory-mapped safetensors for zero-copy loading
- HuggingFace Hub downloader with retry logic

**Key Pattern**: Use `SafetensorsLoader::new()` for local files, `HuggingFaceLoader::from_hub()` for remote models.

#### sutra-tokenizer (~1,800 lines)
Comprehensive tokenization with 3 algorithms:
- **BPE**: Byte Pair Encoding (GPT-2/GPT-3 style)
- **WordPiece**: BERT-style subword tokenization
- **Unigram**: SentencePiece-style probabilistic tokenization
- All tokenizers support special tokens, normalization, pre-tokenization

#### sutra-peft (~1,892 lines)
Parameter-efficient fine-tuning:
- **LoRA**: Low-rank adaptation with learnable A/B matrices (rank << d_model)
- **QLoRA**: LoRA + 4-bit base model quantization (fine-tune on 16GB RAM)
- Adapter merging for inference

#### sutra-training (~1,200 lines)
Training infrastructure:
- Optimizers: Adam, AdamW, SGD with momentum/Nesterov
- Schedulers: Cosine annealing, linear warmup
- Loss functions: Cross-entropy, MSE
- Training loop with gradient accumulation and checkpointing

### Inference Pipeline Flow

Typical inference follows this pattern:
```rust
// 1. Load tokenizer
let tokenizer = BpeTokenizer::from_file("vocab.json", "merges.txt")?;

// 2. Tokenize input
let encoding = tokenizer.encode("Hello, world!")?;

// 3. Load model weights (memory-mapped)
let loader = SafetensorsLoader::new("model.safetensors")?;
let weights = loader.load_all()?;

// 4. Optional: Quantize for efficiency
let quantizer = AwqQuantizer::new(AwqConfig::default());
let quantized = quantizer.quantize(&weights, None)?;

// 5. Run inference (RWKV or Mamba)
let model = RwkvModel::new(config)?;
let output = model.forward(&encoding.ids, &mut state)?;

// 6. Decode tokens
let text = tokenizer.decode(&output.tokens)?;
```

## Development Patterns

### Error Handling
Always use `Result<T>` for fallible operations. Never `unwrap()` in library code (only in examples/tests).

```rust
// Good
pub fn forward(&self, x: &Tensor) -> Result<Tensor> {
    let normed = ops::layer_norm(x, 1e-5)?;
    let activated = ops::activations::gelu(&normed);
    Ok(activated)
}

// Bad (in library code)
let normed = ops::layer_norm(x, 1e-5).unwrap();
```

### Memory Management
Track memory usage for large models:
```rust
let tensor = Tensor::zeros(&[4096, 4096], DType::F32)?;
println!("Memory: {:.2} MB", tensor.memory_usage() / 1_048_576.0);
```

### Testing Patterns
All new features must include tests:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_feature() {
        // Arrange
        let input = Tensor::ones(&[2, 3], DType::F32).unwrap();
        
        // Act
        let result = my_function(&input).unwrap();
        
        // Assert
        assert_eq!(result.shape(), &[2, 3]);
    }
}
```

### Documentation
Document all public APIs with examples:
```rust
/// Performs AWQ quantization on weights.
///
/// # Arguments
///
/// * `weights` - The model weights to quantize (typically FP32)
/// * `calibration_data` - Optional activation data for salience calculation
///
/// # Returns
///
/// Quantized weights with ~7-8x compression
///
/// # Example
///
/// ```
/// let quantizer = AwqQuantizer::new(AwqConfig::default());
/// let quantized = quantizer.quantize(&weights, None)?;
/// ```
pub fn quantize(&self, weights: &Tensor, calibration_data: Option<&Tensor>) -> Result<QuantizedWeights>
```

### Performance Optimization
When optimizing, follow this order:
1. **Algorithm**: Use O(n) architectures (RWKV/Mamba), not O(n²) transformers
2. **Memory**: Quantize to 4-bit, use memory-mapped I/O
3. **Parallelism**: Use `rayon` for data-parallel operations
4. **CPU**: Build with `RUSTFLAGS="-C target-cpu=native"`

## Critical Implementation Details

### AWQ Quantization Bit-Packing
The quantization stores TWO 4-bit values in each byte:
```rust
// High nibble (bits 4-7) stores value at i*2
// Low nibble (bits 0-3) stores value at i*2+1
packed[i] = (data[i*2] << 4) | (data[i*2+1] & 0x0F);
```

Decompression extracts both:
```rust
let high = packed[i] >> 4;        // Upper 4 bits
let low = packed[i] & 0x0F;       // Lower 4 bits
```

### RWKV WKV Kernel Log-Sum-Exp Stabilization
Prevents overflow in exp() operations:
```rust
// Compute max for numerical stability
let max_val = a.max(b);
// exp(a - max) + exp(b - max) instead of exp(a) + exp(b)
let sum = (a - max_val).exp() + (b - max_val).exp();
```

### Mamba Selective Scan
Unlike LTI (Linear Time-Invariant) systems, Mamba's A/B/C matrices change per token:
```rust
// These are LEARNED projections, not fixed parameters
let delta = linear_projection_delta(&x);  // Changes per token
let B = linear_projection_B(&x);          // Changes per token
let C = linear_projection_C(&x);          // Changes per token

// Then discretize and scan with input-dependent dynamics
let A_discrete = (-delta * A_continuous).exp();
let B_discrete = delta * B;
```

### Safetensors Alignment
When loading safetensors, ensure proper alignment for zero-copy:
```rust
// BAD: Transmute without checking alignment (UB)
let data: &[f32] = unsafe { std::mem::transmute(bytes) };

// GOOD: Check alignment or copy if needed
if bytes.as_ptr() as usize % std::mem::align_of::<f32>() != 0 {
    // Copy to properly aligned buffer
}
```

## Common Issues

### "Out of memory" during model loading
- Solution: Use quantization (`AwqQuantizer`) to reduce model size 7-8x
- Check: Model file size vs available RAM

### "Index out of bounds" in quantization
- Likely cause: Row-major vs column-major layout mismatch
- Check: Ensure consistent indexing as `data[row * cols + col]`

### Slow compilation
- Use `cargo check` for fast validation (no codegen)
- Enable incremental compilation (default in debug mode)
- Consider `mold` or `lld` linker for faster linking

### Tests fail after changes
- Run `cargo test --all` to verify all crates
- Check integration tests in `tests/integration_tests.rs`
- Ensure no breaking changes to public APIs

## Commit Conventions

Follow Conventional Commits:
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `refactor:` - Code refactoring
- `test:` - Adding or updating tests
- `perf:` - Performance improvements

Examples:
```
feat: add GPTQ quantization support
fix: correct zero-point quantization in AWQ
docs: update RWKV architecture documentation
perf: optimize WKV kernel with SIMD
```

## VS Code Integration

The project includes pre-configured tasks in `.vscode/tasks.json`:
- **Default build**: `Cmd+Shift+B` runs release build
- **Run examples**: Individual tasks for each example
- Use the Command Palette to access other tasks

## Resources

- README.md: Comprehensive overview and validation results
- QUICKSTART.md: Step-by-step setup guide
- CONTRIBUTING.md: Detailed contribution guidelines
- STATUS.md: Current implementation status
- examples/: Runnable code demonstrations

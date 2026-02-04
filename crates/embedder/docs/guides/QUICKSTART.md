# 🚀 Sutra Embedder - Quick Start Guide

Get up and running with production-grade multi-dimensional embeddings in minutes!

## 📦 Installation

### Prerequisites
- Rust 1.70+ ([Install Rust](https://rustup.rs/))
- 2GB+ RAM recommended  
- Optional: GPU with CUDA/Metal support

### Build from Source

```bash
git clone https://github.com/yourusername/sutra-embedder.git
cd sutra-embedder
cargo build --release
```

## 🎯 Basic Usage

### 1. Check Your Hardware
```bash
./target/release/sutra-embedder hardware
```
This auto-detects your system capabilities and recommends optimal settings.

### 2. List Available Models
```bash
# Show all models
./target/release/sutra-embedder models

# Filter by dimension capability
./target/release/sutra-embedder models --dimensions 768
```

### 3. Generate Embeddings
```bash
# Generate 768-dimensional embeddings (high-quality)
./target/release/sutra-embedder embed \
  --text "Advanced embedding models for production systems" \
  --dimensions 768

# Generate 384-dimensional embeddings (efficient)  
./target/release/sutra-embedder embed \
  --text "Efficient models for edge computing" \
  --dimensions 384
```

> ℹ️ Dimension requests must match or fall below the backing model's native width. The CLI now fails fast instead of silently truncating larger requests.

## 🔧 Advanced Features

### FP16 Mixed Precision (2x Speedup)
Automatically enabled on compatible hardware:
```bash
# FP16 auto-detected on Apple Silicon, NVIDIA GPUs (compute ≥ 6.0)
./target/release/sutra-embedder embed \
  --text "Fast FP16 inference" \
  --dimensions 384
```

**Hardware Support**:
- ✅ Apple Silicon (M1/M2/M3) - Neural Engine
- ✅ NVIDIA GPUs - Tensor Cores (compute capability ≥ 6.0)  
- ✅ AMD GPUs - FP16 instructions
- ⚠️ CPU - Falls back to FP32

### Async Batch Queue (High Throughput)
For library use - non-blocking batch processing:
```rust
use sutra_embedder::{AsyncBatchQueue, EmbedderConfig};

// Unbounded queue for maximum throughput
let queue = AsyncBatchQueue::new(config)?;
let embeddings = queue.embed_async(texts).await?;

// Bounded queue with backpressure (capacity: 100)
let queue = BoundedAsyncBatchQueue::new(config, 100)?;
let results = queue.embed_async(texts).await?;
```

### INT8 Quantization (1.5-2x CPU Speedup)
Dynamic quantization via ONNX Runtime (pure Rust):
```rust
use sutra_embedder::quantization::{QuantizationConfig, quantize_model};

let config = QuantizationConfig::default();
let stats = quantize_model(&input_path, &output_path, config, &calibration_texts).await?;
// 4x model size reduction, 1.5-2x speedup on CPUs with VNNI/DP4A
```

### Model Downloads with Integrity Validation
```bash
# Download specific model with SHA256 validation
./target/release/sutra-embedder download --model bge-base-en-v1.5

# Download all models for offline use
./target/release/sutra-embedder download --model all

# Force re-download with validation
./target/release/sutra-embedder download --model all-MiniLM-L6-v2 --force
```

### Hardware Detection Details
GPU detection works across platforms:
- **NVIDIA**: CUDA via nvidia-smi
- **AMD**: ROCm via rocm-smi (Linux)
- **Apple**: Metal on Apple Silicon
- **Windows**: DirectML-capable GPUs

Example output:
```
Hardware Profile: medium-system
  CPU Cores: 10
  Memory: 16.00 GB
  GPU: Yes (Metal)
  Compute Tier: Desktop
```

### Generate SHA256 Hashes
```bash
# Get hash for specific model (for registry updates)
./target/release/sutra-embedder hash --model bge-base-en-v1.5

# Get hashes for all downloaded models
./target/release/sutra-embedder hash
```

### Performance Benchmarking
```bash
# Auto-detect hardware and run benchmarks (clean output)
./benchmark-clean.sh --profile auto --iterations 100

# Standard benchmark (may show ONNX Runtime allocator logs)
./target/release/sutra-embedder benchmark --profile auto --iterations 100

# Compare against Ollama (requires Ollama running)
./target/release/sutra-embedder compare \
  --model nomic-embed-text \
  --ollama-url http://localhost:11434 \
  --fair-comparison
```

**Note**: ONNX Runtime may show INFO logs about memory allocation (BFCArena). These are informational and can be safely ignored. Use `./benchmark-clean.sh` for filtered output.

## 🎨 Model Selection Guide

Choose the right model for your use case:

### 🚀 **Ultra-Efficient (64-256D)**
- **Model**: `all-MiniLM-L6-v2`
- **Size**: 86MB
- **Best For**: IoT devices, mobile apps, edge computing
- **Memory**: <1KB per embedding
- **Quality**: Good for most applications

### ⚡ **Efficient (384-512D)** ⭐ *Recommended*
- **Models**: `all-MiniLM-L6-v2`, `bge-base-en-v1.5`
- **Size**: 86-416MB
- **Best For**: Desktop apps, web services, general use
- **Memory**: ~1.5KB per embedding
- **Quality**: Excellent balance of speed and accuracy
- **Benchmark**: ~23ms latency, 43 emb/sec

### 🎯 **High-Quality (768D)**
- **Models**: `all-mpnet-base-v2`, `bge-base-en-v1.5`, `e5-base-v2`
- **Size**: 416MB each
- **Best For**: Research, semantic search, high-accuracy tasks
- **Memory**: ~3KB per embedding  
- **Quality**: State-of-the-art accuracy
- **Benchmark**: ~63ms latency, 15.7 emb/sec

### 🔥 **Maximum Quality (1024D+)**
- **Model**: `bge-large-en-v1.5`
- **Size**: 1.2GB
- **Best For**: Research, enterprise applications, maximum accuracy
- **Memory**: ~4KB per embedding
- **Quality**: Best available

### 🌍 **Multilingual (768D)**
- **Model**: `multilingual-e5-base`
- **Size**: 1GB
- **Best For**: 100+ languages, cross-lingual tasks
- **Memory**: ~3KB per embedding
- **Quality**: Best multilingual option

## 💡 Pro Tips & Troubleshooting

### 1. **Hardware Optimization**
```bash
# Let Sutra auto-select optimal model for your hardware
./target/release/sutra-embedder embed \
  --text "Auto-optimized embedding" \
  --dimensions 512
```

### 2. **Memory Optimization**
```bash
# Use smaller dimensions for memory-constrained environments
./target/release/sutra-embedder embed \
  --text "Memory-optimized embedding" \
  --dimensions 256
```

### 3. **Model Download Issues**
```bash
# Check network connectivity and retry
./target/release/sutra-embedder download --model bge-base-en-v1.5

# Models are stored in ./models/ directory
# Verify with: ls -lh models/
```

### 4. **GPU Not Detected**
```bash
# Check hardware detection
./target/release/sutra-embedder hardware

# Force CPU mode if needed (automatic fallback)
CUDA_VISIBLE_DEVICES="" ./target/release/sutra-embedder embed \
  --text "CPU-only embedding" \
  --dimensions 384
```

### 5. **Verbose ONNX Runtime Logs**
```bash
# Use clean script to filter allocator logs
./benchmark-clean.sh --profile auto --iterations 100

# Or manually filter
./target/release/sutra-embedder benchmark --profile auto --iterations 100 2>&1 | \
  grep -v "BFCArena" | grep -v "allocation"

# See LOGGING.md for details
cat LOGGING.md
```

## 🔗 Next Steps

- **Advanced Optimizations**: See [OPTIMIZATIONS.md](OPTIMIZATIONS.md) for FP16, async queues, fused ops
- **Flash Attention**: Check [FLASH_ATTENTION.md](FLASH_ATTENTION.md) for long sequence support
- **Library Integration**: Use `sutra-embedder` as a Rust library in your projects
- **Production Deployment**: Set up model caching and health monitoring  
- **Performance Tuning**: Run benchmarks and optimize for your specific hardware
- **INT8 Quantization**: Explore dynamic quantization for CPU speedup

## 📚 Additional Resources

- [Full Documentation](README.md)
- [Research Background](RESEARCH.md)
- [Optimization Guide](OPTIMIZATIONS.md)
- [Performance Summary](PERFORMANCE_SUMMARY.md)
- [Flash Attention Guide](FLASH_ATTENTION.md)
- [Logging Documentation](LOGGING.md)

---
**Ready to embed?** Start with `./target/release/sutra-embedder models` to see what's available! 🚀

## 📊 Benchmarking & Comparison

### Compare with Ollama
```bash
# Head-to-head comparison with Ollama's nomic-embed-text
./target/release/sutra-embedder compare \
  --model nomic-embed-text:latest \
  --ollama-url http://localhost:11434 \
  --iterations 50
```

**Expected Results** (Apple Silicon):
```
📊 ultra-efficient (256D) vs nomic-embed-text:latest:
  Latency:         1.56x faster (11.07ms vs 17.27ms)
  Throughput:      1.56x higher (90.32 vs 57.92 emb/sec)
  Storage savings: 67.4%

📊 efficient (384D) vs nomic-embed-text:latest:
  Latency:         1.23x faster (14.02ms vs 17.27ms)
  Throughput:      1.23x higher (71.33 vs 57.92 emb/sec)
  Storage savings: 51.2%

📊 high-quality (768D) vs nomic-embed-text:latest:
  Latency:         1.21x faster (14.29ms vs 17.27ms)
  Throughput:      1.21x higher (69.96 vs 57.92 emb/sec)
  Same dimensions: 768D vs 768D
```

**Key Advantages**:
- ✅ Faster at all dimension levels (1.21x - 1.56x)
- ✅ Flexible dimensions (256D/384D/768D) vs fixed 768D
- ✅ Massive storage savings (67% at 256D)
- ✅ Advanced optimizations (FP16, SIMD, async queue)

#### Auto-Detect Hardware and Benchmark
```bash
# Clean output (recommended)
./benchmark-clean.sh --profile auto --iterations 100

# Standard output (may include ONNX allocator logs)
./target/release/sutra-embedder benchmark --profile auto --iterations 100
```

Expected results (on M1 Mac):
```
║  Benchmark Results: efficient                                    ║
║  Latency (avg):    23.19 ms                                    ║
║  Latency (p50):    22.32 ms                                    ║
║  Latency (p95):    26.46 ms                                    ║
║  Latency (p99):    28.32 ms                                    ║
║  Throughput:       43.12 embeddings/sec                       ║
║  Dimensions:         384                                        ║

🚀 Sutra Embedder (Efficient Config):
  Speedup:             2.03x faster
  Memory Reduction:    87.8%
  Dimension Reduction: 50.0%
```

#### Benchmark for Specific Hardware
```bash
# Raspberry Pi profile
./target/release/sutra-embedder benchmark --profile raspberry-pi --iterations 100

# Desktop profile
./target/release/sutra-embedder benchmark --profile desktop --iterations 100

# Server profile
./target/release/sutra-embedder benchmark --profile server --iterations 100

# H100 GPU profile
./target/release/sutra-embedder benchmark --profile h100 --iterations 100
```

### 4. Optimize Models (Coming Soon)

```bash
cargo run --release -- optimize \
  --input models/base-model.safetensors \
  --output models/optimized-rpi.safetensors \
  --target raspberry-pi
```

## Configuration Presets

| Config | Dimensions | Quantization | Best For |
|--------|-----------|--------------|----------|
| `ultra-efficient` | 256 | INT4 | Raspberry Pi, embedded devices |
| `efficient` | 384 | INT8 | Desktops, mobile devices |
| `high-quality` | 768 | None | Servers, workstations |

## Development

### Run Tests
```bash
cargo test
```

### Run with Debug Logging
```bash
RUST_LOG=debug cargo run -- embed --text "test"
```

### Build for Production
```bash
cargo build --release --target=x86_64-unknown-linux-gnu
```

## Next Steps

1. Check out [RESEARCH.md](RESEARCH.md) for technical details
2. Read the full [README.md](README.md) for comprehensive documentation
3. Review [.github/copilot-instructions.md](.github/copilot-instructions.md) for project guidelines

## Troubleshooting

### Build Errors

If you encounter build errors, ensure you have:
- Rust 1.70+ installed (`rustup update`)
- All system dependencies installed

### Model Download Issues

If model download fails:
1. Check internet connection
2. Models are stored in `./models/` directory (project folder)
3. Total storage for all 6 models: 3.6GB
4. All downloads include SHA256 validation for security
5. Manually download models from HuggingFace if needed

### Performance Issues

For better performance:
1. Always use `--release` flag
2. Choose the right config for your hardware
3. First run is slower due to model download
4. Subsequent runs use cached models
5. GPU acceleration detected automatically

## Contributing

See [README.md](README.md) for contribution guidelines.

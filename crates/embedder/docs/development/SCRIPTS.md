# Scripts & Commands Reference

Quick reference for all available scripts and commands in Sutra Embedder.

## 🚀 Quick Commands

### Build & Run
```bash
# Build release version
cargo build --release

# Run with example text
./target/release/sutra-embedder embed --text "Hello world" --dimensions 384

# Check hardware capabilities
./target/release/sutra-embedder hardware
```

### Benchmarking

#### Comprehensive Benchmark Suite (World-Class)
```bash
# Benchmark all dimensions (64D-4096D) with quality + performance metrics
./target/release/sutra-embedder comprehensive-benchmark

# Benchmark specific dimensions only
./target/release/sutra-embedder comprehensive-benchmark -d "256,384,768"

# High-accuracy benchmark (more iterations)
./target/release/sutra-embedder comprehensive-benchmark -i 100

# Custom output directory
./target/release/sutra-embedder comprehensive-benchmark -o production_results

# Run example benchmark suite
./run-comprehensive-benchmarks.sh
```

**Output Files:**
- `benchmark_results.json` - Complete structured data
- `benchmark_results.csv` - Tabular format for Excel/analysis
- `benchmark_report.md` - Human-readable report with methodology

**See [BENCHMARKS.md](BENCHMARKS.md) for complete methodology**

#### Clean Benchmark (Recommended)
```bash
# Clean output with filtered logs
./benchmark-clean.sh --profile auto --iterations 100

# Quick benchmark (10 iterations)
./benchmark-clean.sh --profile auto --iterations 10
```

#### Standard Benchmark
```bash
# May show ONNX Runtime allocator logs
./target/release/sutra-embedder benchmark --profile auto --iterations 100

# Specific hardware profiles
./target/release/sutra-embedder benchmark --profile raspberry-pi --iterations 50
./target/release/sutra-embedder benchmark --profile desktop --iterations 100
./target/release/sutra-embedder benchmark --profile server --iterations 100
```

### Model Management
```bash
# List all available models
./target/release/sutra-embedder models

# List models for specific dimensions
./target/release/sutra-embedder models --dimensions 768 --verbose

# Download specific model (with SHA256 validation)
./target/release/sutra-embedder download --model bge-base-en-v1.5

# Download all 6 models (total 3.6GB)
./target/release/sutra-embedder download --model all

# Verify model integrity
./target/release/sutra-embedder hash --model all-MiniLM-L6-v2

# Check downloaded models
ls -lh models/
du -sh models/
```

**Available Models:**
- `all-MiniLM-L6-v2` (384D, 86MB)
- `bge-base-en-v1.5` (768D, 416MB)
- `all-mpnet-base-v2` (768D, 416MB)
- `bge-large-en-v1.5` (1024D, 1.2GB)
- `e5-base-v2` (768D, 416MB)
- `multilingual-e5-base` (768D, 1GB)

### Hash Generation
```bash
# Generate SHA256 for specific model
./target/release/sutra-embedder hash --model bge-base-en-v1.5

# Generate hashes for all downloaded models
./target/release/sutra-embedder hash
```

### Embedding Generation
```bash
# Generate embeddings with auto-selected model
./target/release/sutra-embedder embed \
  --text "Multi-dimensional embeddings" \
  --dimensions 768

# Efficient embeddings (384D)
./target/release/sutra-embedder embed \
  --text "Efficient model" \
  --dimensions 384

# Ultra-efficient embeddings (256D)
./target/release/sutra-embedder embed \
  --text "Ultra-efficient" \
  --dimensions 256
```

## 📜 Available Scripts

### `benchmark-clean.sh`
Filters ONNX Runtime verbose logs for production-ready output.

**Usage:**
```bash
./benchmark-clean.sh [benchmark-options]
```

**Examples:**
```bash
# Auto-detect hardware, 100 iterations
./benchmark-clean.sh --profile auto --iterations 100

# Desktop profile, 50 iterations  
./benchmark-clean.sh --profile desktop --iterations 50
```

**What it filters:**
- BFCArena creation logs
- Memory allocation logs
- Memory extension logs
- Memory reservation logs

## 🛠️ VS Code Tasks

Available tasks in VS Code (Cmd/Ctrl+Shift+B):

### Build Tasks
- **Build (Release)** - Default build task
- **Build (Debug)** - Debug build
- **Clean** - Clean build artifacts

### Test Tasks
- **Run Tests** - Run all tests
- **Run Benchmarks** - Run Criterion benchmarks

### Utility Tasks
- **Check Hardware** - Detect system capabilities
- **List Models** - Show available models with details
- **Download All Models** - Download all 6 models for offline use
- **Download Individual Model** - Select and download specific model
- **Compute Model Hashes** - Generate SHA256 for registry
- **Verify All Model Hashes** - Check SHA256 for all 6 models
- **Show Models Directory** - List downloaded model files
- **Check Models Storage Size** - Show total storage used

### Embedding Tasks
- **Generate Embeddings (768D)** - High-quality embeddings
- **Generate Embeddings (Efficient 384D)** - Efficient embeddings

### Benchmark Tasks
- **Run Benchmark Suite** - Standard benchmark (may show ONNX logs)
- **Run Benchmark Suite (Clean Output)** - Filtered benchmark output
- **Quick Benchmark (Clean, 10 iterations)** - Fast benchmark with clean output
- **Comprehensive Benchmark (All Dimensions)** - MTEB-style benchmark for all dimensions
- **Comprehensive Benchmark (Quick Test)** - Quick test with 256D, 384D, 768D
- **Comprehensive Benchmark Examples** - Run example benchmark suite script

### Code Quality Tasks
- **Format Code** - Run cargo fmt
- **Clippy (Lint)** - Run cargo clippy
- **Run with Debug Logs** - Run with RUST_LOG=debug

## 🔍 Environment Variables

### Logging Control
```bash
# Show only warnings
RUST_LOG=warn ./target/release/sutra-embedder benchmark --profile auto

# Show info logs (default)
RUST_LOG=info ./target/release/sutra-embedder benchmark --profile auto

# Debug mode (very verbose)
RUST_LOG=debug ./target/release/sutra-embedder embed --text "test"

# Trace everything
RUST_LOG=trace ./target/release/sutra-embedder embed --text "test"
```

### ONNX Runtime Configuration
```bash
# These are set automatically, but can be overridden:
ORT_LOGGING_LEVEL=4 ./target/release/sutra-embedder benchmark --profile auto
# Levels: 0=VERBOSE, 1=INFO, 2=WARNING, 3=ERROR, 4=FATAL
```

### GPU Control
```bash
# Disable GPU (force CPU mode)
CUDA_VISIBLE_DEVICES="" ./target/release/sutra-embedder embed \
  --text "CPU only" \
  --dimensions 384

# Specific GPU
CUDA_VISIBLE_DEVICES=0 ./target/release/sutra-embedder benchmark --profile auto
```

## 📊 Output Filtering

### Manual Log Filtering
```bash
# Filter BFCArena logs
./target/release/sutra-embedder benchmark --profile auto 2>&1 | \
  grep -v "BFCArena" | \
  grep -v "allocation"

# Only show benchmark results
./target/release/sutra-embedder benchmark --profile auto 2>&1 | \
  grep -E "(Benchmark Results|Latency|Throughput|Memory|Dimensions)"

# Suppress all stderr
./target/release/sutra-embedder benchmark --profile auto 2>/dev/null
```

### Save Output
```bash
# Save clean benchmark results
./benchmark-clean.sh --profile auto --iterations 100 > benchmark-results.txt

# Save both stdout and stderr
./target/release/sutra-embedder benchmark --profile auto &> full-output.log
```

## 🔗 Related Documentation

- **[LOGGING.md](LOGGING.md)** - Comprehensive logging documentation
- **[QUICKSTART.md](QUICKSTART.md)** - Getting started guide
- **[README.md](README.md)** - Full documentation
- **[.github/copilot-instructions.md](.github/copilot-instructions.md)** - Development guidelines

## 💡 Pro Tips

1. **Always use `--release` for accurate benchmarks**
   ```bash
   cargo build --release
   ```

2. **Use clean script for presentations/demos**
   ```bash
   ./benchmark-clean.sh --profile auto --iterations 100
   ```

3. **Check hardware first**
   ```bash
   ./target/release/sutra-embedder hardware
   ```

4. **Download models before benchmarking**
   ```bash
   ./target/release/sutra-embedder download --model all-MiniLM-L6-v2
   ```

5. **Verify model integrity**
   ```bash
   ./target/release/sutra-embedder hash --model all-MiniLM-L6-v2
   ```

## 🐛 Troubleshooting Commands

```bash
# Verify build is working
cargo build --release && ./target/release/sutra-embedder --help

# Check model cache location
ls -lh ~/Library/Caches/sutra-embedder/models/  # macOS
ls -lh ~/.cache/sutra-embedder/models/           # Linux

# Test with fallback embeddings (no model required)
./target/release/sutra-embedder embed --text "test" --dimensions 384

# Full debug output
RUST_LOG=debug ./target/release/sutra-embedder embed --text "test" 2>&1 | less
```

---

**Quick Start**: Run `./benchmark-clean.sh --profile auto --iterations 10` for a fast test! 🚀

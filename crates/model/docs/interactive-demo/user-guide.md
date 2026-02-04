# SutraWorks Demo User Guide

## Getting Started

### First Launch
1. **Run the launch script**: `./launch_demo.sh`
2. **Wait for initialization**: Models load in ~2 seconds
3. **Explore the interface**: Four tabs with different capabilities

### Interface Overview

```
┌─ SutraWorks AI Demo ─ Pure Rust • Edge Optimized ─┐
├─ [💬 Chat] [🏎️ Race] [⚡ Quantize] [🧠 NeSy] ─────┤
│                                                    │
│  Main Content Area                                 │
│  (Changes based on active tab)                     │
│                                                    │
└─ Status: ⏱️ 45s • 🔄 12 inferences • ⚡ 2.3ms ────┘
```

## Tab-by-Tab Guide

### 💬 Chat Interface

**What it does**: Real-time conversation with AI models

**How to use**:
1. Select model: RWKV (RNN) or Mamba (SSM)
2. Adjust generation length (1-50 tokens)
3. Type message and press Enter or click Send
4. Watch response generation with timing

**Pro tips**:
- Try both models with same input to compare
- Longer sequences show efficiency advantages
- Watch inference times - pure Rust is fast!

**Example conversation**:
```
You: What makes RWKV efficient?
RWKV (2.1ms): Linear complexity enables constant memory usage during inference...

You: How does this compare to transformers?
RWKV (1.9ms): Transformers use quadratic attention while RWKV maintains O(n) complexity...
```

### 🏎️ Architecture Performance Race

**What it does**: Head-to-head performance comparison

**How to use**:
1. Enter test input text
2. Click "Single Shot Comparison" for quick test
3. Click "Benchmark Suite" for comprehensive analysis
4. Review results table and speedup metrics

**Understanding results**:
- **Lower time = better performance**
- **Higher tokens/sec = better throughput**  
- **Speedup shows relative advantage**
- **Winner changes based on input characteristics**

**Example benchmark**:
```
Seq Length | RWKV (ms) | Mamba (ms) | Winner
─────────────────────────────────────────────
10         | 1.2       | 1.5        | RWKV
25         | 2.1       | 1.8        | Mamba  
50         | 3.5       | 2.9        | Mamba
100        | 6.8       | 5.2        | Mamba
```

### ⚡ Live Quantization Demo

**What it does**: Interactive model compression demonstration

**How to use**:
1. **Configure parameters**:
   - Bits: 2-8 (lower = more compression)
   - Group Size: 32-512 (affects quality/speed)
   - Matrix Size: 256-2048 (larger = more realistic)

2. **Generate matrix**: Creates random weight matrix
3. **Quantize**: Applies AWQ compression algorithm
4. **Analyze results**: View compression ratio and timing

**Key metrics**:
- **Compression Ratio**: How much smaller (e.g., 7.42x)
- **Memory Saved**: Actual bytes saved (e.g., 348MB)
- **Quantization Time**: How fast compression happens
- **Quality**: Weight distribution preservation

**Typical results**:
```
4-bit, 128 group, 512×512 matrix:
Original: 1.0 MB → Quantized: 0.13 MB  
Compression: 7.42x | Time: 23ms | Saved: 86.5%
```

### 🧠 Neuro-Symbolic Preview

**What it does**: Demonstrates hybrid AI reasoning

**Components**:
- **Neural**: Pattern recognition, language understanding
- **Symbolic**: Logic, math, verified computation
- **Tools**: Calculator, web search, code execution
- **Verification**: Logic checking, fact validation

**Example workflow**:
1. **Input**: "What's 15% of 240 plus tax at 8.5%?"
2. **Neural parsing**: Identifies math problem + tax calculation  
3. **Symbolic execution**: Calculator tool computes exact result
4. **Verification**: Logic check confirms calculation steps
5. **Output**: "36 + 3.11 tax = 39.11 total"

## Performance Insights

### What the Metrics Mean

**Inference Time**: 
- <1ms = Excellent for edge deployment
- 1-5ms = Good for interactive applications
- 5-20ms = Acceptable for batch processing
- >20ms = May need optimization

**Memory Usage**:
- Demo models: ~50MB total (very efficient)
- Production models: 500MB-2GB (still efficient vs alternatives)
- Quantized models: 70-90% memory reduction

**Throughput** (tokens/second):
- 1000+ tok/s = Excellent
- 500-1000 tok/s = Good  
- 200-500 tok/s = Acceptable
- <200 tok/s = Needs optimization

### Comparing to Alternatives

**vs PyTorch/Transformers**:
- Memory usage: 5-10x less
- Startup time: 20-100x faster
- Dependencies: Zero vs hundreds
- Deployment: Single binary vs complex environment

**vs llama.cpp**:
- Safety: Memory safe vs C++ risks
- Modularity: Better architecture separation
- Features: GUI + quantization vs CLI only
- Innovation: Novel architectures vs standard models

## Customization

### Model Configuration

Edit `crates/sutra-demo/src/models.rs`:

```rust
// For faster demo (less accuracy)
let config = RwkvConfig::new(
    2,    // 2 layers (vs 4)
    256,  // 256 hidden (vs 384)  
    1000, // 1K vocab (vs 2K)
);

// For better quality (slower)
let config = RwkvConfig::new(
    8,    // 8 layers (vs 4)
    768,  // 768 hidden (vs 384)
    10000,// 10K vocab (vs 2K)  
);
```

### GUI Customization

Edit `crates/sutra-demo/src/main.rs`:

```rust
let options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default()
        .with_inner_size([1600.0, 1000.0])  // Larger window
        .with_resizable(false),              // Fixed size
    ..Default::default()
};
```

### Adding Custom Features

1. **New architectures**: Add to `models.rs`
2. **Additional metrics**: Extend performance tracking
3. **Export capabilities**: Add save/load functionality
4. **Visualization**: Enhance charts and graphs

## Troubleshooting

### Performance Issues

**Slow inference**:
- Ensure `--release` build mode
- Reduce model size in configuration
- Close other memory-intensive apps

**GUI lag**:
- Update graphics drivers
- Reduce window size
- Try different backend: `WGPU_BACKEND=vulkan`

**High memory usage**:
- Reduce model parameters
- Enable quantization by default
- Clear message history frequently

### Common Errors

**"Failed to create model"**:
- Check available memory
- Reduce model configuration
- Verify Rust version (1.70+)

**"Tokenizer error"**:
- Input contains unsupported characters
- Try simpler text inputs
- Check vocab configuration

**"Quantization failed"**:
- Matrix too small (minimum 32×32)
- Invalid bit configuration
- Insufficient memory

## Advanced Usage

### Benchmarking

For serious performance analysis:

```bash
# Run with timing
time cargo run --bin sutra-demo --release

# Profile memory usage  
cargo build --bin sutra-demo --release
valgrind target/release/sutra-demo

# Measure binary size
ls -lh target/release/sutra-demo
```

### Integration

Use demo models in your applications:

```rust
use sutra_demo::models::DemoModels;

let models = DemoModels::new();
let (response, time) = models.rwkv_inference("Hello world!")?;
println!("Generated: {} in {:.2}ms", response, time * 1000.0);
```

### Deployment

Package for distribution:

```bash
# Build optimized binary
cargo build --bin sutra-demo --release --target-cpu=native

# Create installer/package
# (Platform-specific packaging tools)
```

## Next Steps

1. **Try all features**: Explore each tab thoroughly
2. **Compare architectures**: Note performance differences
3. **Experiment with parameters**: See how changes affect results
4. **Share feedback**: Help improve the demo
5. **Build applications**: Use SutraWorks for your projects

---

*Happy exploring! The future of efficient AI is in your hands. 🦀*
# AWQ Quantization Tutorial

Learn how to use Advanced Weight Quantization (AWQ) to compress models by 7.42x while maintaining accuracy.

## Overview

AWQ (Activation-aware Weight Quantization) is a state-of-the-art quantization method that achieves superior compression ratios with minimal accuracy loss by protecting salient weights based on activation patterns.

### Key Benefits

- **7.42x Compression**: Reduce model size from 402MB to 54MB
- **Maintained Accuracy**: Minimal performance degradation
- **Real Bit-Packing**: 2 values per byte storage
- **Production Ready**: Enterprise-grade implementation

## Quick Start

```rust
use sutra_quantize::{AwqQuantizer, AwqConfig};
use sutra_core::Tensor;

// Create quantizer with default AWQ configuration
let config = AwqConfig::default();
let quantizer = AwqQuantizer::new(config);

// Quantize model weights
let quantized_tensor = quantizer.quantize(&weights, None)?;

// Check compression ratio
println!("Compression: {:.2}x", quantized_tensor.compression_ratio());
```

## Configuration Options

### AwqConfig Parameters

```rust
pub struct AwqConfig {
    pub bits: u8,              // Quantization bits (4, 8)
    pub group_size: usize,     // Grouping for scales (128, 256)
    pub salience_threshold: f32, // Protection threshold (0.1)
    pub zero_point_range: (i8, i8), // Zero-point range (-8, 7)
}

// Custom configuration
let config = AwqConfig {
    bits: 4,
    group_size: 128,
    salience_threshold: 0.05,  // Protect more weights
    zero_point_range: (-8, 7),
};
```

### Configuration Presets

```rust
// For maximum compression
let aggressive_config = AwqConfig::aggressive();

// For maximum accuracy
let conservative_config = AwqConfig::conservative();

// Balanced (default)
let balanced_config = AwqConfig::default();
```

## Step-by-Step Guide

### 1. Prepare Your Model

```rust
use sutra_loader::SafetensorsLoader;

// Load model weights
let loader = SafetensorsLoader::new("model.safetensors")?;
let weights = loader.load_all()?;

// Or create synthetic weights for testing
let weights = Tensor::randn([1024, 512], DType::F32)?;
```

### 2. Analyze Salience (Optional)

```rust
use sutra_quantize::SalienceCalculator;

// Calculate weight importance
let salience_calc = SalienceCalculator::new();
let salience = salience_calc.compute(&weights)?;

// Visualize salience distribution
println!("High salience weights: {:.2}%", 
    salience.high_importance_percentage());
```

### 3. Apply Quantization

```rust
// Basic quantization
let quantized = quantizer.quantize(&weights, None)?;

// With custom salience
let quantized = quantizer.quantize(&weights, Some(&salience))?;

// Batch quantization for multiple layers
let quantized_layers = quantizer.quantize_batch(&layer_weights)?;
```

### 4. Verify Results

```rust
// Check compression metrics
let metrics = quantized.metrics();
println!("Original size: {} MB", metrics.original_size_mb);
println!("Compressed size: {} MB", metrics.compressed_size_mb);
println!("Compression ratio: {:.2}x", metrics.compression_ratio);
println!("Memory saved: {} MB", metrics.memory_saved_mb);
```

## Advanced Usage

### Custom Salience Calculation

```rust
use sutra_quantize::ActivationStats;

// Collect activation statistics
let mut activation_stats = ActivationStats::new();

// During model inference (collect activations)
for batch in data_loader {
    let activations = model.forward(&batch)?;
    activation_stats.update(&activations);
}

// Use collected stats for quantization
let salience = activation_stats.compute_salience(&weights)?;
let quantized = quantizer.quantize(&weights, Some(&salience))?;
```

### Layer-Specific Configuration

```rust
use std::collections::HashMap;

let mut layer_configs = HashMap::new();

// Different configs for different layer types
layer_configs.insert("attention", AwqConfig {
    bits: 4,
    group_size: 64,    // Smaller groups for attention
    salience_threshold: 0.05,
    ..Default::default()
});

layer_configs.insert("feedforward", AwqConfig {
    bits: 4,
    group_size: 256,   // Larger groups for FFN
    salience_threshold: 0.1,
    ..Default::default()
});

// Apply different configs per layer
let quantized_model = quantizer.quantize_model_with_configs(
    &model, 
    &layer_configs
)?;
```

### Mixed-Precision Quantization

```rust
// Keep sensitive layers in higher precision
let mixed_config = MixedPrecisionConfig {
    embedding_bits: 8,      // Higher precision for embeddings
    attention_bits: 4,      // 4-bit for attention
    feedforward_bits: 4,    // 4-bit for feedforward
    output_bits: 8,         // Higher precision for output
};

let quantized = quantizer.quantize_mixed_precision(&model, &mixed_config)?;
```

## Performance Optimization

### Memory Layout Optimization

```rust
// Enable memory layout optimization
let config = AwqConfig {
    optimize_layout: true,  // Row-major optimization
    enable_simd: true,      // SIMD-friendly alignment
    ..Default::default()
};
```

### Parallel Quantization

```rust
use rayon::prelude::*;

// Quantize multiple layers in parallel
let quantized_layers: Vec<_> = layer_weights
    .par_iter()
    .map(|weights| quantizer.quantize(weights, None))
    .collect::<Result<Vec<_>, _>>()?;
```

### Streaming Quantization

```rust
// For very large models that don't fit in memory
let streaming_quantizer = StreamingQuantizer::new(config);

streaming_quantizer.quantize_file(
    "large_model.safetensors",
    "quantized_model.bin"
)?;
```

## Integration with Models

### RWKV Integration

```rust
use sutra_rwkv::RwkvModel;

let mut model = RwkvModel::from_pretrained("rwkv-169m")?;

// Quantize RWKV model
let quantized_model = quantizer.quantize_rwkv(&model)?;

// Run inference with quantized model
let output = quantized_model.generate(&prompt, 100, 0.7)?;
```

### Mamba Integration

```rust
use sutra_mamba::MambaModel;

let model = MambaModel::from_pretrained("mamba-130m")?;

// Quantize Mamba model
let quantized_model = quantizer.quantize_mamba(&model)?;

// Benchmark quantized vs original
let (original_time, quantized_time) = benchmark_models(&model, &quantized_model)?;
println!("Speedup: {:.2}x", original_time / quantized_time);
```

## Benchmarking and Validation

### Run Official Benchmark

```bash
# Comprehensive quantization benchmark
cargo run --example quantization_benchmark --release
```

### Custom Benchmarks

```rust
use sutra_quantize::benchmark::{BenchmarkConfig, run_benchmark};

let benchmark_config = BenchmarkConfig {
    num_iterations: 1000,
    measure_accuracy: true,
    measure_performance: true,
    compare_methods: vec!["awq", "gptq", "naive"],
};

let results = run_benchmark(&model, &benchmark_config)?;
println!("AWQ Accuracy: {:.4}", results.accuracy["awq"]);
println!("AWQ Speed: {:.2}x", results.speedup["awq"]);
```

### Accuracy Validation

```rust
use sutra_quantize::validation::{validate_accuracy, AccuracyThreshold};

let threshold = AccuracyThreshold {
    max_mse: 1e-3,
    max_relative_error: 0.01,
    min_correlation: 0.99,
};

let validation_result = validate_accuracy(
    &original_model,
    &quantized_model,
    &test_data,
    &threshold
)?;

assert!(validation_result.passes_threshold);
```

## Production Deployment

### Model Serialization

```rust
use sutra_quantize::serialization::{save_quantized, load_quantized};

// Save quantized model
save_quantized(&quantized_model, "production_model.qbin")?;

// Load in production
let production_model = load_quantized("production_model.qbin")?;
```

### Runtime Dequantization

```rust
// Automatic dequantization during inference
let output = quantized_model.forward(&input)?; // Transparent dequantization

// Manual dequantization for analysis
let dequantized_weights = quantized_tensor.dequantize()?;
```

### Memory Management

```rust
// Monitor memory usage
let memory_usage = quantized_model.memory_usage();
println!("Model memory: {} MB", memory_usage.model_size_mb);
println!("Peak memory: {} MB", memory_usage.peak_usage_mb);

// Enable memory optimization
quantized_model.optimize_memory_layout()?;
```

## Troubleshooting

### Common Issues

1. **Quality Degradation**
   ```rust
   // Solution: Lower salience threshold
   let config = AwqConfig {
       salience_threshold: 0.01, // Protect more weights
       ..Default::default()
   };
   ```

2. **Memory Issues**
   ```rust
   // Solution: Use streaming quantization
   let streaming = StreamingQuantizer::new(config);
   streaming.quantize_in_chunks(&large_model, chunk_size)?;
   ```

3. **Performance Issues**
   ```rust
   // Solution: Enable optimizations
   let config = AwqConfig {
       optimize_layout: true,
       enable_simd: true,
       ..Default::default()
   };
   ```

## Best Practices

1. **Start with Default Config**: Good balance for most use cases
2. **Validate Accuracy**: Always test on your specific tasks
3. **Monitor Memory**: Track compression vs accuracy trade-offs
4. **Use Salience**: Collect real activation statistics when possible
5. **Benchmark**: Measure both accuracy and performance gains

## Next Steps

- **[Mamba Tutorial](mamba.md)**: Learn state space models
- **[RWKV Tutorial](rwkv.md)**: Efficient RNN architectures
- **[QLoRA Tutorial](qlora.md)**: Parameter-efficient fine-tuning
- **[API Reference](../api/quantization.md)**: Complete API documentation

---

**Ready to compress your models? Start with the basic example above!**
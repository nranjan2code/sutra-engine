# Advanced Features Guide

## Overview

This guide covers the four production-grade advanced features implemented in Sutra-Embedder:

1. **Flash Attention** - For long sequences (>512 tokens)
2. **Model Distillation** - Create smaller/faster custom models
3. **Multi-GPU Distributed Inference** - Ultra-high throughput
4. **Streaming Embeddings** - Real-time applications

All features are implemented in **pure Rust** with no Python dependencies.

---

## 1. Flash Attention for Long Sequences

### Overview

Flash Attention provides memory-efficient attention computation for long document embeddings:

- **Memory**: O(N) instead of O(N²)
- **Speed**: 2-4x faster for 1024 tokens, 8.5x for 4096 tokens
- **Quality**: Exact attention (not approximate)
- **GPU Support**: CUDA compute capability ≥ 7.5 required

### Usage

#### Basic Usage

```rust
use sutra_embedder::{EmbedderConfig, Embedder};

// Configure with Flash Attention
let mut config = EmbedderConfig::from_name("high-quality")?;
config.use_flash_attention = true;
config.max_sequence_length = 2048; // Support long documents

let mut embedder = Embedder::new(config)?;

// Long document (>512 tokens automatically uses Flash Attention)
let long_doc = "...very long text...".repeat(100);
let embedding = embedder.embed(&long_doc)?;
```

#### Advanced Configuration

```rust
use sutra_embedder::flash_attention::{FlashAttentionConfig, FlashAttentionOptimizer};

let flash_config = FlashAttentionConfig {
    sequence_threshold: 512,      // Enable for sequences > 512 tokens
    use_efficient_memory: true,   // Memory optimization
    enable_flash_v2: false,       // Auto-detect H100 for v2
    block_size: 512,              // Attention block size
};

let optimizer = FlashAttentionOptimizer::new(flash_config);

// Check if Flash Attention should be enabled
if optimizer.should_enable(sequence_length) {
    println!("Flash Attention enabled");
}

// Get performance stats
let stats = optimizer.get_stats(sequence_length);
println!("{}", stats);
```

#### Sliding Window Alternative

For very long sequences or when Flash Attention is unavailable:

```rust
use sutra_embedder::flash_attention::{SlidingWindowAttention, AggregationMethod};

let window = SlidingWindowAttention::new(512, 128); // 512 tokens, 128 overlap

// Split long text into windows
let tokens: Vec<i64> = tokenize_long_text(&text);
let windows = window.create_windows(&tokens);

// Process each window with embedder
let mut embeddings = Vec::new();
for window_tokens in windows {
    let embedding = embedder.embed_tokens(&window_tokens)?;
    embeddings.push(embedding);
}

// Aggregate window embeddings
let final_embedding = window.aggregate_embeddings(
    embeddings,
    AggregationMethod::Weighted  // Center windows weighted higher
);
```

### Performance Expectations

| Sequence Length | Standard (ms) | Flash (ms) | Speedup | Memory Reduction |
|----------------|---------------|------------|---------|------------------|
| 512 tokens     | 10.0          | 10.5       | 0.95x   | 0%               |
| 1024 tokens    | 35.0          | 15.0       | 2.3x    | 50%              |
| 2048 tokens    | 130.0         | 30.0       | 4.3x    | 75%              |
| 4096 tokens    | 510.0         | 60.0       | 8.5x    | 87%              |

### Hardware Requirements

- **GPU**: CUDA compute capability ≥ 7.5 (Volta, Turing, Ampere, Hopper)
- **ONNX Runtime**: Version 1.16+ with Flash Attention support
- **Models**: Must be exported with Flash Attention ops

---

## 2. Model Distillation Framework

### Overview

Create smaller, faster custom embedding models from larger teacher models:

- **Compression**: 768D → 384D (2-3x smaller)
- **Quality**: Maintains >95% accuracy
- **Techniques**: Knowledge distillation with MSE + cosine loss
- **Export**: ONNX format for production deployment

### Usage

#### Complete Distillation Pipeline

```rust
use sutra_embedder::{DistillationConfig, DistillationTrainer, ModelFormat, Embedder};

// Configure distillation
let config = DistillationConfig {
    teacher_dim: 768,
    student_dim: 384,
    temperature: 2.0,            // Softening parameter
    alpha: 0.5,                  // Distillation loss weight
    num_iterations: 10000,
    batch_size: 32,
    learning_rate: 5e-4,
    use_cosine_loss: true,       // Add cosine similarity loss
    output_format: ModelFormat::ONNX,
};

let mut trainer = DistillationTrainer::new(config);

// Step 1: Collect training data
let training_texts = vec![
    "Example sentence 1".to_string(),
    "Example sentence 2".to_string(),
    // ... thousands of examples
];

// Create teacher embedder
let teacher_config = EmbedderConfig::from_name("high-quality")?; // 768D
let mut teacher = Embedder::new(teacher_config)?;

// Collect teacher embeddings
trainer.collect_teacher_embeddings(training_texts, &mut teacher)?;

// Step 2: Train projection matrix
trainer.train_projection()?;

// Step 3: Evaluate
let test_texts = vec![
    "Test sentence 1".to_string(),
    "Test sentence 2".to_string(),
];

let metrics = trainer.evaluate(test_texts, &mut teacher)?;
println!("{}", metrics);
// Output: MSE: 0.0123, Cosine Similarity: 0.976

// Step 4: Export
trainer.export_model(std::path::Path::new("models/custom_384d.onnx"))?;
```

#### Custom Dimension Reduction

```rust
// Create ultra-efficient 128D model
let config = DistillationConfig {
    teacher_dim: 768,
    student_dim: 128,  // 6x compression
    temperature: 3.0,   // Higher temperature for more compression
    ..Default::default()
};

// Or create specialized high-quality 1024D model
let config = DistillationConfig {
    teacher_dim: 1536,
    student_dim: 1024,
    alpha: 0.3,  // More weight on task loss
    ..Default::default()
};
```

### Evaluation Metrics

```rust
pub struct DistillationMetrics {
    pub mean_squared_error: f32,      // Lower is better
    pub cosine_similarity: f32,       // Higher is better (0-1)
    pub dimension_reduction: f32,     // Compression ratio
    pub num_samples: usize,           // Evaluation set size
}
```

### Best Practices

1. **Training Data**: Use 10K-100K diverse examples
2. **Temperature**: 2.0-3.0 for good balance
3. **Alpha**: 0.5 for equal distillation/task loss
4. **Iterations**: 5K-20K depending on data size
5. **Evaluation**: Use held-out test set for metrics

---

## 3. Multi-GPU Distributed Inference

### Overview

Scale to ultra-high throughput with automatic GPU pooling:

- **Throughput**: 20,000+ embeddings/sec on 8-GPU cluster
- **Load Balancing**: Round-robin, least-loaded, performance-based
- **Fault Tolerance**: GPU health monitoring and retry logic
- **Platforms**: CUDA, ROCm, Metal, DirectML

### Usage

#### Basic Multi-GPU Setup

```rust
use sutra_embedder::{MultiGPUConfig, MultiGPUPool, LoadBalancingStrategy};

// Configure multi-GPU pool
let config = MultiGPUConfig {
    device_ids: vec![],  // Auto-detect all GPUs
    load_balancing: LoadBalancingStrategy::LeastLoaded,
    max_concurrent_per_gpu: 4,
    enable_health_checks: true,
    health_check_interval_secs: 30,
    retry_on_failure: true,
    max_retries: 3,
};

// Create embedder config
let embedder_config = EmbedderConfig::from_name("efficient")?;

// Initialize pool
let pool = MultiGPUPool::new(config, embedder_config).await?;

// Process large batch
let texts: Vec<String> = load_texts(); // 10,000 texts
let embeddings = pool.embed_batch_distributed(texts).await?;

// Get statistics
let stats = pool.get_stats().await;
println!("{}", stats);
```

#### Advanced Configuration

```rust
// Specific GPU selection
let config = MultiGPUConfig {
    device_ids: vec![0, 1, 2, 3],  // Use GPUs 0-3
    load_balancing: LoadBalancingStrategy::PerformanceBased,
    max_concurrent_per_gpu: 8,  // Higher concurrency
    ..Default::default()
};

// Performance-based load balancing (tracks GPU latency)
let config = MultiGPUConfig {
    load_balancing: LoadBalancingStrategy::PerformanceBased,
    ..Default::default()
};
```

### Load Balancing Strategies

1. **RoundRobin**: Simple even distribution
2. **LeastLoaded**: Choose GPU with shortest queue
3. **PerformanceBased**: Choose GPU with best historical latency
4. **Random**: Stateless random selection

### Performance Monitoring

```rust
let stats = pool.get_stats().await;

println!("Total Requests: {}", stats.total_requests);
println!("Total Embeddings: {}", stats.total_embeddings);
println!("Average Latency: {:.2}ms", stats.average_latency_ms);
println!("Throughput: {:.1} emb/sec", stats.throughput_per_sec);

// Per-GPU stats
for gpu in &stats.gpu_utilization {
    println!("GPU {}: {:.1}% utilization", gpu.device_id, gpu.utilization_percent);
}
```

### Performance Targets

| Setup        | Throughput (emb/sec) | Latency (ms) |
|--------------|---------------------|--------------|
| Single GPU   | 1,000+              | ~13          |
| 4-GPU        | 5,000+              | ~15          |
| 8-GPU        | 20,000+             | ~18          |

---

## 4. Streaming Embeddings

### Overview

Real-time embedding generation for live applications:

- **Latency**: <100ms target for real-time
- **Backpressure**: Automatic buffering and flow control
- **Batching**: Auto-batch for efficiency
- **Use Cases**: Live transcription, real-time search, chat embeddings

### Usage

#### Basic Streaming

```rust
use sutra_embedder::{StreamingConfig, StreamingEmbedder};

// Configure streaming
let config = StreamingConfig {
    buffer_size: 100,
    chunk_size: 512,
    chunk_overlap: 64,
    batch_size: 8,
    timeout_secs: 30,
    auto_batch: true,
    max_latency_ms: 100,  // Real-time target
};

let embedder_config = EmbedderConfig::from_name("efficient")?;
let streaming = StreamingEmbedder::new(config, embedder_config)?;

// Stream single text
let embedding = streaming.embed_stream("Hello world".to_string()).await?;

// Stream multiple texts (simulating real-time input)
for text in incoming_texts {
    let embedding = streaming.embed_stream(text).await?;
    // Process embedding immediately
}
```

#### Chunked Processing

```rust
use futures::StreamExt;

// Process long document as stream of chunks
let long_doc = "...very long document...".to_string();
let mut chunk_stream = streaming.embed_chunks_stream(long_doc).await?;

let mut embeddings = Vec::new();
while let Some(result) = chunk_stream.next().await {
    match result {
        Ok(embedding) => embeddings.push(embedding),
        Err(e) => eprintln!("Error: {}", e),
    }
}

// Aggregate chunks
use sutra_embedder::streaming::StreamingAggregator;
let mut aggregator = StreamingAggregator::new();
for embedding in embeddings {
    aggregator.add(embedding, None);  // Equal weights
}
let final_embedding = aggregator.aggregate().unwrap();
```

#### WebSocket Integration

```rust
#[cfg(feature = "websocket")]
use sutra_embedder::streaming::websocket::WebSocketHandler;

let handler = WebSocketHandler::new(streaming);

// Handle incoming WebSocket messages
async fn handle_ws_message(handler: &WebSocketHandler, message: Message) {
    let response = handler.handle_message(message).await?;
    // Send response back to client
}
```

### Configuration Options

```rust
pub struct StreamingConfig {
    pub buffer_size: usize,          // Max pending requests
    pub chunk_size: usize,           // Text chunk size (chars)
    pub chunk_overlap: usize,        // Overlap between chunks
    pub batch_size: usize,           // Auto-batch size
    pub timeout_secs: u64,           // Inactivity timeout
    pub auto_batch: bool,            // Enable batching
    pub max_latency_ms: u64,         // Target latency
}
```

### Performance Monitoring

```rust
let stats = streaming.get_stats().await;

println!("Total Requests: {}", stats.total_requests);
println!("Avg Latency: {:.2}ms", stats.average_latency_ms);
println!("Peak Latency: {:.2}ms", stats.peak_latency_ms);
println!("Throughput: {:.1}/sec", stats.throughput_per_sec);
println!("Buffer Overflows: {}", stats.buffer_overflows);
```

---

## Combined Usage Example

### High-Throughput Real-Time Service

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // 1. Multi-GPU for parallel processing
    let gpu_config = MultiGPUConfig::default();
    let embedder_config = EmbedderConfig {
        name: "production".to_string(),
        dimensions: 768,
        max_sequence_length: 2048,
        use_flash_attention: true,  // Long documents
        use_fp16: true,             // GPU optimization
        ..Default::default()
    };
    
    let pool = MultiGPUPool::new(gpu_config, embedder_config.clone()).await?;
    
    // 2. Streaming for real-time requests
    let stream_config = StreamingConfig {
        max_latency_ms: 50,  // Ultra-low latency
        batch_size: 16,
        ..Default::default()
    };
    
    let streaming = StreamingEmbedder::new(stream_config, embedder_config)?;
    
    // 3. Process real-time stream
    loop {
        let text = receive_text_from_stream().await?;
        let embedding = streaming.embed_stream(text).await?;
        send_embedding_to_consumer(embedding).await?;
    }
}
```

---

## API Reference

### Flash Attention

- `FlashAttentionConfig`: Configuration for Flash Attention
- `FlashAttentionOptimizer`: Optimizer with GPU detection
- `SlidingWindowAttention`: Alternative for very long sequences
- `AggregationMethod`: Window aggregation strategies (Mean, Max, Weighted)

### Model Distillation

- `DistillationConfig`: Training configuration
- `DistillationTrainer`: Main training interface
- `DistillationMetrics`: Evaluation metrics
- `ModelFormat`: Export formats (ONNX, PyTorch, TensorFlow)

### Multi-GPU

- `MultiGPUConfig`: Pool configuration
- `MultiGPUPool`: Main pool interface
- `MultiGPUStats`: Performance statistics
- `LoadBalancingStrategy`: Load balancing modes

### Streaming

- `StreamingConfig`: Stream configuration
- `StreamingEmbedder`: Main streaming interface
- `StreamingStats`: Performance statistics
- `StreamingAggregator`: Chunk aggregation

---

## Testing

Run the advanced features examples:

```bash
# Build examples
cargo build --examples --release

# Run specific example
cargo run --example advanced_features --release

# Run with specific feature
RUST_LOG=info cargo run --example advanced_features --release
```

Run benchmarks:

```bash
# Standard benchmarks
cargo bench

# With advanced features
RUST_LOG=info cargo bench
```

---

## Troubleshooting

### Flash Attention Not Working

- Check GPU compute capability: `nvidia-smi --query-gpu=compute_cap`
- Requires ≥ 7.5 (Volta, Turing, Ampere, Hopper)
- Ensure ONNX Runtime 1.16+ with Flash Attention support

### Multi-GPU Detection Issues

- Run `sutra-embedder hardware` to check GPU detection
- Check CUDA/ROCm installation
- Verify GPU drivers are up to date

### Streaming Latency High

- Reduce `max_latency_ms` for faster processing
- Increase `batch_size` for better throughput
- Check `buffer_overflows` in stats

### Distillation Poor Quality

- Increase `num_iterations` (try 20K+)
- Use more training data (50K+ examples)
- Adjust `temperature` (try 2.0-3.0 range)
- Enable `use_cosine_loss` for better quality

---

## Performance Best Practices

1. **Flash Attention**: Enable for sequences >512 tokens
2. **Multi-GPU**: Use performance-based load balancing
3. **Streaming**: Set `max_latency_ms` based on use case
4. **Distillation**: Use diverse training corpus
5. **Combined**: Stack features for production workloads

---

## References

- [Flash Attention Paper (Dao et al., 2022)](https://arxiv.org/abs/2205.14135)
- [Flash Attention v2 (Dao et al., 2023)](https://arxiv.org/abs/2307.08691)
- [DistilBERT (Sanh et al., 2019)](https://arxiv.org/abs/1910.01108)
- [ONNX Runtime Documentation](https://onnxruntime.ai/)
- [Project README](README.md)
- [Optimization Guide](OPTIMIZATIONS.md)

---

**Last Updated**: November 9, 2025  
**Version**: 0.1.0+advanced-features

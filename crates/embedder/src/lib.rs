pub mod benchmark;
pub mod comprehensive_benchmark;
pub mod custom_ops;
pub mod distillation;
pub mod embedder;
pub mod flash_attention;
pub mod hardware;
pub mod model_registry;
pub mod multi_gpu;
pub mod ollama_client;
pub mod optimization;
pub mod quantization;
pub mod streaming;

pub use benchmark::{BenchmarkResult, BenchmarkSuite};
pub use comprehensive_benchmark::{
    BenchmarkDataGenerator, ComprehensiveBenchmarkSuite, DimensionBenchmarkResult,
    PerformanceMetrics, QualityMetrics, TextCategory,
};
pub use custom_ops::{fused_mean_pool_and_normalize, fused_truncate_and_quantize};
pub use distillation::{DistillationConfig, DistillationMetrics, DistillationTrainer, ModelFormat};
pub use embedder::{
    AsyncBatchQueue, BoundedAsyncBatchQueue, Embedder, EmbedderConfig, QuantizationType,
};
pub use flash_attention::{
    AggregationMethod, FlashAttentionConfig, FlashAttentionOptimizer, FlashAttentionStats,
    SlidingWindowAttention,
};
pub use hardware::HardwareProfile;
pub use model_registry::{ModelInfo, ModelRegistry};
pub use multi_gpu::{LoadBalancingStrategy, MultiGPUConfig, MultiGPUPool, MultiGPUStats};
pub use ollama_client::{EmbeddingQualityResult, OllamaBenchmarkResult, OllamaClient};
pub use quantization::{QuantizationConfig, QuantizationStats};
pub use streaming::{StreamingConfig, StreamingEmbedder, StreamingStats};
pub mod server;

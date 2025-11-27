use async_trait::async_trait;
use anyhow::Result;

/// Trait for embedding providers
///
/// This allows swapping the HTTP-based embedding service (Server Edition)
/// with a local ONNX-based provider (Desktop Edition).
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Generate embedding for a single text
    async fn generate(&self, text: &str, normalize: bool) -> Result<Vec<f32>>;
    
    /// Generate embeddings for multiple texts in batch
    async fn generate_batch(&self, texts: &[String], normalize: bool) -> Vec<Option<Vec<f32>>>;
}

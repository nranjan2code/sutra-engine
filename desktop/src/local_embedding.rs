use anyhow::Result;
use async_trait::async_trait;
use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};
use std::sync::Arc;
use sutra_storage::embedding_provider::EmbeddingProvider;
use tracing::{info, warn};

/// Local embedding provider using ONNX Runtime via fastembed
pub struct LocalEmbeddingProvider {
    model: Arc<TextEmbedding>,
}

impl LocalEmbeddingProvider {
    /// Create a new local embedding provider
    /// 
    /// This will download the model if it doesn't exist.
    /// Uses "nomic-embed-text-v1.5" by default to match server edition.
    pub fn new() -> Result<Self> {
        info!("Initializing LocalEmbeddingProvider (fastembed)...");
        
        let mut options = InitOptions::default();
        options.model_name = EmbeddingModel::NomicEmbedTextV15;
        options.show_download_progress = true;
        
        let model = TextEmbedding::try_new(options)?;
        
        info!("Local embedding model loaded successfully");
        
        Ok(Self {
            model: Arc::new(model),
        })
    }
}

#[async_trait]
impl EmbeddingProvider for LocalEmbeddingProvider {
    async fn generate(&self, text: &str, _normalize: bool) -> Result<Vec<f32>> {
        // fastembed handles normalization by default for most models
        let embeddings = self.model.embed(vec![text], None)?;
        
        embeddings.into_iter().next()
            .ok_or_else(|| anyhow::anyhow!("Failed to generate embedding"))
    }

    async fn generate_batch(&self, texts: &[String], _normalize: bool) -> Vec<Option<Vec<f32>>> {
        match self.model.embed(texts.to_vec(), None) {
            Ok(embeddings) => embeddings.into_iter().map(Some).collect(),
            Err(e) => {
                warn!("Batch embedding failed: {}", e);
                vec![None; texts.len()]
            }
        }
    }
}

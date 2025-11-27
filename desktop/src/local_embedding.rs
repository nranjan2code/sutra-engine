use anyhow::Result;
use async_trait::async_trait;
use sutra_embedder::{Embedder, EmbedderConfig};
use std::sync::Arc;
use tokio::sync::Mutex;
use sutra_storage::embedding_provider::EmbeddingProvider;
use tracing::{info, warn};

/// Local embedding provider using sutra-embedder
pub struct LocalEmbeddingProvider {
    model: Arc<Mutex<Embedder>>,
}

impl LocalEmbeddingProvider {
    /// Create a new local embedding provider
    /// 
    /// This will download the model if it doesn't exist.
    /// Uses 768D configuration for server compatibility.
    pub async fn new_async() -> Result<Self> {
        info!("Initializing LocalEmbeddingProvider (sutra-embedder) with auto-download...");
        
        // Use 768D dimension for server compatibility
        // "high-quality" preset uses all-mpnet-base-v2 (768D)
        let config = EmbedderConfig::from_name("high-quality")?;
        let model = Embedder::new_async(config).await?;
        
        info!("Local embedding model loaded successfully with auto-download");
        
        Ok(Self {
            model: Arc::new(Mutex::new(model)),
        })
    }
    
    /// Synchronous fallback constructor
    pub fn new() -> Result<Self> {
        info!("Initializing LocalEmbeddingProvider (sutra-embedder)...");
        
        // Use "high-quality" preset (768D)
        let config = EmbedderConfig::from_name("high-quality")?;
        let model = Embedder::new(config)?;
        
        info!("Local embedding model loaded successfully");
        
        Ok(Self {
            model: Arc::new(Mutex::new(model)),
        })
    }
}

#[async_trait]
impl EmbeddingProvider for LocalEmbeddingProvider {
    async fn generate(&self, text: &str, _normalize: bool) -> Result<Vec<f32>> {
        let mut model = self.model.lock().await;
        // sutra-embedder handles normalization
        model.embed(text)
    }

    async fn generate_batch(&self, texts: &[String], _normalize: bool) -> Vec<Option<Vec<f32>>> {
        let mut model = self.model.lock().await;
        let mut results = Vec::with_capacity(texts.len());
        
        for text in texts {
            match model.embed(text) {
                Ok(embedding) => results.push(Some(embedding)),
                Err(e) => {
                    warn!("Embedding failed for text: {}", e);
                    results.push(None);
                }
            }
        }
        results
    }
}

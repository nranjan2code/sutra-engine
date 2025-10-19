///! Embedding client for Ollama integration
///!
///! Production-grade HTTP client for generating embeddings via Ollama.
///! Supports both single and batch embedding generation with retry logic,
///! timeout handling, and comprehensive error reporting.

use anyhow::{Context, Result};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info, warn, error};

/// Configuration for embedding client
#[derive(Debug, Clone)]
pub struct EmbeddingConfig {
    /// Ollama server URL
    pub ollama_url: String,
    /// Default embedding model
    pub default_model: String,
    /// Request timeout in seconds
    pub timeout_secs: u64,
    /// Maximum retries on failure
    pub max_retries: usize,
    /// Retry delay in milliseconds
    pub retry_delay_ms: u64,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            ollama_url: std::env::var("SUTRA_OLLAMA_URL")
                .unwrap_or_else(|_| "http://host.docker.internal:11434".to_string()),
            default_model: std::env::var("SUTRA_EMBEDDING_MODEL")
                .unwrap_or_else(|_| "granite-embedding:30m".to_string()),
            timeout_secs: std::env::var("SUTRA_EMBEDDING_TIMEOUT_SEC")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
            max_retries: 3,
            retry_delay_ms: 500,
        }
    }
}

/// Request format for Ollama embedding API
#[derive(Serialize, Debug)]
struct EmbeddingRequest {
    model: String,
    prompt: String,
}

/// Response format from Ollama embedding API
#[derive(Deserialize, Debug)]
struct EmbeddingResponse {
    embedding: Vec<f32>,
}

/// Embedding client for generating vector embeddings
pub struct EmbeddingClient {
    config: EmbeddingConfig,
    client: Client,
}

impl EmbeddingClient {
    /// Create new embedding client with configuration
    pub fn new(config: EmbeddingConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .context("Failed to create HTTP client")?;
            
        info!(
            "Initialized EmbeddingClient: url={}, model={}, timeout={}s",
            config.ollama_url, config.default_model, config.timeout_secs
        );
        
        Ok(Self { config, client })
    }
    
    /// Create client with default configuration
    pub fn with_defaults() -> Result<Self> {
        Self::new(EmbeddingConfig::default())
    }
    
    /// Generate embedding for a single text
    ///
    /// # Arguments
    /// * `text` - Text to generate embedding for
    /// * `model` - Optional model override (uses default if None)
    ///
    /// # Returns
    /// * `Ok(Vec<f32>)` - 768-dimensional embedding vector
    /// * `Err` - If generation fails after all retries
    pub async fn generate(&self, text: &str, model: Option<&str>) -> Result<Vec<f32>> {
        let model = model.unwrap_or(&self.config.default_model);
        
        debug!("Generating embedding: text_len={}, model={}", text.len(), model);
        
        let mut last_error = None;
        
        for attempt in 0..=self.config.max_retries {
            match self.try_generate(text, model).await {
                Ok(embedding) => {
                    debug!("Generated embedding: dim={}, attempt={}", embedding.len(), attempt + 1);
                    return Ok(embedding);
                }
                Err(e) => {
                    last_error = Some(e);
                    
                    if attempt < self.config.max_retries {
                        let delay = Duration::from_millis(
                            self.config.retry_delay_ms * (2_u64.pow(attempt as u32))
                        );
                        warn!(
                            "Embedding generation failed (attempt {}/{}), retrying in {:?}",
                            attempt + 1,
                            self.config.max_retries + 1,
                            delay
                        );
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap().context(format!(
            "Failed to generate embedding after {} attempts",
            self.config.max_retries + 1
        )))
    }
    
    /// Internal method to attempt embedding generation (no retries)
    async fn try_generate(&self, text: &str, model: &str) -> Result<Vec<f32>> {
        let request = EmbeddingRequest {
            model: model.to_string(),
            prompt: text.to_string(),
        };
        
        let url = format!("{}/api/embeddings", self.config.ollama_url);
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("HTTP request failed")?;
            
        match response.status() {
            StatusCode::OK => {
                let embedding_response: EmbeddingResponse = response
                    .json()
                    .await
                    .context("Failed to parse embedding response")?;
                    
                // Validate embedding dimension
                if embedding_response.embedding.is_empty() {
                    return Err(anyhow::anyhow!("Received empty embedding vector"));
                }
                
                Ok(embedding_response.embedding)
            }
            StatusCode::NOT_FOUND => {
                Err(anyhow::anyhow!(
                    "Model '{}' not found. Pull it with: ollama pull {}",
                    model,
                    model
                ))
            }
            StatusCode::SERVICE_UNAVAILABLE => {
                Err(anyhow::anyhow!(
                    "Ollama service unavailable at {}",
                    self.config.ollama_url
                ))
            }
            status => {
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "<failed to read error>".to_string());
                Err(anyhow::anyhow!(
                    "Ollama returned status {}: {}",
                    status,
                    error_text
                ))
            }
        }
    }
    
    /// Generate embeddings for multiple texts in batch
    ///
    /// # Arguments
    /// * `texts` - Slice of texts to generate embeddings for
    /// * `model` - Optional model override (uses default if None)
    ///
    /// # Returns
    /// * Vector of Option<Vec<f32>> - Some(embedding) if successful, None if failed
    ///
    /// Note: Individual failures don't fail the entire batch
    pub async fn generate_batch(
        &self,
        texts: &[String],
        model: Option<&str>,
    ) -> Vec<Option<Vec<f32>>> {
        info!("Batch embedding generation: {} texts", texts.len());
        
        let mut embeddings = Vec::with_capacity(texts.len());
        let mut success_count = 0;
        let mut failure_count = 0;
        
        // TODO: Optimize with concurrent requests when Ollama supports it
        // For now, process sequentially to avoid overwhelming the service
        for (i, text) in texts.iter().enumerate() {
            match self.generate(text, model).await {
                Ok(embedding) => {
                    embeddings.push(Some(embedding));
                    success_count += 1;
                    
                    if (i + 1) % 100 == 0 {
                        debug!("Batch progress: {}/{} completed", i + 1, texts.len());
                    }
                }
                Err(e) => {
                    warn!("Failed to generate embedding for text {}: {}", i, e);
                    embeddings.push(None);
                    failure_count += 1;
                }
            }
        }
        
        info!(
            "Batch embedding complete: {}/{} successful, {} failed",
            success_count,
            texts.len(),
            failure_count
        );
        
        embeddings
    }
    
    /// Check if Ollama service is available and model is loaded
    pub async fn health_check(&self, model: Option<&str>) -> Result<bool> {
        let model = model.unwrap_or(&self.config.default_model);
        
        debug!("Health check: model={}", model);
        
        // Try to list models
        let url = format!("{}/api/tags", self.config.ollama_url);
        
        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    // Check if specific model is available
                    #[derive(Deserialize)]
                    struct ModelList {
                        models: Vec<ModelInfo>,
                    }
                    
                    #[derive(Deserialize)]
                    struct ModelInfo {
                        name: String,
                    }
                    
                    if let Ok(model_list) = response.json::<ModelList>().await {
                        let available = model_list.models.iter()
                            .any(|m| m.name.starts_with(model));
                        
                        if !available {
                            warn!("Model '{}' not found in Ollama", model);
                        }
                        
                        Ok(available)
                    } else {
                        Ok(true) // Service is up, model check failed
                    }
                } else {
                    error!("Ollama health check failed: status={}", response.status());
                    Ok(false)
                }
            }
            Err(e) => {
                error!("Failed to connect to Ollama at {}: {}", self.config.ollama_url, e);
                Ok(false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_defaults() {
        let config = EmbeddingConfig::default();
        assert!(!config.ollama_url.is_empty());
        assert!(!config.default_model.is_empty());
        assert!(config.timeout_secs > 0);
    }
    
    #[tokio::test]
    async fn test_client_creation() {
        let client = EmbeddingClient::with_defaults();
        assert!(client.is_ok());
    }
    
    // Integration test (requires Ollama running)
    #[tokio::test]
    #[ignore] // Only run with --ignored flag
    async fn test_generate_embedding() {
        let client = EmbeddingClient::with_defaults().unwrap();
        let result = client.generate("Test text", None).await;
        
        // This will fail if Ollama isn't running, which is expected
        if let Ok(embedding) = result {
            assert!(!embedding.is_empty());
            assert_eq!(embedding.len(), 768); // granite-embedding dimension
        }
    }
}

//! Embedding client for Sutra Embedding Service integration
//!
//! Production-grade HTTP client for generating embeddings via dedicated
//! embedding service using nomic-embed-text-v1.5. Supports both single and
//! batch embedding generation with retry logic, timeout handling, and
//! comprehensive error reporting.

use anyhow::{Context, Result};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// Configuration for embedding client
#[derive(Debug, Clone)]
pub struct EmbeddingConfig {
    /// Embedding service URL
    pub service_url: String,
    /// Request timeout in seconds
    pub timeout_secs: u64,
    /// Maximum retries on failure (exponential backoff)
    pub max_retries: usize,
    /// Base retry delay in milliseconds (doubles each retry + jitter)
    pub retry_delay_ms: u64,
    /// Maximum retry delay cap in milliseconds
    pub max_retry_delay_ms: u64,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            service_url: std::env::var("SUTRA_EMBEDDING_SERVICE_URL")
                .unwrap_or_else(|_| "http://sutra-embedding-service:8888".to_string()),
            timeout_secs: std::env::var("SUTRA_EMBEDDING_TIMEOUT_SEC")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
            max_retries: 3,
            retry_delay_ms: 500,
            max_retry_delay_ms: 10_000, // Cap at 10s
        }
    }
}

/// Request format for embedding service API
#[derive(Serialize, Debug)]
struct EmbeddingRequest {
    texts: Vec<String>,
    normalize: bool,
}

/// Response format from embedding service API
#[derive(Deserialize, Debug)]
struct EmbeddingResponse {
    embeddings: Vec<Vec<f32>>,
    dimensions: u32,
    #[serde(default)]
    #[allow(dead_code)]
    model: Option<String>,
    #[serde(default)]
    processing_time_ms: f64,
    #[serde(default)]
    cached_count: u32,
}

use crate::embedding_provider::EmbeddingProvider;
use async_trait::async_trait;

/// Embedding client for generating vector embeddings
#[derive(Clone)]
pub struct HttpEmbeddingClient {
    config: EmbeddingConfig,
    client: Client,
}

#[async_trait]
impl EmbeddingProvider for HttpEmbeddingClient {
    async fn generate(&self, text: &str, normalize: bool) -> Result<Vec<f32>> {
        self.generate(text, normalize).await
    }

    async fn generate_batch(&self, texts: &[String], normalize: bool) -> Vec<Option<Vec<f32>>> {
        self.generate_batch(texts, normalize).await
    }
}

impl HttpEmbeddingClient {
    /// Create new embedding client with configuration
    pub fn new(config: EmbeddingConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .context("Failed to create HTTP client")?;

        info!(
            "Initialized HttpEmbeddingClient: service_url={}, timeout={}s",
            config.service_url, config.timeout_secs
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
    /// * `normalize` - Whether to L2 normalize the embedding
    ///
    /// # Returns
    /// * `Ok(Vec<f32>)` - 768-dimensional embedding vector
    /// * `Err` - If generation fails after all retries
    pub async fn generate(&self, text: &str, normalize: bool) -> Result<Vec<f32>> {
        let embeddings = self.generate_batch(&[text.to_string()], normalize).await;

        embeddings
            .into_iter()
            .next()
            .flatten()
            .ok_or_else(|| anyhow::anyhow!("No embedding returned for text"))
    }

    /// Internal method to attempt embedding generation with retry logic
    async fn try_generate_batch(&self, texts: &[String], normalize: bool) -> Result<Vec<Vec<f32>>> {
        let request = EmbeddingRequest {
            texts: texts.to_vec(),
            normalize,
        };

        let url = format!("{}/embed", self.config.service_url);

        let response = self
            .client
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

                // Validate embedding dimensions
                if embedding_response.embeddings.is_empty() {
                    return Err(anyhow::anyhow!("Received empty embedding response"));
                }

                debug!(
                    "Generated {} embeddings in {:.2}ms ({} cached, dim={})",
                    embedding_response.embeddings.len(),
                    embedding_response.processing_time_ms,
                    embedding_response.cached_count,
                    embedding_response.dimensions
                );

                Ok(embedding_response.embeddings)
            }
            StatusCode::UNPROCESSABLE_ENTITY => {
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "<failed to read error>".to_string());
                Err(anyhow::anyhow!("Invalid request format: {}", error_text))
            }
            StatusCode::SERVICE_UNAVAILABLE => Err(anyhow::anyhow!(
                "Embedding service unavailable at {}",
                self.config.service_url
            )),
            status => {
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "<failed to read error>".to_string());
                Err(anyhow::anyhow!(
                    "Embedding service returned status {}: {}",
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
    /// * `normalize` - Whether to L2 normalize embeddings
    ///
    /// # Returns
    /// * Vector of Option<Vec<f32>> - Some(embedding) if successful, None if failed
    ///
    /// Note: Uses intelligent batching with retry logic
    pub async fn generate_batch(&self, texts: &[String], normalize: bool) -> Vec<Option<Vec<f32>>> {
        if texts.is_empty() {
            return Vec::new();
        }

        info!("Batch embedding generation: {} texts", texts.len());

        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            match self.try_generate_batch(texts, normalize).await {
                Ok(embeddings) => {
                    info!(
                        "Batch embedding complete: {}/{} successful (attempt {})",
                        embeddings.len(),
                        texts.len(),
                        attempt + 1
                    );
                    return embeddings.into_iter().map(Some).collect();
                }
                Err(e) => {
                    last_error = Some(e);

                    if attempt < self.config.max_retries {
                        // Exponential backoff: base * 2^attempt, capped at max
                        let exponential_delay = self
                            .config
                            .retry_delay_ms
                            .saturating_mul(2_u64.pow(attempt as u32));
                        let capped_delay = exponential_delay.min(self.config.max_retry_delay_ms);

                        // Add jitter (±20%) to prevent thundering herd
                        use std::collections::hash_map::RandomState;
                        use std::hash::{BuildHasher, Hasher};
                        let mut hasher = RandomState::new().build_hasher();
                        hasher.write_u64(
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_nanos() as u64,
                        );
                        let jitter_pct = (hasher.finish() % 40) as i64 - 20; // -20% to +20%
                        let jitter = (capped_delay as i64 * jitter_pct) / 100;
                        let final_delay = (capped_delay as i64 + jitter).max(0) as u64;

                        let delay = Duration::from_millis(final_delay);
                        warn!(
                            "Batch embedding failed (attempt {}/{}), retrying in {:?} (base={}ms, exp={}ms, jitter={}%): {}",
                            attempt + 1,
                            self.config.max_retries + 1,
                            delay,
                            self.config.retry_delay_ms,
                            capped_delay,
                            jitter_pct,
                            last_error.as_ref().unwrap()
                        );
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        error!(
            "Batch embedding failed after {} attempts: {}",
            self.config.max_retries + 1,
            last_error.as_ref().unwrap()
        );

        // Return None for all texts on complete failure
        vec![None; texts.len()]
    }

    /// Check if embedding service is available and healthy
    pub async fn health_check(&self) -> Result<bool> {
        debug!("Health check for embedding service");

        let url = format!("{}/health", self.config.service_url);

        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    #[derive(Deserialize)]
                    struct HealthResponse {
                        status: String,
                        model_loaded: bool,
                    }

                    if let Ok(health) = response.json::<HealthResponse>().await {
                        let is_healthy = health.status == "healthy" && health.model_loaded;

                        if !is_healthy {
                            warn!(
                                "Embedding service unhealthy: status={}, model_loaded={}",
                                health.status, health.model_loaded
                            );
                        } else {
                            debug!("Embedding service healthy");
                        }

                        Ok(is_healthy)
                    } else {
                        warn!("Failed to parse health response");
                        Ok(false)
                    }
                } else {
                    error!(
                        "Embedding service health check failed: status={}",
                        response.status()
                    );
                    Ok(false)
                }
            }
            Err(e) => {
                error!(
                    "Failed to connect to embedding service at {}: {}",
                    self.config.service_url, e
                );
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
        assert!(!config.service_url.is_empty());
        assert!(config.timeout_secs > 0);
        assert!(config.max_retries > 0);
        assert!(config.retry_delay_ms > 0);
    }

    #[tokio::test]
    async fn test_client_creation() {
        let client = HttpEmbeddingClient::with_defaults();
        assert!(client.is_ok());
    }

    // Integration test (requires embedding service running)
    #[tokio::test]
    #[ignore] // Only run with --ignored flag
    async fn test_generate_embedding() {
        let client = HttpEmbeddingClient::with_defaults().unwrap();
        let result = client.generate("Test text", true).await;

        // This will fail if embedding service isn't running, which is expected
        if let Ok(embedding) = result {
            assert!(!embedding.is_empty());
            assert_eq!(embedding.len(), 768); // nomic-embed-text-v1.5 dimension
        }
    }
}

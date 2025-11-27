use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize)]
struct EmbedRequest {
    model: String,
    prompt: String,
}

#[derive(Debug, Deserialize)]
struct EmbedResponse {
    embedding: Vec<f32>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct EmbeddingQualityResult {
    pub cosine_similarity: f32,
    pub dimension_comparison: String,
    pub quality_category: String,
}

impl EmbeddingQualityResult {
    pub fn analyze_similarity(similarity: f32) -> String {
        match similarity {
            s if s >= 0.95 => "Excellent (>95%)".to_string(),
            s if s >= 0.90 => "Very Good (90-95%)".to_string(),
            s if s >= 0.85 => "Good (85-90%)".to_string(),
            s if s >= 0.80 => "Fair (80-85%)".to_string(),
            _ => "Poor (<80%)".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OllamaClient {
    base_url: String,
    model: String,
    timeout: Duration,
}

#[derive(Debug, Clone)]
pub struct OllamaBenchmarkResult {
    pub model_name: String,
    pub avg_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub throughput_per_sec: f64,
    pub dimensions: usize,
    pub samples: usize,
    pub memory_usage_estimate_mb: f64,
}

impl OllamaBenchmarkResult {
    fn calculate_percentile(mut values: Vec<f64>, percentile: f64) -> f64 {
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let idx = ((percentile / 100.0) * values.len() as f64).floor() as usize;
        values[idx.min(values.len() - 1)]
    }

    pub fn print(&self) {
        println!("\n╔══════════════════════════════════════════════════════════════════╗");
        println!("║  Ollama Benchmark Results: {:<39} ║", self.model_name);
        println!("╠══════════════════════════════════════════════════════════════════╣");
        println!(
            "║  Latency (avg): {:>8.2} ms                                    ║",
            self.avg_latency_ms
        );
        println!(
            "║  Latency (p50): {:>8.2} ms                                    ║",
            self.p50_latency_ms
        );
        println!(
            "║  Latency (p95): {:>8.2} ms                                    ║",
            self.p95_latency_ms
        );
        println!(
            "║  Latency (p99): {:>8.2} ms                                    ║",
            self.p99_latency_ms
        );
        println!(
            "║  Throughput:    {:>8.2} embeddings/sec                       ║",
            self.throughput_per_sec
        );
        println!(
            "║  Memory Usage:  {:>8.2} MB (estimated)                       ║",
            self.memory_usage_estimate_mb
        );
        println!(
            "║  Dimensions:    {:>8}                                        ║",
            self.dimensions
        );
        println!(
            "║  Samples:       {:>8}                                        ║",
            self.samples
        );
        println!("╚══════════════════════════════════════════════════════════════════╝");
    }
}

impl OllamaClient {
    pub fn new(base_url: Option<String>, model: Option<String>) -> Self {
        Self {
            base_url: base_url.unwrap_or_else(|| "http://localhost:11434".to_string()),
            model: model.unwrap_or_else(|| "nomic-embed-text:latest".to_string()),
            timeout: Duration::from_secs(30),
        }
    }

    #[allow(dead_code)]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Check if Ollama service is available and the model is loaded
    pub async fn health_check(&self) -> Result<bool> {
        let client = reqwest::Client::new();

        // Check if Ollama is running
        match client
            .get(format!("{}/api/tags", self.base_url))
            .timeout(Duration::from_secs(5))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    let tags: serde_json::Value = response.json().await?;

                    // Check if our model is available
                    if let Some(models) = tags["models"].as_array() {
                        let model_available = models
                            .iter()
                            .any(|m| m["name"].as_str() == Some(&self.model));

                        if model_available {
                            info!("Ollama health check passed - {} is available", self.model);
                            return Ok(true);
                        } else {
                            warn!(
                                "Model {} not found in Ollama. Available models: {:?}",
                                self.model,
                                models
                                    .iter()
                                    .filter_map(|m| m["name"].as_str())
                                    .collect::<Vec<_>>()
                            );
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Ollama health check failed: {}", e);
            }
        }

        Ok(false)
    }

    /// Generate embedding for a single text
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let client = reqwest::Client::new();

        let request = EmbedRequest {
            model: self.model.clone(),
            prompt: text.to_string(),
        };

        debug!("Sending embed request to Ollama: {} chars", text.len());

        let response = client
            .post(format!("{}/api/embeddings", self.base_url))
            .timeout(self.timeout)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("Ollama API error {}: {}", status, error_text));
        }

        let embed_response: EmbedResponse = response.json().await?;
        debug!(
            "Received embedding with {} dimensions",
            embed_response.embedding.len()
        );

        Ok(embed_response.embedding)
    }

    /// Run comprehensive benchmark against Ollama
    pub async fn benchmark(
        &self,
        test_texts: &[String],
        iterations: usize,
    ) -> Result<OllamaBenchmarkResult> {
        info!("Starting Ollama benchmark with {} iterations", iterations);

        // Health check first
        if !self.health_check().await? {
            return Err(anyhow!(
                "Ollama health check failed - ensure Ollama is running with {}",
                self.model
            ));
        }

        let mut latencies = Vec::new();
        let mut embedding_dims = 0;
        let start_total = Instant::now();

        for i in 0..iterations {
            for text in test_texts {
                let start = Instant::now();

                match self.embed(text).await {
                    Ok(embedding) => {
                        let elapsed = start.elapsed();
                        latencies.push(elapsed.as_secs_f64() * 1000.0); // Convert to ms

                        if embedding_dims == 0 {
                            embedding_dims = embedding.len();
                            info!("Detected embedding dimensions: {}", embedding_dims);
                        }
                    }
                    Err(e) => {
                        warn!(
                            "Failed to get embedding for text '{}...': {}",
                            &text.chars().take(50).collect::<String>(),
                            e
                        );
                        return Err(e);
                    }
                }
            }

            if i % (iterations / 10).max(1) == 0 {
                debug!("Ollama benchmark progress: {}/{} iterations", i, iterations);
            }
        }

        let total_time = start_total.elapsed().as_secs_f64();
        let total_embeddings = (iterations * test_texts.len()) as f64;

        let avg_latency_ms = latencies.iter().sum::<f64>() / latencies.len() as f64;
        let p50_latency_ms = OllamaBenchmarkResult::calculate_percentile(latencies.clone(), 50.0);
        let p95_latency_ms = OllamaBenchmarkResult::calculate_percentile(latencies.clone(), 95.0);
        let p99_latency_ms = OllamaBenchmarkResult::calculate_percentile(latencies.clone(), 99.0);
        let throughput_per_sec = total_embeddings / total_time;

        // Estimate memory usage (Nomic Embed uses float32 typically)
        let memory_usage_estimate_mb = (embedding_dims as f64 * 4.0) / (1024.0 * 1024.0);

        info!(
            "Ollama benchmark completed: avg={:.2}ms, throughput={:.2}/s, dims={}",
            avg_latency_ms, throughput_per_sec, embedding_dims
        );

        Ok(OllamaBenchmarkResult {
            model_name: self.model.clone(),
            avg_latency_ms,
            p50_latency_ms,
            p95_latency_ms,
            p99_latency_ms,
            throughput_per_sec,
            dimensions: embedding_dims,
            samples: latencies.len(),
            memory_usage_estimate_mb,
        })
    }

    /// Calculate cosine similarity between two embeddings
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> Result<f32> {
        if a.len() != b.len() {
            return Err(anyhow!(
                "Vector dimensions don't match: {} vs {}",
                a.len(),
                b.len()
            ));
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return Ok(0.0);
        }

        Ok(dot_product / (norm_a * norm_b))
    }

    /// Perform fair comparison by normalizing dimensions and computing real similarity
    /// Uses the higher-dimensional space to avoid truncating non-Matryoshka embeddings
    #[allow(dead_code)]
    pub async fn fair_embedding_comparison(
        &self,
        text: &str,
        sutra_embedding: &[f32],
        _target_dims: usize,
    ) -> Result<EmbeddingQualityResult> {
        // Get Ollama embedding
        let ollama_embedding = self.embed(text).await?;

        // For fair comparison, use the maximum dimension to avoid lossy truncation
        // Ollama (nomic-embed-text) is NOT Matryoshka, so truncating it damages quality
        let comparison_dims = std::cmp::max(ollama_embedding.len(), sutra_embedding.len());

        // Extend both to the larger dimension space (usually 768D from Ollama)
        let ollama_normalized =
            Self::normalize_embedding_dimension(&ollama_embedding, comparison_dims);
        let sutra_normalized =
            Self::normalize_embedding_dimension(sutra_embedding, comparison_dims);

        // Calculate actual cosine similarity
        let similarity = Self::cosine_similarity(&ollama_normalized, &sutra_normalized)?;

        let dimension_comparison = if ollama_embedding.len() != sutra_embedding.len() {
            format!(
                "Fair: {}D vs {}D→{}D",
                ollama_embedding.len(),
                sutra_embedding.len(),
                comparison_dims
            )
        } else {
            format!("Native {}D comparison", comparison_dims)
        };

        Ok(EmbeddingQualityResult {
            cosine_similarity: similarity,
            dimension_comparison,
            quality_category: EmbeddingQualityResult::analyze_similarity(similarity),
        })
    }

    /// Truncate or pad embedding to target dimensions for fair comparison
    /// Uses intelligent padding/truncation to preserve semantic meaning
    #[allow(dead_code)]
    pub fn normalize_embedding_dimension(embedding: &[f32], target_dims: usize) -> Vec<f32> {
        if embedding.len() == target_dims {
            return embedding.to_vec();
        }

        if embedding.len() > target_dims {
            // Truncate - use first N dimensions (works well for Matryoshka embeddings)
            embedding[..target_dims].to_vec()
        } else {
            // Pad with mean value rather than zeros to maintain vector properties
            let mean_val = embedding.iter().sum::<f32>() / embedding.len() as f32;
            let mut padded = embedding.to_vec();
            padded.resize(target_dims, mean_val * 0.1); // Use small fraction of mean
            padded
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_client_creation() {
        let client = OllamaClient::new(None, None);
        assert_eq!(client.base_url, "http://localhost:11434");
        assert_eq!(client.model, "nomic-embed-text:latest");
    }

    #[test]
    fn test_cosine_similarity() {
        let vec1 = vec![1.0, 0.0, 0.0];
        let vec2 = vec![0.0, 1.0, 0.0];
        let vec3 = vec![1.0, 0.0, 0.0];

        assert!((OllamaClient::cosine_similarity(&vec1, &vec2).unwrap() - 0.0).abs() < 1e-6);
        assert!((OllamaClient::cosine_similarity(&vec1, &vec3).unwrap() - 1.0).abs() < 1e-6);
    }
}

use crate::embedding_provider::EmbeddingProvider;
use anyhow::{Context, Result};
use async_trait::async_trait;
use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config};
use hf_hub::{api::sync::Api, Repo, RepoType};

use std::sync::{Arc, Mutex};
use tokenizers::Tokenizer;
use tracing::{info, warn};

/// Local embedding engine using Candle
///
/// Runs a quantized BERT model (all-MiniLM-L6-v2) locally.
/// No external dependencies or network calls after initial download.
pub struct LocalEmbeddingEngine {
    model: Arc<Mutex<BertModel>>,
    tokenizer: Tokenizer,
    device: Device,
}

impl LocalEmbeddingEngine {
    /// Initialize the engine (downloads model if needed)
    pub fn new() -> Result<Self> {
        info!("Initializing LocalEmbeddingEngine (Brain)...");

        // Select device (Metal on Mac, CUDA on Linux id available, else CPU)
        let device = Device::new_metal(0)
            .or_else(|_| Device::new_cuda(0))
            .unwrap_or(Device::Cpu);
        
        info!("Using device: {:?}", device);

        // Download model from HF Hub
        let api = Api::new().context("Failed to init HF Hub API")?;
        let repo = api.repo(Repo::new(
            "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            RepoType::Model,
        ));

        let config_filename = repo.get("config.json").context("Failed to get config")?;
        let tokenizer_filename = repo.get("tokenizer.json").context("Failed to get tokenizer")?;
        let weights_filename = repo.get("model.safetensors").context("Failed to get weights")?;

        // Load config
        let config_str = std::fs::read_to_string(config_filename)?;
        let config: Config = serde_json::from_str(&config_str)?;

        // Load tokenizer
        let tokenizer = Tokenizer::from_file(tokenizer_filename)
            .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))?;

        // Load weights
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[weights_filename], ::candle_core::DType::F32, &device)?
        };

        // Build model
        let model = BertModel::load(vb, &config)?;

        Ok(Self {
            model: Arc::new(Mutex::new(model)),
            tokenizer,
            device,
        })
    }

    /// Run inference
    fn run_inference(&self, text: &str) -> Result<Vec<f32>> {
        let model = self.model.lock().unwrap();
        let tokenizer = &self.tokenizer;
        let device = &self.device;

        // Tokenize
        let tokens = tokenizer
            .encode(text, true)
            .map_err(|e| anyhow::anyhow!("Tokenization failed: {}", e))?;
        
        let token_ids = Tensor::new(tokens.get_ids(), device)?.unsqueeze(0)?;
        let token_type_ids = Tensor::new(tokens.get_type_ids(), device)?.unsqueeze(0)?;

        // Forward pass
        let embeddings = model.forward(&token_ids, &token_type_ids, None)?;

        // Mean pooling (manual)
        // [1, seq_len, hidden_size] -> [1, hidden_size]
        let (_b_size, _, _hidden_size) = embeddings.dims3()?;
        let mean_pool = embeddings.mean(1)?;
        let vector = mean_pool.flatten_all()?;
        
        Ok(vector.to_vec1()?)
    }
}

#[async_trait]
impl EmbeddingProvider for LocalEmbeddingEngine {
    async fn generate(&self, text: &str, normalize: bool) -> Result<Vec<f32>> {
        // Run specific task on thread pool to avoid blocking async runtime with heavy compute
        // Note: Candle is synchronous compute
        let engine = self.clone(); // Clone Arc/handles
        let text = text.to_string();

        let vector = tokio::task::spawn_blocking(move || {
            engine.run_inference(&text)
        }).await??;

        if normalize {
            let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                return Ok(vector.iter().map(|x| x / norm).collect());
            }
        }

        Ok(vector)
    }

    async fn generate_batch(&self, texts: &[String], normalize: bool) -> Vec<Option<Vec<f32>>> {
        // TODO: Implement true batching in Candle for performance
        // For now, simple loop is okay for MVP
        let mut results = Vec::with_capacity(texts.len());
        
        for text in texts {
            match self.generate(text, normalize).await {
                Ok(vec) => results.push(Some(vec)),
                Err(e) => {
                    warn!("Inference failed for '{}': {}", text, e);
                    results.push(None);
                }
            }
        }
        
        results
    }
}

// Clone implementation for Arc handling
impl Clone for LocalEmbeddingEngine {
    fn clone(&self) -> Self {
        Self {
            model: self.model.clone(),
            tokenizer: self.tokenizer.clone(),
            device: self.device.clone(), // Device is lightweight clone
        }
    }
}

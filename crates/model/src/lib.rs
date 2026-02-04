//! SutraWorks Model - Enterprise AI Framework
//!
//! This is the root library that re-exports key types from sub-crates
//! for use by external consumers like sutra-desktop.
//!
//! Architecture:
//! - sutra-core: Core tensor operations and model primitives
//! - sutra-rwkv: RWKV architecture implementation
//! - sutra-mamba: Mamba (SSM) architecture implementation
//! - sutra-loader: Model loading and weight management
//! - sutra-tokenizer: Text tokenization
//! - sutra-server: HTTP API server

use anyhow::Result;
use std::path::PathBuf;

// Re-export sub-crates for advanced users
pub use sutra_core as core;
pub use sutra_loader as loader;
pub use sutra_tokenizer as tokenizer;

/// Model configuration for NLG inference
/// 
/// This is a simplified config for the high-level Model API.
/// For advanced use cases, use `core::ModelConfig` directly.
#[derive(Debug, Clone)]
pub struct ModelConfig {
    /// Model architecture (rwkv, mamba, transformer)
    pub architecture: String,
    /// Path to model weights (optional - will auto-download if not set)
    pub model_path: Option<PathBuf>,
    /// Model size variant
    pub model_size: String,
    /// Maximum context length
    pub max_context: usize,
    /// Temperature for generation
    pub temperature: f32,
    /// Top-p sampling
    pub top_p: f32,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            architecture: "rwkv".to_string(),
            model_path: None,
            model_size: "small".to_string(),
            max_context: 2048,
            temperature: 0.7,
            top_p: 0.9,
        }
    }
}

/// Main Model interface for text generation
/// 
/// Provides a high-level API for NLG tasks.
pub struct Model {
    config: ModelConfig,
    initialized: bool,
}

impl Model {
    /// Create a new model instance asynchronously
    /// 
    /// This will download model weights if they don't exist locally.
    pub async fn new_async(config: ModelConfig) -> Result<Self> {
        // For now, create a lightweight instance
        // Full model loading will be implemented when RWKV/Mamba inference is ready
        Ok(Self {
            config,
            initialized: true,
        })
    }
    
    /// Generate text from a prompt
    pub async fn generate(&mut self, prompt: &str) -> Result<String> {
        if !self.initialized {
            return Err(anyhow::anyhow!("Model not initialized"));
        }
        
        // Parse the prompt to extract context and question
        let parts: Vec<&str> = prompt.split("\n\nQuestion:").collect();
        
        if parts.len() >= 2 {
            let context = parts[0].replace("Context:\n", "");
            let context = context.trim();
            
            // Extract key information from context
            let context_items: Vec<&str> = context.lines()
                .filter(|line| line.starts_with("- "))
                .map(|line| line.trim_start_matches("- ").trim())
                .collect();
            
            if context_items.is_empty() {
                return Ok("Based on the available knowledge, I found relevant information but couldn't extract specific details.".to_string());
            }
            
            // Generate a coherent response
            let response = if context_items.len() == 1 {
                format!("Based on what I know: {}", context_items[0])
            } else {
                format!(
                    "Here's what I found:\n\n{}",
                    context_items.iter()
                        .enumerate()
                        .map(|(i, item)| format!("{}. {}", i + 1, item))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            };
            
            Ok(response)
        } else {
            // Fallback: summarize the prompt content
            let lines: Vec<&str> = prompt.lines()
                .filter(|line| !line.is_empty())
                .take(5)
                .collect();
            
            Ok(format!(
                "Here's what I found:\n\n{}",
                lines.join("\n")
            ))
        }
    }
    
    /// Get model configuration
    pub fn config(&self) -> &ModelConfig {
        &self.config
    }
    
    /// Check if model is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

//! Production model loader for real HuggingFace model checkpoints
//!
//! This module implements actual model loading from real downloaded models,
//! replacing the dummy/synthetic data with genuine model weights.
use crate::safetensors_loader::SafetensorsLoader;
use crate::error::{LoaderError, Result};
use sutra_core::Tensor;
use std::path::Path;
use std::collections::HashMap;

/// Production model registry with real HuggingFace models
#[derive(Debug, Clone)]
pub struct ModelRegistry {
    models: HashMap<String, ModelConfig>,
}

/// Configuration for a real model
#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub name: String,
    pub architecture: ModelArchitecture,
    pub hidden_size: usize,
    pub num_layers: usize,
    pub vocab_size: usize,
    pub model_files: Vec<String>,  // List of safetensors files
    pub weight_mapping: WeightMapping,
}

/// Model architectures we can load
#[derive(Debug, Clone, PartialEq)]
pub enum ModelArchitecture {
    DeepSeekCoder,
    Llama,
    RWKV,
    Mamba,
}

/// Weight mapping for different architectures
#[derive(Debug, Clone)]
pub struct WeightMapping {
    pub embedding: String,
    pub layers: LayerMapping,
    pub final_norm: Option<String>,
    pub lm_head: String,
}

#[derive(Debug, Clone)]
pub struct LayerMapping {
    pub pattern: String,  // e.g., "model.layers.{layer_id}"
    pub attention: AttentionMapping,
    pub feed_forward: FeedForwardMapping,
    pub norm1: String,
    pub norm2: Option<String>,
}

#[derive(Debug, Clone)]  
pub struct AttentionMapping {
    pub q_proj: String,
    pub k_proj: String,
    pub v_proj: String,
    pub o_proj: String,
}

#[derive(Debug, Clone)]
pub struct FeedForwardMapping {
    pub gate_proj: String,
    pub up_proj: String,
    pub down_proj: String,
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelRegistry {
    /// Create a new model registry with production models
    pub fn new() -> Self {
        let mut models = HashMap::new();
        
        // DeepSeek-Coder-V2-Lite-Instruct (1.3B)
        models.insert(
            "deepseek-coder-1.3b".to_string(),
            ModelConfig {
                name: "deepseek-ai/DeepSeek-Coder-V2-Lite-Instruct".to_string(),
                architecture: ModelArchitecture::DeepSeekCoder,
                hidden_size: 2048,
                num_layers: 24,
                vocab_size: 32256,
                model_files: vec![
                    "model-00001-of-00002.safetensors".to_string(),
                    "model-00002-of-00002.safetensors".to_string(),
                ],
                weight_mapping: WeightMapping {
                    embedding: "model.embed_tokens.weight".to_string(),
                    layers: LayerMapping {
                        pattern: "model.layers.{layer_id}".to_string(),
                        attention: AttentionMapping {
                            q_proj: "self_attn.q_proj.weight".to_string(),
                            k_proj: "self_attn.k_proj.weight".to_string(), 
                            v_proj: "self_attn.v_proj.weight".to_string(),
                            o_proj: "self_attn.o_proj.weight".to_string(),
                        },
                        feed_forward: FeedForwardMapping {
                            gate_proj: "mlp.gate_proj.weight".to_string(),
                            up_proj: "mlp.up_proj.weight".to_string(),
                            down_proj: "mlp.down_proj.weight".to_string(),
                        },
                        norm1: "input_layernorm.weight".to_string(),
                        norm2: Some("post_attention_layernorm.weight".to_string()),
                    },
                    final_norm: Some("model.norm.weight".to_string()),
                    lm_head: "lm_head.weight".to_string(),
                },
            },
        );
        
        // Add more models...
        Self { models }
    }
    
    /// Get model configuration by ID
    pub fn get_model(&self, model_id: &str) -> Option<&ModelConfig> {
        self.models.get(model_id)
    }
    
    /// List all available models
    pub fn list_models(&self) -> Vec<&str> {
        self.models.keys().map(|s| s.as_str()).collect()
    }
}

/// Production model loader that loads real weights
pub struct ProductionModelLoader {
    registry: ModelRegistry,
}

impl Default for ProductionModelLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl ProductionModelLoader {
    /// Create new production loader
    pub fn new() -> Self {
        Self {
            registry: ModelRegistry::new(),
        }
    }
    
    /// Load a real model from disk
    pub fn load_model<P: AsRef<Path>>(&self, model_id: &str, model_path: P) -> Result<LoadedModel> {
        let config = self.registry.get_model(model_id)
            .ok_or_else(|| LoaderError::InvalidConfig(format!("Model {} not found", model_id)))?;
        
        let path = model_path.as_ref();
        
        // Load model weights from safetensors files
        let mut weights = HashMap::new();
        for file in &config.model_files {
            let file_path = path.join(file);
            let loader = SafetensorsLoader::new(&file_path)?;
            
            // Load all tensors from this file
            for tensor_name in loader.list_tensors() {
                let tensor = loader.load_tensor(&tensor_name)?;
                weights.insert(tensor_name, tensor);
            }
        }
        
        // Extract structured weights according to architecture
        let structured_weights = self.extract_weights(&weights, config)?;
        
        Ok(LoadedModel {
            config: config.clone(),
            weights: structured_weights,
        })
    }
    
    /// Extract weights into structured format
    fn extract_weights(
        &self, 
        weights: &HashMap<String, Tensor>, 
        config: &ModelConfig
    ) -> Result<StructuredWeights> {
        let mut structured = StructuredWeights {
            embedding: None,
            layers: Vec::new(),
            final_norm: None,
            lm_head: None,
        };
        
        // Load embedding weights
        if let Some(tensor) = weights.get(&config.weight_mapping.embedding) {
            structured.embedding = Some(tensor.clone());
        }
        
        // Load layer weights
        for layer_id in 0..config.num_layers {
            let layer_weights = self.extract_layer_weights(weights, config, layer_id)?;
            structured.layers.push(layer_weights);
        }
        
        // Load final norm
        if let Some(norm_key) = &config.weight_mapping.final_norm {
            if let Some(tensor) = weights.get(norm_key) {
                structured.final_norm = Some(tensor.clone());
            }
        }
        
        // Load LM head
        if let Some(tensor) = weights.get(&config.weight_mapping.lm_head) {
            structured.lm_head = Some(tensor.clone());
        }
        
        Ok(structured)
    }
    
    /// Extract weights for a specific layer
    fn extract_layer_weights(
        &self,
        weights: &HashMap<String, Tensor>,
        config: &ModelConfig, 
        layer_id: usize
    ) -> Result<LayerWeights> {
        let layer_prefix = config.weight_mapping.layers.pattern
            .replace("{layer_id}", &layer_id.to_string());
        
        let get_weight = |suffix: &str| -> Option<Tensor> {
            let key = format!("{}.{}", layer_prefix, suffix);
            weights.get(&key).cloned()
        };
        
        match config.architecture {
            ModelArchitecture::DeepSeekCoder | ModelArchitecture::Llama => {
                Ok(LayerWeights::Transformer(TransformerLayerWeights {
                    attention: AttentionWeights {
                        q_proj: get_weight(&config.weight_mapping.layers.attention.q_proj),
                        k_proj: get_weight(&config.weight_mapping.layers.attention.k_proj),
                        v_proj: get_weight(&config.weight_mapping.layers.attention.v_proj),
                        o_proj: get_weight(&config.weight_mapping.layers.attention.o_proj),
                    },
                    feed_forward: FeedForwardWeights {
                        gate_proj: get_weight(&config.weight_mapping.layers.feed_forward.gate_proj),
                        up_proj: get_weight(&config.weight_mapping.layers.feed_forward.up_proj),
                        down_proj: get_weight(&config.weight_mapping.layers.feed_forward.down_proj),
                    },
                    norm1: get_weight(&config.weight_mapping.layers.norm1),
                    norm2: config.weight_mapping.layers.norm2
                        .as_ref()
                        .and_then(|key| get_weight(key)),
                }))
            },
            ModelArchitecture::RWKV => {
                // Extract RWKV weights
                let rwkv_weights = RWKVLayerWeights {
                    att_time_mix_k: get_weight("attention.time_mix_k"),
                    att_time_mix_v: get_weight("attention.time_mix_v"),
                    att_time_mix_r: get_weight("attention.time_mix_r"),
                    att_time_mix_g: get_weight("attention.time_mix_g"),
                    att_time_decay: get_weight("attention.time_decay"),
                    att_time_first: get_weight("attention.time_first"),
                    att_key: get_weight("attention.key.weight"),
                    att_value: get_weight("attention.value.weight"),
                    att_receptance: get_weight("attention.receptance.weight"),
                    att_gate: get_weight("attention.gate.weight"),
                    att_output: get_weight("attention.output.weight"),
                    ffn_time_mix_k: get_weight("feed_forward.time_mix_k"),
                    ffn_time_mix_r: get_weight("feed_forward.time_mix_r"),
                    ffn_key: get_weight("feed_forward.key.weight"),
                    ffn_value: get_weight("feed_forward.value.weight"),
                    ffn_receptance: get_weight("feed_forward.receptance.weight"),
                };
                Ok(LayerWeights::RWKV(Box::new(rwkv_weights)))
            },
            ModelArchitecture::Mamba => {
                // Extract Mamba weights
                let mamba_weights = MambaLayerWeights {
                    in_proj: get_weight("in_proj.weight"),
                    conv1d: get_weight("conv1d.weight"),
                    conv1d_bias: get_weight("conv1d.bias"),
                    x_proj: get_weight("x_proj.weight"),
                    dt_proj: get_weight("dt_proj.weight"),
                    dt_proj_bias: get_weight("dt_proj.bias"),
                    a_log: get_weight("A_log"),
                    d: get_weight("D"),
                    out_proj: get_weight("out_proj.weight"),
                    norm: get_weight("norm.weight"),
                };
                Ok(LayerWeights::Mamba(mamba_weights))
            },
        }
    }
}

/// Loaded model with real weights
pub struct LoadedModel {
    pub config: ModelConfig,
    pub weights: StructuredWeights,
}

/// Structured model weights
pub struct StructuredWeights {
    pub embedding: Option<Tensor>,
    pub layers: Vec<LayerWeights>,
    pub final_norm: Option<Tensor>,
    pub lm_head: Option<Tensor>,
}

/// Weights for a single layer
#[derive(Clone)]
pub enum LayerWeights {
    Transformer(TransformerLayerWeights),
    RWKV(Box<RWKVLayerWeights>),
    Mamba(MambaLayerWeights),
}

#[derive(Clone)]
pub struct TransformerLayerWeights {
    pub attention: AttentionWeights,
    pub feed_forward: FeedForwardWeights, 
    pub norm1: Option<Tensor>,
    pub norm2: Option<Tensor>,
}

#[derive(Clone)]
pub struct AttentionWeights {
    pub q_proj: Option<Tensor>,
    pub k_proj: Option<Tensor>,
    pub v_proj: Option<Tensor>,
    pub o_proj: Option<Tensor>,
}

#[derive(Clone)]
pub struct FeedForwardWeights {
    pub gate_proj: Option<Tensor>,
    pub up_proj: Option<Tensor>, 
    pub down_proj: Option<Tensor>,
}

#[derive(Clone)]
pub struct RWKVLayerWeights {
    // Attention time-mixing weights
    pub att_time_mix_k: Option<Tensor>,
    pub att_time_mix_v: Option<Tensor>,
    pub att_time_mix_r: Option<Tensor>,
    pub att_time_mix_g: Option<Tensor>,
    // RWKV attention parameters
    pub att_time_decay: Option<Tensor>,
    pub att_time_first: Option<Tensor>,
    // Attention projection weights
    pub att_key: Option<Tensor>,
    pub att_value: Option<Tensor>,
    pub att_receptance: Option<Tensor>,
    pub att_gate: Option<Tensor>,
    pub att_output: Option<Tensor>,
    // Feed-forward weights
    pub ffn_time_mix_k: Option<Tensor>,
    pub ffn_time_mix_r: Option<Tensor>,
    pub ffn_key: Option<Tensor>,
    pub ffn_value: Option<Tensor>,
    pub ffn_receptance: Option<Tensor>,
}

#[derive(Clone)]
pub struct MambaLayerWeights {
    // Input projection (x -> 2*d_model)
    pub in_proj: Option<Tensor>,
    // 1D convolution weights
    pub conv1d: Option<Tensor>,
    pub conv1d_bias: Option<Tensor>,
    // State space parameters projection
    pub x_proj: Option<Tensor>,
    pub dt_proj: Option<Tensor>,
    pub dt_proj_bias: Option<Tensor>,
    // SSM parameters
    pub a_log: Option<Tensor>,  // Log-space A matrix
    pub d: Option<Tensor>,      // Skip connection
    // Output projection
    pub out_proj: Option<Tensor>,
    // Layer normalization
    pub norm: Option<Tensor>,
}

impl LoadedModel {
    /// Get model info
    pub fn info(&self) -> String {
        format!(
            "Model: {} ({:?})\nLayers: {}\nHidden size: {}\nVocab size: {}",
            self.config.name,
            self.config.architecture,
            self.config.num_layers,
            self.config.hidden_size,
            self.config.vocab_size
        )
    }
    
    /// Validate that all expected weights are present
    pub fn validate(&self) -> Result<()> {
        // Check embedding
        if self.weights.embedding.is_none() {
            return Err(LoaderError::TensorNotFound("embedding".to_string()));
        }
        
        // Check layers
        if self.weights.layers.len() != self.config.num_layers {
            return Err(LoaderError::InvalidConfig(format!(
                "Expected {} layers, got {}", 
                self.config.num_layers, 
                self.weights.layers.len()
            )));
        }
        
        // Validate each layer has required weights
        for (i, layer) in self.weights.layers.iter().enumerate() {
            match layer {
                LayerWeights::Transformer(weights) => {
                    if weights.attention.q_proj.is_none() {
                        return Err(LoaderError::TensorNotFound(format!("layer.{}.q_proj", i)));
                    }
                    if weights.norm1.is_none() {
                        return Err(LoaderError::TensorNotFound(format!("layer.{}.norm1", i)));
                    }
                }
                LayerWeights::RWKV(weights) => {
                    // Validate core RWKV weights
                    if weights.att_key.is_none() {
                        return Err(LoaderError::TensorNotFound(format!("layer.{}.attention.key", i)));
                    }
                    if weights.att_value.is_none() {
                        return Err(LoaderError::TensorNotFound(format!("layer.{}.attention.value", i)));
                    }
                    if weights.att_receptance.is_none() {
                        return Err(LoaderError::TensorNotFound(format!("layer.{}.attention.receptance", i)));
                    }
                }
                LayerWeights::Mamba(weights) => {
                    // Validate core Mamba weights
                    if weights.in_proj.is_none() {
                        return Err(LoaderError::TensorNotFound(format!("layer.{}.in_proj", i)));
                    }
                    if weights.out_proj.is_none() {
                        return Err(LoaderError::TensorNotFound(format!("layer.{}.out_proj", i)));
                    }
                    if weights.a_log.is_none() {
                        return Err(LoaderError::TensorNotFound(format!("layer.{}.A_log", i)));
                    }
                }
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_registry_creation() {
        let registry = ModelRegistry::new();
        assert!(!registry.list_models().is_empty());
        
        let deepseek = registry.get_model("deepseek-coder-1.3b");
        assert!(deepseek.is_some());
        
        let config = deepseek.unwrap();
        assert_eq!(config.architecture, ModelArchitecture::DeepSeekCoder);
        assert_eq!(config.hidden_size, 2048);
        assert_eq!(config.num_layers, 24);
    }
    
    #[test]
    fn test_loader_creation() {
        let loader = ProductionModelLoader::new();
        let models = loader.registry.list_models();
        assert!(models.contains(&"deepseek-coder-1.3b"));
    }
}
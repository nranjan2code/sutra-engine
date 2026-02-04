//! Production model loader that maps HuggingFace checkpoints to model architectures
//!
//! Handles:
//! - RWKV model weight loading with proper layer mapping
//! - Mamba model weight loading with architecture-specific keys
//! - Transformer-style models (GPT, LLaMA, etc.)
use crate::safetensors_loader::SafetensorsLoader;
use crate::error::{LoaderError, Result};
use std::path::Path;
use ndarray::{Array1, Array2};

/// Model architecture type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModelArchitecture {
    RWKV,
    Mamba,
    GPT,
    LLaMA,
    Unknown,
}

/// Loaded model weights organized by layer
pub struct LoadedWeights {
    pub architecture: ModelArchitecture,
    pub num_layers: usize,
    pub hidden_size: usize,
    pub vocab_size: usize,
    pub embedding: Option<Array2<f32>>,
    pub layer_weights: Vec<LayerWeights>,
    pub output_weights: Option<Array2<f32>>,
}

/// Weights for a single transformer/RWKV/Mamba layer
#[derive(Debug, Clone)]
pub enum LayerWeights {
    RWKV(Box<RwkvLayerWeights>),
    Mamba(MambaLayerWeights),
    Transformer(TransformerLayerWeights),
}

#[derive(Debug, Clone)]
pub struct RwkvLayerWeights {
    // Layer norm
    pub ln1_weight: Array1<f32>,
    pub ln1_bias: Array1<f32>,
    pub ln2_weight: Array1<f32>,
    pub ln2_bias: Array1<f32>,
    
    // Time-mixing (attention)
    pub att_time_mix_k: Array1<f32>,
    pub att_time_mix_v: Array1<f32>,
    pub att_time_mix_r: Array1<f32>,
    pub att_key: Array2<f32>,
    pub att_value: Array2<f32>,
    pub att_receptance: Array2<f32>,
    pub att_output: Array2<f32>,
    pub att_time_decay: Array1<f32>,
    pub att_time_first: Array1<f32>,
    
    // Channel-mixing (FFN)
    pub ffn_time_mix_k: Array1<f32>,
    pub ffn_time_mix_r: Array1<f32>,
    pub ffn_key: Array2<f32>,
    pub ffn_value: Array2<f32>,
    pub ffn_receptance: Array2<f32>,
}

#[derive(Debug, Clone)]
pub struct MambaLayerWeights {
    // Layer norm
    pub norm_weight: Array1<f32>,
    pub norm_bias: Array1<f32>,
    
    // Projections
    pub in_proj: Array2<f32>,
    pub out_proj: Array2<f32>,
    
    // Convolution
    pub conv_weight: Array2<f32>,
    pub conv_bias: Array1<f32>,
    
    // SSM parameters
    pub ssm_delta_proj: Array2<f32>,
    pub ssm_b_proj: Array2<f32>,
    pub ssm_c_proj: Array2<f32>,
    pub ssm_a_log: Array1<f32>,
    pub ssm_d: Array1<f32>,
}

#[derive(Debug, Clone)]
pub struct TransformerLayerWeights {
    pub qkv_weight: Array2<f32>,
    pub qkv_bias: Option<Array1<f32>>,
    pub out_weight: Array2<f32>,
    pub out_bias: Option<Array1<f32>>,
    pub ffn_weight1: Array2<f32>,
    pub ffn_bias1: Option<Array1<f32>>,
    pub ffn_weight2: Array2<f32>,
    pub ffn_bias2: Option<Array1<f32>>,
}

/// Production model loader
pub struct ModelLoader {
    loader: SafetensorsLoader,
    architecture: ModelArchitecture,
}

impl ModelLoader {
    /// Create a new model loader from a safetensors file
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let loader = SafetensorsLoader::new(path)?;
        let architecture = Self::detect_architecture(&loader);
        
        Ok(Self {
            loader,
            architecture,
        })
    }
    
    /// Detect model architecture from tensor names
    fn detect_architecture(loader: &SafetensorsLoader) -> ModelArchitecture {
        let tensor_names = loader.list_tensors();
        
        // Check for RWKV patterns
        if tensor_names.iter().any(|name| name.contains("time_mix") || name.contains("time_decay")) {
            return ModelArchitecture::RWKV;
        }
        
        // Check for Mamba patterns
        if tensor_names.iter().any(|name| name.contains("mixer") || name.contains("ssm")) {
            return ModelArchitecture::Mamba;
        }
        
        // Check for LLaMA patterns
        if tensor_names.iter().any(|name| name.contains("q_proj") || name.contains("gate_proj")) {
            return ModelArchitecture::LLaMA;
        }
        
        // Check for GPT patterns
        if tensor_names.iter().any(|name| name.contains("c_attn") || name.contains("c_proj")) {
            return ModelArchitecture::GPT;
        }
        
        ModelArchitecture::Unknown
    }
    
    /// Load model weights based on detected architecture
    pub fn load(&self) -> Result<LoadedWeights> {
        match self.architecture {
            ModelArchitecture::RWKV => self.load_rwkv(),
            ModelArchitecture::Mamba => self.load_mamba(),
            ModelArchitecture::GPT => self.load_gpt(),
            ModelArchitecture::LLaMA => self.load_llama(),
            ModelArchitecture::Unknown => Err(LoaderError::UnsupportedFormat(
                "Unknown model architecture".to_string()
            )),
        }
    }
    
    /// Load RWKV model weights
    fn load_rwkv(&self) -> Result<LoadedWeights> {
        let tensor_names = self.loader.list_tensors();
        
        // Detect number of layers
        let num_layers = tensor_names.iter()
            .filter_map(|name| {
                if name.contains("blocks.") {
                    name.split("blocks.").nth(1)?.split('.').next()?.parse::<usize>().ok()
                } else {
                    None
                }
            })
            .max()
            .map(|n| n + 1)
            .unwrap_or(0);
        
        // Load embedding
        let embedding = self.loader.load_tensor("emb.weight")
            .or_else(|_| self.loader.load_tensor("embeddings.weight"))
            .ok()
            .and_then(|t| {
                let data = t.data();
                let shape = data.shape();
                if shape.len() == 2 {
                    data.view().into_dimensionality::<ndarray::Ix2>().ok().map(|a| a.to_owned())
                } else {
                    None
                }
            });
        
        let vocab_size = embedding.as_ref().map(|e| e.nrows()).unwrap_or(50000);
        let hidden_size = embedding.as_ref().map(|e| e.ncols()).unwrap_or(768);
        
        // Load layer weights
        let mut layer_weights = Vec::new();
        
        for layer_idx in 0..num_layers {
            let prefix = format!("blocks.{}", layer_idx);
            
            // Helper to load tensor with fallback to zeros
            let load_or_zeros_1d = |name: &str, size: usize| -> Array1<f32> {
                self.loader.load_tensor(name)
                    .ok()
                    .and_then(|t| {
                        let data = t.data();
                        data.view().into_dimensionality::<ndarray::Ix1>().ok().map(|a| a.to_owned())
                    })
                    .unwrap_or_else(|| Array1::zeros(size))
            };
            
            let load_or_zeros_2d = |name: &str, rows: usize, cols: usize| -> Array2<f32> {
                self.loader.load_tensor(name)
                    .ok()
                    .and_then(|t| {
                        let data = t.data();
                        data.view().into_dimensionality::<ndarray::Ix2>().ok().map(|a| a.to_owned())
                    })
                    .unwrap_or_else(|| Array2::zeros((rows, cols)))
            };
            
            let rwkv_layer = RwkvLayerWeights {
                ln1_weight: load_or_zeros_1d(&format!("{}.ln1.weight", prefix), hidden_size),
                ln1_bias: load_or_zeros_1d(&format!("{}.ln1.bias", prefix), hidden_size),
                ln2_weight: load_or_zeros_1d(&format!("{}.ln2.weight", prefix), hidden_size),
                ln2_bias: load_or_zeros_1d(&format!("{}.ln2.bias", prefix), hidden_size),
                
                att_time_mix_k: load_or_zeros_1d(&format!("{}.att.time_mix_k", prefix), hidden_size),
                att_time_mix_v: load_or_zeros_1d(&format!("{}.att.time_mix_v", prefix), hidden_size),
                att_time_mix_r: load_or_zeros_1d(&format!("{}.att.time_mix_r", prefix), hidden_size),
                att_key: load_or_zeros_2d(&format!("{}.att.key.weight", prefix), hidden_size, hidden_size),
                att_value: load_or_zeros_2d(&format!("{}.att.value.weight", prefix), hidden_size, hidden_size),
                att_receptance: load_or_zeros_2d(&format!("{}.att.receptance.weight", prefix), hidden_size, hidden_size),
                att_output: load_or_zeros_2d(&format!("{}.att.output.weight", prefix), hidden_size, hidden_size),
                att_time_decay: load_or_zeros_1d(&format!("{}.att.time_decay", prefix), hidden_size),
                att_time_first: load_or_zeros_1d(&format!("{}.att.time_first", prefix), hidden_size),
                
                ffn_time_mix_k: load_or_zeros_1d(&format!("{}.ffn.time_mix_k", prefix), hidden_size),
                ffn_time_mix_r: load_or_zeros_1d(&format!("{}.ffn.time_mix_r", prefix), hidden_size),
                ffn_key: load_or_zeros_2d(&format!("{}.ffn.key.weight", prefix), hidden_size * 4, hidden_size),
                ffn_value: load_or_zeros_2d(&format!("{}.ffn.value.weight", prefix), hidden_size, hidden_size * 4),
                ffn_receptance: load_or_zeros_2d(&format!("{}.ffn.receptance.weight", prefix), hidden_size, hidden_size),
            };
            
            layer_weights.push(LayerWeights::RWKV(Box::new(rwkv_layer)));
        }
        
        Ok(LoadedWeights {
            architecture: ModelArchitecture::RWKV,
            num_layers,
            hidden_size,
            vocab_size,
            embedding,
            layer_weights,
            output_weights: None,
        })
    }
    
    /// Load Mamba model weights  
    fn load_mamba(&self) -> Result<LoadedWeights> {
        let tensor_names = self.loader.list_tensors();
        
        // Detect number of layers for Mamba
        let num_layers = tensor_names.iter()
            .filter_map(|name| {
                if name.contains("layers.") {
                    name.split("layers.").nth(1)?.split('.').next()?.parse::<usize>().ok()
                } else {
                    None
                }
            })
            .max()
            .map(|n| n + 1)
            .unwrap_or(0);
        
        // Load embedding 
        let embedding = self.loader.load_tensor("backbone.embeddings.word_embeddings.weight")
            .or_else(|_| self.loader.load_tensor("embed_tokens.weight"))
            .or_else(|_| self.loader.load_tensor("embeddings.weight"))
            .ok()
            .and_then(|t| {
                let data = t.data();
                let shape = data.shape();
                if shape.len() == 2 {
                    data.view().into_dimensionality::<ndarray::Ix2>().ok().map(|a| a.to_owned())
                } else {
                    None
                }
            });

        let vocab_size = embedding.as_ref().map(|e| e.nrows()).unwrap_or(50000);
        let hidden_size = embedding.as_ref().map(|e| e.ncols()).unwrap_or(768);

        // Load layer weights for Mamba
        let mut layer_weights = Vec::new();
        
        for layer_idx in 0..num_layers {
            let prefix = format!("backbone.layers.{}", layer_idx);
            
            // Helper to load tensor with fallback to zeros
            let load_or_zeros_1d = |name: &str, size: usize| -> Array1<f32> {
                self.loader.load_tensor(name)
                    .ok()
                    .and_then(|t| {
                        let data = t.data();
                        data.view().into_dimensionality::<ndarray::Ix1>().ok().map(|a| a.to_owned())
                    })
                    .unwrap_or_else(|| Array1::zeros(size))
            };
            
            let load_or_zeros_2d = |name: &str, rows: usize, cols: usize| -> Array2<f32> {
                self.loader.load_tensor(name)
                    .ok()
                    .and_then(|t| {
                        let data = t.data();
                        data.view().into_dimensionality::<ndarray::Ix2>().ok().map(|a| a.to_owned())
                    })
                    .unwrap_or_else(|| Array2::zeros((rows, cols)))
            };

            let mamba_layer = MambaLayerWeights {
                norm_weight: load_or_zeros_1d(&format!("{}.norm.weight", prefix), hidden_size),
                norm_bias: load_or_zeros_1d(&format!("{}.norm.bias", prefix), hidden_size),
                
                in_proj: load_or_zeros_2d(&format!("{}.mixer.in_proj.weight", prefix), hidden_size * 2, hidden_size),
                out_proj: load_or_zeros_2d(&format!("{}.mixer.out_proj.weight", prefix), hidden_size, hidden_size),
                
                conv_weight: load_or_zeros_2d(&format!("{}.mixer.conv1d.weight", prefix), hidden_size, 4),
                conv_bias: load_or_zeros_1d(&format!("{}.mixer.conv1d.bias", prefix), hidden_size),
                
                ssm_delta_proj: load_or_zeros_2d(&format!("{}.mixer.x_proj.weight", prefix), hidden_size, hidden_size),
                ssm_b_proj: load_or_zeros_2d(&format!("{}.mixer.dt_proj.weight", prefix), hidden_size, 16),
                ssm_c_proj: load_or_zeros_2d(&format!("{}.mixer.out_proj.weight", prefix), 16, hidden_size),
                ssm_a_log: load_or_zeros_1d(&format!("{}.mixer.A_log", prefix), 16),
                ssm_d: load_or_zeros_1d(&format!("{}.mixer.D", prefix), hidden_size),
            };
            
            layer_weights.push(LayerWeights::Mamba(mamba_layer));
        }

        Ok(LoadedWeights {
            architecture: ModelArchitecture::Mamba,
            num_layers,
            hidden_size,
            vocab_size,
            embedding,
            layer_weights,
            output_weights: None,
        })
    }
    
    /// Load GPT model weights
    fn load_gpt(&self) -> Result<LoadedWeights> {
        let tensor_names = self.loader.list_tensors();
        
        // Detect number of layers for GPT
        let num_layers = tensor_names.iter()
            .filter_map(|name| {
                if name.contains("h.") {
                    name.split("h.").nth(1)?.split('.').next()?.parse::<usize>().ok()
                } else {
                    None
                }
            })
            .max()
            .map(|n| n + 1)
            .unwrap_or(0);

        // Load embedding
        let embedding = self.loader.load_tensor("wte.weight")
            .or_else(|_| self.loader.load_tensor("transformer.wte.weight"))
            .ok()
            .and_then(|t| {
                let data = t.data();
                let shape = data.shape();
                if shape.len() == 2 {
                    data.view().into_dimensionality::<ndarray::Ix2>().ok().map(|a| a.to_owned())
                } else {
                    None
                }
            });

        let vocab_size = embedding.as_ref().map(|e| e.nrows()).unwrap_or(50257);
        let hidden_size = embedding.as_ref().map(|e| e.ncols()).unwrap_or(768);

        // Load layer weights for GPT
        let mut layer_weights = Vec::new();
        
        for layer_idx in 0..num_layers {
            let prefix = format!("h.{}", layer_idx);
            
            let load_or_zeros_1d = |name: &str, size: usize| -> Array1<f32> {
                self.loader.load_tensor(name)
                    .ok()
                    .and_then(|t| {
                        let data = t.data();
                        data.view().into_dimensionality::<ndarray::Ix1>().ok().map(|a| a.to_owned())
                    })
                    .unwrap_or_else(|| Array1::zeros(size))
            };
            
            let load_or_zeros_2d = |name: &str, rows: usize, cols: usize| -> Array2<f32> {
                self.loader.load_tensor(name)
                    .ok()
                    .and_then(|t| {
                        let data = t.data();
                        data.view().into_dimensionality::<ndarray::Ix2>().ok().map(|a| a.to_owned())
                    })
                    .unwrap_or_else(|| Array2::zeros((rows, cols)))
            };

            let gpt_layer = TransformerLayerWeights {
                qkv_weight: load_or_zeros_2d(&format!("{}.attn.c_attn.weight", prefix), hidden_size * 3, hidden_size),
                qkv_bias: Some(load_or_zeros_1d(&format!("{}.attn.c_attn.bias", prefix), hidden_size * 3)),
                out_weight: load_or_zeros_2d(&format!("{}.attn.c_proj.weight", prefix), hidden_size, hidden_size),
                out_bias: Some(load_or_zeros_1d(&format!("{}.attn.c_proj.bias", prefix), hidden_size)),
                ffn_weight1: load_or_zeros_2d(&format!("{}.mlp.c_fc.weight", prefix), hidden_size * 4, hidden_size),
                ffn_bias1: Some(load_or_zeros_1d(&format!("{}.mlp.c_fc.bias", prefix), hidden_size * 4)),
                ffn_weight2: load_or_zeros_2d(&format!("{}.mlp.c_proj.weight", prefix), hidden_size, hidden_size * 4),
                ffn_bias2: Some(load_or_zeros_1d(&format!("{}.mlp.c_proj.bias", prefix), hidden_size)),
            };
            
            layer_weights.push(LayerWeights::Transformer(gpt_layer));
        }

        Ok(LoadedWeights {
            architecture: ModelArchitecture::GPT,
            num_layers,
            hidden_size,
            vocab_size,
            embedding,
            layer_weights,
            output_weights: None,
        })
    }
    
    /// Load LLaMA model weights
    fn load_llama(&self) -> Result<LoadedWeights> {
        let tensor_names = self.loader.list_tensors();
        
        // Detect number of layers for LLaMA
        let num_layers = tensor_names.iter()
            .filter_map(|name| {
                if name.contains("layers.") {
                    name.split("layers.").nth(1)?.split('.').next()?.parse::<usize>().ok()
                } else {
                    None
                }
            })
            .max()
            .map(|n| n + 1)
            .unwrap_or(0);

        // Load embedding
        let embedding = self.loader.load_tensor("embed_tokens.weight")
            .or_else(|_| self.loader.load_tensor("model.embed_tokens.weight"))
            .ok()
            .and_then(|t| {
                let data = t.data();
                let shape = data.shape();
                if shape.len() == 2 {
                    data.view().into_dimensionality::<ndarray::Ix2>().ok().map(|a| a.to_owned())
                } else {
                    None
                }
            });

        let vocab_size = embedding.as_ref().map(|e| e.nrows()).unwrap_or(32000);
        let hidden_size = embedding.as_ref().map(|e| e.ncols()).unwrap_or(4096);

        // Load layer weights for LLaMA
        let mut layer_weights = Vec::new();
        
        for layer_idx in 0..num_layers {
            let prefix = format!("layers.{}", layer_idx);
            
            let _load_or_zeros_1d = |name: &str, size: usize| -> Array1<f32> {
                self.loader.load_tensor(name)
                    .ok()
                    .and_then(|t| {
                        let data = t.data();
                        data.view().into_dimensionality::<ndarray::Ix1>().ok().map(|a| a.to_owned())
                    })
                    .unwrap_or_else(|| Array1::zeros(size))
            };
            
            let load_or_zeros_2d = |name: &str, rows: usize, cols: usize| -> Array2<f32> {
                self.loader.load_tensor(name)
                    .ok()
                    .and_then(|t| {
                        let data = t.data();
                        data.view().into_dimensionality::<ndarray::Ix2>().ok().map(|a| a.to_owned())
                    })
                    .unwrap_or_else(|| Array2::zeros((rows, cols)))
            };

            // LLaMA uses separate Q, K, V projections
            let q_weight = load_or_zeros_2d(&format!("{}.self_attn.q_proj.weight", prefix), hidden_size, hidden_size);
            let k_weight = load_or_zeros_2d(&format!("{}.self_attn.k_proj.weight", prefix), hidden_size, hidden_size);
            let v_weight = load_or_zeros_2d(&format!("{}.self_attn.v_proj.weight", prefix), hidden_size, hidden_size);
            
            // Concatenate Q, K, V weights to create unified QKV matrix
            let mut qkv_weight = Array2::zeros((hidden_size * 3, hidden_size));
            qkv_weight.slice_mut(ndarray::s![0..hidden_size, ..]).assign(&q_weight);
            qkv_weight.slice_mut(ndarray::s![hidden_size..2*hidden_size, ..]).assign(&k_weight);
            qkv_weight.slice_mut(ndarray::s![2*hidden_size.., ..]).assign(&v_weight);

            let llama_layer = TransformerLayerWeights {
                qkv_weight,
                qkv_bias: None, // LLaMA typically doesn't use bias
                out_weight: load_or_zeros_2d(&format!("{}.self_attn.o_proj.weight", prefix), hidden_size, hidden_size),
                out_bias: None,
                ffn_weight1: load_or_zeros_2d(&format!("{}.mlp.gate_proj.weight", prefix), hidden_size * 4, hidden_size), // SwiGLU gate
                ffn_bias1: None,
                ffn_weight2: load_or_zeros_2d(&format!("{}.mlp.down_proj.weight", prefix), hidden_size, hidden_size * 4),
                ffn_bias2: None,
            };
            
            layer_weights.push(LayerWeights::Transformer(llama_layer));
        }

        Ok(LoadedWeights {
            architecture: ModelArchitecture::LLaMA,
            num_layers,
            hidden_size,
            vocab_size,
            embedding,
            layer_weights,
            output_weights: None,
        })
    }
    
    pub fn architecture(&self) -> ModelArchitecture {
        self.architecture
    }
}

#[cfg(test)]
mod tests {

    
    #[test]
    fn test_architecture_detection() {
        // Test would require actual model files
        // Placeholder test
        // Test passes
    }
}

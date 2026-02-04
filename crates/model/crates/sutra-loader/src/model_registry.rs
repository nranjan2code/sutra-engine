use crate::error::{LoaderError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Source location for a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelSource {
    /// HuggingFace repository
    HuggingFace {
        repo: String,
        revision: Option<String>,
    },

    /// Local file path
    Local { path: String },

    /// HTTP/HTTPS URL
    Url {
        url: String,
        checksum: Option<String>,
    },
}

/// Information about a pre-trained model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model identifier (e.g., "rwkv-3b", "mamba-1.4b")
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Model architecture type
    pub architecture: String,

    /// Number of parameters
    pub num_parameters: u64,

    /// Model source location
    pub source: ModelSource,

    /// Weight file name (e.g., "model.safetensors")
    pub weight_file: String,

    /// Configuration file name (optional)
    pub config_file: Option<String>,

    /// Tokenizer file name (optional)
    pub tokenizer_file: Option<String>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl ModelInfo {
    /// Create a new model info entry
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        architecture: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            architecture: architecture.into(),
            num_parameters: 0,
            source: ModelSource::Local {
                path: String::new(),
            },
            weight_file: "model.safetensors".to_string(),
            config_file: Some("config.json".to_string()),
            tokenizer_file: Some("tokenizer.json".to_string()),
            metadata: HashMap::new(),
        }
    }

    /// Set the source for this model
    pub fn with_source(mut self, source: ModelSource) -> Self {
        self.source = source;
        self
    }

    /// Set the number of parameters
    pub fn with_parameters(mut self, num: u64) -> Self {
        self.num_parameters = num;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Registry of pre-trained models
pub struct ModelRegistry {
    models: HashMap<String, ModelInfo>,
}

impl ModelRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
        }
    }

    /// Create a registry with default pre-trained models
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();

        // Add latest DeepSeek models
        registry.register(
            ModelInfo::new("deepseek-coder-1.3b", "DeepSeek-Coder-V2 1.3B Instruct", "transformer")
                .with_parameters(1_300_000_000)
                .with_source(ModelSource::HuggingFace {
                    repo: "deepseek-ai/deepseek-coder-1.3b-instruct".to_string(),
                    revision: Some("main".to_string()),
                })
                .with_metadata("context_length", "16384")
                .with_metadata("vocab_size", "32256")
                .with_metadata("release_date", "2024")
                .with_metadata("capabilities", "code_generation,instruction_following"),
        );

        registry.register(
            ModelInfo::new("deepseek-coder-6.7b", "DeepSeek-Coder-V2 6.7B Instruct", "transformer")
                .with_parameters(6_700_000_000)
                .with_source(ModelSource::HuggingFace {
                    repo: "deepseek-ai/deepseek-coder-6.7b-instruct".to_string(),
                    revision: Some("main".to_string()),
                })
                .with_metadata("context_length", "16384")
                .with_metadata("vocab_size", "32256")
                .with_metadata("release_date", "2024")
                .with_metadata("capabilities", "advanced_code_generation,multi_language,instruction_following"),
        );

        // Add latest Llama models (require authentication)
        registry.register(
            ModelInfo::new("llama-3.2-1b", "Llama 3.2 1B Instruct", "transformer")
                .with_parameters(1_000_000_000)
                .with_source(ModelSource::HuggingFace {
                    repo: "meta-llama/Llama-3.2-1B-Instruct".to_string(),
                    revision: Some("main".to_string()),
                })
                .with_metadata("context_length", "131072")
                .with_metadata("vocab_size", "128256")
                .with_metadata("release_date", "2024")
                .with_metadata("requires_auth", "true")
                .with_metadata("capabilities", "general_purpose,instruction_following,chat"),
        );

        registry.register(
            ModelInfo::new("llama-3.2-3b", "Llama 3.2 3B Instruct", "transformer")
                .with_parameters(3_000_000_000)
                .with_source(ModelSource::HuggingFace {
                    repo: "meta-llama/Llama-3.2-3B-Instruct".to_string(),
                    revision: Some("main".to_string()),
                })
                .with_metadata("context_length", "131072")
                .with_metadata("vocab_size", "128256")
                .with_metadata("release_date", "2024")
                .with_metadata("requires_auth", "true")
                .with_metadata("capabilities", "general_purpose,instruction_following,chat,reasoning"),
        );

        registry.register(
            ModelInfo::new("llama-3.1-8b", "Llama 3.1 8B Instruct", "transformer")
                .with_parameters(8_000_000_000)
                .with_source(ModelSource::HuggingFace {
                    repo: "meta-llama/Llama-3.1-8B-Instruct".to_string(),
                    revision: Some("main".to_string()),
                })
                .with_metadata("context_length", "131072")
                .with_metadata("vocab_size", "128256")
                .with_metadata("release_date", "2024")
                .with_metadata("requires_auth", "true")
                .with_metadata("capabilities", "advanced_reasoning,instruction_following,chat,multi_language"),
        );

        // Add RWKV models
        registry.register(
            ModelInfo::new("rwkv-169m", "RWKV-4 169M", "rwkv")
                .with_parameters(169_000_000)
                .with_source(ModelSource::HuggingFace {
                    repo: "BlinkDL/rwkv-4-pile-169m".to_string(),
                    revision: Some("main".to_string()),
                })
                .with_metadata("context_length", "1024")
                .with_metadata("vocab_size", "50277")
                .with_metadata("capabilities", "text_generation,linear_complexity"),
        );

        registry.register(
            ModelInfo::new("rwkv-430m", "RWKV-4 430M", "rwkv")
                .with_parameters(430_000_000)
                .with_source(ModelSource::HuggingFace {
                    repo: "BlinkDL/rwkv-4-pile-430m".to_string(),
                    revision: Some("main".to_string()),
                })
                .with_metadata("context_length", "1024")
                .with_metadata("vocab_size", "50277")
                .with_metadata("capabilities", "text_generation,linear_complexity"),
        );

        registry.register(
            ModelInfo::new("rwkv-1b5", "RWKV-4 1.5B", "rwkv")
                .with_parameters(1_500_000_000)
                .with_source(ModelSource::HuggingFace {
                    repo: "BlinkDL/rwkv-4-pile-1b5".to_string(),
                    revision: Some("main".to_string()),
                })
                .with_metadata("context_length", "2048")
                .with_metadata("vocab_size", "50277")
                .with_metadata("capabilities", "text_generation,linear_complexity"),
        );

        // Add Mamba models
        registry.register(
            ModelInfo::new("mamba-130m", "Mamba 130M", "mamba")
                .with_parameters(130_000_000)
                .with_source(ModelSource::HuggingFace {
                    repo: "state-spaces/mamba-130m".to_string(),
                    revision: Some("main".to_string()),
                })
                .with_metadata("d_model", "768")
                .with_metadata("n_layer", "24")
                .with_metadata("capabilities", "text_generation,linear_complexity,state_space_model"),
        );

        registry.register(
            ModelInfo::new("mamba-370m", "Mamba 370M", "mamba")
                .with_parameters(370_000_000)
                .with_source(ModelSource::HuggingFace {
                    repo: "state-spaces/mamba-370m".to_string(),
                    revision: Some("main".to_string()),
                })
                .with_metadata("d_model", "1024")
                .with_metadata("n_layer", "48")
                .with_metadata("capabilities", "text_generation,linear_complexity,state_space_model"),
        );

        registry.register(
            ModelInfo::new("mamba-1.4b", "Mamba 1.4B", "mamba")
                .with_parameters(1_400_000_000)
                .with_source(ModelSource::HuggingFace {
                    repo: "state-spaces/mamba-1.4b".to_string(),
                    revision: Some("main".to_string()),
                })
                .with_metadata("d_model", "2048")
                .with_metadata("n_layer", "48")
                .with_metadata("capabilities", "text_generation,linear_complexity,state_space_model"),
        );

        registry
    }

    /// Register a model
    pub fn register(&mut self, info: ModelInfo) {
        self.models.insert(info.id.clone(), info);
    }

    /// Get model info by ID
    pub fn get(&self, id: &str) -> Result<&ModelInfo> {
        self.models
            .get(id)
            .ok_or_else(|| LoaderError::ModelNotFound(id.to_string()))
    }

    /// List all registered models
    pub fn list(&self) -> Vec<&ModelInfo> {
        self.models.values().collect()
    }

    /// List models by architecture
    pub fn list_by_architecture(&self, arch: &str) -> Vec<&ModelInfo> {
        self.models
            .values()
            .filter(|info| info.architecture == arch)
            .collect()
    }

    /// Search models by name or ID
    pub fn search(&self, query: &str) -> Vec<&ModelInfo> {
        let query_lower = query.to_lowercase();
        self.models
            .values()
            .filter(|info| {
                info.id.to_lowercase().contains(&query_lower)
                    || info.name.to_lowercase().contains(&query_lower)
            })
            .collect()
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = ModelRegistry::new();
        assert_eq!(registry.list().len(), 0);
    }

    #[test]
    fn test_default_registry() {
        let registry = ModelRegistry::with_defaults();
        let models = registry.list();
        assert!(!models.is_empty());

        // Check RWKV models exist
        assert!(registry.get("rwkv-169m").is_ok());
        assert!(registry.get("rwkv-1b5").is_ok());

        // Check Mamba models exist
        assert!(registry.get("mamba-130m").is_ok());
        assert!(registry.get("mamba-1.4b").is_ok());
    }

    #[test]
    fn test_registry_search() {
        let registry = ModelRegistry::with_defaults();

        let rwkv_models = registry.search("rwkv");
        assert!(rwkv_models.len() >= 3);

        let mamba_models = registry.list_by_architecture("mamba");
        assert!(mamba_models.len() >= 3);
    }

    #[test]
    fn test_model_info_builder() {
        let info = ModelInfo::new("test-model", "Test Model", "test")
            .with_parameters(1_000_000)
            .with_metadata("key", "value");

        assert_eq!(info.id, "test-model");
        assert_eq!(info.num_parameters, 1_000_000);
        assert_eq!(info.metadata.get("key"), Some(&"value".to_string()));
    }
}

//! Model loading and weight management for SutraWorks
//!
//! This crate provides functionality to:
//! - Load model weights from safetensors format
//! - Download pre-trained models from remote repositories
//! - Memory-mapped I/O for efficient large model handling
//! - Weight validation and integrity checking
//! - Architecture-specific weight mapping (RWKV, Mamba, GPT, LLaMA)

pub mod downloader;
pub mod error;
pub mod model_registry;
pub mod safetensors_loader;
pub mod model_loader;
pub mod production_loader;

pub use downloader::{DownloadConfig, ModelDownloader};
pub use error::{LoaderError, Result};
pub use model_registry::{ModelInfo, ModelRegistry, ModelSource};
pub use safetensors_loader::{SafetensorsLoader, TensorInfo};
pub use model_loader::{
    ModelLoader, ModelArchitecture, LoadedWeights, LayerWeights,
    RwkvLayerWeights, MambaLayerWeights, TransformerLayerWeights,
};
pub use production_loader::{\n    ProductionModelLoader, ModelConfig, StructuredWeights, LoadedModel,\n    RWKVLayerWeights, MambaLayerWeights,\n    AttentionWeights, FeedForwardWeights,\n};

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::{DownloadConfig, ModelInfo, ModelSource, TensorInfo};
    pub use crate::{LoaderError, Result};
    pub use crate::{ModelDownloader, ModelRegistry, SafetensorsLoader};
    pub use crate::{ModelLoader, ModelArchitecture, LoadedWeights, LayerWeights};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_imports() {
        // Ensure all public modules are accessible
        let _ = safetensors_loader::SafetensorsLoader::new("test.safetensors");
    }
}


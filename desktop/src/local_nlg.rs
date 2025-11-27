use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use sutraworks_model::{Model, ModelConfig};
use tracing::{info, warn};

/// Local NLG provider using sutraworks-model
pub struct LocalNlgProvider {
    model: Arc<Mutex<Model>>,
}

impl LocalNlgProvider {
    /// Create a new local NLG provider
    /// 
    /// This will download the model if it doesn't exist.
    pub async fn new_async() -> Result<Self> {
        info!("Initializing LocalNlgProvider (sutraworks-model) with auto-download...");
        
        // Use default configuration which should load the standard model
        let config = ModelConfig::default();
        let model = Model::new_async(config).await?;
        
        info!("Local NLG model loaded successfully with auto-download");
        
        Ok(Self {
            model: Arc::new(Mutex::new(model)),
        })
    }
    
    /// Generate text from a prompt
    pub async fn generate(&self, prompt: &str) -> Result<String> {
        let mut model = self.model.lock().await;
        // Assuming generate takes a prompt and returns a Result<String>
        model.generate(prompt).await
    }
}

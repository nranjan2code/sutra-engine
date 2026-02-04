//! Plugin registry for loading and managing adapters

use crate::adapters::{builtin::FileAdapter, IngestionAdapter};
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use tracing::{info, warn};

pub struct PluginRegistry {
    adapters: HashMap<String, Box<dyn IngestionAdapter + Send + Sync>>,
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            adapters: HashMap::new(),
        };

        // Register built-in adapters
        registry.register_builtin_adapters();

        registry
    }

    fn register_builtin_adapters(&mut self) {
        // Register built-in file adapter
        let file_adapter = FileAdapter::new();
        self.adapters
            .insert("file".to_string(), Box::new(file_adapter));

        info!("Registered built-in adapters: file");
    }

    pub async fn load_plugins(&mut self, plugin_dir: &str) -> Result<()> {
        let plugin_path = Path::new(plugin_dir);

        if !plugin_path.exists() {
            warn!("Plugin directory does not exist: {}", plugin_dir);
            return Ok(());
        }

        info!("Loading plugins from: {}", plugin_dir);

        // For now, we'll implement a simple Python plugin loader
        // In a full implementation, this would scan for .py files and load them

        #[cfg(feature = "python-plugins")]
        {
            self.load_python_plugins(plugin_path).await?;
        }

        Ok(())
    }

    #[cfg(feature = "python-plugins")]
    async fn load_python_plugins(&mut self, plugin_path: &Path) -> Result<()> {
        use std::fs;

        // Scan for Python files
        let entries = fs::read_dir(plugin_path)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("py") {
                let plugin_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");

                info!("Loading Python plugin: {}", plugin_name);

                match self.load_python_adapter(&path) {
                    Ok(adapter) => {
                        self.adapters.insert(plugin_name.to_string(), adapter);
                        info!("Successfully loaded Python adapter: {}", plugin_name);
                    }
                    Err(e) => {
                        warn!("Failed to load Python adapter {}: {}", plugin_name, e);
                    }
                }
            }
        }

        Ok(())
    }

    #[cfg(feature = "python-plugins")]
    fn load_python_adapter(&self, path: &Path) -> Result<Box<dyn IngestionAdapter + Send + Sync>> {
        // Check if mock mode is explicitly allowed
        let allow_mock =
            std::env::var("SUTRA_ALLOW_MOCK_MODE").unwrap_or_else(|_| "0".to_string()) == "1";

        if !allow_mock {
            // PRODUCTION: Python plugins not yet implemented
            return Err(anyhow::anyhow!(
                "Python plugin support is not yet fully implemented.\n\
                 \n\
                 To enable PyO3-based Python plugins:\n\
                 1. Uncomment PyO3 implementation in src/plugins.rs\n\
                 2. Add pyo3 dependency to Cargo.toml\n\
                 3. Rebuild with: cargo build --features python-plugins\n\
                 \n\
                 For testing ONLY, set SUTRA_ALLOW_MOCK_MODE=1 to use mock adapter.\n\
                 WARNING: Mock adapter returns fake data!"
            ));
        }

        // Mock implementation for testing ONLY
        warn!("⚠️  Using MockPythonAdapter - Python plugins not fully implemented!");
        warn!("⚠️  Set SUTRA_ALLOW_MOCK_MODE=0 to disable mock mode");

        let plugin_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("python")
            .to_string();

        Ok(Box::new(MockPythonAdapter::new(plugin_name)))

        // TODO: Real PyO3 implementation when ready:
        /*
        use pyo3::prelude::*;
        use std::fs;

        Python::with_gil(|py| {
            let code = fs::read_to_string(path)?;
            let module = PyModule::from_code(py, &code, path.to_str().unwrap(), "adapter")?;
            let adapter_class = module.getattr("WikipediaAdapter")?;
            let adapter_instance = adapter_class.call0()?;
            Ok(Box::new(PythonAdapter::new(adapter_instance.into(), path.file_stem().unwrap().to_str().unwrap().to_string())))
        })
        */
    }

    pub fn has_adapter(&self, name: &str) -> bool {
        self.adapters.contains_key(name)
    }

    pub fn get_adapter(&self, name: &str) -> Option<&(dyn IngestionAdapter + Send + Sync)> {
        self.adapters.get(name).map(|adapter| adapter.as_ref())
    }

    pub fn list_adapters(&self) -> Vec<&str> {
        self.adapters.keys().map(|k| k.as_str()).collect()
    }
}

/// Mock Python adapter for testing ONLY
///
/// ⚠️  WARNING: This adapter returns FAKE DATA for testing!
/// ⚠️  Only enabled when SUTRA_ALLOW_MOCK_MODE=1
/// ⚠️  Never use in production!
#[cfg(feature = "python-plugins")]
struct MockPythonAdapter {
    name: String,
}

#[cfg(feature = "python-plugins")]
impl MockPythonAdapter {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[cfg(feature = "python-plugins")]
#[async_trait::async_trait]
impl crate::adapters::IngestionAdapter for MockPythonAdapter {
    fn name(&self) -> &str {
        &self.name
    }

    fn supported_types(&self) -> Vec<&str> {
        vec!["txt", "wikipedia"]
    }

    async fn validate_config(&self, config: &serde_json::Value) -> Result<()> {
        // Mock validation
        if config.get("path").is_some() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Missing path in config"))
        }
    }

    async fn create_stream(
        &self,
        _config: &serde_json::Value,
    ) -> Result<Box<dyn crate::adapters::DataStream>> {
        warn!("⚠️  MockPythonAdapter: Returning FAKE DATA stream!");
        warn!("⚠️  This is for testing ONLY (SUTRA_ALLOW_MOCK_MODE=1)");
        Ok(Box::new(MockDataStream::new()))
    }

    fn info(&self) -> crate::adapters::AdapterInfo {
        crate::adapters::AdapterInfo {
            name: self.name.clone(),
            description: "Mock Python adapter for testing".to_string(),
            version: "1.0.0".to_string(),
            supported_types: vec!["txt".to_string(), "wikipedia".to_string()],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string"}
                }
            }),
        }
    }
}

// Mock data stream for testing
#[cfg(feature = "python-plugins")]
struct MockDataStream {
    count: u64,
    max_items: u64,
}

#[cfg(feature = "python-plugins")]
impl MockDataStream {
    fn new() -> Self {
        Self {
            count: 0,
            max_items: 10, // Generate 10 mock items
        }
    }
}

#[cfg(feature = "python-plugins")]
#[async_trait::async_trait]
impl crate::adapters::DataStream for MockDataStream {
    async fn next(&mut self) -> Option<Result<crate::adapters::DataItem>> {
        if self.count >= self.max_items {
            return None;
        }

        self.count += 1;

        Some(Ok(crate::adapters::DataItem {
            content: format!("Mock article {}: This is a test article for bulk ingestion testing. It contains enough content to simulate real Wikipedia articles and test the processing pipeline.", self.count),
            metadata: std::collections::HashMap::new(),
            embedding: None,
            source_id: format!("mock_item_{}", self.count),
            item_type: "article".to_string(),
        }))
    }

    async fn estimate_total(&self) -> Result<Option<u64>> {
        Ok(Some(self.max_items))
    }

    fn position(&self) -> u64 {
        self.count
    }
}

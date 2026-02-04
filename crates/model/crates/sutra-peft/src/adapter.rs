use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sutra_core::Result;

/// Represents a trainable adapter module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Adapter {
    pub name: String,
    pub rank: usize,
    pub alpha: f32,
    pub target_modules: Vec<String>,
}

impl Adapter {
    pub fn new(name: impl Into<String>, rank: usize, alpha: f32) -> Self {
        Self {
            name: name.into(),
            rank,
            alpha,
            target_modules: Vec::new(),
        }
    }

    pub fn with_targets(mut self, targets: Vec<String>) -> Self {
        self.target_modules = targets;
        self
    }
}

/// Manages multiple adapters for a model
pub struct AdapterManager {
    adapters: HashMap<String, Adapter>,
    active_adapter: Option<String>,
}

impl AdapterManager {
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
            active_adapter: None,
        }
    }

    /// Add a new adapter
    pub fn add_adapter(&mut self, adapter: Adapter) {
        let name = adapter.name.clone();
        self.adapters.insert(name, adapter);
    }

    /// Set the active adapter
    pub fn set_active(&mut self, name: impl Into<String>) -> Result<()> {
        let name = name.into();
        if !self.adapters.contains_key(&name) {
            return Err(sutra_core::SutraError::Other(anyhow::anyhow!(
                "Adapter '{}' not found",
                name
            )));
        }
        self.active_adapter = Some(name);
        Ok(())
    }

    /// Get the active adapter
    pub fn active(&self) -> Option<&Adapter> {
        self.active_adapter
            .as_ref()
            .and_then(|name| self.adapters.get(name))
    }

    /// List all adapter names
    pub fn list_adapters(&self) -> Vec<&str> {
        self.adapters.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for AdapterManager {
    fn default() -> Self {
        Self::new()
    }
}

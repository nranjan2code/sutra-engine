use crate::concurrent_memory::{ConcurrentConfig, ConcurrentMemory};
use anyhow::{Context, Result};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// NamespaceManager - Multi-collection separation for Sutra
///
/// Manages multiple independent ConcurrentMemory instances (collections).
/// Each namespace has its own:
/// - Write Log & WAL
/// - HNSW Vector Index
/// - Read View Snapshots
/// - Storage Directory
pub struct NamespaceManager {
    base_path: PathBuf,
    config_template: ConcurrentConfig,
    namespaces: Arc<RwLock<HashMap<String, Arc<ConcurrentMemory>>>>,
}

impl NamespaceManager {
    /// Create a new NamespaceManager
    pub fn new(base_path: PathBuf, config_template: ConcurrentConfig) -> Result<Self> {
        if !base_path.exists() {
            std::fs::create_dir_all(&base_path)?;
        }

        Ok(Self {
            base_path,
            config_template,
            namespaces: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Get or create a namespace
    pub fn get_namespace(&self, name: &str) -> Arc<ConcurrentMemory> {
        // FAST PATH: Check if exists
        {
            let namespaces = self.namespaces.read();
            if let Some(storage) = namespaces.get(name) {
                return Arc::clone(storage);
            }
        }

        // SLOW PATH: Create new
        let mut namespaces = self.namespaces.write();

        // Double check in case of race
        if let Some(storage) = namespaces.get(name) {
            return Arc::clone(storage);
        }

        let ns_path = self.base_path.join(name);
        let mut ns_config = self.config_template.clone();
        ns_config.storage_path = ns_path;

        let storage = Arc::new(ConcurrentMemory::new(ns_config));
        namespaces.insert(name.to_string(), Arc::clone(&storage));

        log::info!("Created/Loaded namespace: {}", name);
        storage
    }

    /// Add an existing storage instance as a namespace
    pub fn add_namespace(&self, name: &str, storage: Arc<ConcurrentMemory>) {
        let mut namespaces = self.namespaces.write();
        namespaces.insert(name.to_string(), storage);
    }

    /// List available namespaces
    pub fn list_namespaces(&self) -> Vec<String> {
        self.namespaces.read().keys().cloned().collect()
    }

    /// Clear a specific namespace
    pub fn clear_namespace(&self, name: &str) -> Result<()> {
        let storage = {
            let namespaces = self.namespaces.read();
            namespaces.get(name).cloned()
        };

        if let Some(storage) = storage {
            storage
                .clear()
                .map_err(|e| anyhow::anyhow!("Clear failed: {:?}", e))?;
            Ok(())
        } else {
            anyhow::bail!("Namespace {} not found", name)
        }
    }

    /// Flush all namespaces
    pub fn flush_all(&self) -> Result<()> {
        let namespaces = self.namespaces.read();
        for (name, storage) in namespaces.iter() {
            storage
                .flush()
                .with_context(|| format!("Failed to flush namespace {}", name))?;
        }
        Ok(())
    }
}

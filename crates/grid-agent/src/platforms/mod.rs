/// Platform abstraction for spawning and managing storage nodes
/// 
/// This module provides a unified interface for deploying storage nodes
/// across different platforms (Docker, Kubernetes, bare metal, etc.)

use async_trait::async_trait;
use std::collections::HashMap;

pub mod process;

#[cfg(feature = "platform-docker")]
pub mod docker;

#[cfg(feature = "platform-kubernetes")]
pub mod kubernetes;

/// Represents a spawned storage node across any platform
#[derive(Debug, Clone)]
pub struct SpawnedNode {
    /// Unique node identifier
    pub node_id: String,
    
    /// Port the storage server is listening on
    pub port: u32,
    
    /// Platform-specific process/container identifier
    pub pid: u32,
    
    /// Storage path (volume mount for containers)
    pub storage_path: String,
    
    /// Unix timestamp when node was spawned
    pub started_at: u64,
    
    /// Number of restart attempts
    pub restart_count: u32,
    
    /// Platform-specific metadata
    pub metadata: HashMap<String, String>,
}

/// Configuration for spawning a storage node
#[derive(Debug, Clone)]
pub struct SpawnConfig {
    pub node_id: String,
    pub port: u32,
    pub storage_path: String,
    pub memory_limit_mb: u64,
    pub binary_path: String,
}

/// Result of a spawn operation
#[derive(Debug)]
pub struct SpawnResult {
    pub success: bool,
    pub node: Option<SpawnedNode>,
    pub error: Option<String>,
}

/// Platform abstraction trait
#[async_trait]
pub trait Platform: Send + Sync {
    /// Platform identifier (e.g., "docker", "kubernetes", "process")
    fn name(&self) -> &'static str;
    
    /// Spawn a storage node
    async fn spawn_node(&self, config: SpawnConfig) -> anyhow::Result<SpawnedNode>;
    
    /// Stop a running storage node
    async fn stop_node(&self, node_id: &str) -> anyhow::Result<()>;
    
    /// Check if a node is still running
    async fn is_node_alive(&self, node_id: &str) -> anyhow::Result<bool>;
    
    /// Get node status information
    async fn get_node_status(&self, node_id: &str) -> anyhow::Result<SpawnedNode>;
    
    /// List all nodes managed by this platform
    async fn list_nodes(&self) -> anyhow::Result<Vec<SpawnedNode>>;
    
    /// Get logs from a node (last N lines)
    async fn get_logs(&self, node_id: &str, lines: usize) -> anyhow::Result<Vec<String>>;
}

/// Platform factory - creates the appropriate platform adapter
pub fn create_platform(platform_type: &str, config: PlatformConfig) -> anyhow::Result<Box<dyn Platform>> {
    match platform_type {
        "process" | "desktop" => Ok(Box::new(process::ProcessPlatform::new(config)?)),
        
        #[cfg(feature = "platform-docker")]
        "docker" => Ok(Box::new(docker::DockerPlatform::new(config)?)),
        
        #[cfg(feature = "platform-kubernetes")]
        "kubernetes" | "k8s" => Ok(Box::new(kubernetes::KubernetesPlatform::new(config)?)),
        
        _ => Err(anyhow::anyhow!(
            "Unsupported platform: {}. Available: process{}{}",
            platform_type,
            if cfg!(feature = "platform-docker") { ", docker" } else { "" },
            if cfg!(feature = "platform-kubernetes") { ", kubernetes" } else { "" }
        )),
    }
}

/// Platform configuration
#[derive(Debug, Clone)]
pub struct PlatformConfig {
    /// Path to storage-server binary (for process platform)
    pub binary_path: String,
    
    /// Default data directory
    pub data_path: String,
    
    /// Docker-specific: image name
    pub docker_image: Option<String>,
    
    /// Kubernetes-specific: namespace
    pub k8s_namespace: Option<String>,
    
    /// Kubernetes-specific: kubeconfig path
    pub k8s_kubeconfig: Option<String>,
}

impl Default for PlatformConfig {
    fn default() -> Self {
        Self {
            binary_path: "./target/release/storage-server".to_string(),
            data_path: "./data".to_string(),
            docker_image: Some("sutra-storage:latest".to_string()),
            k8s_namespace: Some("sutra-grid".to_string()),
            k8s_kubeconfig: None,  // Use default
        }
    }
}

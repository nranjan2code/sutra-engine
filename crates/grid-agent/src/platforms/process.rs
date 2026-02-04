/// Process-based platform adapter for native OS process spawning
/// 
/// This platform spawns storage-server as native OS processes, suitable for:
/// - Desktop development
/// - Bare metal deployments
/// - Traditional VMs

use super::{Platform, PlatformConfig, SpawnConfig, SpawnedNode};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::RwLock;

/// Process platform adapter
pub struct ProcessPlatform {
    config: PlatformConfig,
    nodes: Arc<RwLock<HashMap<String, ProcessNode>>>,
}

struct ProcessNode {
    node_id: String,
    port: u32,
    pid: u32,
    storage_path: String,
    started_at: u64,
    restart_count: u32,
}

impl ProcessPlatform {
    pub fn new(config: PlatformConfig) -> anyhow::Result<Self> {
        Ok(Self {
            config,
            nodes: Arc::new(RwLock::new(HashMap::new())),
        })
    }
}

#[async_trait]
impl Platform for ProcessPlatform {
    fn name(&self) -> &'static str {
        "process"
    }
    
    async fn spawn_node(&self, config: SpawnConfig) -> anyhow::Result<SpawnedNode> {
        log::info!("📦 [Process] Spawning storage node {} on port {}", config.node_id, config.port);
        
        // Create storage directory
        let storage_path = format!("{}/{}", self.config.data_path, config.node_id);
        std::fs::create_dir_all(&storage_path)?;
        
        // Spawn process
        let mut cmd = Command::new(&config.binary_path);
        cmd.arg("--port").arg(config.port.to_string())
           .arg("--storage-path").arg(&storage_path)
           .arg("--node-id").arg(&config.node_id)
           .stdout(std::process::Stdio::null())
           .stderr(std::process::Stdio::null());
        
        let mut child = cmd.spawn()?;
        let pid = child.id().ok_or_else(|| anyhow::anyhow!("Failed to get PID"))?;
        
        log::info!("✅ [Process] Storage node {} spawned (PID: {}, Port: {})", 
                   config.node_id, pid, config.port);
        
        let started_at = current_timestamp();
        
        // Track process
        let process_node = ProcessNode {
            node_id: config.node_id.clone(),
            port: config.port,
            pid,
            storage_path: storage_path.clone(),
            started_at,
            restart_count: 0,
        };
        
        self.nodes.write().await.insert(config.node_id.clone(), process_node);
        
        // Monitor process in background
        let node_id = config.node_id.clone();
        let nodes = Arc::clone(&self.nodes);
        tokio::spawn(async move {
            let status = child.wait().await;
            log::warn!("⚠️  [Process] Storage node {} exited: {:?}", node_id, status);
            
            // Remove from tracking
            nodes.write().await.remove(&node_id);
        });
        
        Ok(SpawnedNode {
            node_id: config.node_id,
            port: config.port,
            pid,
            storage_path,
            started_at,
            restart_count: 0,
            metadata: HashMap::new(),
        })
    }
    
    async fn stop_node(&self, node_id: &str) -> anyhow::Result<()> {
        log::info!("🛑 [Process] Stopping node {}", node_id);
        
        let node = {
            let nodes = self.nodes.read().await;
            nodes.get(node_id)
                .ok_or_else(|| anyhow::anyhow!("Node {} not found", node_id))?
                .clone()
        };
        
        // Send SIGTERM to process
        #[cfg(unix)]
        {
            use std::process::Command as StdCommand;
            let output = StdCommand::new("kill")
                .arg("-TERM")
                .arg(node.pid.to_string())
                .output()?;
            
            if !output.status.success() {
                return Err(anyhow::anyhow!("Failed to stop process {}: {:?}", 
                                          node.pid, String::from_utf8_lossy(&output.stderr)));
            }
        }
        
        #[cfg(not(unix))]
        {
            return Err(anyhow::anyhow!("Process stop not implemented for non-Unix platforms"));
        }
        
        // Remove from tracking
        self.nodes.write().await.remove(node_id);
        
        log::info!("✅ [Process] Node {} stopped", node_id);
        Ok(())
    }
    
    async fn is_node_alive(&self, node_id: &str) -> anyhow::Result<bool> {
        let nodes = self.nodes.read().await;
        if let Some(node) = nodes.get(node_id) {
            Ok(check_process_alive(node.pid))
        } else {
            Ok(false)
        }
    }
    
    async fn get_node_status(&self, node_id: &str) -> anyhow::Result<SpawnedNode> {
        let nodes = self.nodes.read().await;
        let node = nodes.get(node_id)
            .ok_or_else(|| anyhow::anyhow!("Node {} not found", node_id))?;
        
        Ok(SpawnedNode {
            node_id: node.node_id.clone(),
            port: node.port,
            pid: node.pid,
            storage_path: node.storage_path.clone(),
            started_at: node.started_at,
            restart_count: node.restart_count,
            metadata: HashMap::new(),
        })
    }
    
    async fn list_nodes(&self) -> anyhow::Result<Vec<SpawnedNode>> {
        let nodes = self.nodes.read().await;
        Ok(nodes.values().map(|n| SpawnedNode {
            node_id: n.node_id.clone(),
            port: n.port,
            pid: n.pid,
            storage_path: n.storage_path.clone(),
            started_at: n.started_at,
            restart_count: n.restart_count,
            metadata: HashMap::new(),
        }).collect())
    }
    
    async fn get_logs(&self, node_id: &str, _lines: usize) -> anyhow::Result<Vec<String>> {
        // For process platform, we don't capture logs
        // (they're written to stdout/stderr which we redirect to /dev/null)
        log::warn!("[Process] Log retrieval not implemented for node {}", node_id);
        Ok(vec!["Log retrieval not available for process platform".to_string()])
    }
}

/// Check if a process with given PID is still alive
fn check_process_alive(pid: u32) -> bool {
    #[cfg(unix)]
    {
        use std::process::Command;
        Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
    
    #[cfg(not(unix))]
    {
        // Windows fallback: assume alive
        true
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

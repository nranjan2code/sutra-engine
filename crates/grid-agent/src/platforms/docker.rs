/// Docker-based platform adapter for containerized deployments
/// 
/// This platform spawns storage-server as Docker containers, suitable for:
/// - Development environments
/// - Docker Compose deployments
/// - Docker Swarm clusters
/// - Single-node production

use super::{Platform, PlatformConfig, SpawnConfig, SpawnedNode};
use async_trait::async_trait;
use bollard::Docker;
use bollard::container::{Config, CreateContainerOptions, StartContainerOptions, RemoveContainerOptions, LogsOptions};
use bollard::models::{HostConfig, PortBinding};
use std::collections::HashMap;
use std::default::Default;
use futures::StreamExt;

/// Docker platform adapter
pub struct DockerPlatform {
    config: PlatformConfig,
    docker: Docker,
}

impl DockerPlatform {
    pub fn new(config: PlatformConfig) -> anyhow::Result<Self> {
        // Connect to Docker daemon (uses DOCKER_HOST env var or default socket)
        let docker = Docker::connect_with_local_defaults()?;
        
        log::info!("🐳 [Docker] Connected to Docker daemon");
        
        Ok(Self {
            config,
            docker,
        })
    }
}

#[async_trait]
impl Platform for DockerPlatform {
    fn name(&self) -> &'static str {
        "docker"
    }
    
    async fn spawn_node(&self, spawn_config: SpawnConfig) -> anyhow::Result<SpawnedNode> {
        log::info!("🐳 [Docker] Spawning container {} on port {}", spawn_config.node_id, spawn_config.port);
        
        let image = self.config.docker_image.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Docker image not configured"))?;
        
        // Create storage directory on host
        let host_storage_path = format!("{}/{}", self.config.data_path, spawn_config.node_id);
        std::fs::create_dir_all(&host_storage_path)?;
        
        // Port mapping: host_port:container_port
        let mut port_bindings = HashMap::new();
        port_bindings.insert(
            format!("{}/tcp", spawn_config.port),
            Some(vec![PortBinding {
                host_ip: Some("0.0.0.0".to_string()),
                host_port: Some(spawn_config.port.to_string()),
            }]),
        );
        
        // Volume mount: host_path:container_path
        let binds = vec![
            format!("{}:/data", host_storage_path),
        ];
        
        // Container configuration
        let host_config = HostConfig {
            port_bindings: Some(port_bindings),
            binds: Some(binds),
            memory: Some(spawn_config.memory_limit_mb as i64 * 1024 * 1024), // MB to bytes
            ..Default::default()
        };
        
        let container_config = Config {
            image: Some(image.clone()),
            cmd: Some(vec![
                "--port".to_string(),
                spawn_config.port.to_string(),
                "--storage-path".to_string(),
                "/data".to_string(),
                "--node-id".to_string(),
                spawn_config.node_id.clone(),
            ]),
            host_config: Some(host_config),
            labels: Some({
                let mut labels = HashMap::new();
                labels.insert("sutra.node_id".to_string(), spawn_config.node_id.clone());
                labels.insert("sutra.grid".to_string(), "true".to_string());
                labels
            }),
            ..Default::default()
        };
        
        // Create container
        let container_name = format!("sutra-storage-{}", spawn_config.node_id);
        let options = CreateContainerOptions {
            name: container_name.clone(),
            ..Default::default()
        };
        
        let container = self.docker.create_container(Some(options), container_config).await?;
        let container_id = container.id;
        
        log::info!("🐳 [Docker] Created container {} (ID: {})", container_name, container_id);
        
        // Start container
        self.docker.start_container(&container_id, None::<StartContainerOptions<String>>).await?;
        
        log::info!("✅ [Docker] Container {} started on port {}", spawn_config.node_id, spawn_config.port);
        
        // Get container info to extract PID (note: this is the container's init process PID on host)
        let inspect = self.docker.inspect_container(&container_id, None).await?;
        let pid = inspect.state
            .and_then(|s| s.pid)
            .unwrap_or(0) as u32;
        
        let mut metadata = HashMap::new();
        metadata.insert("container_id".to_string(), container_id.clone());
        metadata.insert("container_name".to_string(), container_name);
        metadata.insert("image".to_string(), image.clone());
        
        Ok(SpawnedNode {
            node_id: spawn_config.node_id,
            port: spawn_config.port,
            pid,
            storage_path: host_storage_path,
            started_at: current_timestamp(),
            restart_count: 0,
            metadata,
        })
    }
    
    async fn stop_node(&self, node_id: &str) -> anyhow::Result<()> {
        log::info!("🛑 [Docker] Stopping container for node {}", node_id);
        
        let container_name = format!("sutra-storage-{}", node_id);
        
        // Stop container (10 second timeout)
        self.docker.stop_container(&container_name, None).await?;
        
        // Remove container
        let options = RemoveContainerOptions {
            force: true,
            ..Default::default()
        };
        self.docker.remove_container(&container_name, Some(options)).await?;
        
        log::info!("✅ [Docker] Container {} stopped and removed", node_id);
        Ok(())
    }
    
    async fn is_node_alive(&self, node_id: &str) -> anyhow::Result<bool> {
        let container_name = format!("sutra-storage-{}", node_id);
        
        match self.docker.inspect_container(&container_name, None).await {
            Ok(inspect) => {
                let running = inspect.state
                    .and_then(|s| s.running)
                    .unwrap_or(false);
                Ok(running)
            }
            Err(_) => Ok(false), // Container doesn't exist
        }
    }
    
    async fn get_node_status(&self, node_id: &str) -> anyhow::Result<SpawnedNode> {
        let container_name = format!("sutra-storage-{}", node_id);
        
        let inspect = self.docker.inspect_container(&container_name, None).await?;
        
        let pid = inspect.state
            .as_ref()
            .and_then(|s| s.pid)
            .unwrap_or(0) as u32;
        
        let started_at = inspect.state
            .and_then(|s| s.started_at)
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.timestamp() as u64)
            .unwrap_or(0);
        
        // Extract port from network settings
        let port = inspect.network_settings
            .and_then(|ns| ns.ports)
            .and_then(|ports| {
                ports.iter()
                    .find(|(k, _)| k.contains("/tcp"))
                    .and_then(|(_, bindings)| {
                        bindings.as_ref()
                            .and_then(|b| b.first())
                            .and_then(|pb| pb.host_port.as_ref())
                            .and_then(|p| p.parse::<u32>().ok())
                    })
            })
            .unwrap_or(0);
        
        let mut metadata = HashMap::new();
        metadata.insert("container_id".to_string(), inspect.id.unwrap_or_default());
        metadata.insert("container_name".to_string(), container_name);
        
        Ok(SpawnedNode {
            node_id: node_id.to_string(),
            port,
            pid,
            storage_path: format!("{}/{}", self.config.data_path, node_id),
            started_at,
            restart_count: 0,
            metadata,
        })
    }
    
    async fn list_nodes(&self) -> anyhow::Result<Vec<SpawnedNode>> {
        // List all containers with sutra.grid=true label
        use bollard::container::ListContainersOptions;
        
        let mut filters = HashMap::new();
        filters.insert("label".to_string(), vec!["sutra.grid=true".to_string()]);
        
        let options = ListContainersOptions {
            all: true,
            filters,
            ..Default::default()
        };
        
        let containers = self.docker.list_containers(Some(options)).await?;
        
        let mut nodes = Vec::new();
        for container in containers {
            if let Some(labels) = container.labels {
                if let Some(node_id) = labels.get("sutra.node_id") {
                    match self.get_node_status(node_id).await {
                        Ok(node) => nodes.push(node),
                        Err(e) => log::warn!("Failed to get status for node {}: {}", node_id, e),
                    }
                }
            }
        }
        
        Ok(nodes)
    }
    
    async fn get_logs(&self, node_id: &str, lines: usize) -> anyhow::Result<Vec<String>> {
        let container_name = format!("sutra-storage-{}", node_id);
        
        let options = LogsOptions::<String> {
            stdout: true,
            stderr: true,
            tail: lines.to_string(),
            ..Default::default()
        };
        
        let mut log_stream = self.docker.logs(&container_name, Some(options));
        let mut logs = Vec::new();
        
        while let Some(log_result) = log_stream.next().await {
            match log_result {
                Ok(log_output) => {
                    logs.push(log_output.to_string());
                }
                Err(e) => {
                    log::warn!("Error reading log: {}", e);
                    break;
                }
            }
        }
        
        Ok(logs)
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

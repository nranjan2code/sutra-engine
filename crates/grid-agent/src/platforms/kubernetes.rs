/// Kubernetes-based platform adapter for cloud-native deployments
/// 
/// This platform spawns storage-server as Kubernetes pods, suitable for:
/// - Production Kubernetes clusters
/// - Cloud platforms (EKS, GKE, AKS)
/// - Multi-node distributed deployments
/// - Enterprise-grade orchestration

use super::{Platform, PlatformConfig, SpawnConfig, SpawnedNode};
use async_trait::async_trait;
use k8s_openapi::api::core::v1::{Pod, PodSpec, Container, PersistentVolumeClaim, Volume, VolumeMount, PersistentVolumeClaimSpec, ResourceRequirements};
use k8s_openapi::api::core::v1::ContainerPort;
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use kube::{Api, Client, Config as KubeConfig};
use kube::api::{PostParams, DeleteParams, LogParams};
use std::collections::{HashMap, BTreeMap};

/// Kubernetes platform adapter
pub struct KubernetesPlatform {
    config: PlatformConfig,
    client: Client,
    namespace: String,
}

impl KubernetesPlatform {
    pub async fn new(config: PlatformConfig) -> anyhow::Result<Self> {
        // Load kubeconfig (from file or in-cluster)
        let kube_config = if let Some(kubeconfig_path) = &config.k8s_kubeconfig {
            KubeConfig::from_kubeconfig(&kube::config::KubeConfigOptions {
                config: &std::fs::read_to_string(kubeconfig_path)?,
                ..Default::default()
            }).await?
        } else {
            // Try in-cluster config first, fall back to default kubeconfig
            match KubeConfig::incluster() {
                Ok(cfg) => cfg,
                Err(_) => KubeConfig::from_kubeconfig(&Default::default()).await?,
            }
        };
        
        let client = Client::try_from(kube_config)?;
        let namespace = config.k8s_namespace.clone().unwrap_or_else(|| "sutra-grid".to_string());
        
        log::info!("☸️  [Kubernetes] Connected to cluster (namespace: {})", namespace);
        
        Ok(Self {
            config,
            client,
            namespace,
        })
    }
}

#[async_trait]
impl Platform for KubernetesPlatform {
    fn name(&self) -> &'static str {
        "kubernetes"
    }
    
    async fn spawn_node(&self, spawn_config: SpawnConfig) -> anyhow::Result<SpawnedNode> {
        log::info!("☸️  [Kubernetes] Spawning pod {} on port {}", spawn_config.node_id, spawn_config.port);
        
        let image = self.config.docker_image.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Docker image not configured for Kubernetes"))?;
        
        let pod_name = format!("sutra-storage-{}", spawn_config.node_id);
        let pvc_name = format!("{}-pvc", pod_name);
        
        // Create PVC for persistent storage
        let pvc = self.create_pvc(&pvc_name, &spawn_config).await?;
        
        // Create Pod specification
        let pod = self.create_pod_spec(&pod_name, &pvc_name, image, &spawn_config)?;
        
        // Create pod via API
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), &self.namespace);
        let created_pod = pods.create(&PostParams::default(), &pod).await?;
        
        log::info!("☸️  [Kubernetes] Created pod {}", pod_name);
        
        // Extract metadata
        let uid = created_pod.metadata.uid.unwrap_or_default();
        let mut metadata = HashMap::new();
        metadata.insert("pod_name".to_string(), pod_name.clone());
        metadata.insert("pod_uid".to_string(), uid);
        metadata.insert("namespace".to_string(), self.namespace.clone());
        metadata.insert("pvc_name".to_string(), pvc_name);
        
        log::info!("✅ [Kubernetes] Pod {} created on port {}", spawn_config.node_id, spawn_config.port);
        
        Ok(SpawnedNode {
            node_id: spawn_config.node_id,
            port: spawn_config.port,
            pid: 0, // Kubernetes doesn't expose host PIDs directly
            storage_path: format!("/data/{}", spawn_config.node_id),
            started_at: current_timestamp(),
            restart_count: 0,
            metadata,
        })
    }
    
    async fn stop_node(&self, node_id: &str) -> anyhow::Result<()> {
        log::info!("🛑 [Kubernetes] Stopping pod for node {}", node_id);
        
        let pod_name = format!("sutra-storage-{}", node_id);
        let pvc_name = format!("{}-pvc", pod_name);
        
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), &self.namespace);
        
        // Delete pod
        pods.delete(&pod_name, &DeleteParams::default()).await?;
        
        log::info!("☸️  [Kubernetes] Pod {} deleted", pod_name);
        
        // Delete PVC (optional, depends on retention policy)
        let pvcs: Api<PersistentVolumeClaim> = Api::namespaced(self.client.clone(), &self.namespace);
        if let Err(e) = pvcs.delete(&pvc_name, &DeleteParams::default()).await {
            log::warn!("⚠️  Could not delete PVC {}: {}", pvc_name, e);
        }
        
        log::info!("✅ [Kubernetes] Node {} stopped", node_id);
        Ok(())
    }
    
    async fn is_node_alive(&self, node_id: &str) -> anyhow::Result<bool> {
        let pod_name = format!("sutra-storage-{}", node_id);
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), &self.namespace);
        
        match pods.get(&pod_name).await {
            Ok(pod) => {
                let running = pod.status
                    .and_then(|s| s.phase)
                    .map(|phase| phase == "Running")
                    .unwrap_or(false);
                Ok(running)
            }
            Err(_) => Ok(false),
        }
    }
    
    async fn get_node_status(&self, node_id: &str) -> anyhow::Result<SpawnedNode> {
        let pod_name = format!("sutra-storage-{}", node_id);
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), &self.namespace);
        
        let pod = pods.get(&pod_name).await?;
        
        let started_at = pod.status
            .as_ref()
            .and_then(|s| s.start_time.as_ref())
            .and_then(|t| chrono::DateTime::parse_from_rfc3339(&t.0.to_rfc3339()).ok())
            .map(|dt| dt.timestamp() as u64)
            .unwrap_or(0);
        
        let restart_count = pod.status
            .as_ref()
            .and_then(|s| s.container_statuses.as_ref())
            .and_then(|cs| cs.first())
            .map(|c| c.restart_count)
            .unwrap_or(0) as u32;
        
        // Extract port from pod spec
        let port = pod.spec
            .as_ref()
            .and_then(|s| s.containers.first())
            .and_then(|c| c.ports.as_ref())
            .and_then(|ports| ports.first())
            .map(|p| p.container_port as u32)
            .unwrap_or(0);
        
        let mut metadata = HashMap::new();
        metadata.insert("pod_name".to_string(), pod_name);
        metadata.insert("namespace".to_string(), self.namespace.clone());
        if let Some(uid) = pod.metadata.uid {
            metadata.insert("pod_uid".to_string(), uid);
        }
        
        Ok(SpawnedNode {
            node_id: node_id.to_string(),
            port,
            pid: 0,
            storage_path: format!("/data/{}", node_id),
            started_at,
            restart_count,
            metadata,
        })
    }
    
    async fn list_nodes(&self) -> anyhow::Result<Vec<SpawnedNode>> {
        use kube::api::ListParams;
        
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), &self.namespace);
        
        let lp = ListParams::default().labels("sutra.grid=true");
        let pod_list = pods.list(&lp).await?;
        
        let mut nodes = Vec::new();
        for pod in pod_list {
            if let Some(labels) = pod.metadata.labels {
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
        let pod_name = format!("sutra-storage-{}", node_id);
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), &self.namespace);
        
        let log_params = LogParams {
            tail_lines: Some(lines as i64),
            ..Default::default()
        };
        
        let logs = pods.logs(&pod_name, &log_params).await?;
        Ok(logs.lines().map(|s| s.to_string()).collect())
    }
}

impl KubernetesPlatform {
    /// Create a PersistentVolumeClaim for storage
    async fn create_pvc(&self, pvc_name: &str, config: &SpawnConfig) -> anyhow::Result<PersistentVolumeClaim> {
        let pvcs: Api<PersistentVolumeClaim> = Api::namespaced(self.client.clone(), &self.namespace);
        
        let mut resources = BTreeMap::new();
        resources.insert("storage".to_string(), Quantity(format!("{}Gi", config.memory_limit_mb / 1024)));
        
        let pvc = PersistentVolumeClaim {
            metadata: kube::api::ObjectMeta {
                name: Some(pvc_name.to_string()),
                labels: Some({
                    let mut labels = BTreeMap::new();
                    labels.insert("sutra.grid".to_string(), "true".to_string());
                    labels.insert("sutra.node_id".to_string(), config.node_id.clone());
                    labels
                }),
                ..Default::default()
            },
            spec: Some(PersistentVolumeClaimSpec {
                access_modes: Some(vec!["ReadWriteOnce".to_string()]),
                resources: Some(ResourceRequirements {
                    requests: Some(resources.clone()),
                    limits: Some(resources),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        
        let created_pvc = pvcs.create(&PostParams::default(), &pvc).await?;
        log::info!("☸️  [Kubernetes] Created PVC {}", pvc_name);
        
        Ok(created_pvc)
    }
    
    /// Create a Pod specification
    fn create_pod_spec(&self, pod_name: &str, pvc_name: &str, image: &str, config: &SpawnConfig) -> anyhow::Result<Pod> {
        let mut labels = BTreeMap::new();
        labels.insert("sutra.grid".to_string(), "true".to_string());
        labels.insert("sutra.node_id".to_string(), config.node_id.clone());
        labels.insert("app".to_string(), "sutra-storage".to_string());
        
        let container = Container {
            name: "storage-server".to_string(),
            image: Some(image.to_string()),
            args: Some(vec![
                "--port".to_string(),
                config.port.to_string(),
                "--storage-path".to_string(),
                "/data".to_string(),
                "--node-id".to_string(),
                config.node_id.clone(),
            ]),
            ports: Some(vec![ContainerPort {
                container_port: config.port as i32,
                protocol: Some("TCP".to_string()),
                ..Default::default()
            }]),
            volume_mounts: Some(vec![VolumeMount {
                name: "storage".to_string(),
                mount_path: "/data".to_string(),
                ..Default::default()
            }]),
            resources: Some(ResourceRequirements {
                limits: Some({
                    let mut limits = BTreeMap::new();
                    limits.insert("memory".to_string(), Quantity(format!("{}Mi", config.memory_limit_mb)));
                    limits
                }),
                requests: Some({
                    let mut requests = BTreeMap::new();
                    requests.insert("memory".to_string(), Quantity(format!("{}Mi", config.memory_limit_mb / 2)));
                    requests
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        
        let pod = Pod {
            metadata: kube::api::ObjectMeta {
                name: Some(pod_name.to_string()),
                labels: Some(labels),
                ..Default::default()
            },
            spec: Some(PodSpec {
                containers: vec![container],
                volumes: Some(vec![Volume {
                    name: "storage".to_string(),
                    persistent_volume_claim: Some(k8s_openapi::api::core::v1::PersistentVolumeClaimVolumeSource {
                        claim_name: pvc_name.to_string(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }]),
                restart_policy: Some("Always".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        
        Ok(pod)
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

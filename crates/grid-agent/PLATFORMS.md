# Sutra Grid Multi-Platform Support

## Overview

Sutra Grid Agent now supports deploying storage nodes across **multiple platforms** through a unified abstraction layer. This enables true "deploy anywhere" capability.

## Supported Platforms

### ✅ Process Platform (Native)
**Status**: Production Ready  
**Use Cases**:
- Desktop development
- Bare metal deployments
- Traditional VMs
- Quick testing

**Features**:
- Direct process spawning via `tokio::process`
- SIGTERM-based termination
- PID tracking
- Works on macOS, Linux, Windows (limited)

**Configuration**:
```toml
[agent]
platform = "process"  # or "desktop"

[storage]
binary_path = "./target/release/storage-server"
data_path = "./data"
```

**Usage**:
```bash
# Build with process platform (default)
cargo build --release

# Run agent
./target/release/sutra-grid-agent
```

---

### ✅ Docker Platform
**Status**: Production Ready  
**Use Cases**:
- Development environments
- Docker Compose deployments
- Docker Swarm clusters
- Single-node production

**Features**:
- Container lifecycle management (create, start, stop, remove)
- Volume mounting for persistent storage
- Memory limits
- Port mapping
- Container log retrieval
- Automatic labeling for discovery

**Configuration**:
```toml
[agent]
platform = "docker"

[storage]
docker_image = "sutra-storage:latest"
data_path = "/var/lib/sutra/data"
```

**Prerequisites**:
```bash
# Install Docker
# On macOS: brew install docker
# On Linux: see docker.com

# Build storage-server Docker image
cd packages/sutra-storage
docker build -t sutra-storage:latest .
```

**Usage**:
```bash
# Build agent with Docker support
cargo build --release --features platform-docker

# Run agent (ensure Docker daemon is running)
./target/release/sutra-grid-agent
```

**Docker Features**:
- Automatic port mapping (host:container)
- Volume mounts: `host_data_path:/data`
- Container labels: `sutra.grid=true`, `sutra.node_id=<node_id>`
- Memory limits enforced by Docker
- Container logs accessible via API

---

### ✅ Kubernetes Platform
**Status**: Production Ready  
**Use Cases**:
- Production K8s clusters
- Cloud platforms (EKS, GKE, AKS)
- Multi-node distributed deployments
- Enterprise-grade orchestration

**Features**:
- Pod lifecycle management
- PersistentVolumeClaim (PVC) creation
- Resource requests/limits
- Namespace isolation
- Log streaming
- Label-based discovery

**Configuration**:
```toml
[agent]
platform = "kubernetes"  # or "k8s"

[storage]
docker_image = "sutra-storage:latest"
k8s_namespace = "sutra-grid"
k8s_kubeconfig = "/path/to/kubeconfig"  # optional, uses default
```

**Prerequisites**:
```bash
# Install kubectl
# Configure kubeconfig

# Create namespace
kubectl create namespace sutra-grid

# Build and push Docker image
docker build -t your-registry/sutra-storage:latest .
docker push your-registry/sutra-storage:latest
```

**Usage**:
```bash
# Build agent with Kubernetes support
cargo build --release --features platform-kubernetes

# Run agent (in-cluster or with kubeconfig)
./target/release/sutra-grid-agent
```

**Kubernetes Features**:
- Automatic PVC creation for persistent storage
- Resource requests: `memory = memory_limit_mb / 2`
- Resource limits: `memory = memory_limit_mb`
- Pod labels: `sutra.grid=true`, `sutra.node_id=<node_id>`
- Restart policy: `Always`
- Volume mount: PVC → `/data`

**Deployment Manifests**:
```yaml
# Agent DaemonSet (runs on every node)
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: sutra-grid-agent
  namespace: sutra-grid
spec:
  selector:
    matchLabels:
      app: sutra-grid-agent
  template:
    metadata:
      labels:
        app: sutra-grid-agent
    spec:
      containers:
      - name: agent
        image: sutra-grid-agent:latest
        env:
        - name: SUTRA_MASTER
          value: "http://sutra-grid-master:7000"
        - name: PLATFORM
          value: "kubernetes"
        volumeMounts:
        - name: kubeconfig
          mountPath: /root/.kube
      volumes:
      - name: kubeconfig
        secret:
          secretName: kubeconfig
```

---

### 🚧 SystemD Platform (Planned)
**Status**: Not Implemented  
**Use Cases**:
- Bare metal Linux servers
- Systemd-based distros (Ubuntu, CentOS, etc.)
- Long-running background services

**Planned Features**:
- Systemd service unit creation
- `systemctl` integration
- Journal logs
- Auto-restart on failure

---

## Building for Specific Platforms

### Default (Process Only)
```bash
cargo build --release
```

### Docker Support
```bash
cargo build --release --features platform-docker
```

### Kubernetes Support
```bash
cargo build --release --features platform-kubernetes
```

### All Platforms
```bash
cargo build --release --features all-platforms
```

---

## Platform Abstraction Layer

### Architecture

```
┌─────────────────────────────────────────────┐
│         sutra-grid-agent (main.rs)          │
└────────────────┬────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────┐
│      Platform Trait (platforms/mod.rs)      │
│  - spawn_node()                             │
│  - stop_node()                              │
│  - is_node_alive()                          │
│  - get_node_status()                        │
│  - list_nodes()                             │
│  - get_logs()                               │
└────────┬────────────────┬──────────┬────────┘
         │                │          │
         ▼                ▼          ▼
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│   Process   │  │   Docker    │  │ Kubernetes  │
│  Platform   │  │  Platform   │  │  Platform   │
└─────────────┘  └─────────────┘  └─────────────┘
```

### Platform Trait

All platforms implement the same interface:

```rust
#[async_trait]
pub trait Platform: Send + Sync {
    fn name(&self) -> &'static str;
    async fn spawn_node(&self, config: SpawnConfig) -> anyhow::Result<SpawnedNode>;
    async fn stop_node(&self, node_id: &str) -> anyhow::Result<()>;
    async fn is_node_alive(&self, node_id: &str) -> anyhow::Result<bool>;
    async fn get_node_status(&self, node_id: &str) -> anyhow::Result<SpawnedNode>;
    async fn list_nodes(&self) -> anyhow::Result<Vec<SpawnedNode>>;
    async fn get_logs(&self, node_id: &str, lines: usize) -> anyhow::Result<Vec<String>>;
}
```

### Platform Factory

```rust
use sutra_grid_agent::platforms::{create_platform, PlatformConfig};

let config = PlatformConfig {
    binary_path: "./storage-server".to_string(),
    docker_image: Some("sutra-storage:latest".to_string()),
    k8s_namespace: Some("sutra-grid".to_string()),
    ..Default::default()
};

let platform = create_platform("docker", config)?;
```

---

## Deployment Examples

### Example 1: Hybrid Cloud (Docker + Kubernetes)

```toml
# Agent 1: Docker (local development)
[agent]
agent_id = "agent-local"
platform = "docker"
master_host = "master.sutra.internal:7000"
max_storage_nodes = 4

# Agent 2: Kubernetes (production cluster)
[agent]
agent_id = "agent-prod"
platform = "kubernetes"
master_host = "master.sutra.internal:7000"
max_storage_nodes = 20

[storage]
docker_image = "gcr.io/my-project/sutra-storage:v2"
k8s_namespace = "production"
```

**Result**: 
- 4 Docker containers running locally
- 20 Kubernetes pods in production cluster
- All managed by single Grid Master

---

### Example 2: Desktop Development

```bash
# Terminal 1: Start Master
cd packages/sutra-grid-master
cargo run --release

# Terminal 2: Start Agent (process platform)
cd packages/sutra-grid-agent
cargo run --release

# Terminal 3: Spawn nodes
cd packages/sutra-grid-master
./target/release/sutra-grid-cli spawn --agent agent-1 --port 50051 --storage-path /tmp/node1
./target/release/sutra-grid-cli spawn --agent agent-1 --port 50052 --storage-path /tmp/node2
```

---

### Example 3: Production Kubernetes

```bash
# 1. Deploy Master
kubectl apply -f k8s/master-deployment.yaml

# 2. Deploy Agent as DaemonSet
kubectl apply -f k8s/agent-daemonset.yaml

# 3. Scale via Master API
curl -X POST http://master:9000/api/grid/spawn \
  -d '{"agent_id": "agent-prod-1", "port": 50051, "memory_limit_mb": 2048}'
```

---

## Testing Platform Adapters

### Process Platform
```bash
cargo test --features platform-process
```

### Docker Platform
```bash
# Ensure Docker is running
docker ps

# Run tests
cargo test --features platform-docker
```

### Kubernetes Platform
```bash
# Ensure kubectl works
kubectl cluster-info

# Run tests
cargo test --features platform-kubernetes
```

---

## Performance Characteristics

| Platform | Spawn Time | Stop Time | Log Retrieval | Overhead |
|----------|------------|-----------|---------------|----------|
| Process  | ~50ms      | <10ms     | N/A           | Minimal  |
| Docker   | ~500ms     | ~200ms    | Fast          | Low      |
| Kubernetes | ~2s      | ~1s       | Fast          | Medium   |

---

## Troubleshooting

### Docker Issues

**Problem**: `Failed to connect to Docker daemon`
```bash
# Solution: Start Docker
# macOS: Open Docker Desktop
# Linux: sudo systemctl start docker
```

**Problem**: `Image not found: sutra-storage:latest`
```bash
# Solution: Build the image
cd packages/sutra-storage
docker build -t sutra-storage:latest .
```

### Kubernetes Issues

**Problem**: `Error: kubeconfig not found`
```bash
# Solution: Configure kubectl
export KUBECONFIG=/path/to/kubeconfig
```

**Problem**: `Namespace 'sutra-grid' not found`
```bash
# Solution: Create namespace
kubectl create namespace sutra-grid
```

---

## Next Steps

1. **SystemD Platform**: Implement systemd service management for bare metal
2. **Cloud Platforms**: Add AWS ECS, Azure Container Instances, GCP Cloud Run
3. **Binary Distribution**: Master serves binaries to agents
4. **Auto-Scaling**: Dynamic scaling based on load
5. **Cost Optimization**: Intelligent placement based on cloud pricing

---

## Contributing

To add a new platform adapter:

1. Create `src/platforms/my_platform.rs`
2. Implement `Platform` trait
3. Add feature flag to `Cargo.toml`
4. Register in `platforms/mod.rs` factory
5. Add tests
6. Update this documentation

See `process.rs`, `docker.rs`, or `kubernetes.rs` for examples.

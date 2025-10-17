# Runtime Architecture: All Processes Mapped

## Development Environment (Typical)

### After Storage Server (Current State)

```
┌─────────────────────────────────────────────────────────────────┐
│ Process 1: storage-server (Rust)                                │
│ Port: 50051 (gRPC)                                               │
│ PID: 10000                                                       │
│                                                                  │
│  ┌──────────────────────────────────────────┐                   │
│  │ ConcurrentMemory                          │                   │
│  │  - In-memory knowledge graph              │                   │
│  │  - HNSW vector index                      │                   │
│  │  - Loads storage.dat on startup           │                   │
│  │  - Persists every 50K concepts            │                   │
│  │  - gRPC server (tonic)                    │                   │
│  └──────────────────────────────────────────┘                   │
│                                                                  │
│  Memory: ~100MB (for 10K concepts)                               │
│  Startup: 100ms (loads existing data)                            │
└─────────────────────────────────────────────────────────────────┘
                              ↑
                              │ gRPC (port 50051)
                              │
        ┌─────────────────────┼─────────────────────────┐
        │                     │                         │
┌───────┴───┐         ┌───────┴───┐           ┌─────────┴─────┐
│ Process 2 │         │ Process 3 │           │ Process 4     │
│           │         │           │           │               │
│sutra-api  │         │hybrid/api │           │sutra-control  │
│Port: 8000 │         │Port: 8001 │           │Port: 5000     │
│PID: 10001 │         │PID: 10002 │           │PID: 10003     │
│           │         │           │           │               │
│ ┌────────┐│         │ ┌────────┐│           │ ┌──────────┐  │
│ │SutraAI ││         │ │SutraAI ││           │ │Monitoring│  │
│ │  ↓     ││         │ │  ↓     ││           │ │Dashboard │  │
│ │Storage ││         │ │Storage ││           │ │          │  │
│ │Client  ││         │ │Client  ││           │ │+ Storage │  │
│ │(gRPC)  ││         │ │(gRPC)  ││           │ │  Client  │  │
│ └────────┘│         │ └────────┘│           │ └──────────┘  │
└───────────┘         └───────────┘           └───────────────┘
                              ↑
                              │ HTTP
                              │
                    ┌─────────┴──────────┐
                    │ Process 5          │
                    │                    │
                    │ sutra-client       │
                    │ Port: 3000         │
                    │ PID: 10004         │
                    │                    │
                    │ React SPA          │
                    │ Proxies to :8000   │
                    └────────────────────┘

SOLUTION: All processes share SAME in-memory state via storage server!
```

## Production Deployment (Full Stack)

### Minimal Production (Single Machine)

```
┌────────────────────────────────────────────────┐
│ Machine: sutra-prod-01 (AWS EC2 / GCP VM)     │
│ 4 vCPU, 16GB RAM                               │
├────────────────────────────────────────────────┤
│                                                │
│ ┌────────────────────────────────────────┐    │
│ │ systemd: storage-server.service        │    │
│ │ Port: 50051                             │    │
│ │ Storage: /var/lib/sutra/storage.dat     │    │
│ └────────────────────────────────────────┘    │
│                      ↑                         │
│                      │                         │
│ ┌────────────────────┴──────────────────┐     │
│ │ systemd: sutra-api.service            │     │
│ │ Port: 8000                              │     │
│ │ Workers: 4 (uvicorn)                    │     │
│ │ Env: SUTRA_STORAGE_MODE=server          │     │
│ │      SUTRA_STORAGE_SERVER=localhost:50051│    │
│ └────────────────────────────────────────┘    │
│                      ↑                         │
│                      │                         │
│ ┌────────────────────┴──────────────────┐     │
│ │ nginx: reverse proxy                   │     │
│ │ Port: 80 / 443 (HTTPS)                  │     │
│ │ - Serves static files (React build)     │     │
│ │ - Proxies /api to :8000                  │     │
│ └────────────────────────────────────────┘    │
│                                                │
└────────────────────────────────────────────────┘
```

### High Availability (Multi-Machine)

```
┌─────────────────────────────────────────────────────┐
│ Load Balancer (AWS ALB / nginx)                     │
│ HTTPS → sutra.example.com                           │
└──────────────┬──────────────┬───────────────────────┘
               │              │
       ┌───────┴────┐  ┌──────┴────────┐
       │            │  │               │
┌──────▼─────┐  ┌──▼───────┐  ┌───────▼────┐
│ API Node 1 │  │API Node 2│  │ API Node 3 │
│            │  │          │  │            │
│ sutra-api  │  │sutra-api │  │ sutra-api  │
│ Port: 8000 │  │Port: 8000│  │ Port: 8000 │
└──────┬─────┘  └──┬───────┘  └───────┬────┘
       │           │                   │
       │ gRPC      │ gRPC             │ gRPC
       │           │                   │
       └───────────┼───────────────────┘
                   │
            ┌──────▼───────┐
            │ Storage Node │
            │              │
            │storage-server│
            │Port: 50051   │
            │              │
            │Master Write  │
            └──────────────┘
                   │
                   │ Replication (future)
                   │
         ┌─────────┴─────────┐
         │                   │
    ┌────▼─────┐      ┌─────▼────┐
    │ Replica 1│      │ Replica 2│
    │ (Read)   │      │ (Read)   │
    └──────────┘      └──────────┘
```

## Process Communication Matrix

| Process | Listens On | Connects To | Protocol | Purpose |
|---------|-----------|-------------|----------|---------|
| **storage-server** | :50051 | - | gRPC | Storage service |
| **sutra-api** | :8000 | storage-server:50051 | gRPC | Main REST API |
| **sutra-hybrid/api** | :8001 | storage-server:50051 | gRPC | OpenAI-compatible API |
| **sutra-control** | :5000 | storage-server:50051 | gRPC | Admin dashboard |
| **sutra-client** | :3000 | sutra-api:8000 | HTTP | React dev server |
| **sutra-cli** | - | storage-server:50051 | gRPC | CLI tool (future) |

## Data Flow Examples

### Example 1: User Query via Web UI

```
User Browser
    ↓ HTTP GET /api/query
sutra-client (:3000)
    ↓ proxy to :8000
sutra-api (:8000)
    ↓ SutraAI.ask()
ReasoningEngine
    ↓ RustStorageAdapter
StorageClient (Python)
    ↓ gRPC LearnConcept()
storage-server (:50051)
    ↓ ConcurrentMemory
In-Memory Graph
    ↓ Response
storage-server
    ↓ gRPC response
StorageClient
    ↓ Python dict
ReasoningEngine
    ↓ Format result
sutra-api
    ↓ JSON response
sutra-client
    ↓ Render UI
User sees answer with reasoning paths
```

### Example 2: Learning via Admin Dashboard

```
Admin User
    ↓ HTTP POST /admin/bulk-learn
sutra-control (:5000)
    ↓ Get storage client
StorageClient (Python)
    ↓ gRPC LearnConcept() x 1000
storage-server (:50051)
    ↓ Batch writes
WriteLog (lock-free append)
    ↓ Background reconciliation
ReadView (immutable snapshot)
    ↓ Persist to disk
storage.dat updated
```

### Example 3: Multi-API Consistency

```
Terminal 1: Learn via API 1
    ↓
sutra-api (:8000)
    ↓ gRPC write
storage-server (:50051)
    ↓ Update in-memory graph

Terminal 2: Query via API 2 (immediately)
    ↓
sutra-hybrid/api (:8001)
    ↓ gRPC read
storage-server (:50051)
    ↓ Returns updated data
CONSISTENT! ✓
```

## Environment Configuration

### Development (.env)

```bash
# Storage mode (embedded or server)
SUTRA_STORAGE_MODE=server

# Storage server address
SUTRA_STORAGE_SERVER=localhost:50051

# API configuration
SUTRA_API_PORT=8000
SUTRA_API_HOST=0.0.0.0

# Storage path (for embedded mode or server data dir)
SUTRA_STORAGE_PATH=./knowledge

# Logging
RUST_LOG=info
SUTRA_LOG_LEVEL=INFO
```

### Production (.env.production)

```bash
# Always use server mode in production
SUTRA_STORAGE_MODE=server

# Connect to dedicated storage node
SUTRA_STORAGE_SERVER=storage.internal:50051

# API configuration
SUTRA_API_PORT=8000
SUTRA_API_WORKERS=4
SUTRA_API_HOST=0.0.0.0

# Storage path on storage node
SUTRA_STORAGE_PATH=/var/lib/sutra

# Logging
RUST_LOG=warn
SUTRA_LOG_LEVEL=WARNING

# Monitoring
SENTRY_DSN=https://...
PROMETHEUS_PORT=9090
```

## Resource Requirements

### Storage Server

| Scale | Concepts | RAM | CPU | Disk | Startup Time |
|-------|----------|-----|-----|------|--------------|
| Small | 10K | 100MB | 0.5 core | 50MB | 100ms |
| Medium | 100K | 1GB | 1 core | 500MB | 500ms |
| Large | 1M | 10GB | 2 cores | 5GB | 5s |
| X-Large | 10M | 100GB | 4 cores | 50GB | 50s |

### API Servers (per instance)

| Component | RAM | CPU | Notes |
|-----------|-----|-----|-------|
| sutra-api | 512MB | 0.5 core | Per worker |
| sutra-hybrid/api | 512MB | 0.5 core | Per worker |
| sutra-control | 256MB | 0.2 core | Lightweight |
| sutra-client | - | - | Static files via nginx |

## Monitoring Endpoints

### Storage Server
```bash
# gRPC health check
grpcurl -plaintext localhost:50051 grpc.health.v1.Health/Check

# Storage stats
grpcurl -plaintext localhost:50051 sutra.storage.StorageService/GetStats
```

### API Servers
```bash
# Health check
curl http://localhost:8000/health

# Metrics
curl http://localhost:8000/metrics

# System stats
curl http://localhost:8000/stats
```

### Admin Dashboard
```bash
# Dashboard UI
open http://localhost:5000

# Component status API
curl http://localhost:5000/api/status
```

## Deployment Checklist

### Prerequisites
- [ ] Rust 1.70+ installed
- [ ] Python 3.11+ installed
- [ ] Node.js 18+ (for client)
- [ ] maturin (for Rust bindings)
- [ ] grpcurl (for testing)

### Build
```bash
# Build storage server
cd packages/sutra-storage
cargo build --release --features="server"

# Build Python packages
pip install -e packages/sutra-core
pip install -e packages/sutra-hybrid
pip install -e packages/sutra-api

# Build client
cd packages/sutra-client
npm install
npm run build
```

### Start Services (Development)
```bash
# Terminal 1: Storage server
./packages/sutra-storage/bin/storage-server \
  --storage ./knowledge

# Terminal 2: API
export SUTRA_STORAGE_MODE=server
python -m sutra_api.main

# Terminal 3: Client
cd packages/sutra-client
npm run dev

# Terminal 4: Admin dashboard
export SUTRA_STORAGE_MODE=server
python -m sutra_control.main
```

### Verify
```bash
# Check storage server
grpcurl -plaintext localhost:50051 list

# Check API
curl http://localhost:8000/health

# Check client
open http://localhost:3000

# Check admin
open http://localhost:5000
```

## Conclusion

**With storage server:**
- ✅ All processes share state
- ✅ Hot reload without data loss
- ✅ Consistent views across all APIs
- ✅ Ready for horizontal scaling
- ✅ Clean separation of concerns
- ✅ Production-ready architecture

**One line of code changes everything.**

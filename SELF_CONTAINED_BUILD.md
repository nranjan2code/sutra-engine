# Self-Contained Sutra AI Build System

## Overview

Complete self-contained Docker build system with zero external dependencies. All base images are built from Alpine Linux rootfs without pulling from Docker Hub or any external registry.

## Quick Start

```bash
# Build entire system (first time)
./build.sh

# Deploy system
./sutra-deploy.sh up

# Or install from scratch
./sutra-deploy.sh install
```

## Architecture

### Self-Contained Base Images

All services use locally-built base images:

- **`sutra-base-python:latest`** - Python 3.11 + pip + build tools
- **`sutra-base-rust:latest`** - Rust nightly + cargo + build tools  
- **`sutra-base-runtime:latest`** - Minimal runtime with curl, netcat, bash
- **`sutra-base-node:latest`** - Node.js 18 + npm + yarn
- **`sutra-base-nginx:latest`** - Nginx with standard configuration

### Service Images

All Sutra services built using self-contained bases:

- **`sutra-storage-server:latest`** - Core knowledge graph storage
- **`sutra-api:latest`** - REST API service
- **`sutra-hybrid:latest`** - Semantic AI orchestration
- **`sutra-embedding-service:latest`** - High-performance embeddings
- **`sutra-client:latest`** - Web interface
- **`sutra-control:latest`** - Management UI
- **`sutra-grid-master:latest`** - Grid orchestration
- **`sutra-grid-agent:latest`** - Node management
- **`sutra-bulk-ingester:latest`** - High-performance data ingestion

## Build Process

1. **Download Alpine rootfs** (one-time): `alpine-minirootfs-3.18.4-x86_64.tar.gz`
2. **Build base images** from scratch using Alpine
3. **Build all services** using self-contained bases
4. **Ready for deployment** via docker-compose

## Benefits

✅ **Zero External Dependencies** - No Docker Hub pulls  
✅ **Air-Gap Compatible** - Works in isolated environments  
✅ **Enterprise Ready** - Meets strict security requirements  
✅ **Reproducible** - Identical builds every time  
✅ **Single Path** - One unified build system, no alternatives  

## File Structure

```
base-images/
├── python.Dockerfile          # Python 3.11 base
├── rust.Dockerfile           # Rust nightly base
├── runtime.Dockerfile        # Minimal runtime base  
├── node.Dockerfile           # Node.js 18 base
├── nginx.Dockerfile          # Nginx base
└── alpine-minirootfs-3.18.4-x86_64.tar.gz

build.sh                      # Unified build script
sutra-deploy.sh              # Deployment management
docker-compose-grid.yml      # Service orchestration
```

## Requirements

- Docker Engine 20+
- Docker Compose 2+
- Internet access for initial Alpine download only
- ~4GB free disk space

## Maintenance

- **Update Alpine**: Change version in `build.sh` and base image Dockerfiles
- **Update base images**: Modify `base-images/*.Dockerfile`  
- **Add services**: They'll automatically use self-contained bases
- **Clean rebuild**: `./sutra-deploy.sh clean && ./build.sh`

This system provides complete isolation from external registries while maintaining the full functionality of Sutra AI.
# Sutra AI - Build and Deploy Guide

**Single Source of Truth** for building and deploying the complete Sutra AI system.

## Quick Start

```bash
# 1. Build all Docker images
./build-all.sh

# 2. Deploy the system
./sutra-deploy.sh up

# 3. Verify deployment
./sutra-deploy.sh status
```

## Build System

### Single Build Script

**`./build-all.sh`** - Builds ALL services using official Docker Hub images.

**What it builds (9 services - all required):**
1. Storage Server (Rust) - Core knowledge graph storage
2. API (Python) - REST API interface 
3. Hybrid (Python) - Semantic AI orchestration
4. Client (Node + Nginx) - Interactive web UI
5. Control Center (React + Python) - System management dashboard
6. Grid Master (Rust) - Distributed orchestration
7. Grid Agent (Rust) - Node lifecycle management
8. Bulk Ingester (Rust + Python) - High-performance data ingestion
9. Embedding Service (Python) - nomic-embed-text-v1.5 (768-d)

**Official Images Used:**
- `python:3.11-slim` - Python runtime
- `rust:1.82-slim` - Rust compiler
- `node:18-slim` - Node.js runtime
- `nginx:alpine` - Web server
- `debian:bookworm-slim` - Minimal Linux runtime

**No custom base images** - Everything uses official, well-maintained images from Docker Hub.

### Build Output

The script shows:
- ✅ Successfully built services (all 9 required)
- ⚠️  Any failures (should be ZERO)
- Final image sizes
- Next steps

## Deployment System

### Single Deployment Script

**`./sutra-deploy.sh`** - Manages the entire system lifecycle.

**Commands:**
```bash
./sutra-deploy.sh up        # Start all services
./sutra-deploy.sh down      # Stop all services
./sutra-deploy.sh restart   # Restart services
./sutra-deploy.sh status    # Check service health
./sutra-deploy.sh validate  # Run health checks
./sutra-deploy.sh logs      # View logs (all or specific service)
./sutra-deploy.sh clean     # Remove all data and reset
```

**Services Started:**
- storage-server (port 50051) - Knowledge graph storage
- grid-event-storage (port 50052) - Grid observability
- sutra-api (port 8000) - REST API
- sutra-hybrid (port 8001) - Semantic AI
- sutra-embedding-service (port 8888) - Embeddings
- sutra-client (port 8080) - Web UI
- sutra-control (port 9000) - Management dashboard
- grid-master (ports 7001-7002) - Grid orchestration
- grid-agent-1 (port 8003) - Node management
- grid-agent-2 (port 8004) - Node management

## Complete Fresh Install

**From scratch:**
```bash
# 1. Clean everything (optional)
./sutra-deploy.sh down
docker system prune -af --volumes  # Removes ALL Docker data

# 2. Build fresh images
./build-all.sh

# 3. Deploy
./sutra-deploy.sh up

# 4. Verify
./sutra-deploy.sh status
```

## Clean Rebuild

**To rebuild with new changes:**
```bash
./clean-and-rebuild.sh
```

This script:
1. Stops all containers
2. Removes all Sutra images and volumes
3. Runs `./build-all.sh`
4. Runs `./sutra-deploy.sh up`
5. Verifies deployment

## Access URLs

After deployment:
- **Web UI**: http://localhost:8080
- **Control Center**: http://localhost:9000
- **API**: http://localhost:8000
- **Hybrid API**: http://localhost:8001
- **Embedding Service**: http://localhost:8888

## File Structure

```
sutra-models/
├── build-all.sh              # ⭐ Single build script
├── clean-and-rebuild.sh      # Complete fresh install
├── sutra-deploy.sh           # ⭐ Single deployment script
├── docker-compose-grid.yml   # Service configuration
├── packages/
│   ├── sutra-storage/Dockerfile       # Storage server
│   ├── sutra-api/Dockerfile           # API service
│   ├── sutra-hybrid/Dockerfile        # Hybrid service
│   ├── sutra-client/Dockerfile        # Client UI
│   ├── sutra-control/Dockerfile       # Control center
│   ├── sutra-grid-master/Dockerfile   # Grid master
│   ├── sutra-grid-agent/Dockerfile    # Grid agent
│   └── sutra-bulk-ingester/Dockerfile # Bulk ingester
└── docs/                     # Documentation
```

## Troubleshooting

### Build Issues

**Problem**: Service fails to build
```bash
# Build manually to see full error
docker build -f packages/SERVICE_NAME/Dockerfile -t sutra-SERVICE_NAME:latest .
```

**Problem**: Out of disk space
```bash
# Clean Docker cache
docker system prune -af
```

### Deployment Issues

**Problem**: Services won't start
```bash
# Check logs
./sutra-deploy.sh logs SERVICE_NAME

# Check Docker
docker ps -a | grep sutra
```

**Problem**: Port conflicts
```bash
# Find what's using the port
lsof -i :PORT_NUMBER

# Kill process
kill -9 PID
```

### Complete Reset

**Nuclear option** (removes everything):
```bash
./sutra-deploy.sh down
docker rm -f $(docker ps -aq)
docker rmi -f $(docker images -q | grep sutra)
docker volume rm $(docker volume ls -q | grep sutra)
docker system prune -af --volumes
./build-all.sh
./sutra-deploy.sh up
```

## Development Workflow

### Making Changes

1. **Edit code** in `packages/SERVICE_NAME/`
2. **Rebuild service**: 
   ```bash
   docker build -f packages/SERVICE_NAME/Dockerfile -t sutra-SERVICE_NAME:latest .
   ```
3. **Restart service**:
   ```bash
   docker-compose -f docker-compose-grid.yml restart SERVICE_NAME
   ```

### Testing

```bash
# Check service health
curl http://localhost:8000/health

# Run API tests
curl -X POST http://localhost:8001/sutra/query \
  -H "Content-Type: application/json" \
  -d '{"query": "test"}'
```

## Production Deployment

### Requirements
- Docker Engine 20+
- Docker Compose 2+
- 4GB RAM minimum
- 10GB disk space

### Performance

**Expected image sizes:**
- Storage Server: ~166 MB
- API: ~275 MB
- Hybrid: ~531 MB
- Client: ~83 MB
- Control: ~387 MB
- Grid Master: ~148 MB
- Grid Agent: ~146 MB
- Bulk Ingester: ~245 MB
- Embedding Service: ~1.32 GB

**Total:** ~3.3 GB for complete system

### Security

- All services run as non-root users
- No development tools in production images
- Multi-stage builds strip unnecessary files
- Official base images with security updates

## Documentation

- **PRODUCTION_OPTIMIZATION.md** - Image optimization details
- **OPTIMIZATION_SUMMARY.md** - Investigation and changes
- **QUICK_START_OPTIMIZATION.md** - Quick reference
- **TROUBLESHOOTING.md** - Detailed troubleshooting
- **ARCHITECTURE.md** - System architecture

## Summary

**Two scripts, one workflow:**
1. `./build-all.sh` - Build everything
2. `./sutra-deploy.sh up` - Deploy everything

That's it! Single path to production.

# Docker and Build File Cleanup Summary

## Overview
Cleaned up redundant Docker and build files to maintain only essential files for TCP-based end-to-end deployment.

## Date
2025-10-18

## Files Removed ❌

### Root Level
1. **`docker-compose-app.yml`** - Redundant compose file missing Grid services
2. **`docker-compose-simple.yml`** - Minimal test version with only storage + API
3. **`Makefile`** - Outdated, references non-existent `build-images.sh` from archive

### Package Level
4. **`packages/sutra-control/Dockerfile.fast`** - Unused backend-only variant
5. **`packages/sutra-control/Dockerfile.production`** - Unused duplicate of main Dockerfile

## Files Kept ✅

### Deployment Management
- **`sutra-deploy.sh`** - Single source of truth for all deployment operations
- **`docker-compose-grid.yml`** - Complete production stack (9 services)

### Root Dockerfiles
- **`Dockerfile.test-storage`** - Used for both storage-server and grid-event-storage

### Service Dockerfiles (packages/*)
- **`sutra-api/Dockerfile`** - Primary REST API
- **`sutra-client/Dockerfile`** - Interactive AI interface
- **`sutra-control/Dockerfile`** - Multi-stage React + FastAPI gateway
- **`sutra-hybrid/Dockerfile`** - Semantic embeddings API
- **`sutra-grid-agent/Dockerfile`** - Grid node management
- **`sutra-grid-master/Dockerfile`** - Grid orchestration
- **`sutra-storage/Dockerfile`** - Rust storage library build

## Production Deployment Structure

```
sutra-models/
├── sutra-deploy.sh              # Master deployment script
├── docker-compose-grid.yml      # Complete stack definition
├── Dockerfile.test-storage      # Storage service build
└── packages/
    ├── sutra-api/Dockerfile
    ├── sutra-client/Dockerfile
    ├── sutra-control/Dockerfile
    ├── sutra-hybrid/Dockerfile
    ├── sutra-grid-agent/Dockerfile
    ├── sutra-grid-master/Dockerfile
    └── sutra-storage/Dockerfile
```

## Deployment Commands (Single Source of Truth)

All operations now go through `sutra-deploy.sh`:

```bash
# First-time installation
./sutra-deploy.sh install

# Start all services
./sutra-deploy.sh up

# Stop all services
./sutra-deploy.sh down

# Check status
./sutra-deploy.sh status

# View logs
./sutra-deploy.sh logs [service-name]

# Maintenance menu
./sutra-deploy.sh maintenance

# Complete cleanup
./sutra-deploy.sh clean
```

## Service Architecture

**docker-compose-grid.yml** defines the complete TCP-based stack:

### Storage Layer (TCP Protocol)
- `storage-server` - Main knowledge graph (port 50051)
- `grid-event-storage` - Grid observability (port 50052)

### Grid Infrastructure (TCP Protocol)
- `grid-master` - Orchestration (HTTP 7001, TCP 7002)
- `grid-agent-1` - Node management (TCP 8003)
- `grid-agent-2` - Node management (TCP 8004)

### API Layer (REST)
- `sutra-api` - Primary REST API (port 8000)
- `sutra-hybrid` - Semantic embeddings API (port 8001)

### Web Interfaces (HTTP)
- `sutra-control` - Grid management + monitoring (port 9000)
- `sutra-client` - Interactive AI interface (port 8080)

## Rationale

**Why these files were removed:**
1. Multiple docker-compose files created confusion - `docker-compose-grid.yml` is comprehensive
2. `sutra-deploy.sh` provides all necessary commands with better UX
3. Unused Dockerfile variants added maintenance overhead
4. Old Makefile referenced archived scripts (`build-images.sh`)

**Why these files were kept:**
1. `sutra-deploy.sh` - Modern deployment script with colored output, status checks, maintenance menu
2. `docker-compose-grid.yml` - Single comprehensive stack definition used by deploy script
3. Service Dockerfiles - All actively referenced by docker-compose-grid.yml
4. `Dockerfile.test-storage` - Shared by both storage services

## Migration from Old Build System

**Before (Outdated):**
```bash
make build        # Used non-existent build-images.sh
make dev          # No compose file specified
make k8s-deploy   # K8s files not present
```

**After (Current):**
```bash
./sutra-deploy.sh build     # Builds from docker-compose-grid.yml
./sutra-deploy.sh up        # Complete stack with health checks
./sutra-deploy.sh status    # Service URLs + container status
```

## Archived Files

Older deployment files remain in `archive/` for reference:
- `archive/docker-compose.yml`
- `archive/docker-compose-v2.yml`
- `archive/build-images.sh`
- `archive/deploy-docker-grid.sh`
- `archive/deploy-optimized.sh`

## Next Steps

1. All deployment operations use `./sutra-deploy.sh`
2. No manual docker-compose commands needed
3. All services communicate via TCP binary protocol (no gRPC)
4. Health checks verify all 9 services on startup

## Verification

Check that all essential files are present:
```bash
# Deployment manager
ls -lh sutra-deploy.sh

# Complete stack
ls -lh docker-compose-grid.yml

# Storage build
ls -lh Dockerfile.test-storage

# Service builds
find packages/ -name "Dockerfile" -type f
```

Expected output: 8 files (1 deploy script, 1 compose file, 1 root Dockerfile, 7 service Dockerfiles)

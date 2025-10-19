# Sutra Grid Deployment Guide

## Single Source of Truth

All deployment operations are managed through **`./sutra-deploy.sh`** - this is the only script you need.

## 🚨 CRITICAL: Pre-Deployment Requirements

**Before running any deployment commands, ensure:**

1. **Ollama Service Running:**
```bash
# Install Ollama (if not installed)
curl -fsSL https://ollama.com/install.sh | sh

# Start Ollama and load embedding model
ollama pull granite-embedding:30m

# Verify
curl http://localhost:11434/api/tags | jq '.models[].name'
# Should include: granite-embedding:30m
```

2. **Docker & Docker Compose:**
```bash
docker --version  # 20.10+
docker-compose --version  # 1.29+
```

**⚠️ Without Ollama + embeddings, the system will NOT function correctly.**  
See [`docs/EMBEDDING_TROUBLESHOOTING.md`](docs/EMBEDDING_TROUBLESHOOTING.md) for details.

### 🔥 NEW: Unified Learning Architecture (2025-10-19)

**Key Deployment Change:** Storage server now owns the complete learning pipeline.

```
Storage Server:
  ├─ Embedding Generation (via Ollama HTTP)
  ├─ Association Extraction (Rust NLP)
  ├─ Atomic Storage (HNSW + WAL)
  └─ All clients delegate to this pipeline
```

**What This Means for Deployment:**
- ✅ No client-side embedding configuration needed
- ✅ Consistent behavior across all services
- ✅ Automatic embeddings for ALL learned concepts
- ✅ No "same answer" bug (fixed permanently)

**Environment Variables (Storage Server):**
```bash
SUTRA_OLLAMA_URL=http://host.docker.internal:11434  # Required
SUTRA_EMBEDDING_MODEL=granite-embedding:30m          # Default
SUTRA_EXTRACT_ASSOCIATIONS=true                     # Default
```

---

## Quick Start

### First-Time Installation
```bash
./sutra-deploy.sh install
```

This will:
1. Check prerequisites (Docker, Docker Compose)
2. Build all Docker images
3. Start all services
4. Display service URLs

### Common Operations

```bash
# Start all services
./sutra-deploy.sh up

# Stop all services
./sutra-deploy.sh down

# Restart all services
./sutra-deploy.sh restart

# View system status
./sutra-deploy.sh status

# View logs (all services)
./sutra-deploy.sh logs

# View logs (specific service)
./sutra-deploy.sh logs sutra-api

# Interactive maintenance menu
./sutra-deploy.sh maintenance
```

## Service URLs

Once deployed, access services at:

- **Sutra Control Center**: http://localhost:9000
  - Dashboard: http://localhost:9000/
  - Grid Management: http://localhost:9000/grid
  - Bulk Ingester UI: http://localhost:9000/bulk-ingester ✅
- **Sutra Client (UI)**: http://localhost:8080
- **Sutra API**: http://localhost:8000
- **Sutra Hybrid API**: http://localhost:8001
- **Grid Master (HTTP)**: http://localhost:7001
- **Grid Master (gRPC)**: localhost:7002

## Architecture

### Services
- **Storage Layer**: Main storage (50051), Grid event storage (50052)
- **Grid Infrastructure**: Grid Master (7001/7002), Grid Agents (8003/8004)
- **API Layer**: Sutra API (8000), Sutra Hybrid (8001)
- **Web Interfaces**: Sutra Control (9000), Sutra Client (8080)

### Data Persistence
All data is stored in Docker volumes:
- `storage-data`: Main knowledge graph storage
- `grid-event-data`: Grid observability events
- `agent1-data` & `agent2-data`: Grid agent storage nodes

## Maintenance

### Interactive Maintenance Menu
```bash
./sutra-deploy.sh maintenance
```

Options include:
1. View system status
2. Check service health
3. Restart unhealthy services
4. View logs
5. Clean up unused resources
6. Backup data volumes

### Backup Data
```bash
./sutra-deploy.sh maintenance
# Select option 6
```

Backups are saved to `./backups/` directory.

### Complete Cleanup
```bash
./sutra-deploy.sh clean
```

**Warning**: This removes all containers, volumes, and images. Use with caution!

## Troubleshooting

### Same Answer for All Questions

**Problem:** System returns identical answer regardless of question.

**Status:** ✅ **FIXED** (2025-10-19) - Unified learning architecture guarantees embeddings

**Historical Issue (Pre-2025-10-19):**
- Old architecture: Only Hybrid service generated embeddings
- API and Bulk Ingester learned without embeddings
- Result: "Same answer" bug

**Current Architecture (Post-2025-10-19):**
- Storage server owns complete learning pipeline
- ALL services automatically generate embeddings
- Bug cannot occur with new architecture

**Diagnosis (if issue persists):**
```bash
curl -s http://localhost:8000/stats | jq '.total_embeddings'
# If 0: Ollama not configured or storage corrupted
```

**Solution:**
```bash
# 1. Verify Ollama is running
curl http://localhost:11434/api/tags | jq '.models[].name'
# Should include: granite-embedding:30m

# 2. If Ollama missing, install and configure
ollama pull granite-embedding:30m

# 3. Restart storage server to pick up Ollama
docker restart sutra-storage

# 4. Learn a test fact (any service works now)
curl -X POST http://localhost:8001/sutra/learn \
  -H "Content-Type: application/json" \
  -d '{"text":"Test fact with embedding"}'

# 5. Verify embeddings generated
curl -s http://localhost:8000/stats | jq '.total_embeddings'
# Should be > 0

# 6. Test different queries return different answers
curl -X POST http://localhost:8001/sutra/query \
  -d '{"query":"What is the test?"}' | jq '.answer'
```

**See:** [`docs/UNIFIED_LEARNING_ARCHITECTURE.md`](docs/UNIFIED_LEARNING_ARCHITECTURE.md) for complete architecture documentation.

### Services Not Starting
```bash
# Check logs
./sutra-deploy.sh logs

# Check specific service
./sutra-deploy.sh logs grid-master

# Restart all services
./sutra-deploy.sh restart
```

### Ollama Connection Issues
```bash
# Test from host
curl http://localhost:11434/api/tags

# Test from container
docker exec sutra-hybrid curl http://host.docker.internal:11434/api/tags

# Check environment
docker exec sutra-hybrid env | grep OLLAMA
# Expected: SUTRA_OLLAMA_URL=http://host.docker.internal:11434
```

### Port Conflicts
If ports are already in use, modify `docker-compose-grid.yml` to use different ports.

### Build Errors
```bash
# Clean and rebuild
./sutra-deploy.sh clean
./sutra-deploy.sh install
```

## Development

### Building Specific Services
```bash
docker-compose -f docker-compose-grid.yml build sutra-api
```

### Accessing Container Shells
```bash
docker exec -it sutra-api /bin/sh
docker exec -it sutra-grid-master /bin/sh
```

## Migration from Old Scripts

**Deprecated scripts** (do not use):
- `build-images.sh` → Use `./sutra-deploy.sh build`
- `deploy-optimized.sh` → Use `./sutra-deploy.sh up`
- `deploy-docker-grid.sh` → Use `./sutra-deploy.sh install`
- `docker-compose.yml` → Use `docker-compose-grid.yml`
- `docker-compose-v2.yml` → Use `docker-compose-grid.yml`

These files are kept for reference but should not be used.

## Configuration Files

- **`docker-compose-grid.yml`**: Service definitions (do not edit directly)
- **`sutra-deploy.sh`**: Deployment script (single source of truth)
- **`DEPLOYMENT.md`**: This documentation

## Support

For issues or questions:
1. Check logs: `./sutra-deploy.sh logs`
2. Check status: `./sutra-deploy.sh status`
3. Run maintenance: `./sutra-deploy.sh maintenance`
4. Review documentation: `WARP.md` in project root

# Deployment Infrastructure v2.0

**Status:** ✅ Complete (2025-10-25)

## Executive Summary

Sutra Grid now has a single-path, production-grade deployment infrastructure with zero confusion.

## What Changed

### Before (Fragmented)
- ❌ Multiple conflicting scripts (`build-all.sh`, `build.sh`, `clean-and-rebuild.sh`, `verify-build.sh`)
- ❌ Archive folder with old scripts causing confusion
- ❌ No clear single source of truth
- ❌ HA embedding service build race conditions
- ❌ HAProxy configuration issues

### After (Clean)
- ✅ **One command center**: `sutra-deploy.sh`
- ✅ **Zero archived scripts**: All deleted
- ✅ **Clear documentation**: QUICKSTART.md, DEPLOYMENT.md
- ✅ **Production-ready**: Idempotent, self-healing operations
- ✅ **Fully tested**: Clean install verified

## Architecture

### Single Command Center: `sutra-deploy.sh` v2.0

**Key Features:**

1. **Idempotent Operations**
   - Safe to run multiple times
   - State-aware (CLEAN/BUILT/STOPPED/RUNNING)
   - No side effects from repeated execution

2. **Self-Healing**
   - Auto-fixes HAProxy configuration issues
   - Auto-builds missing images
   - Validates prerequisites before operations

3. **Production-Grade**
   - Fail-fast validation
   - Graceful shutdown with timeouts
   - Comprehensive error handling
   - Clear observability

4. **HA-Aware**
   - Builds embedding service once
   - Uses single image for 3 HA replicas
   - Avoids race conditions
   - Auto-validates configuration

### Command Reference

```bash
# Core Operations
./sutra-deploy.sh install   # Complete first-time setup
./sutra-deploy.sh up         # Start all services
./sutra-deploy.sh down       # Stop all services
./sutra-deploy.sh restart    # Restart all services
./sutra-deploy.sh clean      # Complete system reset

# Monitoring
./sutra-deploy.sh status     # Show service status
./sutra-deploy.sh validate   # Run health checks
./sutra-deploy.sh logs [svc] # View logs

# Maintenance
./sutra-deploy.sh build      # Rebuild images
./sutra-deploy.sh maintenance # Interactive menu
```

## System Configuration

### Services (13 Containers)

| Service | Purpose | Port | Health |
|---------|---------|------|--------|
| storage-server | Main storage | 50051 | ✅ |
| grid-event-storage | Event storage | 50052 | ✅ |
| embedding-1/2/3 | HA replicas | 8888 | - |
| embedding-ha | HAProxy LB | 8888, 8404 | ✅ |
| sutra-api | REST API | 8000 | ✅ |
| sutra-hybrid | Semantic API | 8001 | ✅ |
| sutra-control | Control center | 9000 | ✅ |
| sutra-client | UI client | 8080 | ✅ |
| grid-master | Grid orchestration | 7001-7002 | ✅ |
| grid-agent-1/2 | Node management | 8003-8004 | ✅ |

### Docker Compose Configuration

**File:** `docker-compose-grid.yml`

**Key Features:**
- ✅ Single embedding service build definition
- ✅ All replicas use same image
- ✅ HA embedding service configuration on all storage servers
- ✅ Proper dependency ordering
- ✅ Health checks configured
- ✅ Restart policies set

### Critical Configuration

**Embedding Service (MANDATORY):**
```yaml
SUTRA_EMBEDDING_SERVICE_URL: http://embedding-ha:8888
VECTOR_DIMENSION: 768
```

**Applied to:**
- storage-server ✅
- grid-event-storage ✅

## Issues Resolved

### 1. HA Embedding Race Condition
**Problem:** 3 replicas tried building same image simultaneously  
**Solution:** Build once before docker-compose up

### 2. HAProxy Configuration
**Problem:** Deprecated syntax + missing error file  
**Solution:** Auto-fix to modern syntax on `up`

### 3. Grid Events Storage
**Problem:** Missing embedding service configuration  
**Solution:** Added HA embedding config + dependency

### 4. Build Script Confusion
**Problem:** Multiple conflicting scripts  
**Solution:** Deleted all, single command center

## Documentation Updated

### Core Docs
- ✅ README.md - Removed old script references
- ✅ QUICKSTART.md - 2-command deployment
- ✅ DEPLOYMENT.md - Complete guide
- ✅ WARP.md - Already correct

### Test Results

**Clean Install Test (2025-10-25):**
```bash
./sutra-deploy.sh clean     # ✅ Complete wipe
./sutra-deploy.sh install   # ✅ Build + start
./sutra-deploy.sh status    # ✅ 13 services running
./sutra-deploy.sh validate  # ✅ Critical services healthy
```

**Build Time:**
- Embedding service: ~60s
- All other services: ~120s (parallel)
- Total: ~3-4 minutes cold build

**Startup Time:**
- Services: ~15s
- Health checks: ~30s
- Total: <1 minute

## Maintenance

### Adding New Services

1. Add to `docker-compose-grid.yml`
2. Set proper dependencies (especially embedding-ha)
3. Test with `./sutra-deploy.sh build && ./sutra-deploy.sh up`

### Modifying Existing Services

1. Update Dockerfile in `packages/{service}/`
2. Run `./sutra-deploy.sh build`
3. Run `./sutra-deploy.sh restart`

### Troubleshooting

```bash
# System won't start
./sutra-deploy.sh clean
./sutra-deploy.sh install

# Service failing
./sutra-deploy.sh logs <service-name>

# Health check
./sutra-deploy.sh validate
```

## Future Improvements

### Potential Enhancements
- [ ] Add `./sutra-deploy.sh update` for zero-downtime updates
- [ ] Add `./sutra-deploy.sh backup/restore` commands
- [ ] Add `./sutra-deploy.sh scale <service> <replicas>` command
- [ ] Kubernetes deployment manifests
- [ ] Docker Swarm support

### Not Needed
- ❌ Multiple deployment scripts
- ❌ Archive folder
- ❌ Manual build steps
- ❌ Complex orchestration

## Conclusion

✅ **Production-Ready Deployment Infrastructure**

- Single command center
- Zero confusion
- Self-healing
- Fully tested
- Clean documentation

**Status:** Ready for production deployment and scaling.

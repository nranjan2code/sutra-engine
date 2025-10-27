# Optimized Build Quick Reference

**Fast commands for optimized Docker builds and deployment**

## Essential Commands

### Build All Services (Recommended)
```bash
# Set edition and build all optimized services
export SUTRA_EDITION=simple
./scripts/optimize-images.sh build-all

# Compare original vs optimized sizes
./scripts/optimize-images.sh compare
```

### Deploy Optimized System
```bash
# Deploy with optimized images
SUTRA_VERSION=latest-optimized ./sutra-deploy.sh install

# Check deployment status
./sutra-deploy.sh status
```

## Individual Service Builds

### Heavy ML Services (Use Ultra Optimization)
```bash
# Embedding Service (1.32GB → 838MB)
docker build -f packages/sutra-embedding-service/Dockerfile.ultra \
    -t sutra-embedding:latest-optimized .

# NLG Service (1.39GB → 820MB)  
docker build -f packages/sutra-nlg-service/Dockerfile.ultra \
    -t sutra-nlg:latest-optimized .
```

### Standard Services (Use Simple Optimization)
```bash
# Hybrid Service
docker build -f packages/sutra-hybrid/Dockerfile.simple \
    -t sutra-hybrid:latest-optimized .

# Control Service  
docker build -f packages/sutra-control/Dockerfile.simple \
    -t sutra-control:latest-optimized .

# Client Service
docker build -f packages/sutra-client/Dockerfile.simple \
    -t sutra-client:latest-optimized .
```

### Production Services (Use Optimized)
```bash
# API Service
docker build -f packages/sutra-api/Dockerfile.optimized \
    -t sutra-api:latest-optimized .

# Bulk Ingester
docker build -f packages/sutra-bulk-ingester/Dockerfile.optimized \
    -t sutra-bulk-ingester:latest-optimized .
```

## Build Options

### Performance Options
```bash
# Parallel builds (faster)
PARALLEL=true ./scripts/optimize-images.sh build-all

# Force rebuild without cache
NO_CACHE=true ./scripts/optimize-images.sh build-all

# Build and push to registry
PUSH_IMAGES=true ./scripts/optimize-images.sh build-all
```

### Edition-Specific Builds
```bash
# Development (smaller targets)
SUTRA_EDITION=simple ./scripts/optimize-images.sh build-all

# Production (balanced)  
SUTRA_EDITION=enterprise ./scripts/optimize-images.sh build-all
```

## Verification & Testing

### Size Verification
```bash
# Compare all service sizes
./scripts/optimize-images.sh compare

# List all Sutra images
docker images --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}" | grep sutra
```

### Functional Testing
```bash
# Test optimized deployment
./scripts/optimize-images.sh test

# Run embedding service smoke test
./scripts/smoke-test-embeddings.sh
```

## Common Troubleshooting

### Build Failures
```bash
# Clean Docker cache
docker system prune -a
docker builder prune -a

# Verify Dockerfile paths
ls packages/*/Dockerfile*

# Check build context (run from repo root)
pwd  # Should be /path/to/sutra-models
```

### Size Issues
```bash
# Check actual optimized images exist
docker images | grep "latest-optimized"

# Verify using correct Dockerfile
# Ultra: .ultra (ML services)
# Simple: .simple (frontend/hybrid) 
# Optimized: .optimized (backend services)
```

## Current Optimization Results

| Service | Original | Optimized | Savings |
|---------|----------|-----------|---------|
| Embedding | 1.32GB | 838MB | 482MB (36.5%) |
| NLG | 1.39GB | 820MB | 570MB (41%) |
| Hybrid | 489MB | 378MB | 111MB (22.7%) |
| API | 298MB | 253MB | 45MB (15.1%) |
| Control | N/A | 249MB | New build |
| Client | 92.8MB | 79.5MB | 13.3MB (14.3%) |
| Bulk Ingester | 265MB | 240MB | 25MB (9.4%) |
| **TOTAL ML** | **2.71GB** | **1.66GB** | **1.05GB saved** |

## Docker Compose Override

Create `docker-compose.override.yml`:
```yaml
version: '3.8'
services:
  embedding-service:
    image: sutra-embedding:latest-optimized
  nlg-service:
    image: sutra-nlg:latest-optimized
  hybrid-service:
    image: sutra-hybrid:latest-optimized
  control-service:
    image: sutra-control:latest-optimized
  api-service:
    image: sutra-api:latest-optimized
  bulk-ingester:
    image: sutra-bulk-ingester:latest-optimized
  client-service:
    image: sutra-client:latest-optimized
```

## Cleanup

```bash
# Remove old unoptimized images
docker rmi $(docker images | grep -E "sutra.*latest[^-]" | awk '{print $3}')

# Clean all build cache
docker system prune -a --force
```

---
**Quick Access**: Save this file for fast reference during builds and deployments.
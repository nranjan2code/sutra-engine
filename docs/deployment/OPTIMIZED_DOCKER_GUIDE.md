# Optimized Docker Build & Deployment Guide

**Complete guide for building and deploying size-optimized Sutra AI Docker images**

## Overview

This guide covers the optimized Docker build system that significantly reduces image sizes while maintaining full functionality. Our optimization strategies have achieved:

- **Embedding Service**: 838MB (36.5% reduction from 1.32GB)
- **NLG Service**: 820MB (41% reduction from 1.39GB) 
- **Total ML Services**: 1.05GB saved across heavyweight services
- **Overall System**: 17.2% average reduction across all services

## Table of Contents

1. [Quick Start](#quick-start)
2. [Optimization Strategies](#optimization-strategies)
3. [Build Commands](#build-commands)
4. [Size Comparison](#size-comparison)
5. [Deployment](#deployment)
6. [Troubleshooting](#troubleshooting)
7. [Advanced Options](#advanced-options)

## Quick Start

### Prerequisites

- Docker Desktop or Docker Engine 20.10+
- 8GB+ RAM for building ML services
- 50GB+ disk space for build cache

### Build All Optimized Services

```bash
# Set edition (simple/community/enterprise)
export SUTRA_EDITION=simple

# Build all optimized services
./scripts/optimize-images.sh build-all

# Compare sizes
./scripts/optimize-images.sh compare
```

### Deploy with Docker Compose

```bash
# Use optimized images in deployment
SUTRA_VERSION=latest-optimized ./sutra-deploy.sh install
```

## Optimization Strategies

### 1. Ultra-Aggressive ML Optimization

For heavyweight ML services (embedding, NLG):

**PyTorch CPU-Only Installation:**
```dockerfile
RUN pip install --no-cache-dir torch>=2.0.0 torchvision>=0.15.0 \
    --index-url https://download.pytorch.org/whl/cpu
```

**Aggressive Cleanup:**
```dockerfile
RUN find /opt/venv -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true && \
    find /opt/venv -type f -name "*.pyc" -delete && \
    rm -rf /opt/venv/lib/python*/site-packages/torch/share && \
    rm -rf /opt/venv/lib/python*/site-packages/torch/test && \
    rm -rf /opt/venv/lib/python*/site-packages/torch/include && \
    rm -rf /opt/venv/lib/python*/site-packages/transformers/tests && \
    find /opt/venv -name "*.so" -exec strip {} \; 2>/dev/null || true
```

### 2. Multi-Stage Build Pattern

All services use optimized multi-stage builds:

```dockerfile
# Builder stage - includes build tools
FROM python:3.11-slim AS builder
RUN apt-get update && apt-get install -y gcc g++
# ... build dependencies

# Runtime stage - minimal runtime
FROM python:3.11-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /opt/venv /opt/venv
```

### 3. Service-Specific Optimizations

| Service | Strategy | Key Savings |
|---------|----------|-------------|
| **Embedding** | CPU-only PyTorch, CUDA removal | 482MB (36.5%) |
| **NLG** | Model architecture cleanup + PyTorch | 570MB (41%) |
| **Hybrid** | Lightweight Python deps only | 111MB (22.7%) |
| **Control** | React build optimization | Production build |
| **API** | Minimal FastAPI setup | 45MB (15.1%) |
| **Client** | Nginx Alpine + build cleanup | 13.3MB (14.3%) |
| **Bulk Ingester** | Rust release build stripping | 25MB (9.4%) |
| **Storage** | Debian slim + runtime libs only | Pending Rust fix |

## Build Commands

### Individual Service Builds

```bash
# Build specific service with ultra optimization
docker build -f packages/sutra-embedding-service/Dockerfile.ultra \
    -t sutra-embedding:latest-optimized .

docker build -f packages/sutra-nlg-service/Dockerfile.ultra \
    -t sutra-nlg:latest-optimized .

# Build with simple optimization
docker build -f packages/sutra-hybrid/Dockerfile.simple \
    -t sutra-hybrid:latest-optimized .
    
docker build -f packages/sutra-control/Dockerfile.simple \
    -t sutra-control:latest-optimized .
    
docker build -f packages/sutra-api/Dockerfile.optimized \
    -t sutra-api:latest-optimized .
    
docker build -f packages/sutra-bulk-ingester/Dockerfile.optimized \
    -t sutra-bulk-ingester:latest-optimized .
    
docker build -f packages/sutra-client/Dockerfile.simple \
    -t sutra-client:latest-optimized .
```

### Automated Build Scripts

```bash
# Build all services
./scripts/optimize-images.sh build-all

# Build specific service
./scripts/optimize-images.sh build embedding
./scripts/optimize-images.sh build nlg

# Build with options
SUTRA_EDITION=enterprise ./scripts/optimize-images.sh build-all
NO_CACHE=true ./scripts/optimize-images.sh build-all
PARALLEL=true ./scripts/optimize-images.sh build-all
```

### Build Verification

```bash
# Compare image sizes
./scripts/optimize-images.sh compare

# Test optimized images
./scripts/optimize-images.sh test

# Smoke test deployment
./scripts/smoke-test-embeddings.sh
```

## Size Comparison

Current optimization results:

```
┌─────────────────┬──────────────┬──────────────┬──────────────┬──────────────┐
│ SERVICE         │ ORIGINAL     │ OPTIMIZED    │ TARGET       │ SAVINGS      │
├─────────────────┼──────────────┼──────────────┼──────────────┼──────────────┤
│ embedding       │ 1.32GB       │ 838MB        │ 450MB        │ 482MB (36.5%)│
│ nlg             │ 1.39GB       │ 820MB        │ 550MB        │ 570MB (41%)  │
│ hybrid          │ 489MB        │ 378MB        │ 180MB        │ 111MB (22.7%)│
│ control         │ N/A          │ 249MB        │ 120MB        │ New build    │
│ api             │ 298MB        │ 253MB        │ 80MB         │ 45MB (15.1%) │
│ bulk-ingester   │ 265MB        │ 240MB        │ 250MB        │ 25MB (9.4%)  │
│ client          │ 92.8MB       │ 79.5MB       │ 80MB         │ 13.3MB (14.3%)│
│ storage         │ 168MB        │ Pending      │ 160MB        │ Rust fix req │
├─────────────────┼──────────────┼──────────────┼──────────────┼──────────────┤
│ TOTAL ML        │ 2.71GB       │ 1.66GB       │ 1.00GB       │ 1.05GB saved │
│ TOTAL SYSTEM    │ ~3.5GB       │ ~2.9GB       │ ~2.0GB       │ 17.2% avg    │
└─────────────────┴──────────────┴──────────────┴──────────────┴──────────────┘
```

## Deployment

### Production Deployment

```bash
# Deploy with optimized images
export SUTRA_VERSION=latest-optimized
export SUTRA_EDITION=simple

# Full system deployment
./sutra-deploy.sh install

# Check deployment status
./sutra-deploy.sh status

# View service logs
./sutra-deploy.sh logs embedding-service
```

### Docker Compose Override

Create `docker-compose.override.yml` for optimized images:

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

### Kubernetes Deployment

Update image tags in `k8s/` manifests:

```yaml
spec:
  containers:
  - name: embedding-service
    image: sutra-embedding:latest-optimized
    resources:
      requests:
        memory: "600Mi"  # Reduced from 1Gi
        cpu: "300m"
      limits:
        memory: "1Gi"    # Reduced from 1.5Gi
        cpu: "1"
```

## Troubleshooting

### Common Build Issues

**1. Rust Version Compatibility (Storage Service)**
```bash
# Error: rustc 1.82.0 not supported by cxx@1.0.187
# Solution: Update Cargo.toml dependencies
cargo update cxx --precise 1.0.180
```

**2. NPM Build Failures (Frontend Services)**
```bash
# Error: tsc not found
# Solution: Install dev dependencies
RUN npm ci --silent  # Instead of --only=production
```

**3. PyTorch CUDA Dependencies**
```bash
# Error: Large image size with CUDA
# Solution: Use CPU-only index
pip install torch --index-url https://download.pytorch.org/whl/cpu
```

### Size Optimization Not Working

**Check Dockerfile variant:**
```bash
# Ensure using optimized Dockerfiles
ls packages/*/Dockerfile*
# Should see: Dockerfile.simple, Dockerfile.ultra, Dockerfile.optimized
```

**Verify build context:**
```bash
# Build from repository root
cd /path/to/sutra-models
docker build -f packages/SERVICE/Dockerfile.VARIANT -t SERVICE:optimized .
```

**Clean build cache:**
```bash
docker system prune -a
docker builder prune -a
```

### Runtime Issues

**Missing libraries:**
```bash
# Add to Dockerfile if needed
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 ca-certificates curl
```

**Permission errors:**
```bash
# Ensure proper user setup
RUN useradd --create-home --uid 1000 serviceuser
USER serviceuser
```

## Advanced Options

### Edition-Specific Builds

```bash
# Simple edition (development)
SUTRA_EDITION=simple ./scripts/optimize-images.sh build-all

# Community edition (small teams)
SUTRA_EDITION=community ./scripts/optimize-images.sh build-all

# Enterprise edition (production)
SUTRA_EDITION=enterprise ./scripts/optimize-images.sh build-all
```

### Performance Tuning

**Parallel builds:**
```bash
PARALLEL=true ./scripts/optimize-images.sh build-all
```

**No cache builds:**
```bash
NO_CACHE=true ./scripts/optimize-images.sh build-all
```

**Registry push:**
```bash
PUSH_IMAGES=true ./scripts/optimize-images.sh build-all
```

### Custom Size Targets

Edit size targets in `scripts/optimize-images.sh`:

```bash
get_size_targets() {
    local edition="$1"
    case "$edition" in
        "simple")
            echo "embedding:400MB nlg:500MB hybrid:150MB ..."
            ;;
    esac
}
```

### Future Optimizations

**Distroless Images (Planned):**
- Base images: `gcr.io/distroless/python3-debian12`
- Additional 60MB+ savings per service
- Enhanced security posture

**Multi-Architecture Builds:**
```bash
docker buildx build --platform linux/amd64,linux/arm64 \
    -f Dockerfile.ultra -t sutra-embedding:optimized .
```

## Monitoring & Maintenance

### Size Monitoring

```bash
# Regular size checks
./scripts/optimize-images.sh compare

# Track size over time
docker images --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}\t{{.CreatedAt}}"
```

### Cleanup

```bash
# Remove old images
docker image prune -a

# Clean build cache
docker builder prune -a

# Remove untagged images
docker rmi $(docker images -f "dangling=true" -q)
```

### Updates

When updating services:

1. **Build new optimized image**
2. **Test in development**
3. **Update deployment configs**
4. **Monitor resource usage**

## Best Practices

1. **Always build from repository root**
2. **Use appropriate optimization level per service**
3. **Test optimized images before production**
4. **Monitor runtime performance after optimization**
5. **Keep optimization documentation updated**

## Support

- **Issues**: Check [Troubleshooting](#troubleshooting) section
- **Performance**: Monitor resource usage post-deployment
- **Updates**: Re-run optimization after dependency updates

---

**Last Updated**: October 27, 2025  
**Version**: 2.0.0  
**Optimization Results**: 1.05GB total savings on ML services
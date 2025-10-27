# Docker Image Optimization Guide

## Current State vs Optimized Targets

### Image Size Comparison (Simple Edition)

| Service | Original | Optimized Target | Expected Savings |
|---------|----------|------------------|------------------|
| sutra-embedding-service | 1.32GB | 450MB | **66% (870MB)** |
| sutra-nlg-service | 1.39GB | 550MB | **60% (840MB)** |
| sutra-hybrid | 489MB | 180MB | **63% (309MB)** |
| sutra-control | 387MB | 120MB | **69% (267MB)** |
| sutra-api | 298MB | 80MB | **73% (218MB)** |
| sutra-bulk-ingester | 265MB | 250MB | **6% (15MB)** |
| sutra-storage-server | 168MB | 160MB | **5% (8MB)** |
| sutra-client | 92.8MB | 80MB | **14% (13MB)** |
| **TOTAL** | **4.41GB** | **1.87GB** | **58% (2.54GB)** |

## Key Optimization Strategies

### 1. **PyTorch CPU-Only Distribution**
- **Problem**: Full PyTorch with CUDA support adds ~800MB even when unused
- **Solution**: CPU-only wheels from `https://download.pytorch.org/whl/cpu`
- **Savings**: ~400-500MB per ML service

### 2. **Edition-Specific Dependencies**
- **Simple Edition**: Remove numpy, scipy, scikit-learn from hybrid service
- **Community Edition**: Basic ML libraries only
- **Enterprise Edition**: Full scientific computing stack
- **Savings**: 200-300MB for simple deployments

### 3. **Multi-Stage Build Optimization**
- **Builder Stage**: Full toolchain for compilation
- **Runtime Stage**: Minimal dependencies only
- **Aggressive Cleanup**: Remove tests, docs, dev tools, cache files
- **Savings**: 100-200MB per service

### 4. **Distroless Base Images**
- **Current**: python:3.11-slim (109MB base)
- **Alternative**: gcr.io/distroless/python3 (50MB base)
- **Benefit**: Smaller attack surface, faster startup
- **Savings**: ~60MB per image

### 5. **React Build Optimization**
- **Gzip Compression**: Pre-compress static assets
- **Tree Shaking**: Remove unused JavaScript
- **Source Map Removal**: Delete .map files in production
- **Savings**: 50-100MB for control service

## Implementation Files

### Optimized Dockerfiles Created
```
packages/sutra-embedding-service/Dockerfile.optimized
packages/sutra-nlg-service/Dockerfile.optimized  
packages/sutra-hybrid/Dockerfile.optimized
packages/sutra-control/Dockerfile.optimized
```

### Build Script
```bash
./scripts/optimize-images.sh build-all --edition simple
```

## Edition-Specific Configurations

### Simple Edition (Minimal Dependencies)
```dockerfile
# Hybrid Service - No scientific computing
RUN cat > requirements.txt <<EOF
fastapi>=0.104.0
uvicorn[standard]>=0.24.0
pydantic>=2.0.0
requests>=2.31.0
# REMOVED: numpy, scipy, scikit-learn
EOF
```

### Community Edition (Basic ML)
```dockerfile
# Add basic ML capabilities
numpy>=2.0.0
scikit-learn>=1.3.0
# Still no heavy scipy/advanced ML
```

### Enterprise Edition (Full Stack)
```dockerfile
# Full scientific computing stack
numpy>=2.0.0
scipy>=1.14.0
scikit-learn>=1.5.0
deepspeed>=0.12.0
```

## Production Deployment Checklist

### 1. **Build Optimized Images**
```bash
# Set your target edition
export SUTRA_EDITION=simple

# Build all optimized images
./scripts/optimize-images.sh build-all --edition simple

# Compare sizes
./scripts/optimize-images.sh compare
```

### 2. **Update Docker Compose**
```yaml
services:
  embedding-single:
    image: sutra-embedding-service:${SUTRA_VERSION:-latest}-optimized
    # ... rest of config unchanged
```

### 3. **Registry Push** (if using private registry)
```bash
# Tag for registry
docker tag sutra-embedding-service:latest-optimized registry.company.com/sutra-embedding-service:2.0.0-optimized

# Push optimized images
./scripts/optimize-images.sh build-all --edition simple --push
```

## Performance Benefits

### Deployment Speed
- **Image Pull Time**: 60% faster (2.54GB less to download)
- **Container Startup**: 20-30% faster due to smaller footprint
- **Memory Usage**: 15-25% reduction in runtime memory

### Cost Savings (Cloud Deployment)
- **Storage**: 58% less registry storage costs
- **Bandwidth**: 58% less data transfer costs
- **Compute**: Smaller resource allocation possible

### Security Benefits
- **Attack Surface**: Minimal base images reduce vulnerability exposure
- **Compliance**: Fewer packages = easier security auditing
- **Updates**: Faster security patches due to smaller images

## Build Commands Quick Reference

```bash
# Build all services for simple edition
./scripts/optimize-images.sh build-all --edition simple

# Build specific service without cache
./scripts/optimize-images.sh build embedding --no-cache

# Analyze layer sizes for debugging
./scripts/optimize-images.sh analyze embedding

# Compare original vs optimized
./scripts/optimize-images.sh compare

# Test all optimized images
./scripts/optimize-images.sh test

# Clean up optimized images
./scripts/optimize-images.sh clean
```

## Verification Steps

### 1. **Size Verification**
```bash
docker images | grep sutra | grep optimized
```

### 2. **Functionality Test**
```bash
# Start optimized stack
SUTRA_VERSION=latest-optimized ./sutra-deploy.sh install

# Run smoke tests
./scripts/smoke-test-embeddings.sh
```

### 3. **Performance Monitoring**
```bash
# Monitor resource usage
docker stats

# Check startup times
time docker-compose up -d
```

## Troubleshooting

### Common Issues

1. **Build Failures**
   - Check Docker BuildKit is enabled
   - Ensure sufficient disk space (>10GB free)
   - Verify all base images are available

2. **Runtime Errors** 
   - Missing dependencies for specific editions
   - Path issues in optimized containers
   - Permission problems with non-root users

3. **Size Targets Not Met**
   - Check layer history: `./scripts/optimize-images.sh analyze <service>`
   - Verify cleanup steps executed successfully
   - Consider more aggressive dependency pruning

### Debug Commands
```bash
# Inspect image layers
docker history sutra-embedding-service:latest-optimized --human

# Check installed packages
docker run --rm sutra-embedding-service:latest-optimized pip list

# Test container startup
docker run --rm sutra-embedding-service:latest-optimized python --version
```

## Next Steps

1. **Test optimized images** in development environment
2. **Update CI/CD pipelines** to use optimized Dockerfiles
3. **Monitor production metrics** after deployment
4. **Consider distroless base images** for even smaller footprint
5. **Implement automated size regression testing**

## Maintenance

- **Weekly**: Check for new optimization opportunities
- **Monthly**: Update base images and rebuild
- **Quarterly**: Review edition-specific requirements
- **Per Release**: Validate size targets still met
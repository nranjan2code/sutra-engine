# Deployment Documentation Index

**Complete guide to Sutra AI deployment options and optimization**

## 📚 Documentation Structure

### 🚀 Quick Start
- **[Deployment Guide](DEPLOYMENT.md)** - Main deployment using `sutra-deploy.sh`
- **[Deployment Modes](DEPLOYMENT_MODES.md)** - Different deployment configurations
- **[Deployment Guide (Legacy)](DEPLOYMENT_GUIDE.md)** - Detailed deployment procedures

### 🐳 Docker Optimization (NEW)
- **[Optimized Docker Guide](OPTIMIZED_DOCKER_GUIDE.md)** - Complete optimization guide with 1GB+ savings
- **[Quick Reference](OPTIMIZED_BUILD_QUICK_REF.md)** - Fast command reference for optimized builds
- **[Optimization Results](../AGGRESSIVE_ML_OPTIMIZATION_RESULTS.md)** - Detailed analysis of size reductions

## 🎯 Choose Your Path

### Standard Deployment (Recommended)
**For getting started quickly:**
```bash
./sutra-deploy.sh install
```
- Uses standard Docker images
- Fast build times
- Good for development and testing

### Optimized Deployment (Production)
**For production with resource constraints:**
```bash
export SUTRA_EDITION=simple
./scripts/optimize-images.sh build-all
SUTRA_VERSION=latest-optimized ./sutra-deploy.sh install
```
- **1.05GB saved** on ML services
- 17.2% average size reduction
- Ideal for production environments

## 📊 Optimization Results

| Service | Original | Optimized | Savings | Strategy |
|---------|----------|-----------|---------|----------|
| **Embedding** | 1.32GB | 838MB | 482MB (36.5%) | Ultra PyTorch cleanup |
| **NLG** | 1.39GB | 820MB | 570MB (41%) | Model architecture removal |
| **Hybrid** | 489MB | 378MB | 111MB (22.7%) | Lightweight dependencies |
| **Control** | N/A | 249MB | New build | React optimization |
| **API** | 298MB | 253MB | 45MB (15.1%) | Minimal FastAPI |
| **Client** | 92.8MB | 79.5MB | 13.3MB (14.3%) | Nginx Alpine |
| **Bulk Ingester** | 265MB | 240MB | 25MB (9.4%) | Rust stripping |
| **Storage** | 168MB | Pending | Rust fix | Multi-stage build |

**Total ML Services**: 1.05GB saved  
**Overall System**: 17.2% average reduction

## 🛠️ Build Strategies

### Ultra Optimization (ML Services)
- CPU-only PyTorch installation
- Aggressive cleanup of unused components
- Binary stripping and cache removal
- Target: <500MB for embedding, <600MB for NLG

### Simple Optimization (Frontend/Hybrid)
- Multi-stage builds with minimal runtime
- Production-only dependencies
- Optimized Node.js/Python base images
- Target: Baseline size reduction

### Standard Optimization (Backend Services)
- Balanced approach for API/infrastructure services
- Security-focused with essential runtime libraries
- Moderate size reduction with reliability focus
- Target: 10-20% size reduction

## 📋 Quick Commands Reference

### Complete Deployment
```bash
# Standard deployment
./sutra-deploy.sh install

# Optimized deployment  
export SUTRA_EDITION=simple
./scripts/optimize-images.sh build-all
SUTRA_VERSION=latest-optimized ./sutra-deploy.sh install
```

### Individual Optimized Builds
```bash
# Heavy ML services (use ultra)
docker build -f packages/sutra-embedding-service/Dockerfile.ultra \
    -t sutra-embedding:latest-optimized .

# Frontend services (use simple)  
docker build -f packages/sutra-control/Dockerfile.simple \
    -t sutra-control:latest-optimized .

# Backend services (use optimized)
docker build -f packages/sutra-api/Dockerfile.optimized \
    -t sutra-api:latest-optimized .
```

### Size Verification
```bash
# Compare all service sizes
./scripts/optimize-images.sh compare

# List optimized images
docker images --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}" | grep optimized
```

## 🎯 Use Cases

### Development & Testing
- **Use**: Standard deployment
- **Command**: `./sutra-deploy.sh install`
- **Benefits**: Fast builds, full functionality
- **Resource**: ~3.5GB total

### Production (Resource Constrained)
- **Use**: Optimized deployment
- **Command**: Optimized build + deploy
- **Benefits**: 1GB+ savings, same functionality
- **Resource**: ~2.9GB total

### Production (High Performance)
- **Use**: Enterprise edition with optimization
- **Command**: `SUTRA_EDITION=enterprise` + optimized build
- **Benefits**: Maximum performance with efficient resource usage
- **Resource**: Scaled for load with optimization

## 🔧 Troubleshooting

### Build Issues
- **Rust compatibility**: Check Cargo.toml versions
- **NPM failures**: Include dev dependencies for builds
- **Path issues**: Build from repository root

### Size Issues  
- **Verify Dockerfile**: Use correct optimization level
- **Check context**: Ensure proper build context
- **Clean cache**: Remove old Docker cache

### Runtime Issues
- **Missing libs**: Add runtime dependencies to final stage
- **Permissions**: Verify user setup in containers
- **Health checks**: Use optimized service health endpoints

## 📚 Related Documentation

- **[System Architecture](../ARCHITECTURE.md)** - Overall system design
- **[Production Guide](../PRODUCTION_GUIDE.md)** - Production deployment
- **[Release Management](../release/README.md)** - Version control and releases
- **[Build System](../sutrabuild/README.md)** - Advanced build infrastructure

---

**Last Updated**: October 27, 2025  
**Optimization Status**: 7/8 services optimized, 1.05GB total savings achieved
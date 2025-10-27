# ML Foundation Documentation Index

## Overview

The Sutra ML Foundation provides a comprehensive, edition-aware platform for building scalable ML services. This documentation covers architecture, deployment, and development guidelines for the foundation.

## 📚 Documentation Structure

### Core Foundation
- **[README.md](./README.md)** - Complete ML Foundation architecture and usage guide
- **[DEPLOYMENT.md](./DEPLOYMENT.md)** - Deployment guide for ML services with Docker and Kubernetes

### Service Documentation  
- **[../embedding/SERVICE_OVERVIEW.md](../embedding/SERVICE_OVERVIEW.md)** - Next-generation embedding service
- **[../nlg/README.md](../nlg/README.md)** - Next-generation NLG service

### Architecture Integration
- **[../ARCHITECTURE.md](../ARCHITECTURE.md)** - Main system architecture (updated for ML Foundation)
- **[../SYSTEM_OVERVIEW.md](../SYSTEM_OVERVIEW.md)** - Complete system overview

## 🏗️ Quick Navigation

### Getting Started (5 minutes)
1. Read [README.md](./README.md) - Foundation overview
2. Follow [DEPLOYMENT.md](./DEPLOYMENT.md) - Quick deployment
3. Test ML services with provided examples

### Development (30 minutes)
1. Understand edition-aware architecture
2. Learn BaseMlService patterns
3. Create your first ML service using the foundation

### Production (60 minutes)
1. Configure edition-appropriate resources
2. Set up monitoring and health checks
3. Deploy with Docker/Kubernetes

## 🎯 Key Features

### Edition-Aware Architecture
- **Simple**: Basic models, 32 batch size, 128MB cache
- **Community**: Better models, 64 batch size, 256MB cache  
- **Enterprise**: Best models, 128 batch size, 512MB cache

### Unified Service Pattern
- Consistent FastAPI applications
- Standardized health/metrics/info endpoints
- Automatic model loading and management
- Built-in caching and security

### Performance & Reliability
- 90% code reduction through shared foundation
- Advanced caching with edition limits
- Comprehensive metrics and monitoring
- Production-grade error handling

## 📊 ML Services Comparison

| Service | Purpose | Port | Models | Edition Features |
|---------|---------|------|--------|------------------|
| **Embedding** | Semantic embeddings | 8888 | nomic-embed, all-mpnet, custom | Batch processing, caching |
| **NLG** | Text generation | 8889 | DialoGPT, Gemma-2B, custom | Grounding, prompt validation |
| **Future Services** | TBD | 889X | Various | Foundation-based |

## 🚀 Quick Commands

```bash
# Install foundation
pip install -e packages/sutra-ml-base

# Build ML services
./sutra-optimize.sh build-ml

# Deploy with edition
SUTRA_EDITION=community ./sutra-deploy.sh install

# Test services
curl -s http://localhost:8888/health | jq
curl -s http://localhost:8889/health | jq
```

## 🔗 Related Documentation

### System Architecture
- [WARP.md](../../WARP.md) - AI assistant guidance
- [ARCHITECTURE.md](../ARCHITECTURE.md) - Complete system architecture
- [SYSTEM_OVERVIEW.md](../SYSTEM_OVERVIEW.md) - System components

### Storage & Core
- [storage/](../storage/) - Storage engine documentation
- [grid/](../grid/) - Distributed infrastructure
- [api/](../api/) - API layer documentation

### Operations
- [deployment/](../deployment/) - General deployment guides
- [operations/](../operations/) - Monitoring and maintenance
- [troubleshooting/](../TROUBLESHOOTING.md) - Issue resolution

---

*ML Foundation Documentation v2.0.0*  
*World-Class ML Service Architecture*
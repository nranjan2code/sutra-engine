# Deployment Guide

Complete guide to deploying Sutra across all editions.

> **Important:** This deployment uses `sutra-works-` image prefix. See **[Image Naming Convention](IMAGE_NAMING.md)** for details.

## Contents

- **[Image Naming Convention](IMAGE_NAMING.md)** - Docker image and container naming ⭐ **Start here**
- **[Simple Edition](simple-edition.md)** - Single-node deployment (8 services)
- **[Community Edition](community-edition.md)** - High-availability embedding (HA + load balancing)
- **[Enterprise Edition](enterprise-edition.md)** - Distributed grid with sharding
- **[Production Deployment](production.md)** - Production best practices
- **[Infrastructure](infrastructure.md)** - Infrastructure requirements and setup
- **[Validation](validation.md)** - Deployment validation and healthchecks
- **[Enhancements](enhancements.md)** - Performance tuning and optimizations
- **[Security Requirements (v3.0.0)](../security/README.md)** - Production-grade security
- **[Quality Gates (v3.0.0)](../development/quality-gates.md)** - Automated enforcement

## Production Requirements (v3.0.0)

### Security
- ✅ httpOnly Cookie Authentication (XSS immune)
- ✅ 8-Layer OWASP Security Headers
- ✅ TLS 1.3 with Certificate Authentication
- ✅ 100% Dependency Pinning
- ✅ Security Middleware (230 lines)

### Quality
- ✅ Pre-commit Hooks (9 checks)
- ✅ CI Validation Pipeline
- ✅ Bundle Size Limits
- ✅ Security Scanning (Bandit, npm audit)
- ✅ Credential Detection

### Architecture
- ✅ TCP Binary Protocol (gRPC removed)
- ✅ MessagePack serialization (10-50x faster)
- ✅ Zero localStorage usage

## Quick Start

### Deploy Simple Edition

```bash
# Build images first
SUTRA_EDITION=simple ./sutra-optimize.sh build-all

# Deploy
SUTRA_EDITION=simple ./sutra deploy
```

### Check Status

```bash
./sutra status
docker-compose -f .sutra/compose/production.yml --profile simple ps
```

### Access Services

- **API**: http://localhost:8000
- **Control Panel**: http://localhost:9000
- **Web Client**: http://localhost:8080
- **Embedding Service**: http://localhost:8888
- **NLG Service**: http://localhost:8889
- **Hybrid Service**: http://localhost:8001
- **Storage Server**: localhost:50051 (TCP)

## Editions Overview

### Simple Edition (Default)
**Services**: 9 containers (8 services + 1 storage replica)

- embedding-single (1.36GB)
- nlg-single (1.39GB)
- sutra-hybrid (624MB)
- sutra-control (273MB)
- sutra-api (273MB)
- sutra-bulk-ingester (240MB)
- sutra-storage (164MB)
- sutra-user-storage (164MB)
- sutra-client (79MB)

**Total**: ~4.4GB

**Use Case**: Development, testing, small deployments

### Community Edition
**Services**: 13 containers (8 services + 3 embedding replicas + 1 HAProxy + NLG HA)

Adds high-availability for ML services:
- 3x embedding service replicas (load balanced)
- HAProxy load balancer
- 2x NLG service replicas
- Automatic failover

**Total**: ~6GB

**Use Case**: Production deployments requiring ML service redundancy

### Enterprise Edition
**Services**: 10 unique services (adds grid infrastructure)

Adds to simple edition:
- grid-master (148MB) - Grid coordinator
- grid-agent (146MB) - Distributed workers
- Multi-shard storage with 2PC transactions
- Distributed query processing

**Total**: ~4.76GB

**Use Case**: High-scale distributed deployments, multi-tenant

## Deployment Architecture

```
┌─────────────────────────────────────────────────────┐
│ SUTRA DEPLOYMENT STACK                               │
├─────────────────────────────────────────────────────┤
│                                                     │
│  Frontend Layer                                     │
│    ├─ Client (nginx) :8080                          │
│    └─ Control Panel :9000                           │
│                                                     │
│  API Layer                                          │
│    ├─ API Server :8000                              │
│    └─ Hybrid Service :8001                          │
│                                                     │
│  ML Foundation                                      │
│    ├─ Embedding Service :8888                       │
│    └─ NLG Service :8889                             │
│                                                     │
│  Storage Layer                                      │
│    ├─ Storage Server :50051                         │
│    ├─ User Storage :50053                           │
│    └─ Bulk Ingester :8005                           │
│                                                     │
│  Grid Layer (Enterprise)                            │
│    ├─ Grid Master :7001                             │
│    └─ Grid Agents :7002-7005                        │
│                                                     │
└─────────────────────────────────────────────────────┘
```

## Prerequisites

### System Requirements

**Minimum:**
- CPU: 4 cores
- RAM: 8GB
- Disk: 20GB free
- Docker: 20.10+
- Docker Compose: 2.0+

**Recommended:**
- CPU: 8 cores
- RAM: 16GB
- Disk: 50GB free (SSD preferred)
- Docker: Latest
- Docker Compose: Latest

### Software Requirements

```bash
# Verify Docker
docker --version  # Should be 20.10+

# Verify Docker Compose
docker-compose --version  # Should be 2.0+

# Check available resources
docker info | grep -E "CPUs|Memory"
```

## Environment Variables

### Required

```bash
# None - system works with defaults
```

### Optional

```bash
# Edition selection
export SUTRA_EDITION=simple|community|enterprise  # Default: simple

# Image versioning
export SUTRA_VERSION=latest|v2.0.0  # Default: latest

# Security (production)
export SUTRA_SECURE_MODE=true  # Enables TLS, HMAC, RBAC
export SUTRA_LICENSE_KEY=<your-key>
export SUTRA_LICENSE_SECRET=<your-secret>

# ML configuration
export HF_TOKEN=<huggingface-token>  # For model downloads

# Logging
export SUTRA_LOG_LEVEL=debug|info|warning|error  # Default: info
```

## Deployment Workflow

### 1. Build Images

```bash
SUTRA_EDITION=simple ./sutra-optimize.sh build-all
```

### 2. Verify Images

```bash
./sutra-optimize.sh sizes
docker images | grep sutra
```

### 3. Deploy

```bash
SUTRA_EDITION=simple ./sutra deploy
```

### 4. Validate

```bash
./sutra status
./sutra validate  # Runs smoke tests
```

### 5. Monitor

```bash
# View logs
docker-compose -f .sutra/compose/production.yml --profile simple logs -f

# Check healthchecks
docker ps --filter "name=sutra" --format "table {{.Names}}\t{{.Status}}"
```

## Healthchecks

All services include healthcheck endpoints:

- **API**: `http://localhost:8000/health`
- **Embedding**: `http://localhost:8888/health`
- **NLG**: `http://localhost:8889/health`
- **Hybrid**: `http://localhost:8001/health`
- **Control**: `http://localhost:9000/health`
- **Client**: `http://localhost:8080/` (HTTP 200)
- **Storage**: TCP connection test on port 50051
- **Bulk Ingester**: HTTP health endpoint on 8005

Services are marked healthy after passing checks for 30s.

## Troubleshooting

### Services Not Starting

```bash
# Check logs
docker-compose -f .sutra/compose/production.yml --profile simple logs SERVICE_NAME

# Check resources
docker stats

# Restart specific service
docker-compose -f .sutra/compose/production.yml --profile simple restart SERVICE_NAME
```

### Port Conflicts

```bash
# Check what's using ports
lsof -i :8000
lsof -i :8888

# Change ports in compose file or stop conflicting service
```

### Out of Memory

```bash
# Check memory usage
docker stats --no-stream

# Increase Docker memory limit (Docker Desktop)
# Or reduce services by using simple edition
```

### Image Not Found

```bash
# Verify images exist
docker images | grep sutra

# Rebuild if missing
./sutra-optimize.sh build-all
```

## Stopping Deployment

```bash
# Stop all services
docker-compose -f .sutra/compose/production.yml --profile simple down

# Stop and remove volumes (WARNING: deletes data)
docker-compose -f .sutra/compose/production.yml --profile simple down -v
```

## Next Steps

- [Simple Edition Guide](simple-edition.md)
- [Community Edition Guide](community-edition.md)
- [Enterprise Edition Guide](enterprise-edition.md)
- [Production Best Practices](production.md)
- [Architecture Overview](../architecture/system-overview.md)

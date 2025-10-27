# Sutra AI Documentation Index

**Complete technical documentation for the Sutra AI distributed reasoning system**

---

## 🚀 Getting Started

New to Sutra AI? Start here:

1. **[Quick Start Guide](QUICKSTART.md)** - Get running in 5 minutes
2. **[System Overview](SYSTEM_OVERVIEW.md)** - Understand the architecture
3. **[Tutorial](TUTORIAL.md)** - Build your first knowledge graph

---

## � Release Management (NEW)

**Professional release management for production deployments:**

### Essential Docs
- **[Release Overview](release/README.md)** - Complete release system documentation
- **[Release Process](release/RELEASE_PROCESS.md)** - Step-by-step release workflow
- **[Quick Reference](release/QUICK_REFERENCE.md)** - Command cheat sheet
- **[Versioning Strategy](release/VERSIONING_STRATEGY.md)** - When to bump versions

### Quick Commands
```bash
./sutra-deploy.sh version          # Check current version
./sutra-deploy.sh release patch    # Create bug fix release
./sutra-deploy.sh deploy v2.0.1    # Deploy specific version
```

**Use for:** Creating releases, deploying to customers, version management

---

## 🏗️ Build System (NEW)

**World-class containerized build infrastructure:**

### Essential Docs
- **[Build Overview](sutrabuild/README.md)** - Complete build system documentation
- **[Quick Start](sutrabuild/QUICKSTART.md)** - Start building immediately  
- **[Architecture](sutrabuild/ARCHITECTURE.md)** - Technical deep-dive
- **[Complete Reference](sutrabuild/REFERENCE.md)** - All commands and options
- **[Migration Guide](sutrabuild/MIGRATION.md)** - Move from scattered builds
- **[Maintenance](sutrabuild/MAINTENANCE.md)** - Operations and troubleshooting

### Quick Commands
```bash
./sutrabuild/scripts/build-all.sh --profile simple     # Build all services
./sutrabuild/scripts/build-all.sh --profile simple --parallel  # Parallel builds
./sutrabuild/scripts/health-check.sh http://localhost:8000/health  # Health check
```

**Key Benefits:** 100% reproducible builds, 50%+ faster builds, shared base images, profile-based deployment

---

## Package Documentation

### Core Packages
- [sutra-storage](../packages/sutra-storage/README.md) - Storage engine (Rust)
- [sutra-core](../packages/sutra-core/README.md) - Reasoning engine (Python)
- [sutra-hybrid](../packages/sutra-hybrid/README.md) - Semantic orchestration
- [sutra-api](../packages/sutra-api/README.md) - REST API

### Support Packages
- [sutra-nlg](../packages/sutra-nlg/README.md) - Natural language generation
- [sutra-storage-client-tcp](../packages/sutra-storage-client-tcp/README.md) - TCP client

### Grid Infrastructure
- [sutra-grid-master](../packages/sutra-grid-master/README.md) - Orchestration
- [sutra-grid-agent](../packages/sutra-grid-agent/README.md) - Node management
- [sutra-grid-events](../packages/sutra-grid-events/README.md) - Event system

### UI & Tools
- [sutra-control](../packages/sutra-control/README.md) - Control center
- [sutra-client](../packages/sutra-client/README.md) - Web UI
- [sutra-explorer](../packages/sutra-explorer/README.md) - Storage explorer

---

## Architecture

- [System Architecture](ARCHITECTURE.md) - High-level design
- [Storage Architecture](storage/ARCHITECTURE.md) - Storage engine
- [TCP Protocol](TCP_PROTOCOL_ARCHITECTURE.md) - Binary protocol

---

## 🐳 Docker Optimization (NEW)

**Production-ready optimized Docker builds with massive size savings:**

### Essential Docs
- **[Optimized Docker Guide](deployment/OPTIMIZED_DOCKER_GUIDE.md)** - Complete optimization guide
- **[Quick Reference](deployment/OPTIMIZED_BUILD_QUICK_REF.md)** - Fast command reference
- **[Optimization Results](../docs/AGGRESSIVE_ML_OPTIMIZATION_RESULTS.md)** - Detailed savings analysis

### Quick Commands
```bash
export SUTRA_EDITION=simple
./scripts/optimize-images.sh build-all     # Build all optimized services
./scripts/optimize-images.sh compare       # Compare sizes
SUTRA_VERSION=latest-optimized ./sutra-deploy.sh install  # Deploy optimized
```

### Current Results
- **Embedding Service**: 838MB (36.5% reduction from 1.32GB)
- **NLG Service**: 820MB (41% reduction from 1.39GB)
- **Total ML Services**: 1.05GB saved
- **Overall System**: 17.2% average reduction

**Use for:** Production deployments, resource optimization, cost reduction

---

## Operations

- [Deployment](operations/DEPLOYMENT.md) - Production deployment
- [Monitoring](operations/MONITORING.md) - Observability
- [Production Requirements](operations/PRODUCTION_REQUIREMENTS.md) - Critical config
- [Troubleshooting](./TROUBLESHOOTING.md) - Common issues

---

## Development

- [Development Guide](guides/DEVELOPMENT.md) - Setup
- **[Fast Development Workflow](../FAST_DEVELOPMENT.md)** - **NEW: Single-service updates**
- **[Simple Deployment Solution](../SIMPLE_DEPLOYMENT_SOLUTION.md)** - **NEW: What we built**
- [Contributing](./CONTRIBUTING.md) - How to contribute
- [Testing](development/testing.md) - Test procedures

---

## Security & Compliance

### Dependency Management
- [Quick Start](dependency-management/QUICK_START.md) - Get started in 5 minutes
- [Architecture](dependency-management/ARCHITECTURE.md) - System design
- [Design](dependency-management/DESIGN.md) - Design principles
- [Documentation Index](dependency-management/README.md) - All dependency docs

### Security
- [Security Overview](security/README.md) - Security documentation
- [Production Security](security/PRODUCTION_SECURITY_SETUP.md) - Production setup
- [Security Audit](security/SECURITY_AUDIT_REPORT.md) - Vulnerability analysis

---

## Reference

- [Documentation Audit](DOCUMENTATION_AUDIT_2025.md) - Complete analysis
- [Completion Summary](DOCUMENTATION_COMPLETION_SUMMARY.md) - Roadmap

---

**Last Updated**: October 25, 2025

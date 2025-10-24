# Sutra AI Documentation Index

**Complete documentation navigation for Sutra AI - An explainable AI system that learns in real-time**

Version: 2.0.0 | Last Updated: 2025-10-23

---

## 🚀 Quick Start

**New to Sutra AI?** Start here:
- 📖 [Project Overview](../README.md) - What is Sutra AI and why use it
- 🏃 [Quick Start Guide](guides/QUICK_START.md) - Get up and running in 10 minutes
- 🏗️ [Architecture Overview](../ARCHITECTURE.md) - System design at a glance
- 🔧 [Build & Deploy](operations/BUILD_AND_DEPLOY.md) - Production deployment guide

---

## 📚 Core Documentation

### 🏗️ Architecture & Design
High-level system design, technical deep dives, and scalability architecture

- **[System Architecture](../ARCHITECTURE.md)** - Main architecture document (root)
- **[Deep Dive](architecture/DEEP_DIVE.md)** - Detailed technical design and storage architecture
- **[Scalability Architecture](architecture/SCALABILITY.md)** 🆕 - Sharding, HNSW, HA, distributed features
- **[TCP Binary Protocol](TCP_PROTOCOL_ARCHITECTURE.md)** - Custom 10-50× faster protocol
- **[Unified Learning Architecture](UNIFIED_LEARNING_ARCHITECTURE.md)** - Single source of truth learning
- **[Runtime Architecture](RUNTIME_ARCHITECTURE.md)** - Process communication and deployment
- **[Technical Analysis](architecture/TECHNICAL_ANALYSIS.md)** - SWOT analysis and trade-offs
- **[Enterprise Architecture](architecture/enterprise.md)** - Enterprise deployment patterns

### 🚀 Operations & Deployment
Build, deploy, monitor, and scale Sutra AI in production

- **[Build & Deploy Guide](operations/BUILD_AND_DEPLOY.md)** - Complete build and deployment
- **[Deployment Guide](operations/DEPLOYMENT_GUIDE.md)** - Deployment procedures and configurations
- **[Production Requirements](operations/PRODUCTION_REQUIREMENTS.md)** - Production setup checklist
- **[Optimization Guide](operations/OPTIMIZATION_GUIDE.md)** - Performance tuning and optimization
- **[Scaling Guide](operations/SCALING_GUIDE.md)** - Horizontal and vertical scaling strategies
- **[Monitoring Guide](operations/MONITORING.md)** 🆕 - Observability, metrics, and debugging

### 📖 User Guides
Step-by-step guides for developers and operators

- **[Quick Start](guides/QUICK_START.md)** - Get started in 10 minutes
- **[Best Practices](guides/BEST_PRACTICES.md)** 🆕 - Development best practices and patterns
- **[Troubleshooting](../TROUBLESHOOTING.md)** - Common issues and solutions (root)

---

## 🔧 Component Documentation

### 💾 Storage Layer
High-performance Rust storage engine with sharding and vector search

- **[Sharded Storage](storage/SHARDING.md)** 🆕 - Multi-shard architecture for massive scale
- **[HNSW Optimization](storage/HNSW_OPTIMIZATION.md)** 🆕 - Build-once vector index strategy

### 🧠 Embedding Service
Dedicated high-performance embedding service with 768-dimensional vectors

- **[Service Overview](embedding/SERVICE_OVERVIEW.md)** - Architecture and features
- **[Migration Guide](embedding/MIGRATION_GUIDE.md)** - Migration from Ollama to dedicated service
- **[HA Design](embedding/HA_DESIGN.md)** 🆕 - High availability architecture (planned)

### 📥 Data Ingestion
High-performance bulk data ingestion

- **[Integration Guide](ingestion/INTEGRATION_GUIDE.md)** - Bulk ingester setup

---

## 🆕 What's New (2025-10-23)

### Recently Added Features
1. **Sharded Storage Mode** - 16-256 shards for massive scale (160M-2.5B concepts)
2. **HNSW Build-Once Optimization** - 100× faster vector search
3. **Dedicated Embedding Service** - nomic-embed-text-v1.5 with 768-d vectors
4. **Unified Learning Architecture** - Single source of truth in storage server
5. **TCP Binary Protocol** - 10-50× lower latency than gRPC

### New Documentation (This Release)
- ✅ `docs/architecture/SCALABILITY.md` - Complete scalability architecture
- ✅ `docs/storage/SHARDING.md` - Sharded storage design and configuration
- ✅ `docs/storage/HNSW_OPTIMIZATION.md` - HNSW index optimization guide
- ✅ `docs/operations/MONITORING.md` - Observability and metrics guide
- ✅ `docs/guides/BEST_PRACTICES.md` - Development best practices
- ✅ `docs/embedding/HA_DESIGN.md` - HA embedding service design
- ✅ **This file** - `docs/INDEX.md` - Master documentation index

---

## 🎯 Quick Reference

| I want to... | Read this... |
|--------------|--------------|
| Get started quickly | [Quick Start Guide](guides/QUICK_START.md) |
| Understand the architecture | [ARCHITECTURE.md](../ARCHITECTURE.md) |
| Deploy to production | [Build & Deploy](operations/BUILD_AND_DEPLOY.md) |
| Scale to millions of concepts | [Sharded Storage](storage/SHARDING.md) |
| Optimize performance | [Optimization Guide](operations/OPTIMIZATION_GUIDE.md) |
| Configure embedding service | [Embedding Service](embedding/SERVICE_OVERVIEW.md) |
| Troubleshoot issues | [TROUBLESHOOTING.md](../TROUBLESHOOTING.md) |
| Contribute code | [CONTRIBUTING.md](../CONTRIBUTING.md) |

---

**🔥 Pro Tip**: Bookmark this page for easy navigation to all Sutra AI documentation!

Last Updated: 2025-10-23 | Version: 2.0.0

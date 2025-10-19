# Sutra AI Documentation Index

**Complete guide to all available documentation**

Version 2.0 | Last Updated: 2025-10-19

---

## 🚀 Getting Started

### [Quick Reference](QUICK_REFERENCE.md) ⚡
**Start here for common operations**
- Deployment commands
- Python examples
- REST API quick start
- Configuration presets
- Troubleshooting cheat sheet

### [README](../README.md)
**Project overview and quick start**
- What Sutra AI does
- Architecture overview
- Quick start guide
- Performance benchmarks

### [Migration Guide](MIGRATION_GUIDE.md)
**Upgrading from version 1.0 to 2.0**
- Migration steps
- Feature-by-feature upgrade guide
- Backward compatibility notes
- Testing checklist

---

## 📖 Production Documentation

### [Production Guide](PRODUCTION_GUIDE.md) ⭐
**Complete production documentation** (Master Reference)
- Quick start (deployment + examples)
- Architecture diagrams
- All 4 production features documented
- Configuration reference
- Monitoring & observability
- Complete API reference
- Best practices
- Troubleshooting
- Performance tuning

**Topics Covered:**
- Self-observability system
- Quality gates & confidence calibration
- Streaming responses
- Natural language observability queries

---

## 🔧 Feature-Specific Guides

### [Streaming Guide](STREAMING.md)
**Progressive answer refinement**
- How streaming works (4 stages)
- Python async client
- JavaScript/React/Vue examples
- SSE protocol details
- Performance benchmarks
- Best practices

### [Production Enhancements](PRODUCTION_ENHANCEMENTS.md)
**Self-observability, quality gates, NL queries**
- Event system architecture
- Event types and usage
- Quality gate configuration
- Natural language observability
- Integration examples

### [Unified Learning Architecture](UNIFIED_LEARNING_ARCHITECTURE.md) 🔥
**Storage server as learning authority** (Implemented 2025-10-19)
- Single source of truth design
- Storage server learning pipeline
- TCP client implementation
- Migration from distributed logic
- Benefits and verification

---

## 🏗️ Architecture & Deployment

### [WARP.md](../WARP.md)
**Complete system architecture** (Developer Reference)
- Project overview
- Architecture diagrams
- Package structure
- Development commands
- Testing strategy
- Code style guide
- Production enhancements section

### [Deployment Guide](../DEPLOYMENT.md)
**Production deployment instructions**
- Docker Compose deployment
- Kubernetes manifests
- Manual deployment
- Environment variables
- Service configuration
- Health checks
- Scaling guide

---

## 🔍 Specialized Topics

### [Embedding Troubleshooting](EMBEDDING_TROUBLESHOOTING.md)
**Critical production fixes**
- Ollama service requirements
- TCP architecture details
- Common production errors
- Fix procedures

### [Grid Architecture](grid/architecture/GRID_ARCHITECTURE.md)
**Distributed storage orchestration**
- Grid master architecture
- Agent lifecycle management
- Event-driven monitoring
- Production deployment

### [Production Checklist](../PRODUCTION_CHECKLIST.md)
**Pre-deployment verification**
- Mandatory requirements
- Service checks
- Configuration validation
- Testing procedures

---

## 📚 Documentation by Use Case

### I want to...

#### Deploy Sutra AI
1. [README](../README.md) - Overview
2. [Production Guide](PRODUCTION_GUIDE.md) - Deployment section
3. [Deployment Guide](../DEPLOYMENT.md) - Detailed instructions
4. [Production Checklist](../PRODUCTION_CHECKLIST.md) - Pre-deployment verification

#### Use the Python SDK
1. [Quick Reference](QUICK_REFERENCE.md) - Python examples
2. [Production Guide](PRODUCTION_GUIDE.md) - Python SDK section
3. [Streaming Guide](STREAMING.md) - Async streaming examples

#### Use the REST API
1. [Quick Reference](QUICK_REFERENCE.md) - API quick start
2. [Production Guide](PRODUCTION_GUIDE.md) - API reference section
3. [Streaming Guide](STREAMING.md) - SSE endpoint details

#### Implement streaming responses
1. [Streaming Guide](STREAMING.md) - Complete streaming documentation
2. [Production Guide](PRODUCTION_GUIDE.md) - Streaming section
3. [Quick Reference](QUICK_REFERENCE.md) - Streaming examples

#### Monitor my system
1. [Production Enhancements](PRODUCTION_ENHANCEMENTS.md) - Observability section
2. [Production Guide](PRODUCTION_GUIDE.md) - Monitoring section
3. [Quick Reference](QUICK_REFERENCE.md) - Observability queries

#### Add quality gates
1. [Production Enhancements](PRODUCTION_ENHANCEMENTS.md) - Quality gates section
2. [Production Guide](PRODUCTION_GUIDE.md) - Quality gates section
3. [Quick Reference](QUICK_REFERENCE.md) - Quality gate presets

#### Understand the architecture
1. [WARP.md](../WARP.md) - Complete architecture
2. [Unified Learning Architecture](UNIFIED_LEARNING_ARCHITECTURE.md) - 🔥 NEW unified learning design
3. [Production Guide](PRODUCTION_GUIDE.md) - Architecture overview
4. [README](../README.md) - High-level overview
5. [ARCHITECTURE.md](../ARCHITECTURE.md) - Updated technical details

#### Troubleshoot issues
1. [Quick Reference](QUICK_REFERENCE.md) - Troubleshooting section
2. [Production Guide](PRODUCTION_GUIDE.md) - Troubleshooting section
3. [Embedding Troubleshooting](EMBEDDING_TROUBLESHOOTING.md) - Critical fixes

#### Migrate from version 1.0
1. [Migration Guide](MIGRATION_GUIDE.md) - Complete migration instructions
2. [Production Guide](PRODUCTION_GUIDE.md) - New features overview
3. [Quick Reference](QUICK_REFERENCE.md) - New API examples

#### Develop new features
1. [WARP.md](../WARP.md) - Development guide
2. [Production Guide](PRODUCTION_GUIDE.md) - Architecture section
3. [README](../README.md) - Project structure

---

## 📊 Documentation Statistics

| Document | Lines | Purpose | Audience |
|----------|-------|---------|----------|
| **Production Guide** | 1024 | Master reference | All users |
| **Streaming Guide** | 593 | Streaming feature | Developers |
| **WARP.md** | ~2000 | Architecture | Developers |
| **Production Enhancements** | 444 | Feature details | Developers |
| **Quick Reference** | 439 | Cheat sheet | All users |
| **Migration Guide** | 506 | Version upgrade | System admins |
| **README** | 386 | Project overview | All users |
| **Deployment Guide** | ~500 | Infrastructure | DevOps |

**Total documentation:** ~6000 lines covering all aspects of Sutra AI

---

## 🎯 Recommended Reading Paths

### New User (First Time)
1. [README](../README.md) - 5 min
2. [Quick Reference](QUICK_REFERENCE.md) - 10 min
3. Deploy and try examples
4. [Production Guide](PRODUCTION_GUIDE.md) - As needed

### Existing User (Upgrading to 2.0)
1. [Migration Guide](MIGRATION_GUIDE.md) - 15 min
2. [Production Enhancements](PRODUCTION_ENHANCEMENTS.md) - 10 min
3. [Quick Reference](QUICK_REFERENCE.md) - 5 min
4. [Streaming Guide](STREAMING.md) - Optional

### Developer (Building Features)
1. [WARP.md](../WARP.md) - 30 min
2. [Production Guide](PRODUCTION_GUIDE.md) - 20 min
3. [Production Enhancements](PRODUCTION_ENHANCEMENTS.md) - 15 min
4. [Grid Architecture](grid/architecture/GRID_ARCHITECTURE.md) - Optional

### DevOps (Deploying Production)
1. [Production Checklist](../PRODUCTION_CHECKLIST.md) - 10 min
2. [Deployment Guide](../DEPLOYMENT.md) - 20 min
3. [Production Guide](PRODUCTION_GUIDE.md) - Configuration section
4. [Embedding Troubleshooting](EMBEDDING_TROUBLESHOOTING.md) - 10 min

### API User (Integration)
1. [Quick Reference](QUICK_REFERENCE.md) - REST API section
2. [Production Guide](PRODUCTION_GUIDE.md) - API reference section
3. [Streaming Guide](STREAMING.md) - If using streaming

---

## 🔄 Documentation Version History

### Version 2.0 (Current)
- ✅ Production Guide (master documentation)
- ✅ Streaming Guide (detailed streaming documentation)
- ✅ Production Enhancements (observability, quality gates)
- ✅ Migration Guide (1.0 → 2.0 upgrade)
- ✅ Quick Reference (cheat sheet)
- ✅ Updated README with production features
- ✅ Updated WARP.md with production section

### Version 1.0
- README (basic overview)
- WARP.md (architecture)
- Deployment Guide
- Grid Architecture
- Embedding Troubleshooting

---

## 📝 Contributing to Documentation

### Guidelines
- Keep examples practical and tested
- Include code snippets for all features
- Add troubleshooting sections
- Use consistent formatting
- Link between related docs

### File Locations
```
sutra-models/
├── README.md                        # Project overview
├── DEPLOYMENT.md                     # Deployment instructions
├── WARP.md                          # Architecture guide
├── PRODUCTION_CHECKLIST.md          # Pre-deployment checks
├── docs/
│   ├── INDEX.md                     # This file
│   ├── PRODUCTION_GUIDE.md          # Master production docs
│   ├── STREAMING.md                 # Streaming guide
│   ├── PRODUCTION_ENHANCEMENTS.md   # Feature details
│   ├── QUICK_REFERENCE.md           # Cheat sheet
│   ├── MIGRATION_GUIDE.md           # Version upgrade
│   ├── EMBEDDING_TROUBLESHOOTING.md # Critical fixes
│   └── grid/
│       └── architecture/
│           └── GRID_ARCHITECTURE.md # Grid details
```

---

## 🆘 Getting Help

### Documentation Not Clear?
- Open issue: [GitHub Issues](https://github.com/your-org/sutra-models/issues)
- Include: Document name, section, what's unclear

### Missing Documentation?
- Request via GitHub issue
- Tag as "documentation"
- Describe what you need documented

### Found an Error?
- Create pull request with fix
- Or open issue with details

---

## 📞 Support Resources

### Documentation
- This index
- [Production Guide](PRODUCTION_GUIDE.md) (most comprehensive)
- [Quick Reference](QUICK_REFERENCE.md) (fastest)

### Code Examples
- `examples/` directory
- Code blocks in all documentation
- Inline examples in [Production Guide](PRODUCTION_GUIDE.md)

### Self-Service
```python
# Query your own system
from sutra_core.observability_query import create_observability_interface
obs = create_observability_interface(engine.storage)
obs.query("What issues occurred today?")
```

### External
- GitHub Issues
- Check logs: `./sutra-deploy.sh logs`
- Health endpoint: `curl http://localhost:8001/sutra/health`

---

**Your complete documentation hub!** 📚

All guides maintained and up-to-date for version 2.0.

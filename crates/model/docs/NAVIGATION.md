# Documentation Navigation

This document provides an overview of the documentation structure and how to navigate to find what you need.

## 📁 Documentation Structure

```
docs/
├── README.md                    # Main documentation index
├── getting-started/             # Quick start guides
│   ├── quickstart.md           # Get running in minutes
│   ├── installation.md         # Detailed setup
│   └── first-steps.md          # First application
├── architecture/                # System design
│   ├── overview.md             # High-level architecture
│   ├── components.md           # Crate deep-dive
│   └── performance.md          # Benchmarks & optimization
├── enterprise/                  # Enterprise features
│   ├── deployment.md           # Production deployment
│   ├── demos.md                # Live demonstrations
│   ├── security.md             # Security features
│   └── cost-analysis.md        # TCO calculations
├── tutorials/                   # Step-by-step guides
│   ├── quantization.md         # AWQ tutorial
│   ├── rwkv.md                 # RWKV guide
│   ├── mamba.md                # Mamba tutorial
│   ├── qlora.md                # Fine-tuning guide
│   └── nesy.md                 # Neuro-symbolic AI
├── api/                        # API reference
│   ├── core.md                 # Core types & operations
│   ├── quantization.md         # AWQ API
│   ├── loading.md              # Model loading
│   ├── tokenization.md         # Tokenizers
│   └── training.md             # Training APIs
├── examples/                   # Example guides
│   ├── trading-terminal.md     # Live trading demo
│   ├── e2e-pipeline.md         # End-to-end workflow
│   ├── quantization-benchmark.md # Performance testing
│   └── inference.md            # Model inference
├── deployment/                 # Deployment guides
│   ├── production.md           # Production setup
│   ├── docker.md               # Container deployment
│   ├── cloud.md                # Cloud platforms
│   └── monitoring.md           # Production monitoring
└── contributing/               # Developer guides
    ├── development.md          # How to contribute
    ├── standards.md            # Code conventions
    └── testing.md              # Testing guidelines
```

## 🎯 Quick Navigation by Goal

### I want to get started immediately
→ [Quick Start Guide](getting-started/quickstart.md)

### I want to see live demos
→ [Enterprise Demos](enterprise/demos.md)

### I want to deploy to production
→ [Production Deployment](enterprise/deployment.md)

### I want to understand the system
→ [System Overview](architecture/overview.md)

### I want to learn quantization
→ [Quantization Tutorial](tutorials/quantization.md)

### I want to use the API
→ [API Reference](api/core.md)

### I want to run examples
→ [Examples Guide](examples/trading-terminal.md)

### I want to contribute
→ [Development Guide](contributing/development.md)

## 🚀 Getting Started Path

1. **[Quick Start](getting-started/quickstart.md)** - Set up and run examples (5 minutes)
2. **[Live Trading Demo](examples/trading-terminal.md)** - See AI in action (2 minutes)
3. **[Architecture Overview](architecture/overview.md)** - Understand the design (10 minutes)
4. **[Quantization Tutorial](tutorials/quantization.md)** - Learn compression (20 minutes)
5. **[Production Deployment](enterprise/deployment.md)** - Deploy your system (30 minutes)

## 📊 Documentation by Role

### 👩‍💻 **Developer**
- [Quick Start](getting-started/quickstart.md) - Setup development environment
- [API Reference](api/core.md) - Comprehensive API docs
- [Contributing](contributing/development.md) - Code standards and workflow
- [Architecture](architecture/overview.md) - System design principles

### 🏢 **Enterprise User**
- [Live Demos](enterprise/demos.md) - See real-world applications
- [Production Deployment](enterprise/deployment.md) - Enterprise deployment
- [Security](enterprise/security.md) - Security and compliance
- [Cost Analysis](enterprise/cost-analysis.md) - ROI calculations

### 🎓 **Researcher/Student**
- [Quantization Tutorial](tutorials/quantization.md) - Learn AWQ algorithm
- [RWKV Guide](tutorials/rwkv.md) - Efficient RNN architectures
- [Mamba Guide](tutorials/mamba.md) - State space models
- [Architecture](architecture/overview.md) - Technical deep-dive

### 🚀 **DevOps Engineer**
- [Docker Deployment](deployment/docker.md) - Container strategies
- [Cloud Deployment](deployment/cloud.md) - Cloud platforms
- [Monitoring](deployment/monitoring.md) - Production monitoring
- [Production Setup](deployment/production.md) - Infrastructure

## 📈 Documentation by Experience Level

### 🟢 **Beginner** (New to SutraWorks)
1. [Quick Start](getting-started/quickstart.md)
2. [Trading Terminal Demo](examples/trading-terminal.md)
3. [System Overview](architecture/overview.md)

### 🟡 **Intermediate** (Familiar with AI/ML)
1. [Quantization Tutorial](tutorials/quantization.md)
2. [API Reference](api/core.md)
3. [End-to-End Pipeline](examples/e2e-pipeline.md)

### 🔴 **Advanced** (Ready for Production)
1. [Production Deployment](enterprise/deployment.md)
2. [Docker Deployment](deployment/docker.md)
3. [Contributing Guide](contributing/development.md)

## 🔍 Documentation by Feature

### **Model Compression**
- [Quantization Tutorial](tutorials/quantization.md) - Complete AWQ guide
- [Quantization API](api/quantization.md) - API reference
- [Quantization Benchmark](examples/quantization-benchmark.md) - Performance testing

### **Efficient Architectures**
- [RWKV Tutorial](tutorials/rwkv.md) - O(n) RNN architecture
- [Mamba Tutorial](tutorials/mamba.md) - State space models
- [Architecture Overview](architecture/overview.md) - Design principles

### **Production Deployment**
- [Production Setup](deployment/production.md) - Basic deployment
- [Docker Deployment](deployment/docker.md) - Containerized deployment
- [Cloud Deployment](deployment/cloud.md) - Scalable cloud deployment
- [Monitoring](deployment/monitoring.md) - Production monitoring

### **Enterprise Features**
- [Live Demos](enterprise/demos.md) - Interactive demonstrations
- [Security](enterprise/security.md) - Security and compliance
- [Cost Analysis](enterprise/cost-analysis.md) - Business value

## 📚 External Resources

### **Generated Documentation**
```bash
# Generate and view API documentation
cargo doc --open
```

### **Examples**
```bash
# All working examples are in the examples/ directory
ls examples/
cargo run --example <name> --release
```

### **Source Code**
- **Core Implementation**: `crates/` directory
- **Working Examples**: `examples/` directory
- **Tests**: Each crate has comprehensive tests

## 🎯 Documentation Quality Standards

All documentation follows these principles:

### ✅ **Accurate**
- All examples are tested and working
- Performance claims are measured and verified
- No outdated or misleading information

### ✅ **Complete**
- Comprehensive coverage of all features
- Both basic and advanced use cases
- Troubleshooting and common issues

### ✅ **Practical**
- Working code examples
- Step-by-step instructions
- Copy-paste ready commands

### ✅ **Well-Organized**
- Clear hierarchy and navigation
- Consistent formatting and style
- Cross-references and links

## 📞 Getting Help

If you can't find what you're looking for:

1. **Search the docs** using your browser's search (Ctrl+F)
2. **Check examples** - 7 working demonstrations available
3. **Run API docs** - `cargo doc --open` for detailed API reference
4. **Look at tests** - Comprehensive test suite shows usage patterns
5. **File an issue** - Request documentation improvements

## 🔄 Documentation Updates

Documentation is continuously improved:

- **Examples verified** with each release
- **Performance metrics updated** with benchmarks
- **New features documented** as they're added
- **Community feedback** incorporated regularly

---

**Start exploring**: [📚 Main Documentation Index](README.md)
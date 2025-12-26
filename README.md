<div align="center">

# Sutra AI

### **Production-Grade Explainable AI for Domain-Specific Knowledge**

*Build intelligent systems that reason over your private data with complete audit trails—no frontier LLMs required.*

[![Version](https://img.shields.io/badge/version-3.3.0-blue.svg)](https://github.com/sutraworks/sutra-memory/releases)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Production Ready](https://img.shields.io/badge/status-production--ready-brightgreen.svg)]()
[![Tests](https://img.shields.io/badge/tests-passing-brightgreen.svg)]()
[![Performance](https://img.shields.io/badge/throughput-520_req%2Fs-brightgreen.svg)]()
[![Latency](https://img.shields.io/badge/latency-5--9ms-brightgreen.svg)]()

[Quick Start](#-quick-start) • [Documentation](docs/README.md) • [Architecture](#-architecture) • [Use Cases](#-use-cases) • [Contributing](#-contributing)

</div>

---

## Why Sutra AI?

**Most enterprises don't need general world knowledge. They need explainable reasoning over THEIR proprietary data.**

Traditional LLMs (GPT-4, Claude, etc.) are powerful but come with critical limitations for enterprise use:

- **🔴 Black-box reasoning** → No audit trails for compliance
- **🔴 Expensive at scale** → $0.01-$0.10/query = $100K-$1M/year
- **🔴 Privacy concerns** → Your data sent to external APIs
- **🔴 Can't explain decisions** → Fails regulatory requirements
- **🔴 Massive models** → 100GB-1TB trained on everything

**Sutra AI takes a different approach:**

| Challenge | Traditional LLMs | Sutra AI |
|-----------|-----------------|----------|
| **Model Size** | 100GB-1TB | 500MB-2GB |
| **Cost per Query** | $0.01-$0.10 | ~$0.0001 |
| **Explainability** | Black box | Complete audit trails |
| **Privacy** | External API calls | Self-hosted |
| **Domain Knowledge** | Requires $10K-$100K fine-tuning | Learns from your data |
| **Compliance** | ❌ Not auditable | ✅ FDA/SEC/HIPAA ready |

---

## What is Sutra AI?

**Sutra AI is a domain-specific reasoning engine** that learns exclusively from your proprietary knowledge and provides explainable answers with complete audit trails.

### Core Capabilities

- **🎯 Your Knowledge, Our Reasoning** - Learns from your domain data (protocols, cases, procedures, documents)
- **📊 1000× Smaller Models** - 500MB embedding models vs 100GB+ LLMs
- **🔍 Complete Audit Trails** - Every decision fully traceable for compliance
- **⚡ Real-Time Learning** - Updates instantly without retraining
- **🔒 Privacy-First** - Self-hosted, your data never leaves your infrastructure
- **💰 Cost Effective** - ~$0.0001 per query vs $0.01-$0.10 for LLMs (100-1000× cheaper)
- **📈 Production Grade** - 520 req/sec throughput, 5-9ms latency, 100% test success

### Two Deployment Modes

#### 🖥️ **Desktop Edition** - Self-Contained Native App
Perfect for individual researchers, small teams, or local development.

- Pure Rust native macOS application
- No Docker, no servers, no dependencies
- Complete privacy - all data stays on your machine
- Single ~20MB binary
- Full storage engine and reasoning capabilities

```bash
cargo run -p sutra-desktop
```

[Desktop Documentation →](docs/desktop/README.md)

#### 🏢 **Server Edition** - Distributed Production System
Enterprise-grade deployment for teams and organizations.

- 15-service distributed architecture
- Horizontal scaling with sharding (10M+ concepts)
- High-availability embedding service (3 replicas + HAProxy)
- Grid infrastructure for multi-node orchestration
- Production monitoring and self-observation

[Deployment Guide →](docs/deployment/README.md)

---

## 🚀 Quick Start

### Option 1: Desktop Edition (Fastest)

```bash
# Clone repository
git clone https://github.com/sutraworks/sutra-memory.git
cd sutra-memory

# Build and run (requires Rust)
cargo run -p sutra-desktop
```

**That's it!** You now have a fully functional AI reasoning system running locally.

### Option 2: Server Edition (Production)

```bash
# 1. Build all services
./sutra build

# 2. Deploy (default: Community edition)
./sutra deploy

# 3. Access the interface
open http://localhost:8080
```

**You now have:**
- ✅ 520 req/sec peak throughput
- ✅ 5-9ms average latency
- ✅ Web UI, REST API, and monitoring dashboard
- ✅ Production-ready ML services

[Complete Quick Start Guide →](docs/getting-started/quickstart.md)

---

## ⚡ Key Features

### Production-Ready Architecture

| Feature | Description | Status |
|---------|-------------|--------|
| **Unified Learning Pipeline** | Storage server owns complete learning process | ✅ Production |
| **Quality Gates** | Confidence calibration + "I don't know" responses | ✅ Production |
| **Streaming Responses** | Progressive response refinement (SSE) | ✅ Production |
| **Self-Monitoring** | Natural language operational queries | ✅ Production |
| **HA Embedding** | 3 replicas + HAProxy load balancer | ✅ Production |
| **Sharded Storage** | 4-16 shards for 10M+ concepts | ✅ Production |
| **Zero Data Loss** | Write-Ahead Log + 2PC transactions | ✅ Production |
| **Security** | TLS 1.3, HMAC-SHA256, RBAC | ✅ Production |

### Advanced Capabilities

- **Multi-Path Reasoning (MPPA)** - Consensus-based graph traversal for robust answers
- **Temporal & Causal Reasoning** - Track changes over time, identify cause-effect relationships
- **Semantic Filtering** - Advanced query capabilities beyond keyword search
- **Contradiction Detection** - Identify conflicting information in your knowledge base
- **Grid Event Ingestion** - Self-monitoring using own knowledge graph ("eating our own dogfood")

---

## 🏗️ Architecture

### High-Level System Design

```
┌─────────────────────────────────────────────────────────────┐
│                    Sutra AI Production Stack                │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  🌐 Web Interfaces                                          │
│    ├─ Interactive Client (Streamlit)         :8080         │
│    ├─ Control Center (React)                 :9000         │
│    └─ Storage Explorer (D3.js)                :8100         │
│                                                             │
│  🔌 API Layer                                               │
│    ├─ REST API (FastAPI)                     :8000         │
│    ├─ Semantic Orchestration                 :8001         │
│    └─ Bulk Ingester (Rust)                   :8005         │
│                                                             │
│  🤖 ML Services                                             │
│    ├─ Embedding HA (3 replicas)              :8888         │
│    └─ NLG Service                             :8890         │
│                                                             │
│  💾 Core Infrastructure (Rust)                              │
│    ├─ Storage Engine (TCP)                   :50051        │
│    ├─ Grid Master                             :7001         │
│    └─ Grid Agents                             :7002+        │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Technology Stack

- **Rust** - High-performance storage engine with lock-free concurrent memory
- **Python** - Reasoning engine with Multi-Path Plan Aggregation (MPPA)
- **React** - Modern web interfaces with Material UI
- **FastAPI** - Production REST API layer
- **Docker** - Containerized deployment with Compose orchestration

### Key Innovations

**TCP Binary Protocol** - Custom MessagePack-based protocol for 10-50× better performance than gRPC/REST

**Unified Learning Pipeline** - Single source of truth for all ingestion paths:
```
Client → TCP learn_concept() → Storage Server Pipeline:
  1. Generate Embedding (768-dim vectors)
  2. Extract Associations (semantic analysis)
  3. Store Atomically (HNSW + WAL)
  4. Return concept_id
```

[Complete Architecture Documentation →](docs/architecture/SYSTEM_ARCHITECTURE.md)

---

## 💡 Use Cases

### Healthcare: Clinical Decision Support

```
YOUR DATA: Treatment protocols, safety guidelines, drug interactions, case histories
QUERY: "Is Treatment X safe for this patient profile?"
OUTPUT: Reasoning paths through YOUR protocols with complete audit trail
VALUE: FDA compliance, malpractice protection, quality assurance
```

### Finance: Regulatory Compliance

```
YOUR DATA: Risk models, regulatory rules, historical decisions
QUERY: "Should we approve this credit application?"
OUTPUT: Decision path through YOUR risk framework
VALUE: SEC/FINRA compliance, audit defense, consistent policy
```

### Legal: Precedent Analysis

```
YOUR DATA: Firm's case database, jurisdiction-specific precedents
QUERY: "What's the likely outcome for this contract dispute?"
OUTPUT: Similar cases from YOUR database with outcomes
VALUE: Client explanations, court arguments, billable transparency
```

### Manufacturing: Quality Control

```
YOUR DATA: Quality standards, inspection procedures, defect patterns
QUERY: "Should this batch pass inspection?"
OUTPUT: Decision path through YOUR standards with evidence
VALUE: ISO compliance, defect reduction, audit trails
```

[More Use Cases →](docs/getting-started/use-cases.md)

---

## 📖 Documentation

### Getting Started
- [5-Minute Quickstart](docs/getting-started/quickstart.md)
- [Complete Tutorial](docs/getting-started/tutorial.md)
- [Edition Comparison](docs/getting-started/editions.md) (Simple/Community/Enterprise)

### Deployment
- [Build Guide](docs/build/README.md)
- [Deployment Guide](docs/deployment/README.md)
- [Production Security Setup](docs/security/PRODUCTION_SECURITY_SETUP.md)
- [Desktop Edition](docs/desktop/README.md)

### Development
- [Development Guide](docs/guides/DEVELOPMENT.md)
- [API Reference](docs/api/API_REFERENCE.md)
- [Architecture Deep Dive](docs/architecture/SYSTEM_ARCHITECTURE.md)
- [Contributing Guidelines](docs/CONTRIBUTING.md)

### Operations
- [Release Management](docs/release/README.md)
- [Monitoring & Observability](docs/operations/monitoring.md)
- [Troubleshooting](docs/TROUBLESHOOTING.md)

[📚 Complete Documentation Hub →](docs/README.md)

---

## 🎯 What's New

### December 2025 - Technical Excellence Achieved

**Zero Technical Debt in Core Systems**

- ✅ **Storage Engine Excellence** - 137/137 tests passing, zero warnings, production-ready WAL recovery
- ✅ **Mock Elimination Complete** - All 12 mocks replaced with real connections or fail-fast behavior
- ✅ **Grid Event Ingestion** - Self-monitoring via knowledge graph (query events in natural language)
- ✅ **Fail-Fast Philosophy** - Bulk ingester fails loudly instead of silently discarding data
- ✅ **Graceful Degradation** - Control Center shows "unavailable" instead of crashing

### November 2025 - Desktop Edition Released

- ✅ **Pure Rust Application** - Native macOS app using egui/eframe
- ✅ **No Docker Required** - Self-contained single binary (~20MB)
- ✅ **Full Features** - Chat, Knowledge Browser, Search, Settings
- ✅ **Local Persistence** - WAL-backed storage in ~/Library/Application Support/

### October 2025 - External ML Integration

- ✅ **58× Throughput Improvement** - 9 r/s → 520 r/s peak async throughput
- ✅ **11-20× Faster Latency** - 100-200ms → 5-9ms average response time
- ✅ **100% E2E Test Success** - 3/3 comprehensive tests passing
- ✅ **Production Security** - TLS 1.3, HMAC-SHA256, RBAC, network isolation

[Complete Release Notes →](docs/release/)

---

## 🤝 Contributing

We welcome contributions from the community! Whether you're fixing bugs, adding features, improving documentation, or sharing use cases, your help makes Sutra AI better for everyone.

### How to Contribute

1. **Fork the repository** and create your feature branch
2. **Read the guidelines** - [CONTRIBUTING.md](docs/CONTRIBUTING.md)
3. **Make your changes** - Follow our code style and testing requirements
4. **Submit a pull request** - We'll review and provide feedback

### Areas We Need Help

- 🌍 **Internationalization** - Support for languages beyond English
- 📊 **Visualizations** - Enhanced graph visualization tools
- 📚 **Documentation** - Tutorials, examples, translations
- 🧪 **Testing** - Expand test coverage, add domain-specific tests
- 🎨 **UI/UX** - Improvements to web interfaces

### Development Setup

```bash
# Clone and setup
git clone https://github.com/sutraworks/sutra-memory.git
cd sutra-memory

# Install dependencies
pip install -r requirements-dev.txt
npm install

# Run tests
npm run test:e2e           # E2E tests
pytest tests/ -v           # Python tests
cargo test --workspace     # Rust tests

# Code quality
black packages/            # Format Python
cargo clippy --workspace   # Lint Rust
```

[Development Guide →](docs/guides/DEVELOPMENT.md)

---

## 🌟 Community & Support

### Get Help

- 📖 **Documentation** - [docs/README.md](docs/README.md)
- 💬 **Discussions** - [GitHub Discussions](https://github.com/sutraworks/sutra-memory/discussions)
- 🐛 **Issues** - [Report bugs](https://github.com/sutraworks/sutra-memory/issues)
- 📧 **Email** - support@sutraworks.com

### Stay Updated

- ⭐ **Star this repo** to show your support
- 👀 **Watch releases** for updates
- 🐦 **Follow us** on social media (coming soon)

### Enterprise Support

For production deployments, custom integrations, or enterprise SLAs:
- 📧 Contact: enterprise@sutraworks.com
- 🌐 Website: https://sutraworks.com (coming soon)

---

## 📊 Performance & Scale

### Benchmarks (v3.3.0)

| Metric | Performance |
|--------|-------------|
| **Sequential Throughput** | 117 req/s |
| **Concurrent Throughput** | 520 req/s (async) |
| **Average Latency** | 5-9ms |
| **P95 Latency** | 10-21ms |
| **Success Rate** | 100% |
| **Storage Capacity** | 10M+ concepts |
| **Startup Time** | <50ms (1M vectors) |

### Scale Configuration

| Concept Count | Shards | Use Case | RAM |
|--------------|--------|----------|-----|
| < 100K | 1 | Development | 4GB |
| 1M - 5M | 4 | Production | 16GB |
| 5M - 10M | 8 | High-scale | 32GB |
| 10M+ | 16 | Enterprise | 64GB+ |

[Performance Documentation →](docs/operations/performance.md)

---

## 🔒 Security & Compliance

### Production Security Features

- **TLS 1.3 Encryption** - All network communication encrypted
- **HMAC-SHA256 Authentication** - Cryptographic authentication
- **Role-Based Access Control (RBAC)** - Admin, Writer, Reader, Service roles
- **Network Isolation** - Internal services not exposed to host
- **Audit Logging** - Complete trails for all operations
- **Rate Limiting** - Per-endpoint DDoS protection

### Compliance Readiness

Sutra AI is designed for regulated industries:

- ✅ **HIPAA** - Healthcare privacy and security
- ✅ **SOC 2** - Service organization controls
- ✅ **GDPR** - Data protection and privacy
- ✅ **FDA** - Medical device software compliance
- ✅ **SEC/FINRA** - Financial regulations

[Production Security Setup →](docs/security/PRODUCTION_SECURITY_SETUP.md)

---

## 📜 License

Sutra AI is open-source software licensed under the **MIT License**.

See [LICENSE](LICENSE) for full details.

```
MIT License

Copyright (c) 2025 Sutra Works

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.
```

---

## 🙏 Acknowledgments

Built with world-class open-source technologies:

- **Rust** - Systems programming language for performance and safety
- **Python** - FastAPI, Streamlit, and rich ML ecosystem
- **React** - Modern web interface framework
- **Docker** - Containerization and deployment
- **USearch** - High-performance vector search (HNSW)
- **nomic-embed** - State-of-art embedding models

Inspired by decades of knowledge representation research and recent advances in semantic reasoning.

---

## 🚀 What's Different About Sutra AI?

### Not Just Another Vector Database

**Vector databases** (Pinecone, Weaviate, Milvus) provide similarity search but lack:
- ❌ Graph-based reasoning
- ❌ Multi-path consensus aggregation
- ❌ Temporal and causal reasoning
- ❌ Complete audit trails for compliance

**Sutra AI** provides semantic search **AND** explainable graph reasoning with production-grade features.

### Not a General-Purpose LLM

**We're NOT trying to replace ChatGPT.** Different problem:

| Use Case | Best Solution |
|----------|---------------|
| Creative writing, general chat | ChatGPT, Claude |
| Domain-specific reasoning with audit trails | **Sutra AI** |
| Regulated industry compliance | **Sutra AI** |
| Private enterprise knowledge | **Sutra AI** |
| Cost-sensitive high-volume queries | **Sutra AI** |

### Built for Production

Many research projects claim "production-ready" but lack:
- ❌ High-availability deployment
- ❌ Monitoring and observability
- ❌ Security and authentication
- ❌ Complete documentation
- ❌ Automated testing

**Sutra AI delivers:**
- ✅ 137/137 tests passing with zero warnings
- ✅ 100% E2E test success rate
- ✅ Production security (TLS 1.3, RBAC)
- ✅ Self-monitoring via knowledge graph
- ✅ 5,000+ lines of documentation
- ✅ Semantic versioning + CI/CD

---

<div align="center">

**Ready to build explainable AI for your domain?**

[Get Started](docs/getting-started/quickstart.md) • [Read the Docs](docs/README.md) • [Join Discussions](https://github.com/sutraworks/sutra-memory/discussions)

---

**Built with ❤️ by the Sutra Works team and contributors worldwide**

*Star ⭐ this repository if you find it useful!*

</div>

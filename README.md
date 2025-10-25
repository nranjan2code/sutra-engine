# Sutra AI

**Domain-Specific Reasoning Engine for Your Knowledge**

[![Production Ready](https://img.shields.io/badge/status-production--ready-green)]()
[![Version](https://img.shields.io/badge/version-2.0.0-blue)]()
[![Grade](https://img.shields.io/badge/storage-A+-brightgreen)]()

Explainable reasoning over your private domain knowledge—without frontier LLMs. Built for regulated industries requiring complete audit trails and 1000× lower costs than ChatGPT.

---

## 🎉 What's New (2025-10-25)

**Production-Ready: All P0 Features Complete**

- ✅ **Cross-shard 2PC Transactions** - Zero data loss at scale
- ✅ **Embedding Service HA** - 3 replicas + HAProxy (>95% uptime)
- ✅ **Self-Monitoring** - Sutra monitors itself using own reasoning engine
- ✅ **10M Concept Validation** - Complete scale testing suite
- ✅ **94× Faster Startup** - USearch HNSW persistent indexes
- ✅ **Adaptive Reconciliation** - Self-optimizing storage (80% CPU savings)
- ✅ **Production Grade** - 107 tests passing, DoS protection, input validation
- ✅ **Semantic Query API** - Advanced semantic filtering, temporal reasoning, causal/contradiction detection

**[📖 Complete Documentation](docs/INDEX.md)** | **[🚀 Quick Start](#quick-start)** | **[📊 Benchmarks](#performance)**

---

## Why Sutra AI?

### The Problem: General AI vs Domain-Specific Needs

**Frontier LLMs (GPT-4, Claude, etc.):**
- 🔴 100GB-1TB models trained on everything
- 🔴 Black-box reasoning (no audit trails)
- 🔴 $0.01-$0.10 per query ($100K-$1M/year at scale)
- 🔴 Require fine-tuning for domain knowledge ($10K-$100K)
- 🔴 Privacy concerns (API calls to external services)
- 🔴 Can't explain decisions for compliance

**Most enterprises don't need general world knowledge.  
They need explainable reasoning over THEIR proprietary data.**

### The Sutra Solution

🎯 **Domain-Specific Reasoning Engine**

✅ **Your Knowledge, Our Reasoning** - Learns from your domain data (protocols, cases, procedures)  
✅ **1000× Smaller Models** - 500MB embedding model vs 100GB+ LLMs  
✅ **Complete Audit Trails** - Every decision fully traceable for compliance  
✅ **Real-Time Learning** - Updates instantly without retraining  
✅ **Self-Hosted** - No API calls, your data stays private  
✅ **Cost Effective** - ~$0.0001 per query vs $0.01-$0.10 for LLMs  

**Perfect for:** Healthcare compliance, financial regulations, legal precedents, government accountability—anywhere explainability is mandatory.

---

## How It Works

### The User Provides Knowledge, Sutra Provides Reasoning

**Not a pre-trained world model. A reasoning engine for YOUR domain.**

```
Step 1: Feed Your Domain Knowledge
  ├─→ Hospital treatment protocols
  ├─→ Legal case precedents
  ├─→ Company procedures
  ├─→ Research databases
  └─→ ANY domain-specific knowledge

Step 2: Sutra Builds Reasoning Graph
  ├─→ Extracts concepts and relationships
  ├─→ Generates semantic embeddings (768-d, small model)
  ├─→ Creates connected knowledge graph
  └─→ Real-time updates (no retraining)

Step 3: Query with Full Explainability
  ├─→ Multi-path graph traversal
  ├─→ Semantic similarity matching
  ├─→ Consensus-based aggregation (MPPA)
  └─→ Complete audit trail for every answer
```

### Example: Hospital Compliance System

```
DAY 1: Empty Sutra system
  ↓
Hospital loads:
  • 10,000 treatment protocols
  • 5,000 patient safety guidelines
  • Drug interaction database
  • Historical case outcomes
  ↓
Doctor queries: "Is Treatment X safe for this patient profile?"
  ↓
Sutra reasons using HOSPITAL'S knowledge:
  Path 1: Hospital Protocol #247 → Treatment X approved (conf: 0.87)
  Path 2: Similar Case #1823 → Successful outcome (conf: 0.82)
  Path 3: Drug Database → No interactions (conf: 0.95)
  ↓
Answer with complete FDA-auditable trail
```

### Key Features

**Production-Ready (v2.0)**

| Feature | Description | Status |
|---------|-------------|--------|
| **Unified Learning** | Storage server owns complete pipeline | ✅ |
| **Quality Gates** | Confidence calibration + "I don't know" | ✅ |
| **Streaming** | Progressive response refinement (SSE) | ✅ |
| **Self-Observability** | Natural language operational queries | ✅ |
| **HA Embedding** | 3 replicas + HAProxy load balancer | ✅ |
| **Sharded Storage** | 4-16 shards for 10M-2.5B concepts | ✅ |
| **Zero Data Loss** | Write-Ahead Log + 2PC transactions | ✅ |
| **Semantic Queries** | Filters, temporal/causal chains, contradictions | ✅ |

---

## Architecture

### System Overview

**12-Service Production Ecosystem with TCP Binary Protocol**

```
┌─────────────────────────────────────────────────────────┐
│               Sutra AI Production Stack                 │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  Web Interfaces                                         │
│  ├─ Control Center (React + Material UI)  :9000       │
│  ├─ Interactive Client (Streamlit)        :8080       │
│  └─ Storage Explorer (React + D3.js)      :8100       │
│                                                         │
│  API Layer                                              │
│  ├─ Primary REST API (FastAPI)            :8000       │
│  ├─ Hybrid API (Semantic + NLG)           :8001       │
│  └─ Bulk Ingester (Rust)                  :8005       │
│                                                         │
│  Core Infrastructure (TCP Binary Protocol)              │
│  ├─ Storage Server (Rust - 57K writes/sec) :50051     │
│  ├─ Embedding Service HA (3 replicas)      :8888      │
│  ├─ Grid Master (Orchestration)            :7001-7002  │
│  └─ Event Storage (Self-monitoring)        :50052     │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### Unified Learning Pipeline

**Key Innovation:** Storage server owns complete learning process

```
✅ Unified Architecture (2025-10-19)

ANY Client (API/Hybrid/Bulk/Python)
  └─→ TCP: learn_concept(content, options)
      └─→ Storage Server Pipeline:
          ├─→ 1. Generate Embedding (HA service, 768-d)
          ├─→ 2. Extract Associations (Rust NLP)
          ├─→ 3. Store Atomically (HNSW + WAL)
          └─→ 4. Return concept_id

Benefits:
✅ Single source of truth
✅ Automatic embeddings for ALL paths
✅ Zero code duplication
✅ Guaranteed consistency
```

**[Complete Architecture Documentation →](WARP.md)**

---

## Quick Start

### Prerequisites

- Docker & Docker Compose
- Python 3.11+ (for local development)
- 8GB RAM minimum (16GB recommended)
- macOS, Linux, or Windows with WSL2

### 1. Deploy (Single Command)

```bash
# Clone repository
git clone <repository-url>
cd sutra-models

# Complete installation (builds + starts all services)
./sutra-deploy.sh install
```

### 2. Alternative: Manual Steps

```bash
# Build images only
./sutra-deploy.sh build

# Start services
./sutra-deploy.sh up

# Check status
./sutra-deploy.sh status

# Complete reset
./sutra-deploy.sh clean
```

### 3. Access Services

```bash
open http://localhost:9000    # Control Center (monitoring)
open http://localhost:8080    # Interactive Client (queries)
open http://localhost:8000    # REST API documentation
```

### 4. Try It Out

**Example: Build a Hospital Protocol Knowledge Base**

```bash
# Step 1: Feed your domain knowledge
curl -X POST http://localhost:8001/sutra/learn \
  -H "Content-Type: application/json" \
  -d '{"text": "Hospital Protocol 247: For pediatric sepsis, first-line treatment is ceftriaxone 50mg/kg IV every 12 hours"}'

curl -X POST http://localhost:8001/sutra/learn \
  -H "Content-Type: application/json" \
  -d '{"text": "Patient Case 1823: 8-year-old with sepsis responded well to ceftriaxone, full recovery in 72 hours"}'

curl -X POST http://localhost:8001/sutra/learn \
  -H "Content-Type: application/json" \
  -d '{"text": "Drug Safety Database: Ceftriaxone has no known interactions with acetaminophen or ibuprofen"}'
```

**Step 2: Query with Explainable Reasoning:**
```bash
curl -X POST http://localhost:8001/sutra/query \
  -H "Content-Type: application/json" \
  -d '{"query": "What is the recommended treatment for pediatric sepsis?", "max_paths": 5}'

# Returns reasoning paths through YOUR hospital's protocols:
# Path 1: Protocol 247 → ceftriaxone dosing (confidence: 0.92)
# Path 2: Similar case 1823 → successful outcome (confidence: 0.85)
# Path 3: Drug safety check → no contraindications (confidence: 0.88)
```

**Step 3: Stream Progressive Responses with Quality Gates:**
```bash
curl -X POST http://localhost:8001/sutra/stream/query \
  -H "Content-Type: application/json" \
  -d '{"query": "Is ceftriaxone safe with acetaminophen?", "enable_quality_gates": true}'

# If confidence is low, system returns: "I don't know - insufficient data"
# If confidence is high, provides reasoning path with audit trail
```

**Step 4: Advanced Semantic Queries:**
```bash
# Query with semantic filters
curl -X POST http://localhost:8000/api/semantic/query \
  -H "Content-Type: application/json" \
  -d '{
    "query": "pediatric sepsis treatment",
    "filters": {
      "semantic_type": "clinical_protocol",
      "domain": "pediatrics",
      "min_confidence": 0.8
    }
  }'

# Find temporal chains (events over time)
curl -X POST http://localhost:8000/api/semantic/temporal-chain \
  -H "Content-Type: application/json" \
  -d '{
    "start_concept": "sepsis_diagnosis",
    "end_concept": "patient_recovery",
    "time_range": {"start": "2024-01-01", "end": "2024-12-31"}
  }'

# Detect contradictions in knowledge
curl -X POST http://localhost:8000/api/semantic/contradictions \
  -H "Content-Type: application/json" \
  -d '{"domain": "drug_interactions", "min_confidence": 0.75}'
```

**[Complete Quick Start Guide →](docs/guides/QUICK_START.md)**

---

## Performance

### Production Benchmarks (Verified)

| Operation | Performance | Details |
|-----------|-------------|---------|
| **Learning** | 57,412 concepts/sec | 0.02ms per concept |
| **Query** | <0.01ms | Zero-copy mmap reads |
| **Path Finding** | ~1ms | 3-hop BFS traversal |
| **Vector Search** | <50ms (P50) | HNSW with USearch |
| **Startup** | 3.5ms | 1M vectors from disk (94× faster) |
| **Memory** | ~0.1KB/concept | Excluding embeddings |

### Recent Optimizations (2025-10-24)

**P0.1: AI-Native Adaptive Reconciliation**
- 80% CPU reduction during idle
- 10× lower latency under load (1-5ms vs 10ms)
- Self-optimizing intervals (1-100ms dynamic range)
- Zero configuration required

**P1.5: HNSW Persistent Index (USearch)**
- 94× faster startup (3.5ms vs 5.5min for 1M vectors)
- 24% smaller index files
- SIMD-optimized search
- True mmap persistence (no rebuild)

**P1.2: Parallel Pathfinding**
- 4-8× speedup on multi-path queries
- Rayon work-stealing parallelization
- Optimal for MPPA consensus reasoning

**[Detailed Benchmarks →](docs/performance/BENCHMARKS.md)**

---

## Storage at Scale

### Capacity & Configuration

| Concept Count | Mode | Shards | Use Case |
|--------------|------|--------|----------|
| < 100K | Single | 1 | Development |
| 1M - 5M | Sharded | 4 | Production |
| 5M - 10M | Sharded | 8 | High-scale |
| 10M+ | Sharded | 16 | Enterprise |

### Features

✅ **Cross-Shard 2PC Transactions** - Zero data loss  
✅ **Write-Ahead Log (WAL)** - Automatic crash recovery  
✅ **Parallel Vector Search** - All shards queried simultaneously  
✅ **DoS Protection** - Input validation prevents abuse  
✅ **Memory Safety** - No integer overflow at scale  

**[Storage Guide →](docs/storage/STORAGE_GUIDE.md)**

---

## Documentation

### Essential Reading

- **[Quick Start Guide](docs/guides/QUICK_START.md)** - Get running in 5 minutes
- **[Production Deployment](docs/guides/PRODUCTION_DEPLOYMENT.md)** - Complete production setup
- **[API Reference](docs/api/API_REFERENCE.md)** - All endpoints documented
- **[Architecture Overview](WARP.md)** - System design and patterns
- **[Troubleshooting](TROUBLESHOOTING.md)** - Common issues and fixes

### Component Documentation

- **[sutra-storage](packages/sutra-storage/README.md)** - Rust storage engine (694 lines)
- **[sutra-core](packages/sutra-core/README.md)** - Reasoning engine (318 lines)
- **[sutra-hybrid](packages/sutra-hybrid/README.md)** - Semantic orchestration (109 lines)
- **[sutra-api](packages/sutra-api/README.md)** - REST API (89 lines)
- **All 16 packages documented** - See `packages/*/README.md`

**[Complete Documentation Index →](docs/INDEX.md)**

---

## Project Structure

```
sutra-models/
├── packages/
│   ├── sutra-storage/          # Rust storage (57K writes/sec)
│   ├── sutra-core/            # Graph reasoning engine
│   ├── sutra-hybrid/          # Semantic embeddings + NLG
│   ├── sutra-api/             # REST API (FastAPI)
│   ├── sutra-control/         # React control center
│   ├── sutra-client/          # Streamlit UI
│   ├── sutra-explorer/        # Storage visualization tool
│   ├── sutra-bulk-ingester/   # High-performance ingestion
│   ├── sutra-embedding-service/ # Embedding service HA
│   ├── sutra-grid-master/     # Grid orchestration
│   ├── sutra-grid-agent/      # Node management
│   └── ... (16 packages total)
├── docs/                      # Complete documentation
├── scripts/                   # Testing & validation scripts
├── sutra-deploy.sh           # Single deployment command center
├── QUICKSTART.md             # 2-command quick start
├── DEPLOYMENT.md             # Complete deployment guide
└── README.md                 # This file
```

---

## Development

### Setup

```bash
# Create virtual environment
python3 -m venv venv
source venv/bin/activate

# Install dependencies
pip install -r requirements-dev.txt

# Build Rust components
cd packages/sutra-storage
cargo build --release
```

### Testing

```bash
# Start services
./sutra-deploy.sh up

# Run tests
PYTHONPATH=packages/sutra-core python -m pytest tests/ -v

# Rust tests (includes WAL crash recovery)
cd packages/sutra-storage
cargo test

# Production smoke test
./scripts/smoke-test-embeddings.sh
```

### Code Quality

```bash
make format  # black + isort
make lint    # flake8
make check   # format + lint + test
```

**[Development Guide →](docs/guides/DEVELOPMENT.md)**

---

## Use Cases

### Domain-Specific Use Cases

**Healthcare: Compliance & Clinical Decision Support**
```
YOUR DATA: Treatment protocols, safety guidelines, case histories
USE CASE: "Is this treatment appropriate for this patient?"
OUTPUT: Reasoning paths through YOUR protocols with audit trail
VALUE: FDA compliance, malpractice protection, quality assurance
```

**Finance: Regulatory Compliance & Risk**
```
YOUR DATA: Risk models, regulatory rules, historical decisions
USE CASE: "Should we approve this credit application?"
OUTPUT: Decision path through YOUR risk framework
VALUE: SEC/FINRA compliance, audit defense, consistent policy
```

**Legal: Precedent Analysis & Case Strategy**
```
YOUR DATA: Firm's case database, jurisdiction-specific precedents
USE CASE: "What's the likely outcome for this contract dispute?"
OUTPUT: Similar cases from YOUR database with outcomes
VALUE: Client explanations, court arguments, billable transparency
```

**Manufacturing: Quality Control & Procedures**
```
YOUR DATA: Quality standards, inspection procedures, defect patterns
USE CASE: "Should this batch pass inspection?"
OUTPUT: Decision path through YOUR standards with evidence
VALUE: ISO compliance, defect reduction, audit trails
```

### Why Domain-Specific Beats General AI

✅ **More Accurate** - 100% of knowledge is YOUR domain (not 0.0001% of general model)  
✅ **Fully Explainable** - Complete reasoning paths for compliance  
✅ **Real-Time Updates** - New policy? Update graph instantly  
✅ **Privacy Preserved** - Your data never leaves your infrastructure  
✅ **Cost Effective** - No per-query API fees ($100K-$1M savings)  
✅ **Smaller Models** - 500MB vs 100GB+ (runs on normal servers)

---

## What This Is NOT

- ❌ **Not a general-purpose world model** - Doesn't know "Who won the 1996 Olympics?"
- ❌ **Not pre-trained on internet data** - Starts empty, learns YOUR domain
- ❌ **Not trying to replace ChatGPT** - Different problem: explainable domain reasoning
- ❌ **Not for creative writing** - Built for compliance, not content generation
- ❌ **Not a Wikipedia alternative** - For private enterprise knowledge, not public facts

## What This IS

✅ **Domain-specific reasoning infrastructure** for your proprietary knowledge  
✅ **Explainable AI** for regulated industries requiring audit trails  
✅ **Cost-effective alternative** to frontier LLM APIs ($0.0001 vs $0.01-$0.10 per query)  
✅ **Privacy-preserving** self-hosted system (no external API calls)  
✅ **Real-time learning** system that updates instantly without retraining

---

## Contributing

We welcome contributions aligned with explainable, accountable AI.

**Before contributing:**
1. Read [WARP.md](WARP.md) for architecture overview
2. Check [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines
3. Run tests to verify changes
4. Follow code style (black + isort)

**Areas needing help:**
- Additional NLP languages beyond English
- Performance optimizations
- Documentation improvements
- Test coverage expansion

---

## Research Foundation

Built on published research:

- **Adaptive Focus Learning** - "LLM-Oriented Token-Adaptive Knowledge Distillation" (Oct 2024)
- **Multi-Path Plan Aggregation (MPPA)** - Consensus-based reasoning
- **Graph-Based Reasoning** - Decades of knowledge representation research

No proprietary techniques - all methods from published work.

---

## Status & Roadmap

### Current Status (v2.0.0)

✅ **Production-Ready** - All P0 features complete  
✅ **Storage Grade: A+ (95/100)** - Enterprise durability  
✅ **107 Tests Passing** - Comprehensive test coverage  
✅ **Complete Documentation** - 3,500+ lines, 100% package coverage  

### Roadmap

**Q1 2025**
- [ ] Multi-modal support (text + structured data + tables)
- [ ] Domain-specific template libraries (healthcare, finance, legal)
- [ ] Advanced graph visualization tools
- [ ] Real-time performance monitoring dashboard

**Q2 2025**
- [ ] Distributed reasoning across multiple data centers
- [ ] Bulk knowledge import from enterprise systems (ERP, CRM, databases)
- [ ] Additional language support (Spanish, German, French)
- [ ] Enhanced NLG for regulatory reporting

**Long-term Vision**
- [ ] Provably correct reasoning for critical compliance decisions
- [ ] Formal verification of reasoning paths for safety-critical domains
- [ ] Industry-specific certification (FDA, SEC, ISO) readiness
- [ ] Zero-trust explainable AI with cryptographic audit trails

---

## License

MIT License - See [LICENSE](LICENSE) file

---

## Support

**Documentation:** [docs/INDEX.md](docs/INDEX.md)  
**Issues:** [GitHub Issues](https://github.com/yourusername/sutra-models/issues)  
**Discussions:** [GitHub Discussions](https://github.com/yourusername/sutra-models/discussions)

---

## Acknowledgments

Built with:
- **Rust** - Storage engine and grid infrastructure
- **Python** - Reasoning engine and API layer
- **React** - Control center UI
- **FastAPI** - REST API framework
- **Docker** - Containerization and deployment

---

**Status:** Production-Ready  
**Version:** 2.0.0  
**Last Updated:** 2025-10-24  
**Built with ❤️ by the Sutra AI Team**

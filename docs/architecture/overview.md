# Sutra AI Architecture Overview

**Version 2.0.0** | Domain-Specific Explainable AI for Regulated Industries

## Quick Links

- **[System Architecture](./SYSTEM_ARCHITECTURE.md)** - Complete system design, all layers
- **[Storage Engine](./STORAGE_ENGINE.md)** - Deep dive into Rust storage internals
- **[Technical Analysis](./TECHNICAL_ANALYSIS.md)** - Performance benchmarks and analysis
- **[Scalability](./SCALABILITY.md)** - Sharding, distribution, and scale patterns
- **[Enterprise Edition](./enterprise.md)** - Grid infrastructure and HA

---

## Core Architecture Principles

### 1. Explainability First
Every answer includes complete reasoning paths with confidence scores. No black-box predictions—full audit trails for compliance in medical, legal, and financial domains.

### 2. Real-Time Learning Without Retraining
Add new knowledge without downtime. The system learns incrementally through its unified learning pipeline, with all data written to Write-Ahead Log before memory updates.

### 3. Burst-Tolerant Performance
- **57K writes/sec**: Lock-free append-only WriteLog
- **<0.01ms reads**: Immutable snapshots via Arc<GraphSnapshot>
- **Zero data loss**: WAL with fsync() guarantees (RPO=0)

### 4. Production-Ready
- TLS 1.3 encryption (secure mode)
- JWT authentication with RBAC
- HA embedding services (3 replicas + HAProxy)
- 2PC transactions for cross-shard atomicity
- Self-monitoring via Grid Events

---

## Architecture at a Glance

```
┌─────────────────────────────────────────────────────────┐
│  Layer 5: User Interface (React, Streamlit)            │
└───────────────────────┬─────────────────────────────────┘
┌───────────────────────▼─────────────────────────────────┐
│  Layer 4: API Gateway (FastAPI, Rate Limiting, Auth)   │
└───────────────────────┬─────────────────────────────────┘
┌───────────────────────▼─────────────────────────────────┐
│  Layer 3: Reasoning (Python - PathFinder, MPPA)        │
└───────────────────────┬─────────────────────────────────┘
┌───────────────────────▼─────────────────────────────────┐
│  Layer 2: Protocol (TCP Binary - MessagePack)          │
│  10-50× faster than gRPC                                │
└───────────────────────┬─────────────────────────────────┘
┌───────────────────────▼─────────────────────────────────┐
│  Layer 1: Storage Engine (Rust)                        │
│  • ConcurrentMemory (3-plane architecture)              │
│  • Write-Ahead Log (zero data loss)                     │
│  • HNSW Container (USearch, <50ms startup)              │
│  • Unified Learning Pipeline (server-owned)             │
└─────────────────────────────────────────────────────────┘
```

---

## Key Components

### Storage Server (Rust)
- **ConcurrentMemory**: Lock-free writes, immutable reads, adaptive reconciliation
- **Write-Ahead Log**: fsync() durability, transaction support, crash recovery
- **HNSW Container**: USearch-based vector search with mmap persistence
- **Unified Learning Pipeline**: Embedding + semantic analysis + association extraction
- **TCP Binary Protocol**: 10-50× faster than gRPC, MessagePack serialization

### Reasoning Engine (Python)
- **PathFinder**: Best-first search, BFS, confidence propagation
- **MPPA**: Multi-Path Plan Aggregation with voting
- **QueryProcessor**: Natural language understanding
- **Quality Gates**: Confidence thresholds ("I don't know" when uncertain)

### Service Layer
- **sutra-api** (Port 8000): Primary REST API
- **sutra-hybrid** (Port 8001): Semantic embeddings + NLG
- **sutra-client** (Port 8080): React conversation-first UI
- **sutra-control** (Port 9000): Monitoring dashboard
- **embedding-ha** (Port 8888): 3 replicas + HAProxy (Enterprise)
- **nlg-ha** (Port 8889): 3 replicas + HAProxy (Enterprise)

---

## Data Flow

### Learning Flow
```
Client → API → TCP Binary Protocol → Storage Server
                                      ├─ Embedding Service (HTTP)
                                      ├─ Semantic Analyzer
                                      ├─ Association Extractor
                                      └─ ConcurrentMemory
                                         ├─ WAL → fsync()
                                         ├─ WriteLog (lock-free)
                                         └─ HNSW (incremental)
```

### Reasoning Flow
```
Query → API → ReasoningEngine
              ├─ QueryProcessor → Vector Search (embedding)
              ├─ PathFinder → Graph Traversal (BFS/Best-First)
              └─ MPPA → Consensus Voting
                 └─ Quality Gates → Response (or "I don't know")
```

---

## Performance Characteristics

| Metric | Value | Notes |
|--------|-------|-------|
| **Write Throughput** | 57,412 ops/sec | Lock-free append |
| **Read Latency** | <0.01ms | Zero-copy via Arc |
| **Startup Time** | <50ms | USearch mmap load |
| **Vector Search** | 0.8ms (k=10) | HNSW O(log N) |
| **Path Finding** | 1.2ms (5 hops) | BFS traversal |
| **Crash Recovery** | <1 second | WAL replay |
| **Max Scale** | 10M+ concepts | Sharded (Enterprise) |

---

## Edition Comparison

| Feature | Simple (FREE) | Community ($99/mo) | Enterprise ($999/mo) |
|---------|---------------|-------------------|---------------------|
| **Containers** | 7 | 7 | 16 |
| **Max Concepts** | 100K | 1M | 10M (sharded) |
| **Learn Rate** | 10/min | 100/min | 1000/min |
| **Reason Rate** | 50/min | 500/min | 5000/min |
| **HA Services** | ❌ | ❌ | ✅ (3 replicas) |
| **Grid** | ❌ | ❌ | ✅ |
| **Security** | ❌ | Optional | ✅ Enforced |
| **Sharding** | ❌ | ❌ | ✅ (4-16 shards) |

---

## Technology Stack

| Layer | Technology | Why |
|-------|-----------|-----|
| **Storage** | Rust + Tokio | Memory safety, zero-cost abstractions |
| **Protocol** | MessagePack + TCP | 10-50× faster than gRPC |
| **Reasoning** | Python 3.11+ | Rich ML/NLP ecosystem |
| **API** | FastAPI | Async-first, auto validation |
| **Embedding** | nomic-embed-text-v1.5 | 768-d, state-of-art |
| **Vector Search** | USearch (HNSW) | True persistence, 100× faster |
| **UI** | React 18 + Material Design 3 | Modern, accessible |
| **Orchestration** | Docker Compose + HAProxy | Simple, reliable |

---

## Deployment

### Single Command Install
```bash
./sutra-deploy.sh install
```

### Edition Selection
```bash
# Simple (FREE)
SUTRA_EDITION=simple ./sutra-deploy.sh install

# Community ($99/mo)
SUTRA_EDITION=community SUTRA_LICENSE_KEY=xxx ./sutra-deploy.sh install

# Enterprise ($999/mo)
SUTRA_EDITION=enterprise SUTRA_LICENSE_KEY=xxx SUTRA_SECURE_MODE=true ./sutra-deploy.sh install
```

### Version Management
```bash
# Show version
./sutra-deploy.sh version

# Create release
./sutra-deploy.sh release patch  # 2.0.0 → 2.0.1
./sutra-deploy.sh release minor  # 2.0.0 → 2.1.0
./sutra-deploy.sh release major  # 2.0.0 → 3.0.0

# Deploy version
./sutra-deploy.sh deploy v2.0.1
```

---

## Documentation Structure

```
docs/architecture/
├── overview.md (this file)           - Quick reference
├── SYSTEM_ARCHITECTURE.md            - Complete system design
├── STORAGE_ENGINE.md                 - Rust internals deep dive
├── TECHNICAL_ANALYSIS.md             - Benchmarks and analysis
├── SCALABILITY.md                    - Sharding and distribution
├── enterprise.md                     - Grid and HA setup
└── DEEP_DIVE.md                      - Legacy (archived)
```

---

## Next Steps

1. **For Architects**: Read [SYSTEM_ARCHITECTURE.md](./SYSTEM_ARCHITECTURE.md)
2. **For Storage Engineers**: Read [STORAGE_ENGINE.md](./STORAGE_ENGINE.md)
3. **For DevOps**: Read [SCALABILITY.md](./SCALABILITY.md)
4. **For Deployment**: See `QUICKSTART.md` in repo root

---

*Last Updated: October 27, 2025 | Version 2.0.0*

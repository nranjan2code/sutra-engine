# Sutra AI - System Architecture

**An explainable AI system that learns in real-time without retraining**

Version: 2.0.0 | Status: Production-ready | Last Updated: 2025-10-17

---

## Executive Summary

Sutra AI is a **graph-based reasoning system** with complete explainability. Unlike black-box LLMs, every decision includes the full reasoning path showing how the system arrived at its answer.

**Core Innovation:** Temporal knowledge graphs + semantic embeddings + multi-path reasoning = Explainable AI that learns continuously without retraining.

**Performance:** 57,412 writes/sec, <0.01ms reads, 100% accuracy verified (25,000× faster than previous JSON-based storage).

---

## System Architecture

### High-Level Stack

```
┌──────────────────────────────────────────────────────┐
│                   sutra-api (FastAPI)                 │  External Interface
│              REST API • Rate Limiting • Auth          │
└────────────────────┬─────────────────────────────────┘
                     │
┌────────────────────┴─────────────────────────────────┐
│                 sutra-hybrid (Python)                 │  Semantic Layer
│       Embeddings • Multi-Strategy • Audit Trails      │
└────────────────────┬─────────────────────────────────┘
                     │
┌────────────────────┴─────────────────────────────────┐
│                  sutra-core (Python)                  │  Reasoning Engine
│  Concepts • Associations • PathFinder • MPPA          │
└────────────────────┬─────────────────────────────────┘
                     │
┌────────────────────┴─────────────────────────────────┐
│              sutra-storage (Rust/PyO3)                │  Storage Engine
│  ConcurrentStorage • Memory-Mapped • Lock-Free        │
│       57K writes/sec • <0.01ms reads                  │
└──────────────────────────────────────────────────────┘
```

**Design Principle:** Only `sutra-api` is external-facing. Core, hybrid, and storage are internal implementation details.

---

## Package Structure

### 1. **sutra-storage** (Rust) — Production-Ready Storage Engine

**Purpose:** High-performance, burst-tolerant storage for temporal knowledge graphs.

**Key Features:**
- **57,412 writes/sec** (25,000× faster than JSON baseline)
- **<0.01ms read latency** (zero-copy memory-mapped files)
- Lock-free write log with background reconciliation
- Single-file architecture (`storage.dat`, 512MB initial size)
- Immutable read snapshots (readers never block writers)
- BFS path finding and graph traversal
- 100% test pass rate with verified accuracy

**Innovation:** Dual-plane architecture — writers append to lock-free log, readers access immutable snapshots. Reconciler runs asynchronously every 10ms.

**Documentation:**
- [`packages/sutra-storage/ARCHITECTURE.md`](packages/sutra-storage/ARCHITECTURE.md) — Detailed design
- [`docs/sutra-storage/README.md`](docs/sutra-storage/README.md) — Production guide, benchmarks, API reference

---

### 2. **sutra-core** (Python) — Graph Reasoning Engine

**Purpose:** Core AI reasoning using graph traversal and multi-path consensus.

**Key Components:**
- **ReasoningEngine**: Orchestrates learning, querying, and caching
- **PathFinder**: Multi-strategy graph traversal (best-first, BFS, bidirectional)
- **MultiPathAggregator (MPPA)**: Consensus-based reasoning to prevent single-path errors
- **AssociationExtractor**: Extracts typed relationships (semantic, causal, temporal, hierarchical)

**Reasoning Strategies:**
- Confidence decay (0.85 per hop) for realistic path scoring
- Cycle detection and path diversification
- Path clustering and majority voting
- Robustness analysis with diversity bonus

**Documentation:**
- [`docs/packages/sutra-core.md`](docs/packages/sutra-core.md) — Component guide

---

### 3. **sutra-hybrid** (Python) — Semantic Embeddings Layer

**Purpose:** Combines graph reasoning with optional semantic similarity matching.

**Key Features:**
- Optional semantic embeddings (graph reasoning works standalone)
- Multi-strategy comparison (graph-only vs semantic-enhanced)
- Agreement scoring between strategies
- Full audit trails for compliance

**SutraAI Class:**
- High-level interface for learning and querying
- Knowledge persistence via `save()` and `load()`
- Configurable strategy selection

**Documentation:**
- [`docs/packages/sutra-hybrid.md`](docs/packages/sutra-hybrid.md) — Integration guide

---

### 4. **sutra-api** (FastAPI) — Production REST API

**Purpose:** External HTTP interface with rate limiting and monitoring.

**Endpoints:**
- `POST /learn` — Add knowledge
- `POST /reason` — Query with reasoning paths
- `POST /save` — Persist to disk
- `GET /health` — System status
- `GET /stats` — Performance metrics

**Features:**
- Rate limiting (configurable per endpoint)
- Request validation
- OpenAPI documentation at `/docs`
- CORS support

**Documentation:**
- [`docs/packages/sutra-api.md`](docs/packages/sutra-api.md) — API reference

---

## Core Design Principles

### 1. **Explainability First**
Every decision includes complete reasoning paths. No "magic" — you can trace every step from question to answer.

### 2. **Separation of Concerns**
```
Write Plane:  Lock-free log (throughput optimized)
Read Plane:   Immutable snapshots (latency optimized)
Reconciler:   Async coordination (invisible to users)
```

### 3. **Zero-Copy Philosophy**
Memory-mapped files + direct pointer access = no serialization overhead for internal operations.

### 4. **Temporal Awareness**
Knowledge evolves over time. Storage is log-structured with timestamps for time-travel queries.

### 5. **Graph-Native**
Data structures optimized for graph traversal, not tables or documents. Adjacency lists, BFS, and confidence propagation are first-class operations.

---

## Data Flow

### Learning Flow
```
User Input
    ↓
Content + (optional) Embedding
    ↓
Association Extraction (typed relationships)
    ↓
ConcurrentStorage.learn_concept()
    ↓
Lock-free write log (append-only)
    ↓
Background reconciler (10ms loop)
    ↓
Immutable snapshot update
    ↓
Optional: Flush to storage.dat
```

### Query Flow
```
User Query
    ↓
Semantic Matching (optional embeddings)
    ↓
Concept Retrieval (<0.01ms, zero-copy)
    ↓
PathFinder (multi-strategy BFS)
    ↓
Multi-Path Plan Aggregation (MPPA)
    ↓
Consensus Answer + Confidence + Reasoning Path
    ↓
Audit Trail (logged with timestamp)
```

---

## Performance Characteristics

### Storage (Production Benchmarked)
| Operation       | Latency  | Throughput    | Notes                          |
|-----------------|----------|---------------|--------------------------------|
| Write (learn)   | 0.02ms   | **57,412/sec**| Lock-free log, batched         |
| Read (query)    | <0.01ms  | Millions/sec  | Zero-copy, immutable snapshot  |
| Path finding    | ~1ms     | —             | 3-hop BFS traversal            |
| Reconciliation  | 1-2ms    | 10K/batch     | Background, 10ms interval      |
| Disk flush      | ~100ms   | —             | Manual or auto at 50K concepts |

**Improvement:** 25,000× faster writes, 22,500× faster reads compared to old JSON-based storage.

### Memory
- **Concept**: ~0.1KB (excluding embeddings)
- **Embedding**: ~1.5KB (384-dim float32)
- **1M concepts**: ~2GB total (with embeddings)

### Scaling
- **Vertical**: Tested to 1M+ concepts on single node
- **Horizontal**: Shard by tenant at application layer
- **Storage**: Single `storage.dat` file (grows 2× when full)

---

## Technology Stack

### Storage Engine (Rust)
- **Memory mapping**: `memmap2` for zero-copy I/O
- **Concurrency**: `crossbeam`, `arc-swap` for lock-free structures
- **Python bindings**: `PyO3` for seamless integration
- **Serialization**: Custom binary format (minimal overhead)

### Reasoning Engine (Python)
- **Graph**: Native Python dictionaries + BFS algorithms
- **NLP**: spaCy for text processing (optional)
- **Embeddings**: sentence-transformers (optional)

### API Layer (Python)
- **Web framework**: FastAPI + uvicorn
- **Validation**: Pydantic models
- **Rate limiting**: slowapi

---

## Key Algorithms

### 1. **Multi-Path Plan Aggregation (MPPA)**
Consensus-based reasoning to prevent single-path derailment:
- Find multiple independent paths
- Cluster paths by answer similarity (0.8 threshold)
- Majority voting with diversity bonus
- Return answer + confidence + robustness metrics

### 2. **Confidence Decay**
Realistic confidence propagation through reasoning chains:
```
final_confidence = initial_confidence × (0.85 ^ path_length)
```

### 3. **Path Diversification**
- Cycle detection (visited node tracking)
- Path similarity threshold (0.7 max overlap)
- Alternative route exploration

### 4. **Concurrent Reconciliation**
- Writers: Append to lock-free queue (crossbeam channel)
- Readers: Access immutable snapshot (arc-swap)
- Reconciler: Batch process queue → update snapshot (10ms loop)

---

## What Works (Production Verified)

✅ **Learn new knowledge** — Add concepts and relationships  
✅ **Query with reasoning paths** — Get answers with explanations  
✅ **Save to disk** — Persist knowledge (single `storage.dat` file)  
✅ **Reload from disk** — Restore complete state after restart  
✅ **Multi-strategy reasoning** — Compare graph-only vs semantic-enhanced  
✅ **Audit trails** — Full compliance tracking  
✅ **REST API** — Production-ready HTTP interface  
✅ **Performance** — 57K writes/sec, <0.01ms reads, 100% accuracy  

---

## Current Limitations

1. **Limited reasoning depth** — Works well for 2-3 hops, gets expensive beyond 6 hops
2. **No natural language generation** — Returns concept content, not fluent text
3. **Requires structured input** — Works best with clear factual statements
4. **No common sense reasoning** — Only knows what you teach it
5. **English-only** — NLP components are English-centric
6. **Recovery on restart** — Data persists but auto-load not yet implemented (workaround: load manually)

---

## Quick Start

### Installation
```bash
# Clone and setup
git clone <repo>
cd sutra-models
python3 -m venv venv
source venv/bin/activate

# Install packages
pip install -e packages/sutra-core/
pip install -e packages/sutra-hybrid/
pip install -e packages/sutra-api/
```

### Demo
```bash
# End-to-end workflow
python demo_simple.py           # Basic learning and querying
python demo_end_to_end.py       # Complete workflow
python demo_mass_learning.py    # Performance testing

# Verify storage performance
python verify_concurrent_storage.py
```

### API Server
```bash
cd packages/sutra-api
python -m sutra_api.main
# API available at http://localhost:8000
# Docs at http://localhost:8000/docs
```

---

## Documentation Navigation

### Getting Started
- [`README.md`](README.md) — Project overview, goals, quick start
- [`WARP.md`](WARP.md) — Development guide, commands, configuration

### Architecture Deep Dives
- [`docs/architecture/overview.md`](docs/architecture/overview.md) — System architecture
- [`docs/architecture/enterprise.md`](docs/architecture/enterprise.md) — Deployment, scaling, security
- [`packages/sutra-storage/ARCHITECTURE.md`](packages/sutra-storage/ARCHITECTURE.md) — Storage engine design

### Package Documentation
- [`docs/packages/sutra-core.md`](docs/packages/sutra-core.md) — Reasoning engine
- [`docs/packages/sutra-hybrid.md`](docs/packages/sutra-hybrid.md) — Semantic embeddings
- [`docs/packages/sutra-storage.md`](docs/packages/sutra-storage.md) — Storage API

### Operations
- [`docs/sutra-storage/README.md`](docs/sutra-storage/README.md) — Production guide, benchmarks
- [`docs/sutra-storage/PRODUCTION_STATUS.md`](docs/sutra-storage/PRODUCTION_STATUS.md) — Test results, deployment recommendations
- [`docs/development/setup.md`](docs/development/setup.md) — Development environment
- [`docs/development/testing.md`](docs/development/testing.md) — Testing strategy

### Tutorials and Demos
- [`docs/demos.md`](docs/demos.md) — Demo scripts and examples
- [`docs/TUTORIAL.md`](docs/TUTORIAL.md) — Step-by-step guide

---

## Research Foundation

Built on published research (no proprietary techniques):
- **Adaptive Focus Learning**: "LLM-Oriented Token-Adaptive Knowledge Distillation" (Oct 2024)
- **Multi-Path Plan Aggregation (MPPA)**: Consensus-based reasoning
- **Graph-based reasoning**: Decades of knowledge representation research

---

## Design Trade-offs

### Why Graph-Based?
**Pro:** Inherently explainable — trace every path  
**Con:** Doesn't capture statistical patterns like LLMs

### Why Rust for Storage?
**Pro:** Zero-copy, lock-free, predictable performance  
**Con:** Steeper learning curve than Python-only

### Why Optional Embeddings?
**Pro:** Pure graph = 100% explainable; embeddings = enhanced recall  
**Con:** Some opacity when embeddings are used (but contribution is tracked)

### Why Single-File Storage?
**Pro:** Simple deployment, easy backup, OS-managed paging  
**Con:** File grows large (but memory-mapped, so only active data in RAM)

### Why REST API as Sole Interface?
**Pro:** Clean separation, versioning, polyglot clients  
**Con:** No low-latency in-process API (but Python bindings available internally)

---

## Status

**Version:** 2.0.0  
**Stability:** Production-ready for internal use  
**API:** Stable endpoints, subject to minor changes  
**Performance:** 57,412 writes/sec, <0.01ms reads (verified)  
**Test Coverage:** 100% pass rate on core components  
**Last Tested:** 2025-10-16  

---

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for development guidelines.

**Key Requirements:**
- Run `make test-core` before committing
- Use `make format` for consistent style
- Add tests for new features
- Update documentation for architectural changes

---

## License

MIT License — See [`LICENSE`](LICENSE) file.

---

## Contact

This is an active research project. Issues and pull requests welcome.

**Next Steps:**
1. Read [`docs/architecture/overview.md`](docs/architecture/overview.md) for detailed design
2. Try [`demo_simple.py`](demo_simple.py) to see the system in action
3. Explore [`docs/sutra-storage/README.md`](docs/sutra-storage/README.md) for storage internals
4. Review [`WARP.md`](WARP.md) for development commands

---

*Building explainable AI, one reasoning path at a time.*

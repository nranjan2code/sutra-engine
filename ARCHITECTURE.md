# Sutra AI - System Architecture

**Domain-Specific Reasoning Engine for Your Knowledge**

Explainable reasoning infrastructure that learns from YOUR proprietary data without frontier LLMs.

Version: 2.0.0 | Status: Production-ready | Last Updated: 2025-10-25

## 🚨 CRITICAL PRODUCTION REQUIREMENTS

### Embedding System (MANDATORY)

**⚠️ WARNING:** The system CANNOT function without proper embedding configuration. All production deployments MUST ensure:

1. **Ollama Service**: Must be accessible at `SUTRA_OLLAMA_URL` with `granite-embedding:30m` model loaded
2. **TCP Protocol**: ALL services MUST use `sutra-storage-client-tcp` package - NEVER direct storage access
3. **Message Format**: Unit variants (`GetStats`, `Flush`, `HealthCheck`) send string, not `{variant: {}}`
4. **Vector Serialization**: Always convert numpy arrays to Python lists before TCP transport
5. **Error Handling**: Implement retry logic for TCP connection failures

**Common Failure Modes:**
- "No embedding processor available" → Ollama not accessible or model not loaded
- "can not serialize 'numpy.ndarray' object" → Missing array-to-list conversion
- "wrong msgpack marker" → Incorrect message format for unit variants
- "Connection closed" → TCP client using wrong protocol

---

## Executive Summary

Sutra AI is a **domain-specific reasoning engine** that provides explainable answers over your proprietary knowledge. Unlike frontier LLMs trained on general internet data, Sutra starts empty and learns YOUR domain—hospital protocols, legal cases, financial regulations, manufacturing procedures.

**Core Innovation:** Small embedding models (500MB vs 100GB+ LLMs) + graph-based reasoning + multi-path consensus = Explainable AI with complete audit trails at 1000× lower cost.

**Performance:** 57,412 writes/sec, <0.01ms reads, ~$0.0001 per query (vs $0.01-$0.10 for LLM APIs).

**Target Users:** Regulated industries (healthcare, finance, legal, government) requiring explainable AI with audit trails.

---

## System Architecture

### High-Level (TCP Binary Protocol - Current Production Architecture)

```
┌───────────────────────────────────────────────────────────────────────┐
│                     Unified Learning Architecture                      │
│                        (Implemented 2025-10-19)                       │
├───────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  ANY Client (API/Hybrid/Bulk/Python):                               │
│    └─→ TcpStorageAdapter.learn_concept(content, options)            │
│        └─→ TCP: LearnConceptV2 {content, options}                   │
│            └─→ StorageServer::LearningPipeline:                     │
│                ├─→ 1. Generate embedding (Ollama HTTP)             │
│                ├─→ 2. Extract associations (Rust NLP)              │
│                ├─→ 3. Store atomically (HNSW + WAL)                │
│                └─→ 4. Return concept_id                             │
│                                                                       │
└───────────────────────────────────────────────────────────────────────┘

┌──────────────┐    TCP Binary     ┌─────────────────────────────────────┐
│  sutra-api   │ ──── Protocol ──▶ │      storage-server                 │
│  (FastAPI)   │ ◀────────────────  │   (Rust TCP + Learning Pipeline)   │
└──────────────┘                    │                                     │
        ▲                           │  🔥 NEW: Unified Learning Core:    │
        │                           │  ├─ Embedding Generation (Ollama)  │
        │ TCP Binary Protocol       │  ├─ Association Extraction (NLP)    │
        │                           │  ├─ Atomic Storage (HNSW + WAL)     │
┌──────────────┐                    │  └─ Port 50051                      │
│ sutra-hybrid │ ──────────────────▶│                                     │
│ (Semantic +  │ ◀──────────────────│                                     │
│  NLG Layer)  │                    └─────────────────────────────────────┘
└──────────────┘
        ▲
        │ HTTP
        ▼
┌──────────────┐
│ sutra-ollama │
│  (LLM Server)│
└──────────────┘
```

**Key Design Principles:**
1. **Domain-Specific**: Not pre-trained on internet data—learns from YOUR knowledge
2. **Single Source of Truth**: Storage server owns ALL learning logic (embeddings + associations)
3. **TCP Binary Protocol**: 10-50× lower latency than gRPC using bincode serialization
4. **Small Models**: 500MB embedding model vs 100GB+ frontier LLMs
5. **Complete Explainability**: Full reasoning paths for compliance and audit
6. **Unified Learning Pipeline**: No code duplication - all services delegate to storage server
7. **Atomic Operations**: Complete learning pipeline executes atomically in storage server

---

## Package Structure

### 1. **sutra-storage** (Rust) — Production-Ready Storage Engine

**Purpose:** High-performance, burst-tolerant storage for temporal knowledge graphs.

**Key Features:**
- **57,412 writes/sec** (25,000× faster than JSON baseline)
- **<0.01ms read latency** (zero-copy memory-mapped files)
- **🔥 AI-Native Adaptive Reconciliation** (NEW 2025-10-24) - Self-optimizing with 80% CPU savings
- Lock-free write log with background reconciliation (1-100ms dynamic intervals)
- Single-file architecture (`storage.dat`, 512MB initial size)
- Immutable read snapshots (readers never block writers)
- BFS path finding and graph traversal
- Comprehensive telemetry with predictive health scoring
- 100% test pass rate with verified accuracy

**Innovation:** 
- **Dual-plane architecture**: Writers append to lock-free log, readers access immutable snapshots
- **AI-Native Adaptive Reconciler**: Self-optimizes intervals (1-100ms) based on load with EMA-based prediction
- **USearch HNSW**: True mmap persistence with 94× faster startup (migrated 2025-10-24)

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
Reconciler:   AI-Native adaptive coordination (1-100ms self-optimizing, invisible to users)
```

### 3. **Zero-Copy Philosophy**
Memory-mapped files + direct pointer access = no serialization overhead for internal operations.

### 4. **Temporal Awareness**
Knowledge evolves over time. Storage is log-structured with timestamps for time-travel queries.

### 5. **Graph-Native**
Data structures optimized for graph traversal, not tables or documents. Adjacency lists, BFS, and confidence propagation are first-class operations.

---

## Data Flow

### Unified Learning Flow (2025-10-19)

```
User Input (Content) via ANY Client (API/Hybrid/Bulk/Python)
    ↓
TCP Storage Client (sutra-storage-client-tcp)
    ├─ TcpStorageAdapter.learn_concept(content, options)
    ├─ Convert numpy arrays → Python lists  
    ├─ TCP Message: LearnConceptV2 {content, options}
    └─ ⚠️ CRITICAL: ALL clients use unified TCP protocol
    ↓
Storage Server Learning Pipeline (Single Source of Truth)
    ├─ 🔴 STEP 1: Embedding Generation
    │   ├─ HTTP request → Ollama (granite-embedding:30m, 768 dims)
    │   ├─ ⚠️ FAILS if Ollama not accessible → "No embedding processor available"
    │   └─ Embedding stored with concept
    ├─ 🔴 STEP 2: Association Extraction  
    │   ├─ Rust-based NLP pattern matching
    │   ├─ Typed relationships (semantic, causal, temporal, hierarchical)
    │   └─ Confidence scoring and filtering
    ├─ 🔴 STEP 3: Atomic Storage
    │   ├─ Lock-free write log (append-only, 57K writes/sec)
    │   ├─ HNSW vector indexing for semantic search
    │   ├─ **AI-Native Adaptive Reconciler** (1-100ms dynamic intervals, EMA prediction)
    │   ├─ WAL durability (zero data loss)
    │   └─ Immutable snapshot update
    └─ 🔴 STEP 4: Return concept_id
        └─ Client receives concept_id for further operations

✅ Benefits:
- Single implementation for ALL services
- Automatic embeddings for every concept
- Automatic associations for graph building
- Atomic operations with ACID guarantees
- No "same answer" bug (embeddings always generated)
```

### Query Flow
```
User Query
    ↓
🔴 CRITICAL: Query Embedding Generation
    ├─ OllamaNLPProcessor.get_embedding(query)
    ├─ granite-embedding:30m model (768 dimensions)
    └─ ⚠️ FAILS if no embedding processor → "No embedding processor available"
    ↓
TCP Storage Client - Vector Search
    ├─ Convert numpy query vector → Python list
    ├─ StorageClient.vector_search(query_vector, k=10)
    ├─ Parse response: [[['concept_id', score]]] → [(id, score)]
    └─ ⚠️ FAILS if wrong response parsing
    ↓
Concept Retrieval via TCP
    ├─ StorageClient.query_concept(concept_id)
    ├─ Parse response: [found, id, content, strength, confidence]
    └─ ⚠️ FAILS if expecting dict format
    ↓
PathFinder (multi-strategy graph traversal)
    ↓
Multi-Path Plan Aggregation (MPPA)
    ↓
Consensus Answer + Confidence + Reasoning Paths
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

### 4. **AI-Native Adaptive Reconciliation** 🔥 NEW (2025-10-24)

Self-optimizing reconciliation using online machine learning:

**Architecture:**
- Writers: Append to lock-free queue (crossbeam bounded channel, 100K capacity)
- Readers: Access immutable snapshot (arc-swap, zero-copy)
- **Adaptive Reconciler**: AI-native coordinator with dynamic intervals (1-100ms)

**Intelligence Layer:**
```rust
TrendAnalyzer {
    queue_ema: f64,        // Exponential Moving Average (α=0.3)
    rate_ema: f64,         // Processing rate tracking
    predicted_depth: f64,  // Linear extrapolation
}

calculate_optimal_interval(queue_utilization) -> Duration {
    match utilization {
        0.0..0.20 => 100ms,  // Idle: Save 80% CPU
        0.20..0.70 => 10ms,  // Normal: Original behavior
        0.70..1.00 => 1-5ms, // High load: Aggressive drain (10× faster)
    }
}
```

**Predictive Health Scoring:**
- Real-time queue monitoring with trend analysis
- Health score: 0.0-1.0 (Good → Warning → Critical)
- Predictive alerts at 70% capacity (before issues occur)
- Comprehensive telemetry via Grid events (self-monitoring)

**Performance Impact:**
- **80% CPU reduction** during idle periods (100ms intervals vs 10ms)
- **10× lower latency** during bursts (1-5ms aggressive drain)
- **Zero tuning required** - self-optimizing with defaults

**API:**
```rust
pub struct AdaptiveReconcilerStats {
    pub queue_depth: usize,
    pub queue_utilization: f64,       // 0.0-1.0
    pub predicted_queue_depth: usize, // Trend-based
    pub current_interval_ms: u64,     // Dynamic
    pub health_score: f64,            // 0.0-1.0
    pub recommendation: String,       // "Good" | "Warning" | "Critical"
    pub processing_rate_per_sec: f64,
    pub estimated_lag_ms: u64,
    // ... 10+ metrics
}
```

**See:** [ADAPTIVE_RECONCILIATION_ARCHITECTURE.md](docs/storage/ADAPTIVE_RECONCILIATION_ARCHITECTURE.md) for complete technical details.

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

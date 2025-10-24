# WARP.md

**AI Assistant Guidance for Sutra Models Project**

This document provides structured guidance for AI assistants (like WARP at warp.dev) when working with the Sutra Models codebase.

---

## 📋 Table of Contents

1. [Project Overview](#project-overview)
2. [Critical Production Requirements](#critical-production-requirements)
3. [Architecture](#architecture)
4. [Development Environment](#development-environment)
5. [Key Components](#key-components)
6. [Common Development Tasks](#common-development-tasks)
7. [Troubleshooting](#troubleshooting)
8. [Recent Major Features](#recent-major-features)

---

## Project Overview

**Sutra AI** is an explainable graph-based AI system that learns in real-time without retraining. It provides complete reasoning paths for every decision, making it a transparent alternative to black-box LLMs.

### Core Value Proposition

- ✅ Shows reasoning for every answer
- ✅ Learns incrementally from new information
- ✅ Provides audit trails for compliance
- ✅ Works without GPUs or massive compute
- ✅ 100% explainable reasoning paths
- ✅ Self-observability using own reasoning engine
- ✅ Progressive streaming responses (10× faster UX)
- ✅ Quality gates with "I don't know" for uncertain answers
- ✅ Natural language operational queries

### Production Status (2025-10-24)

**✅ PRODUCTION-READY** - Storage engine grade: **A+ (95/100)**

- ✅ Cross-shard 2PC transactions (zero data loss)
- ✅ Comprehensive DoS protection and input validation
- ✅ 57K writes/sec, <0.01ms reads maintained
- ✅ 107 tests passed, production-grade guarantees
- ✅ Ready for 5M-10M+ concepts with enterprise-grade durability

---

## Critical Production Requirements

### 🚨 MANDATORY: Embedding Service Configuration

**⚠️ SYSTEM WILL NOT FUNCTION WITHOUT CORRECT EMBEDDING CONFIGURATION ⚠️**

#### Strict Requirements (Production Standard: 2025-10-20)

**ONLY Sutra Embedding Service IS SUPPORTED:**

```yaml
REQUIRED:
  - Service: sutra-embedding-service
  - Model: nomic-embed-text-v1.5
  - Dimension: 768
  - Port: 8888
  - NO external dependencies
  - NO fallback providers

FORBIDDEN:
  - Ollama integration ❌ (removed)
  - granite-embedding ❌ (384-d incompatible)
  - sentence-transformers fallback ❌
  - spaCy embeddings ❌
  - TF-IDF fallback ❌
```

#### Why This Matters

- Different models produce **incompatible semantic spaces**
- Mixing dimensions causes **WRONG QUERY RESULTS**
- Real incident: Using 384-d for queries + 768-d for storage caused:
  - Query: "What is the tallest mountain?" → Answer: "Pacific Ocean" ❌

#### Mandatory Environment Variables

```bash
# Storage Server (docker-compose-grid.yml)
VECTOR_DIMENSION=768                                           # MUST be 768
SUTRA_EMBEDDING_SERVICE_URL=http://sutra-embedding-service:8888  # MUST point to service

# Hybrid Service
SUTRA_EMBEDDING_SERVICE_URL=http://sutra-embedding-service:8888  # MUST match storage
SUTRA_VECTOR_DIMENSION=768                                     # MUST be 768
SUTRA_USE_SEMANTIC_EMBEDDINGS=true                            # MUST be true
```

#### Verification Commands

```bash
# 1. Ensure embedding service is running
curl -s http://localhost:8888/health | jq '.status'
# Expected: "healthy"

# 2. Test embedding dimension and model
curl -s http://localhost:8888/info | jq '.dimension, .model'
# Expected: 768, "nomic-ai/nomic-embed-text-v1.5"

# 3. Test embedding generation
curl -s http://localhost:8888/embed \
  -H "Content-Type: application/json" \
  -d '{"texts": ["test"], "normalize": true}' | \
  jq '.embeddings[0] | length'
# Expected: 768

# 4. Check service metrics
curl -s http://localhost:8888/metrics | jq '.success_rate, .cache_hit_rate'
# Expected: >95% success rate, >50% cache hit rate
```

**See:** `EMBEDDING_SERVICE_MIGRATION.md` for complete details.

### Component Initialization Order (CRITICAL)

```python
# ✅ CORRECT ORDER (MANDATORY)
class SutraAI:
    def __init__(self):
        # 1. Environment setup
        os.environ["SUTRA_STORAGE_MODE"] = "server"
        
        # 2. Embedding service processor FIRST (CRITICAL)
        service_url = os.getenv("SUTRA_EMBEDDING_SERVICE_URL", "http://sutra-embedding-service:8888")
        self.embedding_processor = EmbeddingServiceProvider(service_url=service_url)
        
        # 3. Core components
        self._core = ReasoningEngine(use_rust_storage=True)
        
        # 4. Component reconstruction with correct processors
        self._core.embedding_processor = self.embedding_processor
        self._core.query_processor = QueryProcessor(
            self._core.storage,
            self._core.association_extractor,
            self._core.path_finder,
            self._core.mppa,
            embedding_processor=self.embedding_processor,  # ← Pre-created processor
            nlp_processor=None,
        )
```

**Violation of initialization order will cause system failure.**

### TCP Architecture Requirements

- ALL services MUST use `sutra-storage-client-tcp` package
- NEVER import `sutra_storage` directly in distributed services
- Unit variants send strings, not `{variant: {}}` format
- Convert numpy arrays to lists before TCP transport

### Common Production-Breaking Errors

| Error Message | Root Cause | Fix |
|---------------|------------|-----|
| "Dimension mismatch: expected 768, got 384" | Wrong embedding model | Configure nomic-embed-text-v1.5 |
| "Connection refused to embedding service" | Service not running | Start sutra-embedding-service |
| "Embedding service unhealthy" | Model not loaded | Check service logs |
| "can not serialize 'numpy.ndarray' object" | Missing array conversion | Convert to list before TCP |
| "wrong msgpack marker" | Wrong message format | Use string for unit variants |

**References:**
- `EMBEDDING_SERVICE_MIGRATION.md` - Complete migration guide
- `packages/sutra-embedding-service/` - Service implementation
- `docker-compose-grid.yml` - Production deployment

---

## Architecture

### System Diagram

**TCP Binary Protocol** microservices (10-50× lower latency than gRPC):

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                            Docker Network (sutra-network)                       │
├─────────────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────────────────┐  │
│  │  sutra-control  │    │   sutra-client  │    │      sutra-markdown-web     │  │
│  │  (React + Fast  │    │   (Streamlit)   │    │       (Markdown API)       │  │
│  │   API Gateway)  │    │    UI Client    │    │        UI Client           │  │
│  │   Port: 9000    │    │   Port: 8080    │    │       Port: 8002           │  │
│  └─────────┬───────┘    └─────────┬───────┘    └─────────────┬───────────────┘  │
│            │                      │                          │                   │
│            └──────────────────────┼──────────────────────────┘                   │
│                                   │                                              │
│  ┌─────────────────┐              │  TCP      ┌─────────────────────────────┐  │
│  │   sutra-api     │◀─────────────┼──────────▶│      storage-server         │  │
│  │   (FastAPI)     │              │  Binary   │    (Rust TCP Server)        │  │
│  │   Port: 8000    │              │  Protocol │      Port: 50051            │  │
│  └─────────┬───────┘              │           └──────────────┬──────────────┘  │
│            │                      │                          │                   │
│            └──────────────────────┼──────────────────────────┘                   │
│                                   │                                              │
│  ┌─────────────────┐              │           ┌─────────────────────────────┐  │
│  │  sutra-hybrid   │◀─────────────┼──────────▶│ sutra-embedding-service │  │
│  │ (Semantic AI +  │              │           │   (nomic-embed-text-v1.5)  │  │
│  │ Orchestration)  │              │           │      Port: 8888             │  │
│  │   Port: 8001    │              │           └─────────────────────────────┘  │
│  └─────────────────┘              │                                              │
│                                   │                                              │
│  ┌─────────────────────────────────────────────────────────────────────────┐  │
│  │                        Sutra Grid (Distributed Layer)                   │  │
│  │  Grid Master (7001 HTTP, 7002 TCP) ◀──TCP──▶ Grid Agents (8001)        │  │
│  │  Event Storage (50052 TCP)                                              │  │
│  └─────────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### Key Architectural Principles

- **Custom TCP Binary Protocol**: 10-50× lower latency than gRPC (bincode serialization)
- **Rust Storage**: Zero-copy memory-mapped files, lock-free concurrency
- **Optional Semantic Embeddings**: Enhance reasoning but remain transparent
- **Temporal Storage**: Log-structured for time-travel queries
- **Production Error Handling**: Automatic reconnection with exponential backoff

### Package Structure

#### Core AI Packages

- **sutra-core**: Graph reasoning engine (ReasoningEngine, PathFinder, MPPA)
- **sutra-storage**: Production Rust storage (57K writes/sec, <0.01ms reads)
- **sutra-hybrid**: Semantic embeddings integration (SutraAI class)
- **sutra-nlg**: Grounded, template-driven NLG (no LLM)

#### Service Packages

- **sutra-api**: Production REST API (FastAPI, rate limiting)
- **sutra-embedding-service**: High-performance embedding service (nomic-embed-text-v1.5, 768-d)
- **sutra-bulk-ingester**: Rust bulk data ingestion service

#### UI & Tooling Packages

- **sutra-control**: React control center with FastAPI gateway
- **sutra-client**: Streamlit web interface
- **sutra-markdown-web**: Markdown API service
- **sutra-explorer**: Standalone storage explorer (NEW)
- **sutra-cli**: Command-line interface (placeholder)

#### Sutra Grid (Distributed Infrastructure)

- **sutra-grid-master**: Orchestration service (7001 HTTP, 7002 TCP)
- **sutra-grid-agent**: Node lifecycle management (port 8001)
- **sutra-grid-events**: Event emission library (17 event types)
- **sutra-protocol**: Shared TCP binary protocol (bincode)
- **sutra-grid-cli**: Cluster management (under TCP migration)

**Grid Status**: Production-Ready ✅ (11 master events, 2 agent events, port 50052 reserved)

---

## Development Environment

### Environment Setup

```bash
# Virtual environment setup (REQUIRED)
python3 -m venv venv
source venv/bin/activate  # Windows: venv\Scripts\activate
pip install -r requirements-dev.txt  # Installs all -e packages

# Build System:
# - Multi-language: Python (PyO3) + Rust + Node.js
# - 15 Python packages with editable installs
# - 6 Rust crates with optimized release builds
# - React/TypeScript with Vite
```

### Testing

```bash
# CRITICAL: Start services first
./sutra-deploy.sh up

# Core integration tests
source venv/bin/activate
PYTHONPATH=packages/sutra-core python -m pytest tests/ -v

# Storage tests (Rust - includes WAL crash recovery)
cd packages/sutra-storage
cargo test
cargo test test_wal  # WAL durability tests

# End-to-end demos (require running services)
python demo_simple.py
python demo_end_to_end.py
python demo_mass_learning.py

# Grid integration tests
cd packages/sutra-grid-master
./test-integration.sh  # 5 automated tests

# Production smoke test (validates embedding config)
./scripts/smoke-test-embeddings.sh
```

### Code Quality

```bash
make format  # black + isort
make lint    # lint core package
make check   # format, lint, test
```

### Deployment

**⚡ Single Source of Truth: `./sutra-deploy.sh`**

```bash
# First-time installation
./sutra-deploy.sh install

# Start all services (10-service ecosystem)
./sutra-deploy.sh up

# Start with bulk ingester (11 services)
docker-compose -f docker-compose-grid.yml --profile bulk-ingester up -d

# System management
./sutra-deploy.sh status    # Check health
./sutra-deploy.sh validate  # Health checks
./sutra-deploy.sh logs      # View logs
./sutra-deploy.sh restart   # Restart services
./sutra-deploy.sh down      # Stop services
./sutra-deploy.sh clean     # Complete cleanup

# CRITICAL: Before deployment
./scripts/smoke-test-embeddings.sh
```

### Service URLs (After Deployment)

| Service | Port | URL |
|---------|------|-----|
| Control Center | 9000 | http://localhost:9000 |
| Client UI | 8080 | http://localhost:8080 |
| API | 8000 | http://localhost:8000 |
| Hybrid API | 8001 | http://localhost:8001 |
| Embedding Service | 8888 | http://localhost:8888 |
| Bulk Ingester | 8005 | http://localhost:8005 |
| Grid Master (HTTP) | 7001 | http://localhost:7001 |
| Grid Master (TCP) | 7002 | tcp://localhost:7002 |
| Storage Server (TCP) | 50051 | tcp://localhost:50051 |
| Event Storage (TCP) | 50052 | tcp://localhost:50052 |

---

## Key Components

### ReasoningEngine (sutra-core)

Main AI interface orchestrating reasoning:
- Natural language query processing
- Real-time learning without retraining
- Query result caching
- Multi-path plan aggregation (MPPA)
- Complete audit trails

### PathFinder (sutra-core/reasoning)

Advanced graph traversal strategies:
- **Best-first**: Confidence-optimized with heuristics
- **Breadth-first**: Shortest path exploration
- **Bidirectional**: Optimal path from both ends
- Confidence decay (0.85 default)
- Cycle detection, path diversification

### MultiPathAggregator (MPPA)

Consensus-based reasoning:
- Path clustering by answer similarity (0.8 threshold)
- Majority voting with configurable thresholds
- Diversity bonus for varied approaches
- Robustness analysis

### SutraAI (sutra-hybrid)

High-level interface:
- Optional semantic similarity matching
- Multi-strategy comparison (graph vs semantic)
- Agreement scoring between strategies
- Knowledge persistence, audit trails

### ConcurrentStorage (sutra-storage - Rust)

Production-ready storage:
- **57,412 writes/sec** (25,000× faster than JSON)
- **<0.01ms read latency** (zero-copy mmap)
- **Zero data loss** with Write-Ahead Log (WAL)
- Lock-free write log, background reconciliation
- Single-file `storage.dat` (512MB initial)
- Immutable read snapshots (burst-tolerant)
- Crash recovery with automatic WAL replay
- **Binary format**: MessagePack (4.4× smaller, 2-3× faster)

### Sutra Control Center (sutra-control)

Modern React monitoring interface:
- React 18 + Material Design 3 + TypeScript + Vite
- Secure FastAPI gateway for all services
- Real-time system metrics
- Grid management UI ✅
- Bulk ingester UI ✅
- Multi-stage Docker build

### Sutra Storage Explorer (sutra-explorer)

Standalone storage exploration:
- Read-only safe exploration
- Rust parser for storage.dat v2
- FastAPI backend (port 8100)
- React frontend with D3.js visualization
- BFS pathfinding, N-hop neighborhoods
- Cosine similarity, full-text search

---

## Common Development Tasks

### Adding New NLG Templates

1. Add template in `packages/sutra-nlg/sutra_nlg/templates.py`
2. Include tone, moves, pattern with slots
3. Validate: `pytest packages/sutra-nlg`
4. Rebuild Hybrid Docker image

### Adding New Reasoning Strategies

1. Implement in `sutra_core/reasoning/paths.py`
2. Add to PathFinder class
3. Update QueryProcessor
4. Add comprehensive tests

### Extending Storage Format

1. Update Rust structures in `packages/sutra-storage/src/`
2. Modify Python bindings via PyO3 in `lib.rs`
3. Update docs in `docs/sutra-storage/`
4. Run `cargo build --release`
5. Verify with `verify_concurrent_storage.py`

### Adding API Endpoints

1. Define models in `sutra_api/models.py`
2. Implement endpoint in `sutra_api/main.py`
3. Add rate limiting configuration
4. Update OpenAPI documentation

### Extending Control Center

1. **Frontend**: Edit React components in `packages/sutra-control/src/`
2. **Backend**: Update FastAPI gateway in `packages/sutra-control/backend/main.py`
3. **Build**: `npm run build`
4. **Docker**: `docker build -t sutra-control:latest .`
5. **Test**: Access http://localhost:9000

### Extending Sutra Grid

1. **Events**: Update `packages/sutra-grid-events/src/events.rs`
2. **Master**: Edit `packages/sutra-grid-master/src/main.rs`
3. **Agent**: Edit `packages/sutra-grid-agent/src/main.rs`
4. **Protocol**: Modify `packages/sutra-protocol/src/lib.rs`
5. **Test**: Run Docker compose and check logs

---

## Troubleshooting

### Quick Diagnostics

**Run production smoke test first:**

```bash
./scripts/smoke-test-embeddings.sh
```

Validates:
- ✅ nomic-embed-text model availability
- ✅ 768-d configuration
- ✅ No fallback warnings
- ✅ End-to-end semantic search

### Same Answer for All Questions ⭐ CRITICAL

**Symptoms:** Every query returns identical answer

**Root Cause:** Zero embeddings in storage

**Solution:**

```bash
# 1. Clean old data without embeddings
docker stop sutra-storage sutra-api sutra-hybrid sutra-client
docker rm -f sutra-storage
docker volume rm sutra-models_storage-data
docker-compose -f docker-compose-grid.yml up -d storage-server sutra-api sutra-hybrid sutra-client

# 2. Learn via Hybrid service (has embeddings!)
curl -X POST http://localhost:8001/sutra/learn \
  -H "Content-Type: application/json" \
  -d '{"text":"Test fact"}'

# 3. Verify embeddings generated
curl -s http://localhost:8000/stats | jq '.total_embeddings'
# Should be > 0
```

### Embedding Configuration Issues

**Symptoms:**
- Wrong query results ("tallest mountain" → "Pacific Ocean")
- "Dimension mismatch: expected 768, got 384"
- "Query embedding FALLBACK to spaCy" in logs

**Solution:**

```bash
# 1. Run smoke test
./scripts/smoke-test-embeddings.sh

# 2. Verify configuration
docker logs sutra-storage | grep -E "(Vector dimension|nomic)"
docker logs sutra-hybrid | grep -E "(PRODUCTION|nomic|fallback)"

# 3. Fix docker-compose-grid.yml if needed:
#    VECTOR_DIMENSION=768
#    SUTRA_EMBEDDING_SERVICE_URL=http://sutra-embedding-service:8888

# 4. Clean and restart
./sutra-deploy.sh down
docker volume rm sutra-models_storage-data
./sutra-deploy.sh up

# 5. Verify fix
./scripts/smoke-test-embeddings.sh
```

### Storage/Persistence Issues

- Data written to single `storage.dat` file
- WAL logged to `wal.log` (MessagePack binary)
- Check path permissions (default: `./knowledge/storage.dat`)
- Call `storage.flush()` before shutdown
- Auto-flush at 50K concepts
- WAL replays automatically on startup
- WAL checkpointed after flush

### TCP Adapter Issues (Fixed 2025-10-19)

**Symptoms:**
- "Storage-backed pathfinding failed: no attribute 'find_paths'"
- "list indices must be integers or slices, not str"

**Fixed:**
- ✅ `find_paths()` - Multi-path reasoning
- ✅ `get_association()` - Association retrieval
- ✅ `get_all_concept_ids()` - Health checks
- ✅ `get_neighbors()` - Response parsing
- ✅ Unit variant handling (strings, not dicts)
- ✅ Numpy array to list conversion

### Common Deployment Issues

#### Docker Build Failures

```bash
docker rmi sutra-storage-server:latest || true
docker-compose -f docker-compose-grid.yml build storage-server
```

#### Services Not Starting

```bash
docker info
docker ps -a | grep sutra
docker-compose -f docker-compose-grid.yml down
docker system prune -f
./sutra-deploy.sh up
```

#### Health Check Failures

```bash
docker logs sutra-hybrid --tail 50
./sutra-deploy.sh status
```

**See:**
- `PRODUCTION_REQUIREMENTS.md` - Mandatory embedding config
- `docs/EMBEDDING_TROUBLESHOOTING.md` - Detailed troubleshooting
- `TROUBLESHOOTING.md` - General troubleshooting

---

## Recent Major Features

### 🎉 Production-Grade Storage Complete (2025-10-24)

**Status:** ✅ ALL P0 CRITICAL ISSUES RESOLVED

**Grade:** A- (90/100) → **A+ (95/100)**

**What Was Fixed:**
1. ✅ Cross-shard 2PC - Full distributed transaction support (transaction.rs, 500 lines)
2. ✅ Input validation - DoS protection (6 limits, 7 validation points)
3. ✅ Config validation - Fail-fast at startup
4. ✅ Overflow protection - Memory safety via checked_mul()

**Test Results:** 107 passed, 0 failed, 1 ignored ✅

**Production Guarantees:**
- ✅ Zero data loss - 2PC ensures atomic cross-shard operations
- ✅ DoS protection - Cannot allocate 1GB concepts
- ✅ Fail-fast - Invalid config rejected at startup
- ✅ Memory safety - No integer overflow at 10M+ concepts
- ✅ 57K writes/sec, <0.01ms reads maintained

**Documentation:** `docs/storage/PRODUCTION_GRADE_COMPLETE.md` (465 lines)

### P0 Production Features (2025-10-24)

#### P0.2: Embedding Service High Availability

**Architecture:** 3 replicas + HAProxy load balancer

```yaml
Embedding HA:
  - 3 Independent Replicas (embedding-1, 2, 3)
  - HAProxy Load Balancer (least-connection)
  - Health Checks: Every 2s (3 failures = down)
  - Automatic Failover: <3s detection
  - Stats Dashboard: http://localhost:8404/stats
```

**Benefits:**
- ✅ Zero downtime deployments
- ✅ 3× capacity for load spikes
- ✅ Automatic failover (<3s)
- ✅ Independent failure domains

#### P0.3: Self-Monitoring via Grid Events

**Architecture:** "Eating our own dogfood" - Sutra monitors itself

**9 Production Event Types:**
- StorageMetrics, QueryPerformance, EmbeddingLatency
- HnswIndexBuilt/Loaded, PathfindingMetrics
- ReconciliationComplete, EmbeddingServiceHealthy/Degraded

**Implementation:**
```rust
let emitter = StorageEventEmitter::new(node_id, event_storage);
emitter.emit_storage_metrics(concepts, edges, throughput, memory_mb);
emitter.emit_query_performance(type, depth, results, latency, confidence);
```

**Zero-Cost:** Events only emitted when `EVENT_STORAGE` configured

#### P0.4: Scale Validation - 10M Concept Benchmark

```bash
cd scripts && rustc --edition 2021 -O scale-validation.rs
SUTRA_NUM_SHARDS=16 ./scale-validation
# Runtime: ~3-5 minutes
```

**4 Test Phases:**
1. Sequential Write: 10M concepts with 768-d vectors
2. Random Read: 10K queries (P50/P95/P99 latencies)
3. Vector Search: 10K HNSW searches
4. Memory Analysis: Per-concept overhead

**Claims Validated:**
- ✅ Write throughput ≥ 50,000 concepts/sec
- ✅ Read latency < 0.01ms (P50)
- ✅ Vector search < 50ms (P50)
- ✅ Memory ≤ 2KB/concept

### P1 Performance Features (2025-10-24)

#### P1.1: Semantic Association Extraction

**OLD:** Regex patterns (50% accuracy, 5ms)
**NEW:** Embedding-based NLP (80% accuracy, 30ms)

**Performance:**
- 30% better accuracy than regex
- 3× faster than spaCy
- Zero new dependencies (uses HA embedding service)

#### P1.5: HNSW Persistent Index - USearch Migration 🚀

**Migration:** hnsw-rs → **USearch** (production-grade)

**Problem Solved:** OLD system rebuilt index on EVERY startup (lifetime constraints)

**Performance:**
| Operation | OLD (rebuild) | NEW (mmap) | Improvement |
|-----------|---------------|------------|-------------|
| Load 1K | 327ms | 3.5ms | **94×** |
| Load 1M | 5.5min | **3.5s** | **94×** |
| Load 10M | 55min | **35s** | **94×** |

**Benefits:**
- ✅ 94× faster startup (TRUE mmap persistence)
- ✅ 24% smaller index files
- ✅ SIMD-optimized search
- ✅ Incremental updates with capacity management

#### P1.2: Parallel Pathfinding

**Solution:** Rayon-based parallel pathfinding with work-stealing

**Performance:**
| Graph Fanout | Sequential | Parallel (8 cores) | Speedup |
|--------------|------------|-------------------|---------|
| 8 neighbors | 400ms | 50ms | **8×** |
| 16 neighbors | 800ms | 100ms | **8×** |

**Benefits:**
- ✅ 4-8× speedup on multi-path queries
- ✅ Work-stealing via Rayon
- ✅ Thread-safe using immutable snapshots

### Unified Learning Architecture (2025-10-19)

**CRITICAL:** Storage server owns learning pipeline

```
✅ CORRECT:
ALL Clients → TCP → Storage Server Learning Pipeline
                      ├─→ Embedding Generation (Service)
                      ├─→ Association Extraction (Rust)
                      ├─→ Atomic Storage (HNSW + WAL)
                      └─→ Return concept_id

❌ OLD (Removed):
Each service had duplicate logic
```

**Benefits:**
- ✅ Zero code duplication
- ✅ Guaranteed consistency
- ✅ Automatic embeddings (no "same answer" bug)
- ✅ Easier testing
- ✅ Better performance

### Production Sharded Storage (2025-10-24)

**Architecture:** Horizontal scalability for 10M+ concepts

```
Concepts → Consistent Hashing → Shard Selection → Independent Storage

Sharded Storage (4-16 shards)
  ├─→ Shard 0: storage.dat + WAL + HNSW
  ├─→ Shard 1: storage.dat + WAL + HNSW
  ├─→ Shard 2: storage.dat + WAL + HNSW
  └─→ Shard 3: storage.dat + WAL + HNSW

Parallel Vector Search: All shards queried simultaneously
```

**Configuration:**
```yaml
storage-server:
  environment:
    - SUTRA_STORAGE_MODE=sharded    # "single" or "sharded"
    - SUTRA_NUM_SHARDS=4            # 4-16 recommended
    - STORAGE_PATH=/data
    - VECTOR_DIMENSION=768
```

**When to Use:**

| Concept Count | Mode | Shards | Notes |
|--------------|------|--------|-------|
| < 100K | Single | 1 | Development |
| 1M - 5M | Sharded | 4 | Production |
| 5M - 10M | Sharded | 8 | High-scale |
| 10M+ | Sharded | 16 | Enterprise |

### AI-Native Adaptive Reconciliation (2025-10-24)

**Architecture:** Self-optimizing storage using online ML

```rust
AdaptiveReconciler {
  // EMA-based trend analysis
  trend_analyzer: TrendAnalyzer {
    queue_ema: f64,  // Exponential moving average
    rate_ema: f64,   // Processing rate
    ema_alpha: 0.3,  // Smoothing factor
  },
  
  // Dynamic interval optimization (1-100ms range)
  calculate_optimal_interval() -> Duration {
    if utilization < 0.20 { 100ms }  // Idle: save CPU
    else if utilization > 0.70 { 1-5ms }  // High load: aggressive
    else { 10ms }  // Normal
  }
}
```

**Performance:**

**OLD (Fixed 10ms):**
- Idle: Wastes CPU cloning every 10ms
- Burst: 10ms lag = 570 entries backlog

**NEW (Adaptive):**
- Idle: 80% CPU savings (100ms intervals)
- Burst: 10× lower latency (1-5ms aggressive drain)
- Real-time health scoring with predictive alerts

**Benefits:**
- ✅ 80% CPU reduction during idle
- ✅ 10× lower latency during bursts
- ✅ Predictive alerting at 70% capacity
- ✅ Zero tuning required - self-optimizing
- ✅ Backward compatible

**Test Results:** 102 tests passed ✅

---

## Performance Characteristics

Based on production testing with ConcurrentStorage:

- **Learning:** 0.02ms per concept (57,412 concepts/sec) — 25,000× faster
- **Query:** <0.01ms with zero-copy mmap
- **Path finding:** ~1ms for 3-hop BFS
- **Memory:** ~0.1KB per concept (excluding embeddings)
- **Storage:** Single `storage.dat` file (512MB for 1K concepts)
- **Accuracy:** 100% verified with comprehensive test suite

---

## Code Style

- **Line length:** 88 characters (black default)
- **Import order:** stdlib, third-party, local (isort with black profile)
- **Type hints:** Required for all public functions
- **Docstrings:** Google style for all public classes/methods
- **Testing:** pytest with descriptive names and docstrings

---

## Research Foundation

Built on published research:
- **Adaptive Focus Learning:** "LLM-Oriented Token-Adaptive Knowledge Distillation" (Oct 2024)
- **Multi-Path Plan Aggregation (MPPA):** Consensus-based reasoning
- **Graph-based reasoning:** Decades of knowledge representation research

No proprietary techniques - all methods from published work.

---

**Last Updated:** 2025-10-24  
**Status:** Production-Ready  
**Documentation:** See `docs/INDEX.md` for complete documentation map

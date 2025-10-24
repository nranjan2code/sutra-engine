# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## 🚨 CRITICAL: Embedding System Requirements (MANDATORY)

**✅ HIGH-PERFORMANCE EMBEDDING SERVICE (2025-10-20) ✅**

**Current Status**: **PRODUCTION-READY EMBEDDING SERVICE**  
**Architecture**: Dedicated high-performance service using **nomic-embed-text-v1.5**  
**Performance**: 10x faster than previous Ollama-based system

**NEVER IGNORE THESE REQUIREMENTS - SYSTEM WILL NOT FUNCTION WITHOUT THEM:**

### **1. STRICT EMBEDDING SERVICE REQUIREMENT (Production Standard: 2025-10-20)**

**⚠️ ONLY Sutra Embedding Service IS SUPPORTED ⚠️**

Sutra AI uses **dedicated embedding service architecture** with ZERO dependencies:

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

**Why This Matters:**
- Different models produce **incompatible semantic spaces**
- Mixing dimensions causes **WRONG QUERY RESULTS**
- Real example: Using 384-d for queries + 768-d for storage caused:
  - Query: "What is the tallest mountain?" → Answer: "Pacific Ocean" ❌
- Dedicated service ensures **consistent performance**

**Mandatory Environment Variables:**
```bash
# Storage Server (docker-compose-grid.yml)
VECTOR_DIMENSION=768                                           # MUST be 768
SUTRA_EMBEDDING_SERVICE_URL=http://sutra-embedding-service:8888  # MUST point to service

# Hybrid Service
SUTRA_EMBEDDING_SERVICE_URL=http://sutra-embedding-service:8888  # MUST match storage
SUTRA_VECTOR_DIMENSION=768                                     # MUST be 768
SUTRA_USE_SEMANTIC_EMBEDDINGS=true                            # MUST be true
```

**Verification:**
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

**See:** `EMBEDDING_SERVICE_MIGRATION.md` for complete details and deployment guide.

### **2. MANDATORY: Component Initialization Order (CRITICAL)**

**⚠️ EMBEDDING SERVICE INITIALIZATION IS CRITICAL ⚠️**

Following the 2025-10-20 embedding service migration, **strict initialization order** is now **MANDATORY**:

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
        logger.info("Recreated QueryProcessor with EmbeddingServiceProvider")
```

**Violation of initialization order will cause system failure.**

**Reference**: `packages/sutra-embedding-service/` (production implementation)

### **3. Embedding Service Configuration**
   - Must be running at `SUTRA_EMBEDDING_SERVICE_URL` (default: `http://sutra-embedding-service:8888`)
   - Uses nomic-embed-text-v1.5 with 768-dimensional output
   - Includes intelligent caching and batch processing for performance

### **4. TCP Architecture is MANDATORY**
   - ALL services MUST use `sutra-storage-client-tcp` package
   - NEVER import `sutra_storage` directly in distributed services
   - Unit variants send strings, not `{variant: {}}` format
   - Convert numpy arrays to lists before TCP transport

### **5. Common Production-Breaking Errors:**
   - "Dimension mismatch: expected 768, got 384" → Embedding service not configured correctly
   - "Connection refused to embedding service" → SUTRA_EMBEDDING_SERVICE_URL incorrect or service not running
   - "Embedding service unhealthy" → Service startup failed or model not loaded
   - "can not serialize 'numpy.ndarray' object" → Missing array conversion
   - "wrong msgpack marker" → Wrong message format for unit variants

**References:**
- **EMBEDDING_SERVICE_MIGRATION.md** - Complete migration guide (READ THIS FIRST)
- **packages/sutra-embedding-service/** - Service implementation
- **docker-compose-grid.yml** - Production deployment configuration

## 🔥 NEW: Unified Learning Architecture (Implemented 2025-10-19)

**CRITICAL ARCHITECTURAL CHANGE - ALL DEVELOPMENT MUST FOLLOW THIS PATTERN:**

### Single Source of Truth: Storage Server Owns Learning

```
✅ CORRECT Architecture (Current):

ALL Clients → TCP → Storage Server Learning Pipeline
                      ├─→ Embedding Generation (Service)
                      ├─→ Association Extraction (Rust)
                      ├─→ Atomic Storage (HNSW + WAL)
                      └─→ Return concept_id

❌ OLD Architecture (Removed):
Each service had duplicate logic for embeddings/associations
```

### Implementation Rules

1. **Clients MUST delegate learning to storage server:**
   ```python
   # ✅ CORRECT - Delegate to storage
   concept_id = storage.learn_concept(
       content=content,
       generate_embedding=True,  # Storage server handles this
       extract_associations=True,
   )
   
   # ❌ WRONG - Client-side embedding generation
   embedding = embedding_service.generate(content)  # DON'T DO THIS
   storage.add_concept(concept, embedding)
   ```

2. **ReasoningEngine.learn() extracts and passes parameters:**
   ```python
   # ✅ CORRECT - Extract from kwargs
   generate_embedding = kwargs.get("generate_embedding", True)
   extract_associations = kwargs.get("extract_associations", True)
   
   concept_id = self.storage.learn_concept(
       content=content,
       generate_embedding=generate_embedding,
       extract_associations=extract_associations,
       # ... other individual parameters
   )
   
   # ❌ WRONG - Pass options dict
   concept_id = self.storage.learn_concept(
       content=content,
       options=kwargs  # TcpStorageAdapter expects individual params
   )
   ```

3. **TCP client response parsing handles list format:**
   ```python
   # ✅ CORRECT - Handle both dict and list formats
   if isinstance(result, list) and len(result) > 0:
       return result[0]  # Storage returns ['concept_id']
   elif isinstance(result, dict) and "concept_id" in result:
       return result["concept_id"]
   ```

4. **Storage server implements complete pipeline:**
   - `learning_pipeline.rs` - Orchestrates entire flow
   - `embedding_client.rs` - Embedding service HTTP integration
   - `association_extractor.rs` - Pattern-based NLP
   - All atomically committed to storage

### Benefits

✅ **Zero code duplication** - One implementation for all clients  
✅ **Guaranteed consistency** - Same behavior everywhere  
✅ **Automatic embeddings** - No "same answer" bug  
✅ **Easier testing** - Mock storage server instead of each client  
✅ **Better performance** - Batch operations in one place  

### Migration Complete

- ✅ Phase 1: Storage server learning pipeline (2025-10-19)
- ✅ ReasoningEngine updated to use unified API
- ✅ TCP client response parsing fixed
- ✅ Docker build cache issues resolved
- ✅ End-to-end testing verified (Eiffel Tower, Great Wall, Mount Everest)

**See:** `docs/UNIFIED_LEARNING_ARCHITECTURE.md` for complete design documentation.

## 🚀 NEW: Production Sharded Storage (Implemented 2025-10-24)

**CRITICAL: HORIZONTAL SCALABILITY FOR 10M+ CONCEPTS**

### Sharded Storage Architecture

Sutra AI now supports **production-grade sharded storage** for horizontal scalability:

```
✅ PRODUCTION Architecture (Current):

Concepts → Consistent Hashing → Shard Selection → Independent Storage

Sharded Storage (4-16 shards)
  ├─→ Shard 0: storage.dat + WAL + HNSW index
  ├─→ Shard 1: storage.dat + WAL + HNSW index  
  ├─→ Shard 2: storage.dat + WAL + HNSW index
  └─→ Shard 3: storage.dat + WAL + HNSW index

Parallel Vector Search: All shards queried simultaneously
Performance: O(log N) per shard = O(log(N/S)) total where S = shard count
```

### Configuration

**Environment Variables (docker-compose-grid.yml):**
```yaml
storage-server:
  environment:
    - SUTRA_STORAGE_MODE=sharded    # "single" or "sharded"
    - SUTRA_NUM_SHARDS=4            # Number of shards (4-16 recommended)
    - STORAGE_PATH=/data            # Base path for shard directories
    - VECTOR_DIMENSION=768          # MUST be 768 (nomic-embed-text)
```

### LearningStorage Trait (Production Pattern)

The **unified learning pipeline** works with both single and sharded storage via the `LearningStorage` trait:

```rust
// Common trait for all storage backends
pub trait LearningStorage {
    fn learn_concept(&self, id: ConceptId, content: Vec<u8>, 
                     vector: Option<Vec<f32>>, strength: f32, 
                     confidence: f32) -> Result<u64>;
    
    fn learn_association(&self, source: ConceptId, target: ConceptId,
                        assoc_type: AssociationType, 
                        confidence: f32) -> Result<u64>;
}

// Implemented for:
// - ConcurrentMemory (single storage)
// - ShardedStorage (distributed storage)
// - Arc<T> where T: LearningStorage (shared ownership)
```

### Benefits

✅ **Horizontal scalability** - Add shards to handle more concepts  
✅ **Parallel operations** - All shards queried simultaneously  
✅ **Load balancing** - Consistent hashing distributes load evenly  
✅ **Independent failure** - Each shard has own WAL + durability  
✅ **Zero code changes** - Trait-based polymorphism handles both modes  

### Performance Characteristics

**Single Storage (< 1M concepts):**
- Write: 57,412 concepts/sec
- Read: <0.01ms (zero-copy)
- Vector search: O(log N) with HNSW

**Sharded Storage (1M-10M+ concepts):**
- Write: 57,412 concepts/sec per shard
- Read: <0.01ms per shard
- Vector search: O(log(N/S)) with parallel shard queries
- Shard count: 4-16 recommended (configurable)

### When to Use Sharded Storage

| Concept Count | Mode | Shards | Notes |
|--------------|------|--------|-------|
| < 100K | Single | 1 | Optimal for development |
| 100K - 1M | Single | 1 | Monitor performance |
| 1M - 5M | Sharded | 4 | Production recommended |
| 5M - 10M | Sharded | 8 | High-scale deployment |
| 10M+ | Sharded | 16 | Enterprise scale |

### Migration from Single to Sharded

**Zero-downtime migration:**
1. Update `docker-compose-grid.yml` with shard configuration
2. Restart storage server: `docker-compose restart storage-server`
3. Data automatically distributed across shards
4. Clients continue working without changes

**See:** `docs/storage/SHARDING.md` for complete migration guide.

## Project Overview

Sutra AI is an explainable graph-based AI system that learns in real-time without retraining. It provides complete reasoning paths for every decision, making it a transparent alternative to black-box LLMs.

**Core Value Proposition:**
- Shows reasoning for every answer
- Learns incrementally from new information  
- Provides audit trails for compliance
- Works without GPUs or massive compute
- 100% explainable reasoning paths
- **NEW:** Self-observability using own reasoning engine
- **NEW:** Progressive streaming responses (10x faster UX)
- **NEW:** Quality gates with "I don't know" for uncertain answers
- **NEW:** Natural language operational queries

## Architecture

**TCP Binary Protocol** microservices architecture with containerized deployment. All services communicate via high-performance TCP binary protocol (10-50× lower latency than gRPC) with a secure React-based control center for monitoring.

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
│  │  sutra-hybrid   │◀───────────────┼──────────▶│ sutra-embedding-service │  │
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

**Key Architectural Principles:**
- **Custom TCP Binary Protocol**: 10-50× lower latency than gRPC, using bincode serialization
- All graph/vector ops run in storage-server; no in-process storage in API/Hybrid
- Rust storage provides zero-copy memory-mapped files and lock-free concurrency
- Optional semantic embeddings enhance reasoning but remain transparent
- Temporal log-structured storage for time-travel queries
- Production-grade error handling with automatic reconnection and exponential backoff

### Package Structure

#### Core AI Packages
- **sutra-core**: Core graph reasoning engine with concepts, associations, and multi-path plan aggregation (MPPA)
- **sutra-storage**: Production-ready Rust storage with ConcurrentStorage (57K writes/sec, <0.01ms reads), single-file architecture, and lock-free concurrency  
- **sutra-hybrid**: Semantic embeddings integration (SutraAI class) that combines graph reasoning with optional similarity matching
- **sutra-nlg**: Grounded, template-driven NLG (no LLM) used by Hybrid for human-like, explainable responses

#### Service Packages
- **sutra-api**: Production REST API with FastAPI, rate limiting, and comprehensive endpoints
- **sutra-embedding-service**: Dedicated high-performance embedding service (nomic-embed-text-v1.5, 768-d)
- **sutra-bulk-ingester**: High-performance Rust service for bulk data ingestion with TCP storage integration (production-ready)

#### UI & Tooling Packages
- **sutra-control**: Modern React-based control center with secure FastAPI gateway for system monitoring and management
- **sutra-client**: Streamlit-based web interface for interactive AI queries and knowledge exploration
- **sutra-markdown-web**: Markdown API service for document processing and content management
- **sutra-explorer**: Standalone storage explorer for deep visualization and analysis of storage.dat files (NEW)
- **sutra-cli**: Command-line interface (placeholder)

### Production Enhancements (NEW)
- **Self-Observability** (`sutra_core/events.py`): Event emission to knowledge graph for NL queries
- **Quality Gates** (`sutra_core/quality_gates.py`): Confidence calibration and uncertainty quantification
- **Streaming** (`sutra_core/streaming.py`): Progressive answer refinement with SSE
- **Observability Queries** (`sutra_core/observability_query.py`): Natural language operational debugging

#### Sutra Grid (Distributed Infrastructure)
- **sutra-grid-master**: Rust-based orchestration service managing agents and storage nodes (7001 HTTP binary distribution, 7002 TCP agent connections)
- **sutra-grid-agent**: Rust-based agent with TCP server for storage node lifecycle management (port 8001)
- **sutra-grid-events**: Event emission library with 17 structured event types, TCP-based async background worker
- **sutra-protocol**: Shared TCP binary protocol library using bincode serialization
- **sutra-grid-cli**: Command-line tool for cluster management (under migration to TCP)

**Grid Status**: Production-Ready ✅  
- Master: 11 events emitted (agent lifecycle, node operations)
- Agent: 2 events emitted (node crash, restart)
- Reserved Storage: Port 50052 for Grid events
- Testing: End-to-end verified with `test-integration.sh`
- Documentation: See `docs/grid/architecture/GRID_ARCHITECTURE.md` for complete details

## Development Commands

### Environment Setup

**Build System Overview:**
- **Multi-language workspace**: Python (PyO3) + Rust + Node.js
- **Python packages**: 15 packages with editable installs via `requirements-dev.txt`
- **Rust workspace**: 6 crates with optimized release builds (`Cargo.toml`)
- **Frontend**: React/TypeScript with Vite build system (`package.json`)
- **No Makefile**: Uses direct tooling (pytest, cargo, npm, docker-compose)

```bash
# Virtual environment setup (REQUIRED)
python3 -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate
pip install -r requirements-dev.txt  # Installs all -e packages automatically

# Development dependencies structure:
# -e packages/sutra-core/
# -e packages/sutra-hybrid/ 
# -e packages/sutra-api/
# -e packages/sutra-nlg/
# -e packages/sutra-storage-client-tcp/
# Plus: pytest, black, isort, flake8, mypy, fastapi, etc.
```

### Testing

**Current Test Pipeline Status:**
- **Integration tests**: 24 tests (3 deselected, 21 executed)
- **Requires running services**: Tests expect localhost:8000 (API) and localhost:8001 (Hybrid)
- **Service dependency**: Must run `./sutra-deploy.sh up` before testing

```bash
# CRITICAL: Start services first (tests require running system)
./sutra-deploy.sh up

# Core integration tests (with proper environment)
source venv/bin/activate
PYTHONPATH=packages/sutra-core python -m pytest tests/ -v

# Storage tests (Rust - includes WAL crash recovery tests)
cd packages/sutra-storage
cargo test
cargo test test_wal  # Specific WAL durability tests

# End-to-end demos (require services running)
python demo_simple.py
python demo_end_to_end.py
python demo_mass_learning.py
python demo_wikipedia_learning.py

# Verify storage performance
python verify_concurrent_storage.py

# Grid integration tests
cd packages/sutra-grid-master
./test-integration.sh  # Automated end-to-end (5 tests)

# Manual Grid testing (3 terminals required)
# Terminal 1: Reserved storage for events
cd packages/sutra-storage
./bootstrap-grid-events.sh

# Terminal 2: Grid Master
cd packages/sutra-grid-master
EVENT_STORAGE=localhost:50052 GRID_MASTER_TCP_PORT=7002 cargo run --release

# Terminal 3: Grid Agent  
cd packages/sutra-grid-agent
EVENT_STORAGE=localhost:50052 cargo run --release

# Note: CLI tool is under migration to TCP protocol
# Use Control Center UI at http://localhost:9000/grid for Grid management

# Production Smoke Test (validates embedding configuration)
./scripts/smoke-test-embeddings.sh
# Checks: nomic-embed-text availability, 768-d config, no fallbacks, e2e semantic search
```

### Code Quality
```bash
# Format code (black + isort)
make format

# Lint core package
make lint

# Full quality check
make check  # Runs format, lint, and test
```

### Deployment

**⚡ Single Source of Truth: `./sutra-deploy.sh`**

#### Hybrid + NLG (Docker)
- Build: `docker build -f packages/sutra-hybrid/Dockerfile -t sutra-hybrid:nlg .`
- Run: `docker run --rm -p 8001:8000 -e SUTRA_STORAGE_SERVER=storage-server:50051 -e SUTRA_NLG_ENABLED=true -e SUTRA_NLG_TONE=friendly sutra-hybrid:nlg`

Environment variables:
- `SUTRA_NLG_ENABLED` (default true) — enable grounded NLG post-processing
- `SUTRA_NLG_TONE` — friendly|formal|concise|regulatory

All deployment operations are managed through one script:

```bash
# First-time installation
./sutra-deploy.sh install

# Start all services (10-service ecosystem with embedding service - verified working)
./sutra-deploy.sh up

# Start with bulk ingester (11-service ecosystem)
docker-compose -f docker-compose-grid.yml --profile bulk-ingester up -d

# Stop all services
./sutra-deploy.sh down

# Restart services
./sutra-deploy.sh restart

# Check system status
./sutra-deploy.sh status

# Validate system health (including embedding service)
./sutra-deploy.sh validate

# View logs (all services or specific)
./sutra-deploy.sh logs
./sutra-deploy.sh logs sutra-api

# Interactive maintenance menu
./sutra-deploy.sh maintenance

# Complete cleanup
./sutra-deploy.sh clean
```

**✅ VERIFIED DEPLOYMENT STATUS (2025-10-20):**

| Service | Port | Status | Health | Function |
|---------|------|---------|---------|----------|
│ Storage Server │ 50051 │ ✅ Running │ Healthy │ Core knowledge graph │
│ Grid Event Storage │ 50052 │ ✅ Running │ Healthy │ Grid observability │
│ Sutra API │ 8000 │ ✅ Running │ Healthy │ REST API │
│ Sutra Hybrid │ 8001 │ ✅ Running │ Healthy │ Semantic AI orchestration │
│ **Embedding Service** │ **8888** │ ✅ **Running** │ **Healthy** │ **nomic-embed-text-v1.5 (768-d)** │
│ Control Center │ 9000 │ ✅ Running │ Healthy │ Management UI │
│ Client UI │ 8080 │ ✅ Running │ Healthy │ Interactive interface │
│ Grid Master │ 7001-7002 │ ✅ Running │ Healthy │ Orchestration │
│ Grid Agent 1 │ 8003 │ ✅ Running │ Healthy │ Node management │
│ Grid Agent 2 │ 8004 │ ✅ Running │ Healthy │ Node management │

**End-to-End Verification:**
- ✅ Learning pipeline: POST `/sutra/learn` → concept stored
- ✅ Query pipeline: POST `/sutra/query` → semantic retrieval (89.9% similarity)
- ✅ Embedding service: nomic-embed-text-v1.5 (768-d) operational
- ✅ TCP binary protocol: All services communicating
- ✅ Health checks: All endpoints responding

**Service URLs (after deployment):**
- Sutra Control Center: http://localhost:9000
- Sutra Client (UI): http://localhost:8080  
- Sutra API: http://localhost:8000
- Sutra Hybrid API: http://localhost:8001
- Sutra Embedding Service: http://localhost:8888
- Sutra Bulk Ingester: http://localhost:8005 (Rust service for production data ingestion)
- Grid Master (HTTP Binary Distribution): http://localhost:7001
- Grid Master (TCP Agent Protocol): localhost:7002

**Individual service development** (requires storage server):
```bash
export SUTRA_STORAGE_SERVER=localhost:50051
uvicorn sutra_api.main:app --host 0.0.0.0 --port 8000
```

**⚠️ CRITICAL: Before deploying, verify embedding configuration:**
```bash
# Run production smoke test
./scripts/smoke-test-embeddings.sh

# If tests fail, see PRODUCTION_REQUIREMENTS.md for fixes
```

**See:**
- **`PRODUCTION_REQUIREMENTS.md`** - Mandatory embedding configuration (READ FIRST)
- **`DEPLOYMENT.md`** - Comprehensive deployment documentation

### Build and Distribution

**⚡ Single Source of Truth: `./build-all.sh` + `./sutra-deploy.sh`**

See **BUILD_AND_DEPLOY.md** for complete build and deployment guide.

**Quick Build (All 9 Services):**
```bash
# Build all Docker images (ZERO failures required)
./build-all.sh

# Expected result: 9/9 services built successfully
# - Storage Server (~166 MB)
# - API (~275 MB)
# - Hybrid (~531 MB)
# - Client (~83 MB)
# - Control Center (~387 MB)
# - Grid Master (~148 MB)
# - Grid Agent (~146 MB)
# - Bulk Ingester (~245 MB)
# - Embedding Service (~1.32 GB)
# Total: ~3.3 GB
```

**Official Base Images (NO custom images):**
- `python:3.11-slim` - Python runtime
- `rust:1.82-slim` - Rust compiler (v1.82 required for indexmap compatibility)
- `node:18-slim` - Node.js runtime
- `nginx:alpine` - Web server
- `debian:bookworm-slim` - Minimal Linux runtime

**Build Verification:**
```bash
# Verify all 9 images built
docker images | grep "^sutra" | wc -l
# Expected: 9

# Check image sizes
docker images | grep "^sutra"
```

**Production Deployment:**
```bash
# Deploy complete system
./sutra-deploy.sh up

# Verify health
./sutra-deploy.sh status
```

**See BUILD_AND_DEPLOY.md for:**
- Complete troubleshooting guide
- Development workflow
- Clean rebuild procedures
- Production requirements

## Key Components

### ReasoningEngine (sutra-core)
The main AI interface that orchestrates reasoning components:
- Natural language query processing
- Real-time learning without retraining
- Query result caching for performance  
- Multi-path plan aggregation (MPPA) for consensus
- Complete audit trails

### PathFinder (sutra-core/reasoning)
Advanced graph traversal with multiple search strategies:
- **Best-first**: Confidence-optimized with heuristics
- **Breadth-first**: Shortest path exploration
- **Bidirectional**: Optimal path finding from both ends
- Confidence decay (0.85 default) for realistic propagation
- Cycle detection and path diversification

### MultiPathAggregator (MPPA) (sutra-core/reasoning)
Consensus-based reasoning that prevents single-path derailment:
- Path clustering by answer similarity (0.8 threshold)
- Majority voting with configurable thresholds
- Diversity bonus for varied reasoning approaches
- Robustness analysis with multiple metrics

### SutraAI (sutra-hybrid)
High-level interface combining graph reasoning with semantic embeddings:
- Optional semantic similarity matching
- Multi-strategy comparison (graph-only vs semantic-enhanced)
- Agreement scoring between strategies
- Knowledge persistence and audit trails

### ConcurrentStorage (sutra-storage - Rust)
Production-ready storage architecture with enterprise-grade durability:
- **57,412 writes/sec** (25,000× faster than JSON baseline)
- **<0.01ms read latency** (zero-copy memory-mapped files)
- **Zero data loss** with Write-Ahead Log (WAL) integration
- Lock-free write log with background reconciliation
- Single-file storage (`storage.dat`) with 512MB initial size
- Immutable read snapshots for burst-tolerant performance
- Path finding and graph traversal (BFS)
- Crash recovery with automatic WAL replay
- 100% test pass rate with verified accuracy

**Durability Guarantees:**
- Every write logged to WAL before in-memory structures
- Automatic crash recovery on startup
- WAL checkpoint on flush (safe truncation)
- RPO (Recovery Point Objective): 0 (zero data loss)
- Tested with comprehensive crash simulation tests

### Sutra Control Center (sutra-control)
Modern React-based monitoring and management interface with **Complete UI Integration**:
- **Frontend**: React 18 with Material Design 3, TypeScript, and Vite
- **Backend**: Secure FastAPI gateway providing REST APIs for all services
- **Real-time Updates**: Live system metrics and performance monitoring
- **Grid Management**: Complete web UI for Grid agents and storage nodes ✅
- **Bulk Ingester UI**: Integrated web interface for high-performance data ingestion ✅
- **Navigation**: Full sidebar with Dashboard, Components, Analytics, Knowledge Graph, Reasoning Engine, Bulk Ingestion, Grid Management, and Settings
- **Grid API**: REST endpoints for spawn/stop operations, status monitoring ✅
- **Features**: System health monitoring, performance metrics, knowledge graph visualization, Grid cluster management, bulk data ingestion interface
- **Architecture**: Multi-stage Docker build combining React SPA with Python gateway
- **Access**: http://localhost:9000 (containerized deployment)
- **Grid UI**: Accessible at http://localhost:9000/grid with real-time monitoring
- **Bulk Ingester UI**: Accessible at http://localhost:9000/bulk-ingester with ingestion management

### Sutra Storage Explorer (sutra-explorer)
Standalone application for deep exploration and visualization of storage.dat files **independently** from running Sutra services:
- **Read-Only**: Safe exploration without modification risk
- **Rust Parser**: Zero-copy binary parser for storage.dat v2 format (concepts, edges, vectors)
- **FastAPI Backend**: REST API with 10+ endpoints for exploration (port 8100)
- **React Frontend**: Interactive UI with graph visualization, search, path finding
- **Graph Features**: BFS pathfinding, N-hop neighborhoods, force-directed visualization (D3.js)
- **Vector Operations**: Cosine similarity between concept embeddings
- **Full-Text Search**: Content substring matching with highlighting
- **Architecture**: Multi-stage Docker build (Rust → React → Python)
- **Deployment**: Standalone container with volume mounting for storage files
- **Access**: http://localhost:8100 (API), http://localhost:3000 (UI)
- **Use Cases**: Debugging storage files, offline analysis, data auditing, knowledge graph visualization
- **Documentation**: See `packages/sutra-explorer/README.md` for complete guide
### Configuration

### Environment Variables
```bash
# Storage server address (all services)
export SUTRA_STORAGE_SERVER="storage-server:50051"

# Grid Master address (for Control Center integration)
export SUTRA_GRID_MASTER="localhost:7000"

# Service ports
export SUTRA_API_PORT="8000"
export SUTRA_HYBRID_PORT="8001"
export SUTRA_CLIENT_PORT="8080"
export SUTRA_CONTROL_PORT="9000"
export SUTRA_MARKDOWN_PORT="8002"
export SUTRA_EMBEDDING_SERVICE_PORT="8888"
export SUTRA_STORAGE_PORT="50051"

# Rate limits
export SUTRA_RATE_LIMIT_LEARN="30"
export SUTRA_RATE_LIMIT_REASON="60"

# Production settings
export ENVIRONMENT="production"
export PYTHONPATH=/app:$PYTHONPATH
```

### Reasoning Configuration

### Grounded NLG Configuration (Hybrid)
- Grounded responses only (no LLM); template-driven
- Optional parameters on `/sutra/query`:
  - `tone`: friendly|formal|concise|regulatory
  - `moves`: e.g., ["define","evidence"]
- Fallback: if NLG fails, Hybrid returns raw retrieved answer
- **Confidence decay**: 0.85 per reasoning step
- **Max reasoning depth**: 6 hops
- **Consensus threshold**: 50% agreement for multi-path aggregation
- **Path similarity threshold**: 0.7 maximum overlap for diversification

## Performance Characteristics

Based on production testing with ConcurrentStorage:
- **Learning**: **0.02ms per concept** (57,412 concepts/sec) — 25,000× faster than old system
- **Query**: **<0.01ms** with zero-copy memory-mapped access
- **Path finding**: ~1ms for 3-hop BFS traversal
- **Memory**: ~0.1KB per concept (excluding embeddings)
- **Storage**: Single `storage.dat` file (512MB for 1K concepts)
- **Vector search**: Product quantization for 4× compression (in development)
- **Accuracy**: 100% verified with comprehensive test suite

## Testing Strategy

The system has comprehensive testing at multiple levels:

1. **Production smoke test**: `scripts/smoke-test-embeddings.sh` (validates embedding configuration)
2. **Integration tests**: `tests/` directory with cross-package functionality
3. **End-to-end demos**: `demo_simple.py`, `demo_end_to_end.py`, `demo_mass_learning.py`
4. **API tests**: `packages/sutra-api/tests/` and `packages/sutra-hybrid/tests/`
5. **Performance verification**: `verify_concurrent_storage.py` (production benchmarks)
6. **Storage tests**: Rust unit and integration tests in `packages/sutra-storage/`

### Test Locations
- **`scripts/smoke-test-embeddings.sh`**: Production validation (model, config, e2e semantic search)
- **Root `tests/`**: Integration tests and query processor tests
- **`demo_*.py`**: End-to-end workflow demonstrations
- **`verify_concurrent_storage.py`**: Performance benchmarking (57K writes/sec verified)
- **Package-specific tests**: In respective `packages/*/tests/` directories

**⚠️ Before every deployment, run:** `./scripts/smoke-test-embeddings.sh`

## Common Development Tasks

### Adding New NLG Templates (sutra-nlg)
1. Add a template in `packages/sutra-nlg/sutra_nlg/templates.py` or persist as concepts in storage (future)
2. Include tone, moves, and pattern with slots
3. Validate via `pytest packages/sutra-nlg`
4. Rebuild Hybrid Docker image to include changes

### Adding New Reasoning Strategies
1. Implement in `sutra_core/reasoning/paths.py`
2. Add to PathFinder class
3. Update QueryProcessor to support new strategy
4. Add comprehensive tests

### Extending Storage Format
1. Update Rust structures in `packages/sutra-storage/src/`
2. Modify Python bindings via PyO3 in `lib.rs`
3. Update memory layout documentation in `docs/sutra-storage/`
4. Run `cargo build --release` and verify with `verify_concurrent_storage.py`
5. Ensure single-file `storage.dat` format compatibility

### Adding API Endpoints
1. Define request/response models in `sutra_api/models.py`
2. Implement endpoint in `sutra_api/main.py`
3. Add rate limiting configuration
4. Update OpenAPI documentation

### Extending Control Center
1. **Frontend Changes**: Edit React components in `packages/sutra-control/src/`
2. **Backend Changes**: Update FastAPI gateway in `packages/sutra-control/backend/main.py`
3. **Build Process**: Run `npm run build` to create production build
4. **Docker**: Rebuild container with `docker build -t sutra-control:latest .`
5. **Testing**: Access at http://localhost:9000 after `docker compose up -d`

### Extending Sutra Grid
1. **Adding New Event Types**: Update `packages/sutra-grid-events/src/events.rs` with new event variants
2. **Master Changes**: Edit orchestration logic in `packages/sutra-grid-master/src/main.rs`
3. **Agent Changes**: Edit node management in `packages/sutra-grid-agent/src/main.rs`
4. **Protocol Updates**: Modify message types in `packages/sutra-protocol/src/lib.rs`
5. **Testing**: Run Docker compose and verify with `docker logs` commands

**Grid Event Flow**: Master/Agent → EventEmitter → Async Worker → TCP Binary Protocol → Sutra Storage (port 50052)

**Grid Control Center Integration**: ✅ **COMPLETED**
- ✅ React Grid dashboard with agent/node topology view
- ✅ REST API endpoints for all Grid operations  
- ✅ Real-time monitoring and status updates
- ✅ Interactive spawn/stop operations via web UI
- ✅ Comprehensive documentation and troubleshooting guides

**Future Enhancements**:
- Natural language queries for events ("Show me all crashed nodes today")
- Advanced visualizations and network topology diagrams
- Automated operations and auto-scaling capabilities

### Using Sutra Storage Explorer
1. **Standalone Deployment**: `cd packages/sutra-explorer && docker-compose up -d`
2. **Mount Storage File**: Set `STORAGE_FILE_PATH` environment variable to your storage.dat location
3. **Access UI**: Navigate to http://localhost:3000 for interactive exploration
4. **API Access**: REST API available at http://localhost:8100/docs
5. **Features**: Search concepts, visualize graphs, find paths, calculate similarities
6. **Development**: See `packages/sutra-explorer/README.md` for local development setup
7. **Use Cases**: Debugging storage issues, auditing knowledge graphs, offline analysis

## Troubleshooting

### Quick Diagnostics

**Run the production smoke test first:**
```bash
./scripts/smoke-test-embeddings.sh
```
This validates:
- ✅ nomic-embed-text model availability
- ✅ Storage server 768-d configuration
- ✅ Hybrid service embedding configuration
- ✅ No fallback warnings
- ✅ End-to-end semantic search

**If smoke test fails, see `PRODUCTION_REQUIREMENTS.md` for detailed fixes.**

---

### Import Errors
```bash
# Solution: Set PYTHONPATH for core tests
PYTHONPATH=packages/sutra-core python -m pytest tests/ -v

# Or run integration tests from root
python -m pytest tests/ -v
```

### Virtual Environment Issues
```bash
# Recreate environment
python3 -m venv venv
source venv/bin/activate
pip install -e packages/sutra-core/
pip install -r requirements-dev.txt
```

### Storage/Persistence Issues
- **ConcurrentStorage**: Data written to single `storage.dat` file
- **WAL (Write-Ahead Log)**: All writes logged to `wal.log` for durability
- Check storage path permissions (default: `./knowledge/storage.dat`)
- Call `storage.flush()` manually before shutdown
- Auto-flush triggers at 50K concepts (configurable)
- Verify performance with `verify_concurrent_storage.py`
- Monitor stats: `storage.stats()` shows writes, drops, concepts, edges
- WAL automatically replays on startup for crash recovery
- WAL is checkpointed (truncated) after successful flush

### Same Answer for All Questions ⭐ CRITICAL

**Symptoms:** Every query returns identical answer, regardless of question.

**Root Cause:** Zero embeddings in storage (concepts learned without embedding generation).

**Quick Diagnosis:**
```bash
curl -s http://localhost:8000/stats | jq '.total_embeddings'
# If 0: System is non-functional
```

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

**Prevention:**
- Always learn via `/sutra/learn` (Hybrid) not `/learn` (API)
- Verify embedding service is running BEFORE learning data
- Check `total_embeddings` matches `total_concepts`

**See:**
- **`PRODUCTION_REQUIREMENTS.md`** - Strict embedding requirements and migration guide
- **`docs/EMBEDDING_TROUBLESHOOTING.md`** - Detailed troubleshooting
- **`TROUBLESHOOTING.md`** - General troubleshooting

### TCP Adapter Issues (Fixed 2025-10-19)

**Symptoms:**
- "Storage-backed pathfinding failed: no attribute 'find_paths'"
- "list indices must be integers or slices, not str"
- "Failed to get neighbors via TCP"

**Root Cause:** Missing methods in `TcpStorageAdapter` and incorrect response parsing.

**Fixed Methods:**
- ✅ `find_paths()` - Multi-path reasoning support
- ✅ `get_association()` - Association retrieval for path building
- ✅ `get_all_concept_ids()` - Health check support
- ✅ `get_neighbors()` - Fixed response parsing (dict vs list)

**TCP Client Fixes:**
- ✅ Unit variant handling (`GetStats`, `Flush`, `HealthCheck` send strings, not dicts)
- ✅ Response parsing for nested list formats
- ✅ Numpy array to list conversion before TCP transport

**Verification:**
```bash
# Restart services to pick up fixes
docker restart sutra-api sutra-hybrid sutra-client

# Test pathfinding
curl -X POST http://localhost:8001/sutra/query \
  -H "Content-Type: application/json" \
  -d '{"query":"test query"}'
# Should not show "pathfinding failed" warnings
```

### Embedding Configuration Issues ⚠️ PRODUCTION-CRITICAL

**Symptoms:**
- Wrong query results despite correct storage ("tallest mountain" → "Pacific Ocean")
- "Dimension mismatch: expected 768, got 384"
- "Query embedding FALLBACK to spaCy" in logs
- Semantic search returns irrelevant results

**Root Cause:** Environment variable mismatch between services.

**Quick Fix:**
```bash
# 1. Run production smoke test to diagnose
./scripts/smoke-test-embeddings.sh

# 2. Verify configuration
docker logs sutra-storage | grep -E "(Vector dimension|nomic)"
docker logs sutra-hybrid | grep -E "(PRODUCTION|nomic|fallback)"

# 3. If configuration is wrong, fix docker-compose-grid.yml:
#    storage-server:
#      environment:
#        - VECTOR_DIMENSION=768
#        - SUTRA_EMBEDDING_MODEL=nomic-embed-text
#    sutra-hybrid:
#      environment:
#        - SUTRA_EMBEDDING_MODEL=nomic-embed-text
#        - SUTRA_VECTOR_DIMENSION=768

# 4. Clean and restart
./sutra-deploy.sh down
docker volume rm sutra-models_storage-data
./sutra-deploy.sh up

# 5. Verify fix
./scripts/smoke-test-embeddings.sh
```

**Prevention:**
- **ALWAYS** set `SUTRA_EMBEDDING_MODEL=nomic-embed-text` for storage AND hybrid
- **ALWAYS** set `VECTOR_DIMENSION=768` for storage
- **ALWAYS** run smoke test before deploying
- **NEVER** allow fallback embeddings (spaCy, TF-IDF, sentence-transformers)

**See `PRODUCTION_REQUIREMENTS.md` for complete details and incident postmortem.**

### Common Deployment Issues (2025-10-20)

#### Docker Build Failures
**Symptoms:** "failed to solve: image already exists" during builds

**Solution:**
```bash
# Remove problematic images
docker rmi sutra-storage-server:latest || true

# Rebuild specific service
docker-compose -f docker-compose-grid.yml build storage-server

# Or rebuild all services
./sutra-deploy.sh build
```

#### Port Conflicts (11434 - Ollama)
**Symptoms:** "bind: address already in use" on port 11434

**Solution:**
```bash
# Check what's using the port
lsof -i :11434

# Stop local Ollama if running
killall ollama

# Or modify docker-compose to use different port mapping
# ports:
#   - "11435:11434"  # Map to different external port
```

#### Services Not Starting
**Symptoms:** Empty status output from `./sutra-deploy.sh status`

**Solution:**
```bash
# Check Docker daemon
docker info

# Check for conflicting containers
docker ps -a | grep sutra

# Clean up and restart
docker-compose -f docker-compose-grid.yml down
docker system prune -f
./sutra-deploy.sh up
```

#### Health Check Failures
**Symptoms:** Services show "health: starting" or "unhealthy"

**Solution:**
```bash
# Check service logs
docker logs sutra-hybrid --tail 50

# Wait for Ollama model download (can take 5-10 minutes)
docker logs sutra-ollama | grep -E "(pulling|success)"

# Verify model availability
curl -s http://localhost:11434/api/tags | jq '.models[].name'
# Should show: "nomic-embed-text:latest"
```

#### Test Failures Due to Missing Services
**Symptoms:** "Connection refused" errors in tests

**Solution:**
```bash
# ALWAYS start services before testing
./sutra-deploy.sh up

# Wait for health checks to pass
./sutra-deploy.sh status

# Then run tests
source venv/bin/activate
PYTHONPATH=packages/sutra-core python -m pytest tests/ -v
```

## Code Style

- **Line length**: 88 characters (black default)
- **Import order**: stdlib, third-party, local (isort with black profile)
- **Type hints**: Required for all public functions
- **Docstrings**: Google style for all public classes and methods
- **Testing**: pytest with descriptive test names and docstrings

## Research Foundation

Built on published research:
- **Adaptive Focus Learning**: "LLM-Oriented Token-Adaptive Knowledge Distillation" (Oct 2024)
- **Multi-Path Plan Aggregation (MPPA)**: Consensus-based reasoning
- **Graph-based reasoning**: Decades of knowledge representation research

No proprietary techniques - all methods are from published work.
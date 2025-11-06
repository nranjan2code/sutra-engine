# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Important Note on Image Naming

This deployment uses the `sutra-works-` prefix for all Docker images to avoid conflicts with other Sutra deployments. See `docs/deployment/IMAGE_NAMING.md` for details. All `./sutra` CLI commands automatically use the correct image naming.

## Project Overview

Sutra AI is a **production-grade, domain-specific explainable AI platform** providing semantic reasoning over private knowledge graphs. Unlike general-purpose LLMs, Sutra starts empty and learns exclusively from your proprietary data with complete audit trails and millisecond-level query responses.

**Core Value:** 1000× smaller models, 100× lower cost, 100% explainable reasoning paths for regulated industries (healthcare, finance, legal) and knowledge-intensive domains (DevOps observability, enterprise knowledge management).

## Build & Test Commands

### Build System

```bash
# Build all services (unified entry point)
sutra build                         # Builds all services for current SUTRA_EDITION
SUTRA_EDITION=enterprise sutra build  # Build all enterprise services (10 services)

# Build specific services
sutra build storage api             # Build only storage and api

# Check build status
sutra status                        # Show running containers and built images

# Validate built images
sutra validate                      # Verify all essential images exist
```

### Testing

```bash
# Unit tests (Python)
PYTHONPATH=packages/sutra-core python -m pytest tests/ -v

# Run specific test markers
pytest -v -m unit                   # Fast unit tests only
pytest -v -m integration            # Integration tests (requires services running)

# Rust tests (storage engine, WAL recovery, 2PC transactions)
cd packages/sutra-storage && cargo test

# Production smoke tests (validates embedding services and HTTP health endpoints)
sutra test smoke

# Integration tests (validates Docker deployment)
sutra test integration

# Scripts-based testing
./scripts/smoke-test-embeddings.sh  # Comprehensive embedding service validation
./scripts/integration-test.sh       # End-to-end workflow testing
```

### Rust Development

```bash
# Build storage engine
cd packages/sutra-storage
cargo build --release               # Production build (optimized)
cargo build                         # Debug build (faster compilation)

# Run storage server binary
cargo run --bin storage-server

# Check workspace
cargo check --workspace             # Check all workspace packages
cargo clippy --workspace            # Linting

# Run specific test
cargo test test_wal_recovery
```

### Deployment

```bash
# Build nginx proxy (required for all deployments)
cd .sutra/compose
docker build -t sutra-works-nginx-proxy:latest -f nginx/Dockerfile nginx/
cd ../..

# Deploy by edition (profile-based Docker Compose)
export SUTRA_EDITION=simple    # or community, enterprise
export SUTRA_VERSION=latest
docker-compose -f .sutra/compose/production.yml --profile simple up -d

# Production deployment with security
export SUTRA_SECURE_MODE=true
export SUTRA_EDITION=enterprise
export SUTRA_SSL_CERT_PATH=/path/to/cert.pem
export SUTRA_SSL_KEY_PATH=/path/to/key.pem
docker-compose -f .sutra/compose/production.yml --profile enterprise up -d

# Stop services
docker-compose -f .sutra/compose/production.yml down

# View logs (note: all containers use sutra-works- prefix)
docker logs sutra-works-storage         # Storage server logs
docker logs sutra-works-api             # API service logs
docker logs sutra-works-nginx-proxy     # Nginx proxy logs

# Access services through nginx proxy
curl http://localhost:8080/api/health       # API via proxy
curl http://localhost:8080/sutra/health     # Hybrid service via proxy
open http://localhost:8080/                 # Main UI via proxy
```

## Architecture Overview

### Multi-Language Workspace

- **Rust Workspace** (packages/sutra-storage, sutra-protocol, sutra-grid-*, sutra-bulk-ingester)
  - Storage engine: Log-structured, HNSW vector indexing, WAL for durability
  - TCP binary protocol: Custom MessagePack-based (NOT gRPC)
  - Grid infrastructure: Distributed orchestration and sharding
  - Bulk ingestion: High-throughput data loading

- **Python Packages** (packages/sutra-core, sutra-api, sutra-hybrid, sutra-client, sutra-control)
  - Reasoning engine: Multi-Path Plan Aggregation (MPPA), graph traversal
  - REST APIs: FastAPI-based services
  - ML orchestration: Embedding and NLG service coordination
  - Web UIs: Streamlit client, React control center

### Core Components

**Storage Server (Rust)** - Port 50051 TCP
- **ConcurrentMemory**: Lock-free write log + immutable read views
- **Write-Ahead Log (WAL)**: Zero data loss, automatic crash recovery
- **Unified Learning Pipeline**: Server owns complete learning process:
  1. Embedding generation (calls HA service at :8888)
  2. Semantic analysis (rule-based + pattern matching)
  3. Association extraction (NLP + ML-based relationship discovery)
  4. Atomic storage (2PC for cross-shard writes)
- **HNSW Vector Index**: USearch with mmap persistence (94× faster startup vs rebuild)
- **Sharded Storage**: 4-16 shards for 10M+ concepts with 2PC transactions

**Reasoning Engine (Python)**
- `sutra-core`: Graph traversal, MPPA consensus reasoning, query processing
- Real-time learning without retraining
- Quality gates: "I don't know" responses when confidence < threshold

**ML Foundation Services**
- **Embedding Service** (:8888 HAProxy → :8889-8891 replicas)
  - Model: nomic-embed-text-v1.5 (768-dimensional vectors)
  - HA architecture: 3 replicas + load balancer
- **NLG Service** (:8890)
  - Grounded text generation with strict/balanced/creative modes
  - Edition-aware scaling (Simple/Community/Enterprise)

**API Layer**
- `sutra-api` (:8000): Primary REST API (FastAPI)
- `sutra-hybrid` (:8001): Semantic orchestration service
- `sutra-bulk-ingester` (:8005): High-throughput Rust-based ingestion

**Web Interfaces**
- `sutra-control` (:9000): React + Material UI control center
- `sutra-client` (:8080): Streamlit interactive query interface
- `sutra-explorer` (:8100): React + D3.js storage visualization

### Data Flow Pattern: Unified Learning

**CRITICAL**: All learning MUST route through the storage server's TCP `learn_concept()` method. Never bypass this pipeline.

```rust
// packages/sutra-storage/src/learning_pipeline.rs
pub async fn learn_concept(&mut self, content: String) -> Result<ConceptId> {
    // 1. Generate embedding via HA service
    let embedding = self.embedding_client.generate(content).await?;

    // 2. Extract associations (semantic extractor)
    let associations = self.semantic_extractor.extract(&content)?;

    // 3. Store atomically (2PC if sharded)
    self.store_atomically(content, embedding, associations).await
}
```

**Why this matters:**
- Single source of truth for learning
- Automatic embeddings for ALL ingestion paths
- Zero code duplication across clients
- Guaranteed consistency with ACID semantics

### TCP Binary Protocol (NOT gRPC)

Sutra uses a custom MessagePack-based binary protocol for 10-50× better performance than gRPC/REST.

**Architectural Policy**: Sutra will NEVER support SQL, Cypher, or GraphQL. See `docs/architecture/NO_SQL_POLICY.md` for rationale.

```rust
// packages/sutra-protocol/src/messages.rs
#[derive(Serialize, Deserialize)]
pub enum StorageRequest {
    LearnConcept { content: String, metadata: Option<ConceptMetadata> },
    QueryGraph { query: String, max_paths: u32 },
    GetConcept { concept_id: ConceptId },
}
```

### Security Modes

**Development (default)**: NO authentication, NO encryption - localhost only
```bash
sutra deploy  # Unsafe for production
```

**Production**: TLS 1.3, HMAC-SHA256, RBAC, network isolation
```bash
SUTRA_SECURE_MODE=true sutra deploy
docker logs sutra-storage | grep "Authentication: ENABLED"  # Verify
```

Security is **fully integrated** as of October 2025. See `docs/security/PRODUCTION_SECURITY_SETUP.md`.

## Key Architectural Patterns

### Edition System

```bash
export SUTRA_EDITION=simple|community|enterprise
```

**Impact:**
- **Simple**: 8 services, single storage instance, lower rate limits (FREE)
- **Community**: 8 services, HA embedding (3 replicas), 10× higher limits ($99/mo)
- **Enterprise**: 10 services (+ grid-master, grid-agent), 16 shards, 100× limits ($999/mo)

Edition affects: service count, shard configuration, embedding model selection, rate limits, resource allocation.

### Cargo Workspace Structure

```
Cargo.toml (workspace root)
├── packages/sutra-storage/        # Core storage engine + binary
├── packages/sutra-protocol/       # TCP protocol definitions
├── packages/sutra-grid-master/    # Grid orchestration
├── packages/sutra-grid-agent/     # Node management
├── packages/sutra-grid-events/    # Event library (26 event types)
└── packages/sutra-bulk-ingester/  # High-throughput ingestion
```

Shared dependencies defined in `[workspace.dependencies]` for consistency.

### Version Management

**Single source of truth**: `VERSION` file (currently 3.0.0)

```bash
# Check version
sutra version                     # Show current version
cat VERSION                       # Direct file access (e.g., 3.0.0)

# Create releases (manual VERSION file update)
echo "3.0.1" > VERSION           # Bug fix: 3.0.0 → 3.0.1
echo "3.1.0" > VERSION           # Feature: 3.0.0 → 3.1.0
echo "4.0.0" > VERSION           # Breaking: 3.0.0 → 4.0.0

# Commit, tag, and push to trigger automated builds
git add VERSION
git commit -m "Release v3.0.1"
git tag -a v3.0.1 -m "Release version 3.0.1"
git push origin main --tags
```

See `docs/release/` for complete release management documentation.

### Self-Monitoring Architecture (Revolutionary)

Sutra monitors itself using its own reasoning engine - NO external tools (Prometheus, Grafana, Datadog).

**26 Production Event Types** (`packages/sutra-grid-events/src/events.rs`):
- Agent lifecycle: Registered, Heartbeat, Degraded, Offline, Recovered, Unregistered
- Node lifecycle: SpawnRequested/Succeeded/Failed, StopRequested/Succeeded/Failed, Crashed, Restarted
- Performance: StorageMetrics, QueryPerformance, EmbeddingLatency, HnswIndexBuilt, PathfindingMetrics

**Natural Language Operational Queries:**
```python
"Show cluster status"
"What caused the 2am crash?"  # Returns complete causal chain in ~30ms
"Which agents went offline this week?"
"Why did node-abc123 crash?"  # Automatic root cause analysis
"Which storage node is slowest?"
```

See `docs/sutra-platform-review/DEVOPS_SELF_MONITORING.md` for complete case study.

## Testing Considerations

### Test Markers

```python
@pytest.mark.unit           # Fast unit tests
@pytest.mark.integration    # Requires services running
@pytest.mark.slow           # Resource-intensive tests
```

### Coverage Requirements

- Minimum coverage: 70% (enforced by pytest.ini)
- Reports: HTML (htmlcov/), XML (coverage.xml), terminal
- Coverage command: `pytest --cov=packages/sutra-core --cov=packages/sutra-api`

### WAL Testing (Rust)

Storage engine tests include crash recovery scenarios:
```bash
cd packages/sutra-storage
cargo test test_wal_recovery       # Tests write-ahead log replay
cargo test test_2pc_transaction    # Tests cross-shard atomicity
```

## Important File Locations

**Build & Deploy:**
- `./sutra` - Unified Python CLI (single entry point for all operations)
- `./sutra-optimize.sh` - Backend build orchestration (called by ./sutra)
- `.sutra/compose/production.yml` - Docker Compose file with edition profiles
- `VERSION` - Single source of truth for versioning

**Storage Engine (Rust):**
- `packages/sutra-storage/src/lib.rs` - Module exports and definitions
- `packages/sutra-storage/src/concurrent_memory.rs` - Lock-free core
- `packages/sutra-storage/src/learning_pipeline.rs` - Unified learning pipeline
- `packages/sutra-storage/src/wal.rs` - Write-Ahead Log
- `packages/sutra-storage/src/transaction.rs` - 2PC coordinator
- `packages/sutra-storage/src/bin/storage_server.rs` - TCP server binary

**Reasoning Engine (Python):**
- `packages/sutra-core/sutra_core/` - 42 Python modules
- `packages/sutra-core/sutra_core/reasoning.py` - MPPA algorithm
- `packages/sutra-core/sutra_core/graph.py` - Concept/Association data structures

**Documentation Hub:**
- `docs/README.md` - Main navigation with user journeys
- `docs/architecture/SYSTEM_ARCHITECTURE.md` - Complete technical architecture
- `docs/getting-started/quickstart.md` - 5-minute setup guide
- `docs/deployment/README.md` - Deployment guide
- `docs/release/README.md` - Release management

**Scripts:**
- `scripts/smoke-test-embeddings.sh` - Production smoke tests
- `scripts/integration-test.sh` - End-to-end integration tests
- `scripts/ci-validate.sh` - CI validation

## Common Anti-Patterns to Avoid

❌ **Don't bypass unified learning**: All ingestion MUST go through storage server TCP `learn_concept()`
❌ **Don't ignore editions**: Features/performance vary significantly by `SUTRA_EDITION`
❌ **Don't assume security**: Default mode has NO authentication (development only)
❌ **Don't skip semantic versioning**: Update VERSION file, commit, and tag properly (see Version Management section)
❌ **Don't use non-existent scripts**: Use `sutra` command as the single entry point
❌ **Don't position as "regulated industries only"**: Market spans 20+ industries ($200B+ TAM)
❌ **Don't forget temporal/causal reasoning**: Core differentiator vs. vector databases
❌ **Don't use gRPC/SQL/GraphQL**: Sutra uses custom TCP binary protocol exclusively

## Development Workflow

### Single Service Updates (Fast Development)

```bash
# Rebuild and restart specific service (faster than full rebuild)
docker-compose -f .sutra/compose/production.yml up -d --build sutra-api
docker-compose -f .sutra/compose/production.yml up -d --build sutra-embedding-service

# Hot-reload development (Python/React changes apply instantly)
docker-compose -f docker-compose-grid.yml -f docker-compose.dev.yml up
```

### Code Quality

```bash
# Pre-commit hooks (9 checks: black, isort, flake8, bandit, etc.)
pre-commit run --all-files

# Manual formatting
black packages/
isort packages/

# Rust linting
cargo clippy --workspace -- -D warnings
```

## Documentation Structure

User journeys:
1. **New Users**: `docs/getting-started/` → quickstart.md → tutorial.md
2. **Developers**: `docs/build/` → `docs/deployment/` → `docs/architecture/`
3. **DevOps**: `docs/deployment/` → `docs/release/` → `docs/security/`
4. **Contributors**: `docs/guides/DEVELOPMENT.md` → `docs/CONTRIBUTING.md`

## References

- **Architecture**: `docs/architecture/SYSTEM_ARCHITECTURE.md` (100+ lines)
- **Platform Review**: `docs/sutra-platform-review/` (complete technical/market analysis)
- **Build System**: `docs/build/README.md`
- **Deployment**: `docs/deployment/README.md`
- **Release Process**: `docs/release/RELEASE_PROCESS.md`
- **Copilot Instructions**: `.github/copilot-instructions.md` (detailed AI assistant guidance)

## Scale & Performance Targets

- **Startup**: <50ms for 1M vectors (HNSW mmap persistence)
- **Write throughput**: 57K writes/sec
- **Read latency**: <10ms
- **Target scale**: 10M+ concepts across 16 shards
- **Memory efficiency**: <0.1% overhead for self-monitoring events

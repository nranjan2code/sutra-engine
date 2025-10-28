# Sutra AI - AI Assistant Instructions

**Domain-Specific Explainable AI System for Regulated Industries**

## Project Architecture

Sutra AI is a **12-service distributed system** that provides explainable reasoning over domain-specific knowledge graphs. Unlike general LLMs, it starts empty and learns from YOUR proprietary data (medical protocols, legal precedents, financial regulations) with complete audit trails.

### Core Components

**Storage Layer (Rust):**
- `sutra-storage` - High-performance graph storage (57K writes/sec, <0.01ms reads)
- Binary TCP protocol with Write-Ahead Log (WAL) for durability
- USearch HNSW vector indexing with persistent mmap (94x faster startup)
- Cross-shard 2PC transactions, adaptive reconciliation

**Reasoning Engine (Python):**
- `sutra-core` - Graph traversal with Multi-Path Plan Aggregation (MPPA)
- Real-time learning without retraining
- Quality gates: "I don't know" responses when confidence < threshold
- Progressive streaming with confidence-based refinement

**API & Services:**
- `sutra-api` (FastAPI) - REST endpoints on :8000
- `sutra-hybrid` - Semantic orchestration on :8001  
- `sutra-embedding-service` - HA embedding cluster (3 replicas + HAProxy) on :8888
- `sutra-bulk-ingester` (Rust) - High-throughput data ingestion on :8005

## Essential Workflows

### Build System (2025-10-28)
```bash
# Build all services (single :latest tag strategy)
SUTRA_EDITION=simple ./sutra-optimize.sh build-all      # 8 services (4.4GB)
SUTRA_EDITION=enterprise ./sutra-optimize.sh build-all  # 10 services (4.76GB)

# Check what was built
./sutra-optimize.sh sizes

# Build individual service
SUTRA_EDITION=simple ./sutra-optimize.sh build-service storage
```

**Image Tagging:**
- All services: `sutra-<service>:${SUTRA_VERSION:-latest}`
- Single tag strategy (no `:latest-optimized` or intermediate tags)
- Compose file: `.sutra/compose/production.yml`

### Deployment System
```bash
# Deploy by edition (profile-based)
SUTRA_EDITION=simple ./sutra deploy        # 8 services
SUTRA_EDITION=community ./sutra deploy     # HA configuration
SUTRA_EDITION=enterprise ./sutra deploy    # 10 services with grid

# Check status
./sutra status                             # Overall health
docker compose ps                          # Raw Docker status
```

**Service Breakdown:**
- **Simple/Community**: 8 services (storage, api, hybrid, embedding, nlg, bulk-ingester, client, control)
- **Enterprise**: +2 grid services (grid-master, grid-agent)

### Release Management (2025-10-26)
```bash
./sutra-deploy.sh version          # Show current version (2.0.0)
./sutra-deploy.sh release patch    # Bug fix release (2.0.0 → 2.0.1)
./sutra-deploy.sh release minor    # Feature release (2.0.0 → 2.1.0)
./sutra-deploy.sh release major    # Breaking changes (2.0.0 → 3.0.0)
git push origin main --tags        # Trigger automated builds
./sutra-deploy.sh deploy v2.0.1    # Deploy specific version
```

### Development Testing
```bash
# Full integration test
PYTHONPATH=packages/sutra-core python -m pytest tests/ -v

# Production smoke test (validates embeddings)
./scripts/smoke-test-embeddings.sh

# Rust tests (WAL recovery, 2PC transactions)
cd packages/sutra-storage && cargo test
```

### Security Modes
- **Development (default):** No auth/encryption - localhost only
- **Production:** `SUTRA_SECURE_MODE=true` - TLS 1.3, HMAC-SHA256, RBAC
- Security code complete but **NOT YET integrated** into main binary

## Project-Specific Patterns

### Unified Learning Pipeline
**Critical:** Storage server owns complete learning process. ALL data ingestion routes through TCP `learn_concept()`:
```rust
// packages/sutra-storage/src/services/learning_service.rs
pub async fn learn_concept(&mut self, content: String) -> Result<ConceptId> {
    let embedding = self.embedding_client.generate(content).await?;
    let associations = self.extract_associations(&content)?;
    self.store_atomically(content, embedding, associations).await
}
```

### Multi-Language Workspace
- **Rust:** High-performance storage, grid infrastructure, bulk ingestion
- **Python:** AI reasoning, APIs, UIs (except React components)
- **Cargo workspace:** `packages/sutra-{storage,protocol,grid-*,bulk-ingester}`  
- **Python packages:** `packages/sutra-{core,api,hybrid,client,control}`

### Edition System
```bash
SUTRA_EDITION=simple|community|enterprise
# Affects: feature availability, shard count, embedding model selection
# See: docker-compose profiles and license validation
```

### TCP Binary Protocol (Not gRPC)
Custom MessagePack-based protocol for 10-50x lower latency:
```rust
// packages/sutra-protocol/src/messages.rs
#[derive(Serialize, Deserialize)]
pub enum StorageRequest {
    LearnConcept { content: String, metadata: Option<ConceptMetadata> },
    QueryGraph { query: String, max_paths: u32 },
    GetConcept { concept_id: ConceptId },
}
```

## Integration Points

### Embedding Service HA Architecture
- HAProxy load balancer (`:8888`) → 3 embedding replicas (`:8889-8891`)
- Automatic failover, health checks every 30s
- Storage server connects via `SUTRA_EMBEDDING_SERVICE_URL`

### Cross-Shard Coordination  
- Grid Master (`:7001`) orchestrates 4-16 storage shards
- 2PC coordinator for atomic writes across shards
- Parallel vector search queries all shards simultaneously

### Self-Monitoring with Grid Events
System monitors itself using its own reasoning engine:
```python
# Query operational state in natural language
"Which services are consuming the most CPU?"
"Show me failed requests in the last hour"
# Answers come from Grid Events stored in concept graph
```

## Critical File Locations

- **Build script:** `./sutra-optimize.sh` (929 lines, single-tag build system)
- **Main deployment:** `./sutra-deploy.sh` (1100+ lines, handles releases)
- **Version control:** `VERSION` (single source of truth, currently 2.0.0)
- **Compose file:** `.sutra/compose/production.yml` (unified, profile-based)
- **Storage engine:** `packages/sutra-storage/src/` (14K+ LOC Rust)
- **Reasoning core:** `packages/sutra-core/sutra_core/` (42 Python modules)
- **Architecture docs:** `WARP.md` (AI assistant guidance, 1600+ lines)
- **Documentation hub:** `docs/README.md` (navigation, user journeys)
- **Build docs:** `docs/build/README.md` (build system guide)
- **Deployment docs:** `docs/deployment/README.md` (deployment guide)
- **Release docs:** `docs/release/README.md` (release management)
- **Smoke tests:** `scripts/smoke-test-embeddings.sh` (production validation)

## Documentation Structure (2025-10-28)

**Organized by User Journey:**
```
docs/
├── README.md                    # Main hub with navigation
├── getting-started/             # New users start here
│   ├── README.md               # Prerequisites, quickstart
│   ├── quickstart.md           # 5-minute setup
│   ├── editions.md             # Edition comparison
│   └── tutorial.md             # Complete walkthrough
├── build/                       # Building services
│   ├── README.md               # Build hub
│   └── building-services.md    # Detailed build guide
├── deployment/                  # Deploying services
│   ├── README.md               # Complete deployment guide
│   ├── docker-compose.md       # Compose details
│   └── editions/               # Edition configs
├── release/                     # Release management
│   ├── README.md               # Release guide
│   └── RELEASE_PROCESS.md      # Complete workflow
├── architecture/                # Technical deep dives
│   ├── SYSTEM_ARCHITECTURE.md
│   ├── storage/
│   ├── ml-foundation/
│   └── protocols/
└── guides/                      # Feature guides
```

**User Journeys:**
1. **New Users**: getting-started/ → quickstart.md → tutorial.md
2. **Developers**: build/ → deployment/ → architecture/
3. **DevOps**: deployment/ → release/ → monitoring
4. **Contributors**: guides/ → architecture/ → development/

## Release Management System

### Overview
Professional version control for customer deployments with centralized versioning and automated builds.

### Key Files
- **VERSION** - Single source of truth for all package versions (2.0.0)
- **sutra-deploy.sh** - Release commands (version, release, deploy)
- **docker-compose-grid.yml** - All services use `${SUTRA_VERSION:-latest}`
- **.github/workflows/release.yml** - Automated builds on tag push
- **docs/release/** - Complete documentation (5 docs, 44KB)

### Common Release Tasks

**Check version:**
```bash
./sutra-deploy.sh version
cat VERSION  # 2.0.0
```

**Create release:**
```bash
# Bug fixes
./sutra-deploy.sh release patch    # 2.0.0 → 2.0.1

# New features
./sutra-deploy.sh release minor    # 2.0.0 → 2.1.0

# Breaking changes
./sutra-deploy.sh release major    # 2.0.0 → 3.0.0
```

**What happens:**
1. VERSION file updated
2. README.md badge updated
3. Git commit created
4. Git tag created (e.g., v2.0.1)
5. User pushes: `git push origin main --tags`
6. GitHub Actions builds all images
7. Docker images tagged and pushed to registry

**Deploy version:**
```bash
./sutra-deploy.sh deploy v2.0.1
```

### Semantic Versioning
```
MAJOR.MINOR.PATCH
  │     │     │
  │     │     └─ Bug fixes (backward compatible)
  │     └─────── New features (backward compatible)
  └───────────── Breaking changes
```

### Documentation
- **docs/release/README.md** - Overview and navigation
- **docs/release/RELEASE_PROCESS.md** - Complete workflow (500+ lines)
- **docs/release/QUICK_REFERENCE.md** - Command cheat sheet
- **docs/release/VERSIONING_STRATEGY.md** - When to bump versions
- **docs/release/SETUP_COMPLETE.md** - Implementation details

## Common Anti-Patterns to Avoid

- **Don't bypass unified learning:** All ingestion MUST go through storage server TCP protocol
- **Don't use generic AI advice:** This is domain-specific reasoning, not general chatbot
- **Don't ignore editions:** Features/performance vary significantly by SUTRA_EDITION
- **Don't assume security:** Default mode has NO authentication (development only)
- **Don't forget WAL:** Storage operations are transactional with crash recovery
- **Don't manually edit VERSION:** Use `./sutra-deploy.sh release <type>` for version bumps
- **Don't skip release docs:** See `docs/release/` for complete release workflow

## Performance Expectations

- **Learning:** 57K concepts/sec  
- **Query:** <0.01ms (zero-copy mmap)
- **Startup:** 3.5ms for 1M vectors (persistent HNSW)
- **Memory:** ~0.1KB per concept (excluding embeddings)
- **Target scale:** 10M+ concepts across 16 shards

When working on this codebase, prioritize explainability, audit trails, and domain-specific accuracy over general AI capabilities. Every reasoning path should be traceable for compliance requirements.
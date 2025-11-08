# Sutra AI - AI Assistant Instructions

**General-Purpose Semantic Reasoning Engine with Temporal, Causal, and Explainable AI**

## Project Architecture

Sutra AI is a **12-service distributed system** that provides explainable reasoning over domain-specific knowledge graphs. Unlike general LLMs, it starts empty and learns from YOUR proprietary data with complete audit trails. Sutra combines semantic understanding, temporal reasoning, causal analysis, and explainability for ANY knowledge-intensive industry - not just regulated industries.

**Key Capabilities:**
- ✅ **Temporal Reasoning** (before/after/during relationships)
- ✅ **Causal Understanding** (X causes Y, multi-hop chains, root cause analysis)
- ✅ **Semantic Classification** (9 types: Entity, Event, Rule, Temporal, Negation, Condition, Causal, Quantitative, Definitional)
- ✅ **Self-Monitoring** (monitors itself via Grid events - revolutionary "eat your own dogfood")
- ✅ **Complete Explainability** (MPPA reasoning paths with confidence scores)
- ✅ **Real-Time Learning** (<10ms concept ingestion, no retraining)

**Market Applications:** DevOps observability ($20B), enterprise knowledge management ($30B), content platforms ($50B), supply chain ($15B), customer support ($12B), scientific research ($5B), and any domain requiring explainable decisions.

### Core Components

**Storage Layer (Rust):**
- `sutra-storage` - High-performance graph storage with Write-Ahead Log (WAL) for durability
- Binary TCP protocol for low-latency communication
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

### Build System (2025-11-03)
```bash
# Build all services (single :latest tag strategy)
SUTRA_EDITION=simple sutra build                        # 8 services (4.4GB)
SUTRA_EDITION=enterprise sutra build                    # 10 services (4.76GB)

# Check what was built
sutra status

# Build individual service
SUTRA_EDITION=simple sutra build storage
```

**Image Tagging:**
- All services: `sutra-<service>:${SUTRA_VERSION:-latest}`
- Single tag strategy (no `:latest-optimized` or intermediate tags)
- Compose file: `.sutra/compose/production.yml`

### Deployment System
```bash
# Deploy by edition (profile-based)
SUTRA_EDITION=simple sutra deploy         # 8 services
SUTRA_EDITION=community sutra deploy      # HA configuration
SUTRA_EDITION=enterprise sutra deploy     # 10 services with grid

# Check status
sutra status                               # Overall health
docker ps                                  # Raw Docker status
```

**Service Breakdown:**
- **Simple/Community**: 8 services (storage, api, hybrid, embedding, nlg, bulk-ingester, client, control)
- **Enterprise**: +2 grid services (grid-master, grid-agent)

### Release Management (2025-11-03)
```bash
sutra version                      # Show current version (3.0.0)
# Manual release process:
echo "3.0.1" > VERSION            # Update version file (patch/minor/major)
git add VERSION
git commit -m "Release v3.0.1"
git tag -a v3.0.1 -m "Release version 3.0.1"
git push origin main --tags        # Trigger automated builds
sutra deploy                       # Deploy current version
```

### Development Testing
```bash
# Python unit tests
PYTHONPATH=packages/sutra-core python -m pytest tests/ -v

# E2E tests (web-based UI automation)
npm run test:e2e              # Run continuous learning tests (3 tests)
npm run test:e2e:ui           # Interactive UI mode
npm run test:e2e:debug        # Debug mode
npm run test:e2e:report       # View HTML report

# Performance stress tests (November 2025)
python3 scripts/stress_test.py --quick    # Quick validation (10-25 requests)
python3 scripts/stress_test.py            # Comprehensive tests
# Expected: 9+ req/sec, 100% success rate, <200ms latency

# Production smoke test (validates embeddings)
sutra test smoke

# Integration test (validates container deployment)
sutra test integration

# Validation test (validates Docker images)
sutra validate

# Rust tests (WAL recovery, 2PC transactions)
cd packages/sutra-storage && cargo test
```

### Security Modes
- **Development (default):** No auth/encryption - localhost only
- **Production:** `SUTRA_SECURE_MODE=true` - TLS 1.3, HMAC-SHA256, RBAC
- Security code **FULLY INTEGRATED** as of October 2025

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
Custom MessagePack-based protocol for low latency:
```rust
// packages/sutra-protocol/src/messages.rs
#[derive(Serialize, Deserialize)]
pub enum StorageRequest {
    LearnConcept { content: String, metadata: Option<ConceptMetadata> },
    QueryGraph { query: String, max_paths: u32 },
    GetConcept { concept_id: ConceptId },
}
```

**Architectural Policy:** Sutra will NEVER support SQL, Cypher, or GraphQL (see `docs/architecture/NO_SQL_POLICY.md`).

## Integration Points

### Embedding Service HA Architecture
- HAProxy load balancer (`:8888`) → 3 embedding replicas (`:8889-8891`)
- Automatic failover, health checks every 30s
- Storage server connects via `SUTRA_EMBEDDING_SERVICE_URL`

### Cross-Shard Coordination  
- Grid Master (`:7001`) orchestrates 4-16 storage shards
- 2PC coordinator for atomic writes across shards
- Parallel vector search queries all shards simultaneously

### Self-Monitoring with Grid Events (Revolutionary Innovation)
System monitors itself using its own reasoning engine - **NO external tools (Prometheus, Grafana, Datadog)**:

**Architecture:**
```
Grid Components → EventEmitter (Rust) → Sutra Storage (TCP) → Natural Language Queries
```

**26 Production Event Types:**
- Agent Lifecycle: AgentRegistered, AgentHeartbeat, AgentDegraded, AgentOffline, AgentRecovered, AgentUnregistered
- Node Lifecycle: SpawnRequested, SpawnSucceeded, SpawnFailed, StopRequested, StopSucceeded, StopFailed, NodeCrashed, NodeRestarted
- Performance: StorageMetrics, QueryPerformance, EmbeddingLatency, HnswIndexBuilt, PathfindingMetrics, ReconciliationComplete

**Natural Language Queries:**
```python
# Operational queries
"Show cluster status"
"What caused the 2am crash?"  # Returns complete causal chain in ~30ms
"Which agents went offline this week?"
"Show all spawn failures today"

# Root cause analysis
"Why did node-abc123 crash?"  # Automatic causal chain discovery
"What happened before the cluster went critical?"
"Which node has the highest restart count?"

# Performance analysis
"Which storage node is slowest?"
"Show embedding cache hit rate trends"
"Compare query latency across shards"
```

**Production Metrics:**
- Event volume: 30 events/sec sustained, 100+ burst
- Query latency: 12-34ms (faster than Elasticsearch)
- Storage overhead: <0.1% (for 16M concepts)
- Cost savings: 96% vs. traditional stack ($46K → $1.8K/year)

**Key Files:**
- Event library: `packages/sutra-grid-events/src/events.rs` (26 event types, 500+ LOC)
- Event emitter: `packages/sutra-grid-events/src/emitter.rs` (TCP binary protocol)
- Production usage: `packages/sutra-grid-master/src/main.rs` (Grid Master emits 10+ event types)
- Case study: `docs/sutra-platform-review/DEVOPS_SELF_MONITORING.md` (complete documentation)

## Critical File Locations

- **Unified CLI:** `./sutra` (Python CLI, single entry point for all operations)
- **Build script:** `./sutra-optimize.sh` (backend build orchestration, called by ./sutra)
- **Version control:** `VERSION` (single source of truth, currently 3.0.0)
- **Compose file:** `.sutra/compose/production.yml` (unified, profile-based)
- **Storage engine:** `packages/sutra-storage/src/` (14K+ LOC Rust)
- **Reasoning core:** `packages/sutra-core/sutra_core/` (42 Python modules)
- **Architecture docs:** `WARP.md` (AI assistant guidance, 1600+ lines)
- **Documentation hub:** `docs/README.md` (navigation, user journeys)
- **Build docs:** `docs/build/README.md` (build system guide)
- **Deployment docs:** `docs/deployment/README.md` (deployment guide)
- **Release docs:** `docs/release/README.md` (release management)
- **E2E tests:** `tests/e2e/` (3 continuous learning tests, web-based)
- **Test docs:** `tests/e2e/README.md` (complete test documentation)
- **Page objects:** `e2e/page-objects.ts` (LoginPage, ChatPage)
- **Smoke tests:** `scripts/smoke-test-embeddings.sh` (production validation)

## Financial Intelligence System (November 2025)

**Production-Grade Financial Data Ingestion and Analysis:**
- **Complete E2E System:** `docs/case-studies/financial-intelligence/` (comprehensive documentation suite)
- **Bulk Financial Ingester:** `scripts/production_scale_financial.py` (100+ companies, optimized concurrency)
- **Google Sheets Integration:** `packages/sutra-bulk-ingester/src/adapters/google_sheets.rs` (GOOGLEFINANCE API)
- **E2E Validation:** `tests/production_e2e_financial.py` (76 tests, 100% success rate)
- **Data Persistence:** `scripts/validate_financial_persistence.py` (WAL validation across restarts)
- **Production Monitor:** `scripts/production_monitor.py` (real-time performance tracking)
- **Query Examples:** `examples/query_financial_data.py`, `examples/test_live_queries.py`

**Key Achievements:**
- **100% E2E Success Rate:** All 76 tests passing across ingestion → storage → query workflow
- **Optimized Performance:** 0.14 concepts/sec ingestion rate with max_concurrent=2 (vs 0% success with 10)
- **WAL Persistence:** Complete data retention across Docker container restarts
- **Semantic Understanding:** 80% semantic query success rate for financial concepts
- **Production Scale:** Validated for 100+ companies with comprehensive error handling

**Architecture Highlights:**
- Google Finance → Sutra semantic concepts with temporal/causal relationships
- ThreadPoolExecutor with retry logic and exponential backoff (30→60 second timeouts)
- Docker networking with nginx proxy (port 8080) and internal service discovery
- Complete monitoring stack with performance metrics and error tracking

**Documentation Coverage:**
- **System Overview:** `docs/case-studies/financial-intelligence/README.md`
- **Data Ingestion:** `docs/case-studies/financial-intelligence/data-ingestion.md`
- **Testing & Validation:** `docs/case-studies/financial-intelligence/testing-validation.md`
- **Query & Retrieval:** `docs/case-studies/financial-intelligence/query-retrieval.md`
- **Architecture Design:** `docs/case-studies/financial-intelligence/architecture-design.md`
- **Production Deployment:** `docs/case-studies/financial-intelligence/production-deployment.md`

## Platform Review Documentation (November 2025)

**Complete Technical and Strategic Review:**
- **Platform Review:** `docs/sutra-platform-review/README.md` (navigation hub)
- **Technical Analysis:** `docs/sutra-platform-review/DEEP_TECHNICAL_REVIEW.md` (A+ grade, 9.4/10)
- **Market Analysis:** `docs/sutra-platform-review/REAL_WORLD_USE_CASES.md` ($10B → $200B+ TAM)
- **Self-Monitoring Case Study:** `docs/sutra-platform-review/DEVOPS_SELF_MONITORING.md` (96% cost savings)
- **Developer Guide:** `docs/sutra-platform-review/QUICK_START_SELF_MONITORING.md` (5-minute setup)
- **Blog Post:** `docs/sutra-platform-review/BLOG_POST_SELF_MONITORING.md` (Hacker News ready)

**Key Insights from Review:**
- Sutra was positioned as "compliance for regulated industries" but is actually **general-purpose semantic reasoning**
- Self-monitoring proves capabilities at production scale (zero external observability tools)
- Market opportunity expanded from $10B (compliance) to $200B+ (knowledge-intensive industries)
- DevOps observability ($20B market) identified as immediate go-to-market opportunity

## Performance Optimization (November 2025)

**50-70× Throughput Improvements Achieved:**
- **Sequential**: 0.13 → 9.06 req/sec (70× faster, 7542ms → 107ms)
- **Concurrent**: 0.13 → 6.52 req/sec (49× faster, 14888ms → 306ms)
- **Async**: 0% → 100% success rate (∞ improvement, fixed dimension mismatch)

**Critical Fixes:**
1. **Config-Driven Dimensions**: Eliminated hardcoded 768-dim validation
   - Now reads `VECTOR_DIMENSION` env var (supports 256/512/768)
   - Removed 15s retry penalty from dimension mismatches
   - Files: `packages/sutra-storage/src/embedding_client.rs`

2. **TCP Connection Resilience**: Fixed `NoneType.sendall` crashes
   - Added connection state validation and timeout handling
   - Implemented graceful reconnection with exponential backoff
   - Files: `packages/sutra-storage-client-tcp/sutra_storage_client/__init__.py`

3. **Production Monitoring**: Request tracking and slow query detection
   - Logs requests >1s, connection lifecycle, request counts
   - Files: `packages/sutra-storage/src/tcp_server.rs`

4. **Connection Pooling**: Keep-alive and connection reuse
   - Optimized aiohttp connector settings for concurrent workloads
   - Files: `scripts/stress_test.py`

**Documentation:**
- **Performance Guide**: `docs/architecture/PERFORMANCE_OPTIMIZATION.md` (complete guide with benchmarks)
- **Troubleshooting**: `docs/guides/troubleshooting.md` (performance section added)
- **System Architecture**: `docs/architecture/SYSTEM_ARCHITECTURE.md` (updated to v3.0.0)

**Testing:**
```bash
# Run performance stress tests
python3 scripts/stress_test.py --quick
# Expected: 9+ req/sec, 100% success, <200ms latency
```

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
- **VERSION** - Single source of truth for all package versions (3.0.0)
- **./sutra** - Unified CLI for build, deploy, test, status commands
- **sutra-optimize.sh** - Backend build orchestration (called by ./sutra)
- **.sutra/compose/production.yml** - Docker Compose with edition profiles
- **.github/workflows/release.yml** - Automated builds on tag push
- **docs/release/** - Complete documentation (5 docs, 44KB)

### Common Release Tasks

**Check version:**
```bash
sutra version                      # Show current version (3.0.0)
cat VERSION                        # 3.0.0
```

**Create release:**
```bash
# Update VERSION file manually based on change type
echo "3.0.1" > VERSION  # Bug fixes (patch)
echo "3.1.0" > VERSION  # New features (minor)
echo "4.0.0" > VERSION  # Breaking changes (major)

# Commit and tag
git add VERSION
git commit -m "Release v3.0.1"
git tag -a v3.0.1 -m "Release version 3.0.1"
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
sutra deploy                       # Deploy current version
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
- **Don't ignore editions:** Features/performance vary significantly by SUTRA_EDITION
- **Don't assume security:** Default mode has NO authentication (development only)
- **Don't forget WAL:** Storage operations are transactional with crash recovery
- **Don't skip semantic versioning:** Update VERSION file, commit, and tag properly
- **Don't skip release docs:** See `docs/release/` for complete release workflow
- **Don't use removed scripts:** Use `sutra` command as single entry point, not external scripts
- **Don't position as "regulated industries only":** Use cases span 20+ industries ($200B+ market)
- **Don't ignore self-monitoring capabilities:** We already have production proof (Grid events)
- **Don't forget temporal/causal reasoning:** This is the core differentiator vs. vector databases

## Scale Targets

- **Startup:** Fast loading with persistent HNSW indexes
- **Target scale:** 10M+ concepts across 16 shards
- **Architecture:** Optimized for continuous learning workloads

## Market Positioning

- **Narrow (old):** "Compliance tool for healthcare/finance/legal" ($10B market)
- **Broad (correct):** "Explainable reasoning for knowledge-intensive industries" ($200B+ market)
- **Immediate opportunity:** DevOps observability ($20B) - we already have proof (self-monitoring)
- **Financial Intelligence:** Production-grade system for market data analysis with 100% success rate (November 2025)

## Real-World Production Systems

**1. DevOps Self-Monitoring (October 2025):**
- 26 event types, 30 events/sec sustained, 12-34ms query latency
- 96% cost savings vs traditional stack ($46K → $1.8K/year)
- Natural language queries: "What caused the 2am crash?" returns complete causal chain

**2. Financial Intelligence (November 2025):**
- Google Finance integration for 100+ AI/tech companies
- E2E system: ingestion → storage → semantic queries (76 tests, 100% success)
- Temporal/causal relationships: "Why did NVIDIA stock drop after earnings?"
- Production deployment with WAL persistence and real-time monitoring

When working on this codebase, prioritize explainability, audit trails, and domain-specific accuracy over general AI capabilities. Every reasoning path should be traceable for compliance requirements AND operational understanding.
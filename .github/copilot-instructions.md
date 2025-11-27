# Sutra AI - AI Assistant Instructions
**Updated: November 27, 2025 - Desktop Edition Released (Local AI Integrated)**

**General-Purpose Semantic Reasoning Engine with Temporal, Causal, and Explainable AI**

## Project Architecture

Sutra AI is available in **two deployment modes**:

1. **Server Edition**: 15-service distributed system for production deployments
2. **Desktop Edition**: Pure Rust native application for personal/offline use (NEW!)

Unlike general LLMs, Sutra starts empty and learns from YOUR proprietary data with complete audit trails. Sutra combines semantic understanding, temporal reasoning, causal analysis, and explainability for ANY knowledge-intensive industry.

**CURRENT STATUS (November 27, 2025):**
- ✅ **Phase 1 COMPLETE:** Production Foundation Deployment
- ✅ **Desktop Edition:** Pure Rust native app released (v3.3.0) with **Local AI**
- ✅ **Deployment Status:** 15 containers running, 23 Docker images built
- ✅ **Service Health:** All 5 core services validated (API, Hybrid, ML-Base, Embedding, NLG)
- ✅ **System Architecture:** Production-grade multi-service mesh operational
- 🚀 **Phase 2 In Progress:** ML Service Optimization & Local AI Integration

**Key Capabilities:**
- ✅ **Temporal Reasoning** (before/after/during relationships)
- ✅ **Causal Understanding** (X causes Y, multi-hop chains, root cause analysis)
- ✅ **Semantic Classification** (9 types: Entity, Event, Rule, Temporal, Negation, Condition, Causal, Quantitative, Definitional)
- ✅ **Self-Monitoring** (monitors itself via Grid events - revolutionary "eat your own dogfood")
- ✅ **Complete Explainability** (MPPA reasoning paths with confidence scores)
- ✅ **Real-Time Learning** (<10ms concept ingestion, no retraining)

**Market Applications:** DevOps observability ($20B), enterprise knowledge management ($30B), content platforms ($50B), supply chain ($15B), customer support ($12B), scientific research ($5B), and any domain requiring explainable decisions.

## Desktop Edition (NEW - November 2025)

**Pure Rust native application - No Docker, no servers, no external dependencies.**

### Quick Start
```bash
# Build and run
cargo build -p sutra-desktop --release
cargo run -p sutra-desktop

# Create macOS app bundle
cd desktop && ./scripts/build-macos.sh
```

### Architecture
```
┌─────────────────────────────────────────────────────────────┐
│                    Sutra Desktop App                         │
├─────────────────────────────────────────────────────────────┤
│  UI Layer (egui)  →  App Controller  →  sutra-storage        │
│                                              ↓               │
│                                      Local AI (fastembed)    │
└─────────────────────────────────────────────────────────────┘
```

**Key Features:**
- 🚀 **Native Performance**: Pure Rust from storage to UI
- 🧠 **Local AI**: Built-in ONNX embedding models (fastembed)
- 🔒 **Complete Privacy**: All data stays on your machine
- 📦 **Self-Contained**: Single binary, ~20MB
- 🎨 **Modern UI**: Premium dark theme with animations
- 💾 **Full Storage Engine**: Reuses `sutra-storage` crate (no code duplication)
- 💬 **Slash Commands**: Modern `/learn`, `/search`, `/help`, `/stats` interface with autocomplete
- 🔍 **Keyword Search**: Intelligent text search with stop word filtering (shared with enterprise)

**Slash Commands:**
| Command | Shortcut | Description |
|---------|----------|-------------|
| `/learn <text>` | `/l` | Teach new knowledge |
| `/search <query>` | `/s`, `/find` | Search knowledge base |
| `/help` | `/h`, `/?` | Show all commands |
| `/clear` | `/c` | Clear chat history |
| `/stats` | `/status` | Show knowledge statistics |

**Autocomplete (Type `/` to trigger):**
- `↑`/`↓` arrows to navigate
- `Enter` to accept selection
- `Esc` to close
- Click or hover with mouse
- Auto-selects first option

**Data Location:** `~/Library/Application Support/ai.sutra.SutraDesktop/`

**Documentation:**
- `docs/desktop/README.md` - Overview and usage
- `docs/desktop/ARCHITECTURE.md` - Internal design
- `docs/desktop/BUILDING.md` - Build instructions
- `docs/desktop/UI_COMPONENTS.md` - UI component reference

### Core Components

**Storage Layer (Rust):**
- `sutra-storage` - High-performance graph storage with Write-Ahead Log (WAL) for durability
- Binary TCP protocol for low-latency communication
- USearch HNSW vector indexing with persistent mmap (94x faster startup)
- Cross-shard 2PC transactions, adaptive reconciliation
- **`text_search()` method** - Keyword-based search with stop word filtering (shared by Desktop & Enterprise)

**Local AI Layer (Rust):**
- `fastembed` - ONNX Runtime wrapper for local embedding generation
- `nomic-embed-text-v1.5` - High-quality embedding model (same as server)
- **No API Keys** - Runs completely offline

**Reasoning Engine (Python - Server Only):**
- `sutra-core` - Graph traversal with Multi-Path Plan Aggregation (MPPA)
- Real-time learning without retraining
- Quality gates: "I don't know" responses when confidence < threshold
- Progressive streaming with confidence-based refinement

**Desktop Application (Rust):**
- `sutra-desktop` - Native GUI using egui/eframe
- Directly uses `sutra-storage` crate (no duplication)
- Self-contained with local file persistence

## Essential Workflows

### Desktop Edition (NEW)
```bash
# Development build and run
cargo run -p sutra-desktop

# Release build
cargo build -p sutra-desktop --release

# Create macOS app bundle
cd desktop && ./scripts/build-macos.sh
```

### Build System (Server Edition)
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

### Deployment System (UPDATED November 2025)
```bash
# Current production deployment (Phase 1 Complete)
SUTRA_EDITION=simple ./sutra deploy    # 15 containers, 23 images - OPERATIONAL ✅

# Check deployment status
./sutra status                          # All services healthy ✅
docker ps | grep sutra-works           # 15 containers running ✅

# Production validation
./validate_production.sh               # Internal connectivity validated ✅
```

**Current Service Status:**
- **Production Deployment**: 15 containers operational with health monitoring
- **Core Services**: API ✅ Hybrid ✅ ML-Base ✅ Embedding ✅ NLG ✅
- **Storage Layer**: Graph database and WAL operational ✅
- **Architecture**: Multi-service production mesh validated ✅

**Phase 1 Achievement (November 19, 2025):**
- ✅ Built all required Docker images (23 total)
- ✅ Deployed production-grade architecture (15 containers)
- ✅ Validated core service functionality and health monitoring
- ✅ Established foundation for Phase 2 ML optimization

### Release Management (2025-11-26)
```bash
sutra version                      # Show current version (3.3.0)
echo "3.3.0" > VERSION            # Current version
git add VERSION
git commit -m "Release v3.3.0: Desktop Edition"
git tag -a v3.3.0 -m "Release version 3.3.0 - Desktop Edition released"
git push origin main --tags        # Trigger automated builds
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

### Self-Monitoring with Grid Events (Revolutionary Innovation - Eating Our Own Dogfood) 🔥
System monitors itself using its own reasoning engine - **NO external tools (Prometheus, Grafana, Datadog)**:

**Architecture:**
```
Grid Components → EventEmitter (Rust) → Sutra Storage (TCP) → Natural Language Queries
                      ↓
              USES SUTRA'S OWN REASONING ENGINE
```

**Why This Is Revolutionary:**
- ✅ **Eating Own Dogfood:** We use Sutra to monitor Sutra (zero external dependencies)
- ✅ **Temporal Reasoning:** "What happened before the crash?" (not just logs)
- ✅ **Causal Analysis:** "Why did node-abc123 fail?" (automatic root cause discovery)
- ✅ **Natural Language:** No PromQL, Lucene, SQL - just English
- ✅ **Complete Audit Trail:** Every state change captured as semantic concept
- ✅ **Cost Efficiency:** 96% savings vs Datadog/New Relic ($46K → $1.8K/year)

**Implementation Status (v3.0.1):**
- ✅ **Event Library:** 26 event types defined (1659 LOC in `sutra-grid-events`)
- ✅ **Event Emitter:** TCP-based async emitter with background worker
- ✅ **Integrated:** Grid Master + Grid Agent use EventEmitter
- 🔨 **Partial Emission:** 4/26 event types actively emitted (AgentOffline, AgentDegraded, NodeCrashed, NodeRestarted)
- 🔨 **TODO:** Emit remaining 22 event types for complete observability

**26 Event Types (Defined in `packages/sutra-grid-events/src/events.rs`):**
```rust
// Agent Lifecycle (2/6 emitted)
AgentRegistered, AgentHeartbeat, AgentDegraded ✅, AgentOffline ✅, AgentRecovered, AgentUnregistered

// Node Lifecycle (2/9 emitted)
SpawnRequested, SpawnSucceeded, SpawnFailed, StopRequested, StopSucceeded, StopFailed,
NodeCrashed ✅, NodeRestarted ✅, NodeHealthy

// Performance (0/11 emitted - TODO)
StorageMetrics, QueryPerformance, EmbeddingLatency, CacheHit, CacheMiss,
HnswIndexBuilt, PathfindingMetrics, ReconciliationComplete, GridRebalance,
ShardSplit, ShardMerge
```

**Natural Language Queries (Ready, Waiting for Events):**
```python
# Operational queries
"Show cluster status"  # Once all events emitted
"What caused the 2am crash?"  # Complete causal chain in ~30ms
"Which agents went offline this week?"
"Show all spawn failures today"

# Root cause analysis
"Why did node-abc123 crash?"  # Automatic causal chain discovery
"What happened before the cluster went critical?"
"Which node has the highest restart count?"

# Performance analysis
"Which storage node is slowest?"  # Once StorageMetrics emitted
"Show embedding cache hit rate trends"  # Once EmbeddingLatency emitted
"Compare query latency across shards"  # Once QueryPerformance emitted
```

**Production Targets (When All 26 Events Emitted):**
- Event volume: 30 events/sec sustained, 100+ burst
- Query latency: 12-34ms (faster than Elasticsearch)
- Storage overhead: <0.1% (for 16M concepts)
- Cost savings: 96% vs. traditional stack ($46K → $1.8K/year)

**Current Priority:** Emit all 26 event types to prove "eating own dogfood" thesis at production scale

**Key Files:**
- Event library: `packages/sutra-grid-events/src/events.rs` (26 event types, 500+ LOC)
- Event emitter: `packages/sutra-grid-events/src/emitter.rs` (TCP binary protocol)
- Production usage: `packages/sutra-grid-master/src/main.rs` (Grid Master emits 10+ event types)
- Case study: `docs/sutra-platform-review/DEVOPS_SELF_MONITORING.md` (complete documentation)

## Critical File Locations

- **Unified CLI:** `./sutra` (Python CLI, single entry point for all operations)
- **Build script:** `./sutra-optimize.sh` (backend build orchestration, called by ./sutra)
- **Version control:** `VERSION` (single source of truth, currently 3.3.0)
- **Compose file:** `.sutra/compose/production.yml` (unified, profile-based)
- **Storage engine:** `packages/sutra-storage/src/` (14K+ LOC Rust)
- **Reasoning core:** `packages/sutra-core/sutra_core/` (42 Python modules)
- **Storage adapter:** `packages/sutra-core/sutra_core/storage/tcp_adapter.py` (ONLY adapter - TCP binary protocol)

**Desktop Edition:**
- **Desktop app:** `desktop/src/` (Pure Rust GUI application)
- **Desktop main:** `desktop/src/main.rs` (entry point, window setup)
- **Desktop app:** `desktop/src/app.rs` (controller, uses sutra-storage)
- **Desktop UI:** `desktop/src/ui/` (sidebar, chat, knowledge, settings, status_bar)
- **Desktop theme:** `desktop/src/theme.rs` (color palette, styling)
- **Desktop build:** `desktop/scripts/build-macos.sh` (macOS app bundle)
- **Desktop docs:** `docs/desktop/` (README, ARCHITECTURE, BUILDING, UI_COMPONENTS)

**Documentation:**
- **Architecture docs:** `.github/copilot-instructions.md` (AI assistant guidance - updated November 2025)
- **Documentation hub:** `docs/README.md` (navigation, user journeys)
- **Build docs:** `docs/build/README.md` (build system guide)
- **Deployment docs:** `docs/deployment/README.md` (deployment guide)
- **Release docs:** `docs/release/README.md` (release management)
- **Architecture changes:** `docs/architecture/CLEAN_ARCHITECTURE_IMPLEMENTATION.md` (v3.0.1 changes)
- **E2E tests:** `tests/e2e/` (3 continuous learning tests, web-based)
- **Test docs:** `tests/e2e/README.md` (complete test documentation)
- **Page objects:** `e2e/page-objects.ts` (LoginPage, ChatPage)
- **Smoke tests:** `scripts/smoke-test-embeddings.sh` (production validation)

## Clean Architecture Implementation (November 2025 - v3.0.1)

**Simplified to Single TCP Backend:**
- **Removed:** `RustStorageAdapter` (573 LOC), `GrpcStorageAdapter` (200+ LOC), `connection.py` (80+ LOC)
- **Total:** 1000+ LOC dead code eliminated
- **Result:** Single initialization path, no mode switching, clearer architecture

**Storage Adapter (Production):**
```python
# ONLY ONE ADAPTER - TCP Binary Protocol
from sutra_core.storage import TcpStorageAdapter

# Initialize (automatic via SUTRA_STORAGE_SERVER env var)
storage = TcpStorageAdapter(
    server_address=os.environ.get("SUTRA_STORAGE_SERVER", "storage-server:50051"),
    vector_dimension=768,
)
```

**Installation Modes:**
```bash
# Production (minimal dependencies)
pip install sutra-core[server]  # 20MB (TCP client only)
pip install sutra-hybrid         # 103MB (no sklearn)

# Development (with local storage - NOT RECOMMENDED)
pip install sutra-core[local]   # 30MB (includes sqlalchemy/hnswlib)
pip install sutra-hybrid[tfidf] # 115MB (includes sklearn)
```

**Key Changes:**
- ❌ **Removed:** `use_rust_storage` flag (always True, now removed)
- ❌ **Removed:** `SUTRA_STORAGE_MODE` env var (always "server")
- ❌ **Removed:** Embedded mode (RustStorageAdapter with local files)
- ❌ **Removed:** gRPC mode (deprecated, TCP is 10-50× faster)
- ✅ **Kept:** TCP Binary Protocol (ONLY backend)
- ✅ **Made Optional:** sklearn (12MB), sqlalchemy (5MB), hnswlib (5MB)

**Documentation:**
- Complete guide: `docs/architecture/CLEAN_ARCHITECTURE_IMPLEMENTATION.md`
- Analysis: `docs/architecture/CLEAN_ARCHITECTURE_ANALYSIS.md`

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
- **System Architecture**: `docs/architecture/SYSTEM_ARCHITECTURE.md` (updated to v3.0.1)

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
- **VERSION** - Single source of truth for all package versions (3.0.1, Phase 1 complete)
- **./sutra** - Unified CLI for build, deploy, test, status commands
- **sutra-optimize.sh** - Backend build orchestration (called by ./sutra)
- **.sutra/compose/production.yml** - Docker Compose with edition profiles
- **.github/workflows/release.yml** - Automated builds on tag push
- **docs/release/** - Complete documentation (5 docs, 44KB)

### Common Release Tasks

**Check version:**
```bash
sutra version                      # Show current version (3.0.1)
cat VERSION                        # 3.0.1
```

**Create release:**
```bash
# Update VERSION file manually based on change type
echo "3.0.2" > VERSION  # Bug fixes (patch)
echo "3.1.0" > VERSION  # New features (minor)
echo "4.0.0" > VERSION  # Breaking changes (major)

# Commit and tag
git add VERSION
git commit -m "Release v3.0.2"
git tag -a v3.0.2 -m "Release version 3.0.2"
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
- **Don't forget temporal/causal reasoning:** This is the core differentiator vs. vector databases
- **Don't treat self-monitoring as "nice to have":** This is THE killer feature - Sutra monitoring Sutra (eating own dogfood)
- **Don't add external observability tools:** We prove our value by NOT using Prometheus/Grafana/Datadog
- **Don't emit events to external systems:** All Grid events → Sutra Storage → queryable via natural language

## Scale Targets

- **Startup:** Fast loading with persistent HNSW indexes
- **Target scale:** 10M+ concepts across 16 shards
- **Architecture:** Optimized for continuous learning workloads

## Market Positioning

- **Narrow (old):** "Compliance tool for healthcare/finance/legal" ($10B market)
- **Broad (correct):** "Explainable reasoning for knowledge-intensive industries" ($200B+ market)
- **KILLER DEMO:** Self-monitoring ($20B DevOps observability market) - Sutra monitors itself without Prometheus/Grafana/Datadog
- **Eating Own Dogfood:** Grid events → Sutra Storage → Natural language queries ("What caused the 2am crash?")
- **Current Status:** 26 event types defined (1659 LOC), 4 actively emitted, 22 to complete
- **Financial Intelligence:** Production-grade system for market data analysis with 100% success rate (November 2025)

## Real-World Production Systems

**1. DevOps Self-Monitoring (October 2025 - FLAGSHIP "EATING OWN DOGFOOD"):**
- **Status:** Foundation complete, partial implementation (4/26 events)
- **Architecture:** 1659 LOC (sutra-grid-events) defining 26 event types
- **Active Emissions:** AgentOffline, AgentDegraded, NodeCrashed, NodeRestarted
- **Target Emissions:** All 26 types → 30 events/sec sustained → 12-34ms query latency
- **Proof Point:** 96% cost savings vs traditional stack ($46K → $1.8K/year)
- **Killer Demo:** "What caused the 2am crash?" returns complete causal chain (when complete)
- **Market:** $20B DevOps observability - proving Sutra monitors Sutra WITHOUT Prometheus/Grafana
- **CURRENT PRIORITY:** Emit remaining 22 event types to complete production proof

**2. Financial Intelligence (November 2025 - PRODUCTION COMPLETE):**
- Google Finance integration for 100+ AI/tech companies
- E2E system: ingestion → storage → semantic queries (76 tests, 100% success)
- Temporal/causal relationships: "Why did NVIDIA stock drop after earnings?"
- Production deployment with WAL persistence and real-time monitoring
- **Status:** ✅ COMPLETE AND VALIDATED

**Development Philosophy:**
When working on this codebase, prioritize:
1. **Self-Monitoring First:** This proves our thesis - Sutra monitoring Sutra is the killer demo
2. **Explainability Always:** Every reasoning path must be traceable (compliance + operational understanding)
3. **Eating Own Dogfood:** Use Sutra's capabilities to build/monitor/validate Sutra itself
4. **Domain-Specific Accuracy:** Start empty, learn from YOUR data, not general AI slop

## Current Phase Status (November 19, 2025)

### ✅ PHASE 1 COMPLETE: Production Foundation Deployment
**Achievement:** Successfully deployed 15-container production architecture with validated internal connectivity

**What Was Accomplished:**
- **Production Deployment:** 15 containers, 23 Docker images built successfully
- **Core Service Validation:** All 5 services operational (API, Hybrid, ML-Base, Embedding, NLG)
- **System Architecture:** Multi-service production mesh validated with health monitoring
- **Internal Connectivity:** Service-to-service communication confirmed operational
- **Foundation Established:** Ready for Phase 2 optimization with external services

**Production Metrics Achieved:**
- Deployment Success: 100% (15/15 containers running)
- Service Health: 100% (5/5 core services operational)  
- System Resilience: Auto-recovery and orchestration working
- Performance Baseline: Internal connectivity validated
- Monitoring Infrastructure: Health checks and status reporting active

### 🚀 PHASE 2 READY: ML Service Optimization
**Next Priority:** Integrate external advanced services for 4x performance improvements

**External Services Prepared:**
- **sutra-embedder:v1.0.1** - 4x faster Rust embedding service (ready for integration)
- **sutraworks-model:v1.0.0** - Enterprise AI framework with RWKV/Mamba (ready for integration)
- **Performance Targets:** 50ms → 25ms embeddings, enhanced NLG capabilities
- **Integration Strategy:** Replace internal services with advanced external counterparts

**Phase 2 Objectives:**
1. ONNX model quantization for embedding optimization
2. External service integration with performance validation
3. GPU acceleration for enterprise AI models
4. Comprehensive benchmarking and performance measurement
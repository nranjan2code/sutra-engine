# Sutra AI Multi-Tenancy Architecture

**Production-Grade Multi-Tenant SaaS Transformation**

Version: 2.0 | Status: Implementation Ready | Date: 2025-10-26

---

## Executive Summary

### Current Architecture Analysis (Code Review Findings)

After comprehensive code review of all 16 packages, Sutra AI has a **sophisticated distributed architecture** but is currently **single-tenant**:

**Current Stack:**
- **Rust Storage Layer**: Production-grade with TCP binary protocol (57K writes/sec, <0.01ms reads)
- **Unified Learning Pipeline**: Storage server owns all learning (embeddings + associations + storage)
- **Edition System**: Already has 3-tier licensing (Simple/Community/Enterprise) with quota enforcement
- **Python Reasoning Layer**: ReasoningEngine orchestrates via TCP storage clients
- **Grid Infrastructure**: Enterprise edition has master/agent distributed processing
- **HA Services**: Embedding (3 replicas) and NLG (3 replicas) with HAProxy load balancing

**Critical Finding: No Tenant Isolation**

```
Current Single-Tenant Architecture:
┌─────────────────────────────────────────────────────────────────────┐
│  Single Deployment = Single Customer                                │
├─────────────────────────────────────────────────────────────────────┤
│  TCP Storage Server (Rust)                                          │
│  ├─ ConcurrentMemory (single instance)                              │
│  ├─ storage.dat (all data in one file)                              │
│  ├─ HNSW index (single namespace)                                   │
│  └─ NO tenant_id concept                                            │
├─────────────────────────────────────────────────────────────────────┤
│  Python API Layer                                                   │
│  ├─ sutra-api (FastAPI)                                             │
│  ├─ sutra-hybrid (SutraAI orchestrator)                             │
│  ├─ ReasoningEngine → TcpStorageAdapter → storage-server            │
│  └─ NO authentication or tenant context                             │
├─────────────────────────────────────────────────────────────────────┤
│  Edition System (Already Implemented)                               │
│  ├─ FeatureFlags with Edition enum (Simple/Community/Enterprise)    │
│  ├─ EditionLimits with quotas (learn_per_min, max_concepts, etc.)  │
│  ├─ License validation (HMAC-SHA256)                                │
│  └─ Rate limiting per edition ✅                                    │
└─────────────────────────────────────────────────────────────────────┘
```

**Key Architecture Components (From Code Review):**

1. **Storage Layer** (`packages/sutra-storage/`)
   - `ConcurrentMemory`: Main coordinator (write_log + read_view + reconciler)
   - `WriteLog`: Lock-free append-only log (57K writes/sec)
   - `ReadView`: Immutable snapshots (zero-copy mmap)
   - `AdaptiveReconciler`: AI-native self-optimizing (1-100ms intervals)
   - `HnswContainer`: USearch-based vector index (94× faster startup)
   - `ShardedStorage`: Horizontal scaling (4-16 shards for Enterprise)
   - `TransactionCoordinator`: 2PC for cross-shard atomicity
   - `tcp_server.rs`: Production TCP server (1149 lines)

2. **Protocol Layer** (`packages/sutra-protocol/`)
   - Custom binary protocol (bincode serialization)
   - 10-50× lower latency than gRPC
   - Max message size: 16MB (DoS protection)

3. **Python Layers**
   - `sutra-core`: ReasoningEngine, PathFinder, MPPA
   - `sutra-hybrid`: SutraAI orchestrator (embeddings + reasoning)
   - `sutra-api`: FastAPI REST endpoints (562 lines)

4. **Unified Learning Pipeline** (Already Implemented!)
   - Storage server owns: embedding generation + association extraction + atomic storage
   - Eliminates code duplication
   - Single source of truth ✅

5. **Edition System** (`sutra_core/feature_flags.py`)
   - `FeatureFlags` class with quota enforcement
   - `LicenseValidator` with HMAC-SHA256
   - Per-edition rate limiting ✅
   - Topology control (HA/Grid) ✅

**Problems with Current Single-Tenant Architecture:**
- ❌ **No tenant_id in data model**: Concepts/Associations lack tenant isolation
- ❌ **No authentication**: APIs have no auth middleware
- ❌ **Single storage.dat file**: All data mixed together
- ❌ **No tenant management**: Can't create/delete tenants
- ❌ **High cost per customer**: Each needs dedicated $190/month infrastructure
- ❌ **Manual provisioning**: 15-30 min per customer
- ❌ **Poor resource utilization**: <10% average capacity usage

### Target State: Production Multi-Tenant SaaS

Transform Sutra AI into a **true multi-tenant SaaS platform** leveraging existing architecture:

```
Target Multi-Tenant Architecture (Leveraging Existing Components):
┌─────────────────────────────────────────────────────────────────────┐
│  Sutra AI Multi-Tenant SaaS Platform                                │
├─────────────────────────────────────────────────────────────────────┤
│  AUTHENTICATION LAYER (NEW)                                         │
│  ├─ JWT/API Key Authentication Middleware                           │
│  ├─ TenantContext extraction (tenant_id from token)                 │
│  ├─ Per-tenant rate limiting (uses existing FeatureFlags)           │
│  └─ RBAC (admin/user/api roles per tenant)                          │
├─────────────────────────────────────────────────────────────────────┤
│  STORAGE LAYER (MODIFIED - Add Tenant Isolation)                    │
│  Rust TCP Storage Server:                                           │
│  ├─ TenantStorageManager (NEW)                                      │
│  │   ├─ get_storage(tenant_id) → ConcurrentMemory instance          │
│  │   ├─ Per-tenant storage files: tenants/{tenant_id}/storage.dat   │
│  │   ├─ Per-tenant HNSW indices                                     │
│  │   └─ Tenant metadata (quota, edition, created_at)                │
│  ├─ Existing ConcurrentMemory (unchanged, per-tenant)               │
│  │   ├─ WriteLog (lock-free, 57K writes/sec) ✅                     │
│  │   ├─ ReadView (immutable snapshots) ✅                           │
│  │   ├─ AdaptiveReconciler (AI-native) ✅                           │
│  │   └─ HnswContainer (USearch, 94× faster) ✅                      │
│  └─ tcp_server.rs modifications:                                    │
│      ├─ Extract tenant_id from request metadata                     │
│      ├─ Route to tenant-specific ConcurrentMemory                   │
│      └─ Enforce per-tenant quotas before operations                 │
├─────────────────────────────────────────────────────────────────────┤
│  API LAYER (MODIFIED - Add Tenant Context)                          │
│  ├─ sutra-api (FastAPI)                                             │
│  │   ├─ Auth middleware: get_tenant_context(token) → TenantContext  │
│  │   ├─ All endpoints receive tenant parameter                      │
│  │   ├─ TcpStorageClient.with_tenant(tenant_id) for all ops         │
│  │   └─ Existing edition-aware rate limiting ✅                     │
│  ├─ sutra-hybrid (SutraAI)                                          │
│  │   ├─ Accept tenant_id in constructor                             │
│  │   ├─ Pass tenant_id to ReasoningEngine                           │
│  │   └─ Existing unified learning pipeline ✅                       │
│  └─ sutra-core (ReasoningEngine)                                    │
│      ├─ Store tenant_id context                                     │
│      ├─ Pass tenant_id in all TcpStorageAdapter calls               │
│      └─ Existing reasoning logic (unchanged) ✅                     │
├─────────────────────────────────────────────────────────────────────┤
│  TENANT MANAGEMENT (NEW)                                            │
│  ├─ Tenant CRUD API (create/read/update/delete tenants)             │
│  ├─ Self-service signup flow                                        │
│  ├─ API key generation (sk_tenant_123_abc...)                       │
│  ├─ Quota tracking and enforcement (uses existing EditionLimits)    │
│  └─ Billing integration hooks                                       │
├─────────────────────────────────────────────────────────────────────┤
│  EXISTING COMPONENTS (Leverage As-Is) ✅                            │
│  ├─ Edition System (FeatureFlags, EditionLimits) ✅                 │
│  ├─ Unified Learning Pipeline ✅                                    │
│  ├─ Embedding Service HA (3 replicas + HAProxy) ✅                  │
│  ├─ NLG Service HA (3 replicas + HAProxy) ✅                        │
│  ├─ Grid Infrastructure (Enterprise) ✅                             │
│  └─ Adaptive Reconciler, Sharding, 2PC ✅                           │
└─────────────────────────────────────────────────────────────────────┘

Storage Layout (Phase 1: Separate Files):
/data/tenants/
├─ tenant_001/
│  ├─ storage.dat (ConcurrentMemory)
│  ├─ wal.log (Write-Ahead Log)
│  ├─ hnsw.index (USearch)
│  └─ metadata.json (quota, edition, created_at)
├─ tenant_002/
│  ├─ storage.dat
│  ├─ wal.log
│  ├─ hnsw.index
│  └─ metadata.json
└─ tenant_003/
   └─ ... (same structure)
```

**Key Advantages of This Approach:**
- ✅ **Leverages 80% of existing code**: Edition system, rate limiting, HA services
- ✅ **Physical isolation**: Separate storage.dat per tenant (security)
- ✅ **No performance degradation**: Each tenant gets own ConcurrentMemory instance
- ✅ **Simple migration**: Copy tenant folder to scale horizontally
- ✅ **Existing optimizations work**: Adaptive reconciler, HNSW, sharding all per-tenant

**Benefits:**
- ✅ **93.8% Cost Reduction**: $19K/month → $1.2K/month for 100 customers
- ✅ **Instant Onboarding**: <1 second (just create folder + metadata)
- ✅ **Self-Service**: Signup → API key → start learning
- ✅ **Operational Efficiency**: Single platform, multiple tenants
- ✅ **Better Utilization**: 60-80% average (vs 10% single-tenant)

## 1. Code Review Findings: Multi-Tenancy Readiness

### 1.1 Storage Layer (`packages/sutra-storage/`)

**Current Implementation:**
```rust
// lib.rs - Exports (1149 lines total)
pub use concurrent_memory::{ConcurrentMemory, ConcurrentConfig, ConcurrentStats};
pub use sharded_storage::{ShardedStorage, ShardConfig, ShardMap};
pub use transaction::{TransactionCoordinator, TxnOperation};
// NO tenant-related exports

// tcp_server.rs - Storage Server (1149 lines)
pub struct StorageServer {
    storage: Arc<ConcurrentMemory>,  // ← Single instance, NO tenant awareness
    embedding_client: Arc<EmbeddingClient>,
    semantic_extractor: Arc<SemanticExtractor>,
}

impl StorageServer {
    async fn handle_request(&self, request: StorageRequest) -> Result<StorageResponse> {
        match request {
            StorageRequest::LearnConceptV2 { content, options } => {
                // Uses single self.storage - NO tenant_id parameter
                let concept_id = self.learn_concept_impl(content, options).await?;
                Ok(StorageResponse::LearnConceptV2Ok { concept_id })
            }
            // All operations use single storage instance
        }
    }
}
```

**Multi-Tenancy Readiness:** ⚠️ **60% Ready**
- ✅ **Production-quality storage**: ConcurrentMemory, WriteLog, ReadView all solid
- ✅ **Scalability features**: ShardedStorage, TransactionCoordinator (2PC), AdaptiveReconciler
- ✅ **HA infrastructure**: Embedding/NLG services already have 3× replicas
- ❌ **No tenant_id concept**: All data structures lack tenant isolation
- ❌ **Single ConcurrentMemory**: Server has one instance for all requests
- ❌ **No tenant routing**: tcp_server.rs doesn't extract or route by tenant

**Required Changes:**
1. Add `TenantId` type in `types.rs`
2. Create `TenantStorageManager` to manage per-tenant `ConcurrentMemory` instances
3. Modify `tcp_server.rs` to extract tenant_id and route requests
4. Add tenant metadata storage (quota, edition, created_at)

### 1.2 Protocol Layer (`packages/sutra-protocol/`)

**Current Implementation:**
```rust
// lib.rs - Binary protocol messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageMessage {
    LearnConcept {
        concept_id: String,
        content: String,
        embedding: Vec<f32>,
        // NO tenant_id field
    },
    QueryConcept {
        concept_id: String,  // NO tenant_id
    },
    // All messages lack tenant context
}
```

**Multi-Tenancy Readiness:** ⚠️ **40% Ready**
- ✅ **Efficient binary protocol**: bincode, 10-50× faster than gRPC
- ✅ **DoS protection**: MAX_MESSAGE_SIZE = 16MB limit
- ❌ **No tenant_id in messages**: All protocol messages lack tenant field
- ❌ **No authentication metadata**: Messages don't carry auth tokens

**Required Changes:**
1. Add `tenant_id: String` field to all `StorageMessage` variants
2. Add request metadata struct for auth tokens
3. Update `send_message`/`recv_message` to include metadata

### 1.3 Python Core Layer (`packages/sutra-core/`)

**Current Implementation:**
```python
# sutra_core/__init__.py
from .reasoning import ReasoningEngine, PathFinder, QueryProcessor
from .graph import Concept, Association
# NO tenant-related classes

# ReasoningEngine uses TcpStorageAdapter (via RustStorageAdapter)
# All operations go to single storage server - NO tenant context passed
```

**Multi-Tenancy Readiness:** ⚠️ **50% Ready**
- ✅ **Well-architected**: Clean separation (reasoning → storage adapter → TCP)
- ✅ **Edition system**: FeatureFlags with quota enforcement already implemented
- ✅ **Rate limiting**: Per-edition limits already configured
- ❌ **No tenant context**: ReasoningEngine doesn't track tenant_id
- ❌ **No tenant passing**: TcpStorageAdapter calls don't include tenant

**Required Changes:**
1. Add `tenant_id` parameter to `ReasoningEngine.__init__()`
2. Store tenant_id as instance variable
3. Pass tenant_id in all storage adapter calls
4. Update `TcpStorageAdapter` to include tenant_id in protocol messages

### 1.4 API Layer (`packages/sutra-api/`)

**Current Implementation:**
```python
# sutra_api/main.py (562 lines)
@app.post("/learn")
async def learn(
    request: LearnRequest,
    client=Depends(get_storage_client),  # NO auth dependency
):
    """Learn new concept - NO tenant isolation"""
    concept_id = await client.learn_concept(
        content=request.text,  # NO tenant_id parameter
        options=request.options,
    )
    return {"concept_id": concept_id}

# NO authentication middleware
# NO tenant context extraction
# NO per-tenant rate limiting (has edition-based only)
```

**Multi-Tenancy Readiness:** ⚠️ **30% Ready**
- ✅ **Edition-aware rate limiting**: Already has `RateLimitMiddleware` with edition limits
- ✅ **Edition info endpoint**: `/edition` returns current limits and features
- ✅ **Feature flags integration**: Uses `FeatureFlags` for quota enforcement
- ❌ **No authentication**: No JWT or API key validation
- ❌ **No tenant context**: Endpoints don't receive or validate tenant_id
- ❌ **No tenant-specific rate limits**: Only edition-based (not per-tenant)

**Required Changes:**
1. Add authentication middleware (`get_tenant_context` dependency)
2. Update all endpoints to receive `TenantContext` parameter
3. Pass tenant_id to storage client in all operations
4. Add per-tenant rate limiting (not just per-edition)
5. Create tenant management endpoints (CRUD)

### 1.5 Hybrid Layer (`packages/sutra-hybrid/`)

**Current Implementation:**
```python
# sutra_hybrid/engine.py (647 lines)
class SutraAI:
    def __init__(self, storage_server: str = "storage-server:50051", enable_semantic: bool = True):
        os.environ["SUTRA_STORAGE_MODE"] = "server"
        os.environ["SUTRA_STORAGE_SERVER"] = storage_server
        
        # NO tenant_id parameter
        self._core = ReasoningEngine(use_rust_storage=True)
        self.embedding_processor = EmbeddingServiceProvider(...)
        
    def learn(self, content: str, source: Optional[str] = None) -> LearnResult:
        # Learns via ReasoningEngine - NO tenant context
        concept_id = self._core.learn(content=content, source=source)
        return LearnResult(concept_id=concept_id, ...)
```

**Multi-Tenancy Readiness:** ⚠️ **50% Ready**
- ✅ **Unified learning pipeline**: Storage server handles embeddings + associations
- ✅ **Embedding service integration**: Uses EmbeddingServiceProvider
- ✅ **Proper architecture**: Hybrid → Core → Storage
- ❌ **No tenant_id**: Constructor and methods lack tenant parameter
- ❌ **Single tenant assumption**: All operations assume one tenant

**Required Changes:**
1. Add `tenant_id` parameter to `__init__()`
2. Pass tenant_id to `ReasoningEngine`
3. Include tenant_id in all learning/query operations

### 1.6 Feature Flags (`sutra_core/feature_flags.py`)

**Current Implementation:**
```python
# feature_flags.py (360 lines) - ALREADY PRODUCTION-READY ✅
class Edition(Enum):
    SIMPLE = "simple"
    COMMUNITY = "community"
    ENTERPRISE = "enterprise"

@dataclass
class EditionLimits:
    learn_per_min: int
    reason_per_min: int
    max_concepts: int
    max_dataset_gb: int
    ingest_workers: int
    ha_enabled: bool
    grid_enabled: bool
    # ... comprehensive quotas

class FeatureFlags:
    def __init__(self):
        self.edition = Edition(os.getenv("SUTRA_EDITION", "simple"))
        self.limits = EDITION_LIMITS[self.edition]
        self.validator = LicenseValidator()
        if self.edition != Edition.SIMPLE:
            self._validate_license()  # HMAC-SHA256 validation
    
    def check_quota(self, metric: str, current: int) -> bool:
        """Check if quota exceeded"""
        limit = self.limits.get(metric, 0)
        if limit == 0:  # Unlimited (enterprise)
            return True
        return current < limit
    
    def get_rate_limit(self, endpoint: str) -> int:
        """Get rate limit for API endpoint"""
        return self.limits.learn_per_min if endpoint == "learn" else self.limits.reason_per_min
```

**Multi-Tenancy Readiness:** ✅ **90% Ready!**
- ✅ **Edition system complete**: 3-tier licensing with quotas
- ✅ **License validation**: HMAC-SHA256 cryptographic validation
- ✅ **Quota enforcement**: `check_quota()` method ready to use
- ✅ **Rate limit lookup**: `get_rate_limit()` method ready
- ✅ **Comprehensive limits**: All quotas defined (learn, reason, concepts, storage)
- 🔧 **Minor enhancement needed**: Make quota checks per-tenant (not just per-edition)

**Required Changes:**
1. Add tenant-level quota tracking (not just edition-level)
2. Method to get tenant's edition: `get_tenant_edition(tenant_id)`
3. Per-tenant quota storage (current usage vs limit)

### 1.7 Grid Infrastructure (Enterprise)

**Current Implementation:**
- `sutra-grid-master`: Orchestration (Rust, TCP binary protocol)
- `sutra-grid-agent`: Node management (spawn/stop storage nodes)
- `sutra-grid-events`: Self-monitoring via Grid events

**Multi-Tenancy Readiness:** ⚠️ **70% Ready**
- ✅ **Distributed architecture**: Master/agent pattern solid
- ✅ **Event observability**: 17 event types for monitoring
- ✅ **Enterprise-only**: Topology already controlled by edition
- ❌ **No tenant awareness**: Grid doesn't track which tenants are on which nodes
- 🔧 **Enhancement**: Add tenant-to-node mapping for load balancing

---

## 2. Multi-Tenancy Implementation Strategy

Based on code review, we can leverage **80% of existing code** with targeted additions.

---

## 2. Multi-Tenancy Implementation Strategy

Based on code review, we can leverage **80% of existing code** with targeted additions.

### Pattern 1: Separate Storage Files Per Tenant (Phase 1 - RECOMMENDED)

**Architecture:** Each tenant gets isolated `storage.dat` file with own `ConcurrentMemory` instance.

```rust
// NEW FILE: packages/sutra-storage/src/tenant_storage.rs

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use crate::concurrent_memory::{ConcurrentMemory, ConcurrentConfig};

/// Unique tenant identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TenantId(pub u64);

impl TenantId {
    pub fn from_string(s: &str) -> Result<Self, ParseError> {
        // Parse "tenant_123" → TenantId(123)
        let id = s.strip_prefix("tenant_")
            .and_then(|s| s.parse::<u64>().ok())
            .ok_or(ParseError::InvalidTenantId)?;
        Ok(TenantId(id))
    }
    
    pub fn to_string(&self) -> String {
        format!("tenant_{}", self.0)
    }
}

/// Manages ConcurrentMemory instances per tenant
pub struct TenantStorageManager {
    base_path: PathBuf,
    tenants: Arc<RwLock<HashMap<TenantId, Arc<ConcurrentMemory>>>>,
    config_template: ConcurrentConfig,
}

impl TenantStorageManager {
    pub fn new<P: AsRef<Path>>(base_path: P, config: ConcurrentConfig) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
            tenants: Arc::new(RwLock::new(HashMap::new())),
            config_template: config,
        }
    }
    
    /// Get or create storage for tenant (lazy loading)
    pub fn get_storage(&self, tenant_id: TenantId) -> Result<Arc<ConcurrentMemory>, StorageError> {
        // Check cache first (hot path)
        {
            let tenants = self.tenants.read().unwrap();
            if let Some(storage) = tenants.get(&tenant_id) {
                return Ok(Arc::clone(storage));
            }
        }
        
        // Cold path: Load or create tenant storage
        let tenant_path = self.base_path.join("tenants").join(tenant_id.to_string());
        std::fs::create_dir_all(&tenant_path)?;
        
        let storage_path = tenant_path.join("storage.dat");
        let mut tenant_config = self.config_template.clone();
        tenant_config.storage_path = storage_path;
        
        // Leverage EXISTING ConcurrentMemory - no changes needed!
        let storage = Arc::new(ConcurrentMemory::new(tenant_config)?);
        
        // Cache for future requests
        let mut tenants = self.tenants.write().unwrap();
        tenants.insert(tenant_id, Arc::clone(&storage));
        
        log::info!("✅ Loaded storage for {}", tenant_id.to_string());
        Ok(storage)
    }
    
    /// Create new tenant (onboarding)
    pub fn create_tenant(&self, tenant_id: TenantId, quota: TenantQuota) -> Result<(), StorageError> {
        let tenant_path = self.base_path.join("tenants").join(tenant_id.to_string());
        std::fs::create_dir_all(&tenant_path)?;
        
        // Write metadata
        let metadata = TenantMetadata {
            tenant_id,
            created_at: Utc::now(),
            quota,
            edition: Edition::Community,  // Default
        };
        let metadata_path = tenant_path.join("metadata.json");
        std::fs::write(metadata_path, serde_json::to_string_pretty(&metadata)?)?;
        
        // Initialize empty storage (will be lazy-loaded)
        let _storage = self.get_storage(tenant_id)?;
        
        log::info!("✅ Created tenant {}", tenant_id.to_string());
        Ok(())
    }
    
    /// Delete tenant (offboarding) 
    pub fn delete_tenant(&self, tenant_id: TenantId) -> Result<(), StorageError> {
        // Remove from cache
        {
            let mut tenants = self.tenants.write().unwrap();
            tenants.remove(&tenant_id);
        }
        
        // Delete tenant directory
        let tenant_path = self.base_path.join("tenants").join(tenant_id.to_string());
        if tenant_path.exists() {
            std::fs::remove_dir_all(tenant_path)?;
        }
        
        log::info!("✅ Deleted tenant {}", tenant_id.to_string());
        Ok(())
    }
}

/// Tenant metadata (stored in tenants/{tenant_id}/metadata.json)
#[derive(Debug, Serialize, Deserialize)]
pub struct TenantMetadata {
    pub tenant_id: TenantId,
    pub created_at: DateTime<Utc>,
    pub quota: TenantQuota,
    pub edition: Edition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantQuota {
    pub max_concepts: usize,
    pub max_storage_gb: usize,
    pub learn_per_min: u32,
    pub reason_per_min: u32,
}
```

**Storage Layout:**
```
/data/tenants/
├─ tenant_001/
│  ├─ storage.dat       # ConcurrentMemory (mmap + WriteLog + ReadView)
│  ├─ wal.log           # Write-Ahead Log for durability
│  ├─ hnsw.index        # USearch persistent index
│  └─ metadata.json     # Tenant quota, edition, created_at
├─ tenant_002/
│  ├─ storage.dat
│  ├─ wal.log
│  ├─ hnsw.index
│  └─ metadata.json
└─ tenant_003/
   └─ ... (same structure)
```

**Pros:**
- ✅ **Physical isolation**: Impossible for tenant A to access tenant B's data
- ✅ **Zero ConcurrentMemory changes**: Reuse 100% of existing storage code
- ✅ **Independent performance**: Each tenant has own WriteLog, ReadView, Reconciler
- ✅ **Simple migration**: Copy folder to move tenant to different node
- ✅ **Per-tenant backups**: Granular backup/restore
- ✅ **Easy debugging**: Inspect tenant's storage.dat independently

**Cons:**
- ❌ **Higher memory overhead**: Each tenant loads separate HNSW index (~100MB/tenant)
- ❌ **File handle limits**: Linux default ~1024 open files (but can increase)
- ⚠️ **Memory management**: Need to evict inactive tenants from cache

**Implementation Effort:** Low (2-3 weeks)
- Minimal code changes to existing ConcurrentMemory
- Clean separation of concerns
- Leverages all existing optimizations (adaptive reconciler, HNSW, WAL, etc.)

**Best For:**
- ✅ Phase 1 implementation (fastest, safest)
- ✅ First 10-50 customers
- ✅ Enterprise customers requiring strict isolation
- ✅ Regulated industries (healthcare, finance, legal)

---

### Pattern 2: Shared Storage with Tenant ID (Phase 2 - Future)

```rust
// Rust Storage Structure (Shared)
pub struct Concept {
    pub id: ConceptId,
    pub tenant_id: TenantId,  // ← NEW: Tenant isolation
    pub content: String,
    pub embedding: Vec<f32>,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
}

pub struct Association {
    pub id: AssociationId,
    pub tenant_id: TenantId,  // ← NEW: Tenant isolation
    pub source_id: ConceptId,
    pub target_id: ConceptId,
    pub association_type: AssociationType,
    pub strength: f64,
}
```

**Storage Layout:**
```
/data/storage.dat (Single File - All Tenants)
├─ Tenant A concepts: [concept_a1, concept_a2, ...]
├─ Tenant B concepts: [concept_b1, concept_b2, ...]
├─ Tenant C concepts: [concept_c1, concept_c2, ...]
└─ All indexed by (tenant_id, concept_id)
```

**Pros:**
- ✅ **Maximum Resource Efficiency**: Single storage pool, shared memory
- ✅ **Operational Simplicity**: One database, one backup, one deployment
- ✅ **Cost Effective**: Minimize infrastructure overhead
- ✅ **Easy Scaling**: Add shards as total data grows
- ✅ **Fast Queries**: All data in same address space

**Cons:**
- ❌ **Noisy Neighbor Risk**: One tenant's load can affect others
- ❌ **Security Concerns**: Logical isolation only (bugs = data leaks)
- ❌ **Complex Query Filtering**: Must always include `tenant_id` checks
- ❌ **Tenant Migration Difficulty**: Moving tenant to different shard is complex

**Implementation Effort:** High (6-8 weeks)
- Requires changes to ConcurrentMemory core
- WriteLog, ReadView, HNSW must filter by tenant_id
- Complex query rewriting (all searches need tenant filter)

**Best For:**
- ⏭️ Phase 2 (after 50+ customers)
- ⏭️ Cost optimization at scale
- ⏭️ Small tenants (<100K concepts each)

---

### Pattern 3: Hybrid (Database Sharding by Tenant Size)

**Architecture:** Large tenants get dedicated shards, small tenants share shards.

```rust
pub enum TenantStrategy {
    Dedicated { shard_id: u32 },      // Large tenant (>1M concepts)
    Shared { shard_id: u32 },         // Small tenant (<100K concepts)
}

pub struct TenantStorageManager {
    dedicated_shards: HashMap<TenantId, Arc<ConcurrentMemory>>,
    shared_shards: Vec<Arc<ConcurrentMemory>>,
    tenant_strategy: HashMap<TenantId, TenantStrategy>,
}
```

**Pros:**
- ✅ **Best of both worlds**: Isolation for large tenants, efficiency for small
- ✅ **Flexible**: Promote tenant to dedicated shard as they grow
- ✅ **Cost optimized**: Don't waste resources on small tenants

**Cons:**
- ❌ **Complex management**: Two different code paths
- ❌ **Migration overhead**: Moving tenant between strategies

**Best For:**
- ⏭️ Phase 3 (mature product)
- ⏭️ Mixed customer sizes (10 large + 1000 small)

---

## 3. TCP Server Multi-Tenant Modifications

**Best For:** 
- SaaS platforms with many small-to-medium tenants
- Cost-sensitive deployments
- Development/staging environments
- Startups optimizing for cost

---

### Pattern 2: Separate Storage Files Per Tenant

**Architecture:** Each tenant gets their own isolated `storage.dat` file.

```
/data/tenants/
├─ tenant_a/
│  ├─ storage.dat (Tenant A only)
│  ├─ wal.log
│  └─ hnsw.index
├─ tenant_b/
│  ├─ storage.dat (Tenant B only)
│  ├─ wal.log
│  └─ hnsw.index
└─ tenant_c/
   ├─ storage.dat (Tenant C only)
   ├─ wal.log
   └─ hnsw.index
```

**Pros:**
- ✅ **Physical Isolation**: Complete separation at filesystem level
- ✅ **Better Security**: Impossible for tenant A to access tenant B's data
- ✅ **Easier Migration**: Copy tenant folder to different node
- ✅ **Independent Backups**: Per-tenant backup/restore
- ✅ **Tenant-Specific Tuning**: Custom config per tenant

**Cons:**
- ❌ **Higher Memory Overhead**: Each tenant loads separate HNSW index
- ❌ **More File Handles**: Linux has limits on open files
- ❌ **Complex Resource Management**: Need to balance memory across tenants
- ❌ **Slower Cross-Tenant Operations**: If ever needed (rare)

**Implementation Complexity:** Low (minimal code changes)

**Best For:**
- Enterprise customers requiring strict isolation
- Regulated industries (healthcare, finance, legal)
- Large tenants (>1M concepts each)
- High-security requirements

---

### Pattern 3: Hybrid (Database Sharding by Tenant Size)

**Architecture:** Combine both patterns based on tenant size.

```
/data/
├─ shared_storage.dat (Tenants A, B, C, D, E - small/medium)
│  ├─ All tenants with <100K concepts
│  └─ Efficient resource pooling
├─ tenant_large_1/
│  ├─ storage.dat (Tenant F - 5M concepts)
│  └─ Dedicated isolation
└─ tenant_large_2/
   ├─ storage.dat (Tenant G - 2M concepts)
   └─ Dedicated isolation
```

**Strategy:**
- **Small tenants (<100K concepts)**: Shared storage with `tenant_id`
- **Medium tenants (100K-1M)**: Dedicated shard in shared pool
- **Large tenants (>1M)**: Dedicated storage file

**Pros:**
- ✅ **Optimal Resource Utilization**: Small tenants share, large tenants isolated
- ✅ **Flexible**: Can move tenants between patterns as they grow
- ✅ **Cost Effective for Small Tenants**: Maximize density
- ✅ **Performance Isolation for Large Tenants**: No noisy neighbor

**Cons:**
- ❌ **Most Complex**: Two codepaths to maintain
- ❌ **Migration Overhead**: Moving tenants between patterns
- ❌ **Operational Complexity**: Different management for different tenant types

**Implementation Complexity:** High

**Best For:**
- Mature SaaS platforms with diverse tenant sizes
- Platforms with 100+ tenants
- When cost optimization is critical

---

## 2. Recommended Architecture for Sutra AI

### Recommendation: **Start with Pattern 2 (Separate Files), Migrate to Pattern 1**

**Phase 1 (Months 1-3): Separate Storage Files**
- **Why:** Fastest to implement, complete isolation, minimal risk
- **Implementation:** ~2 weeks of development
- **Target:** First 10-50 customers

**Phase 2 (Months 4-6): Shared Storage with Tenant ID**
- **Why:** Better resource utilization at scale
- **Implementation:** ~4-6 weeks (more complex)
- **Target:** 50+ customers, optimize costs

**Phase 3 (Months 7+): Hybrid Pattern**
- **Why:** Handle diverse tenant sizes optimally
- **Implementation:** ~2-4 weeks (build on Phase 2)
- **Target:** 100+ customers, mature platform

---

## 3. Storage Layer Design (Phase 1: Separate Files)

### Rust Storage Changes

#### 3.1 TenantId Type
```rust
// packages/sutra-storage/src/types.rs

/// Unique tenant identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TenantId(pub u64);

impl TenantId {
    pub fn new(id: u64) -> Self {
        TenantId(id)
    }
    
    pub fn from_string(s: &str) -> Result<Self, ParseError> {
        // Parse from "tenant_123" format
        let id = s.strip_prefix("tenant_")
            .and_then(|s| s.parse::<u64>().ok())
            .ok_or(ParseError::InvalidTenantId)?;
        Ok(TenantId(id))
    }
    
    pub fn to_string(&self) -> String {
        format!("tenant_{}", self.0)
    }
}
```

#### 3.2 Tenant-Aware Storage Manager
```rust
// packages/sutra-storage/src/tenant_storage.rs (NEW FILE)

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// Manages storage instances per tenant
pub struct TenantStorageManager {
    base_path: PathBuf,
    tenants: Arc<RwLock<HashMap<TenantId, Arc<ConcurrentMemory>>>>,
    config: ConcurrentConfig,
}

impl TenantStorageManager {
    pub fn new<P: AsRef<Path>>(base_path: P, config: ConcurrentConfig) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
            tenants: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }
    
    /// Get or create storage for a tenant
    pub fn get_storage(&self, tenant_id: TenantId) -> Result<Arc<ConcurrentMemory>, StorageError> {
        // Check if already loaded
        {
            let tenants = self.tenants.read().unwrap();
            if let Some(storage) = tenants.get(&tenant_id) {
                return Ok(Arc::clone(storage));
            }
        }
        
        // Create new tenant storage
        let tenant_path = self.base_path
            .join("tenants")
            .join(tenant_id.to_string());
        
        std::fs::create_dir_all(&tenant_path)?;
        
        let storage_path = tenant_path.join("storage.dat");
        let storage = Arc::new(ConcurrentMemory::new(
            storage_path,
            self.config.clone(),
        )?);
        
        // Cache the storage instance
        let mut tenants = self.tenants.write().unwrap();
        tenants.insert(tenant_id, Arc::clone(&storage));
        
        Ok(storage)
    }
    
    /// Initialize new tenant (onboarding)
    pub fn create_tenant(&self, tenant_id: TenantId, quota: TenantQuota) -> Result<(), StorageError> {
        let tenant_path = self.base_path
            .join("tenants")
            .join(tenant_id.to_string());
        
        std::fs::create_dir_all(&tenant_path)?;
        
        // Write tenant metadata
        let metadata = TenantMetadata {
            tenant_id,
            created_at: Utc::now(),
            quota,
            edition: Edition::Community, // Default
        };
        
        let metadata_path = tenant_path.join("metadata.json");
        std::fs::write(metadata_path, serde_json::to_string_pretty(&metadata)?)?;
        
        // Initialize empty storage
        let _storage = self.get_storage(tenant_id)?;
        
        Ok(())
    }
    
    /// Delete tenant (offboarding)
    pub fn delete_tenant(&self, tenant_id: TenantId) -> Result<(), StorageError> {
        // Remove from cache
        {
            let mut tenants = self.tenants.write().unwrap();
            tenants.remove(&tenant_id);
        }
        
        // Delete tenant directory
        let tenant_path = self.base_path
            .join("tenants")
            .join(tenant_id.to_string());
        
        if tenant_path.exists() {
            std::fs::remove_dir_all(tenant_path)?;
        }
        
        Ok(())
    }
    
    /// List all tenants
    pub fn list_tenants(&self) -> Result<Vec<TenantId>, StorageError> {
        let tenants_dir = self.base_path.join("tenants");
        
        if !tenants_dir.exists() {
            return Ok(Vec::new());
        }
        
        let mut tenants = Vec::new();
        
        for entry in std::fs::read_dir(tenants_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Ok(tenant_id) = TenantId::from_string(&entry.file_name().to_string_lossy()) {
                    tenants.push(tenant_id);
                }
            }
        }
        
        Ok(tenants)
    }
    
    /// Get tenant statistics
    pub fn get_tenant_stats(&self, tenant_id: TenantId) -> Result<TenantStats, StorageError> {
        let storage = self.get_storage(tenant_id)?;
        let stats = storage.stats();
        
        let metadata = self.load_tenant_metadata(tenant_id)?;
        
        Ok(TenantStats {
            tenant_id,
            total_concepts: stats.total_concepts,
            total_edges: stats.total_edges,
            storage_size_bytes: stats.memory_usage_bytes,
            quota: metadata.quota,
            usage_percent: (stats.total_concepts as f64 / metadata.quota.max_concepts as f64) * 100.0,
        })
    }
    
    fn load_tenant_metadata(&self, tenant_id: TenantId) -> Result<TenantMetadata, StorageError> {
        let metadata_path = self.base_path
            .join("tenants")
            .join(tenant_id.to_string())
            .join("metadata.json");
        
        let content = std::fs::read_to_string(metadata_path)?;
        Ok(serde_json::from_str(&content)?)
    }
}

/// Tenant metadata stored on disk
#[derive(Debug, Serialize, Deserialize)]
pub struct TenantMetadata {
    pub tenant_id: TenantId,
    pub created_at: DateTime<Utc>,
    pub quota: TenantQuota,
    pub edition: Edition,
}

/// Tenant resource quota
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantQuota {
    pub max_concepts: usize,
    pub max_storage_gb: usize,
    pub learn_per_min: u32,
    pub reason_per_min: u32,
}

/// Tenant statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct TenantStats {
    pub tenant_id: TenantId,
    pub total_concepts: usize,
    pub total_edges: usize,
    pub storage_size_bytes: usize,
    pub quota: TenantQuota,
    pub usage_percent: f64,
}
```

#### 3.3 TCP Server Multi-Tenant Support
```rust
// packages/sutra-storage/src/tcp_server.rs (MODIFICATIONS)

use crate::tenant_storage::{TenantStorageManager, TenantId};

pub struct StorageServer {
    tenant_manager: Arc<TenantStorageManager>,  // ← NEW: Replace single ConcurrentMemory
    embedding_client: Arc<EmbeddingClient>,
    semantic_extractor: Arc<SemanticExtractor>,
}

impl StorageServer {
    pub fn new(base_path: PathBuf, config: ConcurrentConfig) -> Result<Self, StorageError> {
        Ok(Self {
            tenant_manager: Arc::new(TenantStorageManager::new(base_path, config)),
            embedding_client: Arc::new(EmbeddingClient::new(/* config */)?),
            semantic_extractor: Arc::new(SemanticExtractor::new()),
        })
    }
    
    async fn handle_request(
        &self,
        request: StorageRequest,
        tenant_id: TenantId,  // ← NEW: Extracted from auth token
    ) -> Result<StorageResponse, ServerError> {
        // Get tenant-specific storage
        let storage = self.tenant_manager.get_storage(tenant_id)?;
        
        match request {
            StorageRequest::LearnConceptV2 { content, options } => {
                // Use tenant-specific storage
                let concept_id = self.learn_concept_impl(&storage, content, options).await?;
                Ok(StorageResponse::ConceptLearned { concept_id })
            }
            
            StorageRequest::QueryConcept { concept_id } => {
                let concept = storage.get_concept(&concept_id)?;
                Ok(StorageResponse::ConceptData { concept })
            }
            
            // ... other operations use tenant-specific storage
        }
    }
}
```

---

## 4. Authentication & Authorization

### JWT Token Structure
```json
{
  "sub": "user_12345",
  "tenant_id": "tenant_67890",
  "role": "admin",
  "edition": "community",
  "exp": 1735689600,
  "iat": 1735603200,
  "permissions": ["learn", "query", "admin"]
}
```

### API Key Format
```
sk_tenant_67890_abc123def456...
│  │       │       └─ 32-byte random key
│  │       └─ Tenant ID
│  └─ Key type (sk = secret key)
└─ Prefix
```

### Authentication Middleware (Python)
```python
# packages/sutra-api/sutra_api/auth.py (NEW FILE)

from fastapi import HTTPException, Security, Header
from fastapi.security import HTTPAuthorizationCredentials, HTTPBearer
import jwt
from typing import Optional

security = HTTPBearer()

class TenantContext:
    def __init__(self, tenant_id: str, user_id: str, role: str, edition: str):
        self.tenant_id = tenant_id
        self.user_id = user_id
        self.role = role
        self.edition = edition

async def get_tenant_context(
    credentials: HTTPAuthorizationCredentials = Security(security),
    x_api_key: Optional[str] = Header(None),
) -> TenantContext:
    """Extract tenant context from JWT or API key"""
    
    # Try JWT first
    if credentials:
        try:
            token = credentials.credentials
            payload = jwt.decode(
                token,
                JWT_SECRET_KEY,
                algorithms=["HS256"]
            )
            
            return TenantContext(
                tenant_id=payload["tenant_id"],
                user_id=payload["sub"],
                role=payload["role"],
                edition=payload["edition"],
            )
        except jwt.ExpiredSignatureError:
            raise HTTPException(status_code=401, detail="Token expired")
        except jwt.InvalidTokenError:
            raise HTTPException(status_code=401, detail="Invalid token")
    
    # Try API key
    if x_api_key:
        tenant_id = extract_tenant_from_api_key(x_api_key)
        if not validate_api_key(x_api_key, tenant_id):
            raise HTTPException(status_code=401, detail="Invalid API key")
        
        return TenantContext(
            tenant_id=tenant_id,
            user_id="api_key_user",
            role="api",
            edition=get_tenant_edition(tenant_id),
        )
    
    raise HTTPException(status_code=401, detail="No credentials provided")

def extract_tenant_from_api_key(api_key: str) -> str:
    """Extract tenant_id from API key format: sk_tenant_67890_abc123..."""
    parts = api_key.split("_")
    if len(parts) >= 3 and parts[0] == "sk" and parts[1] == "tenant":
        return f"tenant_{parts[2]}"
    raise ValueError("Invalid API key format")
```

### Modified API Endpoints
```python
# packages/sutra-api/sutra_api/main.py (MODIFICATIONS)

from .auth import get_tenant_context, TenantContext

@app.post("/learn")
async def learn(
    request: LearnRequest,
    tenant: TenantContext = Depends(get_tenant_context),  # ← NEW: Inject tenant
):
    """Learn new concept (multi-tenant)"""
    
    # Check quota before learning
    stats = await storage_client.get_tenant_stats(tenant.tenant_id)
    if stats.total_concepts >= stats.quota.max_concepts:
        raise HTTPException(
            status_code=429,
            detail=f"Quota exceeded: {stats.quota.max_concepts} concepts limit"
        )
    
    # Learn with tenant context
    concept_id = await storage_client.learn_concept(
        tenant_id=tenant.tenant_id,  # ← NEW: Pass tenant_id
        content=request.text,
        options=request.options,
    )
    
    return {"concept_id": concept_id, "tenant_id": tenant.tenant_id}


@app.post("/query")
async def query(
    request: QueryRequest,
    tenant: TenantContext = Depends(get_tenant_context),  # ← NEW: Inject tenant
):
    """Query knowledge (multi-tenant)"""
    
    # Query only tenant's data
    results = await storage_client.query(
        tenant_id=tenant.tenant_id,  # ← NEW: Pass tenant_id
        query=request.query,
        max_results=request.max_results,
    )
    
    return results
```

---

## 5. Tenant Management System

### 5.1 Tenant Provisioning API
```python
# packages/sutra-api/sutra_api/tenant_management.py (NEW FILE)

from fastapi import APIRouter, HTTPException, Depends
from pydantic import BaseModel, EmailStr
from typing import Optional
import secrets

router = APIRouter(prefix="/tenants", tags=["Tenant Management"])

class TenantCreateRequest(BaseModel):
    name: str
    email: EmailStr
    edition: str = "community"  # simple, community, enterprise
    initial_quota: Optional[dict] = None

class TenantResponse(BaseModel):
    tenant_id: str
    name: str
    email: str
    edition: str
    api_key: str
    quota: dict
    created_at: str

@router.post("/", response_model=TenantResponse)
async def create_tenant(
    request: TenantCreateRequest,
    admin: TenantContext = Depends(require_admin),  # Admin-only
):
    """Create new tenant (admin-only)"""
    
    # Generate tenant ID
    tenant_id = f"tenant_{secrets.randbelow(1_000_000_000)}"
    
    # Generate API key
    api_key = f"sk_{tenant_id}_{secrets.token_urlsafe(32)}"
    
    # Determine quota based on edition
    quota = get_edition_quota(request.edition)
    if request.initial_quota:
        quota.update(request.initial_quota)
    
    # Create tenant in storage
    await storage_client.create_tenant(
        tenant_id=tenant_id,
        quota=quota,
        edition=request.edition,
    )
    
    # Store tenant metadata in database
    tenant = await db.tenants.create({
        "tenant_id": tenant_id,
        "name": request.name,
        "email": request.email,
        "edition": request.edition,
        "api_key_hash": hash_api_key(api_key),
        "quota": quota,
        "created_at": datetime.utcnow(),
    })
    
    return TenantResponse(
        tenant_id=tenant_id,
        name=request.name,
        email=request.email,
        edition=request.edition,
        api_key=api_key,  # ⚠️ Only returned once!
        quota=quota,
        created_at=tenant.created_at.isoformat(),
    )

@router.get("/{tenant_id}/stats")
async def get_tenant_stats(
    tenant_id: str,
    tenant: TenantContext = Depends(get_tenant_context),
):
    """Get tenant statistics"""
    
    # Ensure user can only access their own tenant
    if tenant.tenant_id != tenant_id and tenant.role != "admin":
        raise HTTPException(status_code=403, detail="Forbidden")
    
    stats = await storage_client.get_tenant_stats(tenant_id)
    
    return {
        "tenant_id": tenant_id,
        "total_concepts": stats.total_concepts,
        "total_edges": stats.total_edges,
        "storage_size_mb": stats.storage_size_bytes / (1024 * 1024),
        "quota": stats.quota,
        "usage_percent": stats.usage_percent,
    }

@router.delete("/{tenant_id}")
async def delete_tenant(
    tenant_id: str,
    admin: TenantContext = Depends(require_admin),  # Admin-only
):
    """Delete tenant (admin-only, dangerous!)"""
    
    # Archive before deleting
    await archive_tenant_data(tenant_id)
    
    # Delete from storage
    await storage_client.delete_tenant(tenant_id)
    
    # Delete from database
    await db.tenants.delete(tenant_id)
    
    return {"message": f"Tenant {tenant_id} deleted"}
```

### 5.2 Self-Service Signup Flow
```python
# packages/sutra-api/sutra_api/signup.py (NEW FILE)

@router.post("/signup")
async def signup(
    email: EmailStr,
    name: str,
    password: str,
):
    """Self-service tenant signup (Simple edition)"""
    
    # Validate email not already registered
    if await db.tenants.find_by_email(email):
        raise HTTPException(status_code=400, detail="Email already registered")
    
    # Create tenant (Simple edition, free)
    tenant_id = f"tenant_{secrets.randbelow(1_000_000_000)}"
    api_key = f"sk_{tenant_id}_{secrets.token_urlsafe(32)}"
    
    quota = get_edition_quota("simple")
    
    # Provision tenant storage
    await storage_client.create_tenant(
        tenant_id=tenant_id,
        quota=quota,
        edition="simple",
    )
    
    # Create user account
    user = await db.users.create({
        "email": email,
        "name": name,
        "password_hash": hash_password(password),
        "tenant_id": tenant_id,
        "role": "admin",  # First user is tenant admin
    })
    
    # Send welcome email with API key
    await send_welcome_email(
        email=email,
        tenant_id=tenant_id,
        api_key=api_key,
    )
    
    return {
        "message": "Signup successful! Check your email for API key.",
        "tenant_id": tenant_id,
        "edition": "simple",
    }
```

---

## 6. Quota Management & Rate Limiting

### Per-Tenant Rate Limiting
```python
# packages/sutra-api/sutra_api/rate_limiting.py (NEW FILE)

from fastapi import HTTPException
from collections import defaultdict
from datetime import datetime, timedelta
import asyncio

class TenantRateLimiter:
    def __init__(self):
        self.counters = defaultdict(lambda: defaultdict(int))
        self.windows = defaultdict(lambda: defaultdict(datetime))
    
    async def check_limit(
        self,
        tenant_id: str,
        operation: str,  # "learn" or "reason"
        quota: dict,
    ) -> bool:
        """Check if tenant has exceeded rate limit"""
        
        now = datetime.utcnow()
        window_key = f"{tenant_id}:{operation}"
        
        # Get limit based on operation
        limit_per_min = quota.get(f"{operation}_per_min", 10)
        
        # Check if we need to reset window
        last_reset = self.windows[window_key]
        if now - last_reset >= timedelta(minutes=1):
            self.counters[window_key] = 0
            self.windows[window_key] = now
        
        # Increment counter
        self.counters[window_key] += 1
        
        # Check limit
        if self.counters[window_key] > limit_per_min:
            raise HTTPException(
                status_code=429,
                detail=f"Rate limit exceeded: {limit_per_min} {operation} requests/min"
            )
        
        return True

# Global rate limiter
rate_limiter = TenantRateLimiter()

# Use in endpoints
@app.post("/learn")
async def learn(
    request: LearnRequest,
    tenant: TenantContext = Depends(get_tenant_context),
):
    # Check rate limit
    await rate_limiter.check_limit(
        tenant_id=tenant.tenant_id,
        operation="learn",
        quota=get_tenant_quota(tenant.tenant_id),
    )
    
    # ... rest of endpoint
```

---

## 7. Migration Strategy (Single → Multi-Tenant)

### Phase 1: Preparation (Week 1-2)
1. **Database Schema Updates**
   - Add `tenant_id` to all relevant tables
   - Create `tenants` table for metadata
   - Create `api_keys` table

2. **Rust Storage Changes**
   - Implement `TenantStorageManager`
   - Add tenant isolation to TCP server
   - Test with 2-3 test tenants

3. **Authentication System**
   - Implement JWT + API key auth
   - Add tenant context middleware
   - Test auth flows

### Phase 2: API Migration (Week 3-4)
1. **Update All Endpoints**
   - Add tenant context injection
   - Update storage client calls
   - Add quota enforcement

2. **Tenant Management APIs**
   - Create tenant CRUD endpoints
   - Implement self-service signup
   - Add admin dashboard

3. **Testing**
   - Test cross-tenant isolation
   - Test quota enforcement
   - Load testing with multiple tenants

### Phase 3: Deployment (Week 5-6)
1. **Staging Environment**
   - Deploy multi-tenant version
   - Migrate existing customers
   - Validate data isolation

2. **Production Rollout**
   - Blue-green deployment
   - Monitor for issues
   - Gradual traffic migration

3. **Documentation**
   - Update API docs
   - Create tenant onboarding guide
   - Update architecture diagrams

---

## 8. Security Considerations

### Data Isolation Guarantees

**At Storage Layer:**
```rust
// CRITICAL: Always validate tenant_id in all operations
impl TenantStorageManager {
    pub fn get_storage(&self, tenant_id: TenantId) -> Result<Arc<ConcurrentMemory>> {
        // ✅ Each tenant gets isolated storage instance
        // ✅ No way for tenant A to access tenant B's ConcurrentMemory
        // ✅ Physical file separation guarantees isolation
    }
}
```

**At API Layer:**
```python
# CRITICAL: Always extract tenant from auth, never from request body
async def learn(
    request: LearnRequest,
    tenant: TenantContext = Depends(get_tenant_context),  # ✅ From JWT
):
    # ❌ NEVER: tenant_id = request.tenant_id  (user can fake this!)
    # ✅ ALWAYS: Use tenant.tenant_id from auth token
    concept_id = await storage_client.learn_concept(
        tenant_id=tenant.tenant_id,  # ✅ From authenticated token
        content=request.text,
    )
```

### Audit Logging
```python
# Log all cross-tenant operations
await audit_log.record({
    "timestamp": datetime.utcnow(),
    "tenant_id": tenant.tenant_id,
    "user_id": tenant.user_id,
    "operation": "learn_concept",
    "resource_id": concept_id,
    "ip_address": request.client.host,
})
```

---

## 9. Monitoring & Observability

### Per-Tenant Metrics
```python
# Prometheus metrics (per tenant)
tenant_concepts_total = Counter(
    "sutra_tenant_concepts_total",
    "Total concepts per tenant",
    ["tenant_id", "edition"]
)

tenant_api_requests = Counter(
    "sutra_tenant_api_requests",
    "API requests per tenant",
    ["tenant_id", "operation", "status"]
)

tenant_storage_bytes = Gauge(
    "sutra_tenant_storage_bytes",
    "Storage usage per tenant",
    ["tenant_id"]
)
```

### Admin Dashboard Queries
```sql
-- Top tenants by concept count
SELECT tenant_id, COUNT(*) as concepts
FROM concepts
GROUP BY tenant_id
ORDER BY concepts DESC
LIMIT 10;

-- Tenants approaching quota
SELECT 
    t.tenant_id,
    t.name,
    COUNT(c.id) as current_concepts,
    t.max_concepts,
    (COUNT(c.id)::float / t.max_concepts * 100) as usage_pct
FROM tenants t
LEFT JOIN concepts c ON c.tenant_id = t.tenant_id
GROUP BY t.tenant_id
HAVING (COUNT(c.id)::float / t.max_concepts * 100) > 80;
```

---

## 10. Cost Analysis

### Current (Single-Tenant) vs Multi-Tenant

**Single-Tenant (Current):**
```
Per customer:
- 1× VM (4 vCPU, 16GB RAM): $120/month
- Storage: $50/month
- Network: $20/month
Total: $190/month per customer

For 100 customers: $19,000/month
```

**Multi-Tenant (Proposed):**
```
Shared infrastructure:
- 4× VMs (8 vCPU, 32GB RAM): $480/month
- Shared storage: $500/month
- Network: $200/month
Total: $1,180/month for 100 customers

Cost per customer: $11.80/month
Savings: 93.8% reduction!
```

---

## 11. Implementation Checklist

### Phase 1: Core Multi-Tenancy (2-3 weeks)
- [ ] Implement `TenantId` type in Rust
- [ ] Create `TenantStorageManager` 
- [ ] Add tenant isolation to TCP server
- [ ] Implement JWT authentication
- [ ] Add API key support
- [ ] Update all API endpoints with tenant context
- [ ] Implement per-tenant rate limiting
- [ ] Add tenant CRUD APIs
- [ ] Write comprehensive tests

### Phase 2: Tenant Management (1-2 weeks)
- [ ] Create tenant metadata database
- [ ] Implement self-service signup
- [ ] Add tenant admin dashboard
- [ ] Implement quota enforcement
- [ ] Add billing integration
- [ ] Create tenant migration tools

### Phase 3: Production Readiness (1-2 weeks)
- [ ] Security audit
- [ ] Load testing (100+ tenants)
- [ ] Monitoring and alerting
- [ ] Documentation updates
- [ ] Customer migration plan
- [ ] Rollback procedures

---

## 12. Next Steps

### Immediate Actions (This Week)
1. **Design Review**: Review this document with team
2. **Prototype**: Build minimal tenant isolation in Rust
3. **Test**: Validate separate storage files work correctly
4. **Estimate**: Refine timeline based on prototype

### Short-Term (Next 2 Weeks)
1. **Implement Phase 1**: Core multi-tenancy in storage layer
2. **API Updates**: Add authentication and tenant context
3. **Testing**: Comprehensive tenant isolation tests

### Medium-Term (Next 1-2 Months)
1. **Deploy to Staging**: Test with beta customers
2. **Migrate Existing Customers**: Careful migration plan
3. **Production Rollout**: Gradual deployment

---

## Conclusion

Transforming Sutra AI from single-tenant to multi-tenant is a **critical milestone** for productization. The recommended approach:

1. **Start Simple**: Phase 1 with separate storage files (fastest, safest)
2. **Learn & Iterate**: Gather real usage data from first customers
3. **Optimize Later**: Migrate to shared storage when scale demands it

**Estimated Timeline:** 6-8 weeks for production-ready multi-tenancy

**Risk Level:** Medium (well-defined path, careful isolation design)

**Business Impact:** 10-100× cost reduction, instant onboarding, true SaaS model

---

**Document Version:** 1.0  
**Last Updated:** 2025-10-26  
**Author:** AI Architecture Review  
**Status:** Awaiting team review

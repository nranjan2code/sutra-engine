# Sutra Storage

**Production-grade knowledge graph storage engine designed for 10M+ concepts with real-time learning.**

Version: 0.1.0  
Language: Rust (14,242 LOC across 34 modules)  
License: MIT

---

## Overview

Sutra Storage is a custom storage engine built specifically for temporal, continuously-learning knowledge graphs. It's **not a database** — it's optimized for the unique requirements of AI reasoning systems that learn in real-time without retraining.

### Key Features

- **🚀 Performance**: 57,412 writes/sec, <0.01ms reads (zero-copy memory-mapped)
- **🔒 Durability**: Write-Ahead Log (WAL) with MessagePack binary format - zero data loss
- **📈 Scalability**: Horizontal sharding (4-16 shards) for 10M+ concepts
- **⚡ Concurrency**: Lock-free writes, immutable read snapshots, adaptive reconciliation
- **🎯 Vector Search**: USearch HNSW with 94× faster startup (true mmap persistence)
- **🔄 ACID Transactions**: 2PC coordinator for cross-shard atomicity
- **🧠 AI-Native**: Self-optimizing adaptive reconciler with EMA trend analysis
- **🌐 Distributed**: TCP binary protocol (10-50× lower latency than gRPC)
- **📊 Observable**: 17 Grid event types for self-monitoring

### Architecture at a Glance

```
┌─────────────────────────────────────────────────────────────┐
│                    Sutra Storage Architecture                │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    │
│  │   Shard 0   │    │   Shard 1   │    │   Shard N   │    │
│  ├─────────────┤    ├─────────────┤    ├─────────────┤    │
│  │ Write Log   │    │ Write Log   │    │ Write Log   │    │
│  │ Read View   │    │ Read View   │    │ Read View   │    │
│  │ HNSW Index  │    │ HNSW Index  │    │ HNSW Index  │    │
│  │ WAL         │    │ WAL         │    │ WAL         │    │
│  └─────────────┘    └─────────────┘    └─────────────┘    │
│         │                   │                   │            │
│         └───────────────────┼───────────────────┘            │
│                             │                                │
│              ┌──────────────▼──────────────┐                │
│              │ 2PC Transaction Coordinator │                │
│              └──────────────┬──────────────┘                │
│                             │                                │
│              ┌──────────────▼──────────────┐                │
│              │      TCP Binary Server      │                │
│              │     (Custom Protocol)       │                │
│              └──────────────┬──────────────┘                │
│                             │                                │
│                  ┌──────────▼──────────┐                    │
│                  │    Client APIs       │                    │
│                  │ (Python, Rust, ...)  │                    │
│                  └─────────────────────┘                    │
└─────────────────────────────────────────────────────────────┘
```

---

## Core Concepts

### 1. **Log-Structured Storage**

Append-only design for maximum write throughput:

- **Write Log**: Lock-free, in-memory, never blocks
- **Read View**: Immutable snapshots, structural sharing via `im` crate
- **Adaptive Reconciler**: Background merging with EMA-based trend analysis (1-100ms dynamic intervals)
- **mmap Store**: Persistent single-file storage (`storage.dat`)

### 2. **Sharded Architecture**

Horizontal scaling via consistent hashing:

```rust
// Concept ID → Shard mapping
let shard_id = hash(concept_id) % num_shards;
```

- **4-16 shards** (configurable via `SUTRA_NUM_SHARDS`)
- Independent per-shard WAL, HNSW index, reconciler
- Parallel operations across all shards
- Even load distribution

### 3. **2PC Transactions**

Cross-shard atomicity for associations:

```
Phase 1: PREPARE
  ├─→ Lock resources on source shard
  └─→ Lock resources on target shard

Phase 2: COMMIT or ABORT
  ├─→ All prepared successfully → COMMIT
  └─→ Any failure → ROLLBACK (automatic)
```

### 4. **HNSW Vector Indexing**

USearch-based persistent index:

- **94× faster startup** (true mmap persistence, not rebuild)
- **24% smaller files** (single-file format with compression)
- **SIMD-optimized** search
- **Incremental updates** with automatic capacity management

### 5. **Write-Ahead Log (WAL)**

MessagePack binary format for durability:

- **Zero data loss** guarantee
- **Automatic crash recovery** on startup
- **4.4× smaller** than JSON (MessagePack migration 2025-10-24)
- **2-3× faster** serialization

---

## Module Structure

### Core Storage (9 modules)

| Module | LOC | Purpose |
|--------|-----|---------|
| `types.rs` | 208 | Data structures (ConceptId, AssociationType, etc.) |
| `segment.rs` | - | Log-structured storage segments |
| `manifest.rs` | - | Metadata management |
| `lsm.rs` | - | LSM tree for indexing |
| `store.rs` | - | Graph store interface |
| `index.rs` | - | Graph indexing |
| `wal.rs` | 561 | Write-Ahead Log (MessagePack binary) |
| `quantization.rs` | - | Vector compression (PQ) |
| `vectors.rs` | 630 | Vector storage |

### Concurrent Memory System (7 modules)

| Module | LOC | Purpose |
|--------|-----|---------|
| `write_log.rs` | 388 | Lock-free append-only write buffer |
| `read_view.rs` | - | Immutable read snapshots |
| `reconciler.rs` | - | Background reconciliation (DEPRECATED) |
| `adaptive_reconciler.rs` | 490 | 🔥 AI-native adaptive reconciliation |
| `concurrent_memory.rs` | 800+ | Main concurrent storage implementation |
| `mmap_store.rs` | - | Memory-mapped persistent storage |
| `parallel_paths.rs` | - | Parallel pathfinding with Rayon |

### Scalability & Distribution (4 modules)

| Module | LOC | Purpose |
|--------|-----|---------|
| `hnsw_container.rs` | - | USearch-based HNSW with true mmap |
| `sharded_storage.rs` | 400+ | Horizontal scaling with consistent hashing |
| `storage_trait.rs` | - | LearningStorage trait for polymorphism |
| `transaction.rs` | 500+ | 🔥 2PC coordinator for cross-shard atomicity |

### Unified Learning Pipeline (3 modules)

| Module | LOC | Purpose |
|--------|-----|---------|
| `embedding_client.rs` | - | HTTP client for embedding service |
| `semantic_extractor.rs` | - | NLP-based association extraction |
| `learning_pipeline.rs` | - | Orchestrates embedding + extraction + storage |

### Self-Monitoring & Network (3 modules)

| Module | LOC | Purpose |
|--------|-----|---------|
| `event_emitter.rs` | - | StorageEventEmitter for Grid events |
| `tcp_server.rs` | 400+ | Production TCP server (custom binary protocol) |
| `reasoning_store.rs` | - | Reasoning-specific storage API |

**Total**: 34 modules, 14,242 LOC

---

## API Reference

### 1. ConcurrentMemory (Single-Node Storage)

Primary API for single-node deployments (<1M concepts).

#### Configuration

```rust
use sutra_storage::{ConcurrentMemory, ConcurrentConfig, AdaptiveReconcilerConfig};
use std::path::PathBuf;

let config = ConcurrentConfig {
    storage_path: PathBuf::from("./storage"),
    memory_threshold: 50_000,  // Flush after 50K concepts
    vector_dimension: 768,     // nomic-embed-text dimension
    adaptive_reconciler_config: AdaptiveReconcilerConfig {
        base_interval_ms: 10,
        min_interval_ms: 1,    // High load: aggressive 1ms
        max_interval_ms: 100,  // Idle: CPU-saving 100ms
        ema_alpha: 0.3,        // Smoothing factor
        queue_warning_threshold: 0.7,  // 70% capacity alert
        queue_critical_threshold: 0.9, // 90% capacity critical
    },
    ..Default::default()
};

// Validate configuration
config.validate().expect("Invalid config");

let storage = ConcurrentMemory::new(config);
```

#### Learning

```rust
use sutra_storage::types::ConceptId;

// Learn concept with embedding
let concept_id = ConceptId::new_with_content("Eiffel Tower");
let content = b"The Eiffel Tower is in Paris".to_vec();
let embedding = vec![0.1, 0.2, ..., 0.9]; // 768-d vector
let strength = 1.0;
let confidence = 0.9;

let sequence = storage
    .learn_concept(concept_id, content, Some(embedding), strength, confidence)
    .expect("Failed to learn concept");

println!("Learned concept at sequence: {}", sequence);
```

#### Querying

```rust
// Query concept
if let Some(concept) = storage.get_concept(concept_id) {
    println!("Content: {:?}", concept.content);
    println!("Strength: {}", concept.strength);
}

// Get neighbors
let neighbors = storage.get_neighbors(concept_id);
for (neighbor_id, assoc_type, strength) in neighbors {
    println!("  → {} ({:?}, strength: {})", 
             neighbor_id.to_hex(), assoc_type, strength);
}

// Find path (BFS)
let path = storage.find_path(start_id, end_id, max_depth);
if let Some(path) = path {
    println!("Path: {:?}", path);
}

// Parallel pathfinding (4-8× faster)
let paths = storage.find_paths_parallel(start_id, end_id, 6, 10);
for path_result in paths {
    println!("Path: {:?} (confidence: {:.2}%)", 
             path_result.path, path_result.confidence * 100.0);
}
```

#### Vector Search

```rust
// HNSW vector search
let query_vector = vec![0.1, 0.2, ..., 0.9];
let k = 10;
let ef_search = 50;

let results = storage.vector_search(&query_vector, k, ef_search);
for (concept_id, distance) in results {
    println!("{}: {:.4}", concept_id.to_hex(), distance);
}
```

#### Statistics

```rust
let stats = storage.stats();
println!("Concepts: {}", stats.concepts);
println!("Edges: {}", stats.edges);
println!("Vectors: {}", stats.vectors);
println!("Written: {}", stats.written);
println!("Dropped: {}", stats.dropped);
println!("Pending: {}", stats.pending);
println!("Reconciliations: {}", stats.reconciliations);

// Adaptive reconciler stats
let reconciler_stats = storage.reconciler_stats();
println!("Queue depth: {}/{}", 
         reconciler_stats.queue_depth,
         reconciler_stats.queue_capacity);
println!("Queue utilization: {:.2}%", 
         reconciler_stats.queue_utilization * 100.0);
println!("Current interval: {}ms", reconciler_stats.current_interval_ms);
println!("Processing rate: {:.0} entries/sec", 
         reconciler_stats.processing_rate_per_sec);
println!("Health score: {:.2}/1.00 ({})", 
         reconciler_stats.health_score,
         reconciler_stats.recommendation);
```

#### Persistence

```rust
// Manual flush (auto-flush at memory_threshold)
storage.flush().expect("Flush failed");

// Graceful shutdown
drop(storage); // Flushes on drop
```

---

### 2. ShardedStorage (Multi-Node Storage)

Production API for horizontal scaling (1M-10M+ concepts).

#### Configuration

```rust
use sutra_storage::{ShardedStorage, ShardConfig, ConcurrentConfig};

let shard_config = ShardConfig {
    num_shards: 8,  // 4-16 recommended
    base_path: PathBuf::from("./sharded_storage"),
    shard_config: ConcurrentConfig {
        vector_dimension: 768,
        memory_threshold: 50_000,
        ..Default::default()
    },
};

let storage = ShardedStorage::new(shard_config)
    .expect("Failed to initialize sharded storage");
```

#### Learning (Automatic Routing)

```rust
// Concept automatically routed to correct shard
let sequence = storage.learn_concept(
    concept_id,
    content,
    Some(embedding),
    strength,
    confidence,
).expect("Failed to learn");
```

#### Cross-Shard Associations (2PC)

```rust
use sutra_storage::types::AssociationType;

// Source and target on different shards → 2PC transaction
let source_id = ConceptId::new_with_content("Paris");
let target_id = ConceptId::new_with_content("France");

let sequence = storage.create_association(
    source_id,
    target_id,
    AssociationType::Hierarchical,
    0.9,
).expect("Failed to create association");

// 2PC ensures atomicity: both shards commit or both rollback
```

#### Statistics

```rust
let aggregated_stats = storage.aggregated_stats();
println!("Total concepts: {}", aggregated_stats.total_concepts);
println!("Total edges: {}", aggregated_stats.total_edges);
println!("Total vectors: {}", aggregated_stats.total_vectors);

// Per-shard breakdown
for (shard_id, shard_stats) in &aggregated_stats.shard_stats {
    println!("Shard {}: {} concepts, {} edges",
             shard_id,
             shard_stats.concept_count,
             shard_stats.edge_count);
}
```

---

### 3. TCP Storage Server (Network API)

Standalone server for distributed architecture.

#### Starting the Server

```bash
# Environment variables
export STORAGE_PATH=/data
export STORAGE_HOST=0.0.0.0
export STORAGE_PORT=50051
export VECTOR_DIMENSION=768
export SUTRA_STORAGE_MODE=sharded  # or "single"
export SUTRA_NUM_SHARDS=4
export SUTRA_EMBEDDING_SERVICE_URL=http://embedding-ha:8888
export EVENT_STORAGE=grid-event-storage:50051  # Optional: Grid events

# Run server
cargo run --release --bin storage-server
```

#### Docker Deployment

```yaml
# docker-compose-grid.yml
storage-server:
  image: sutra-storage-server:latest
  ports:
    - "50051:50051"
  volumes:
    - storage-data:/data
  environment:
    - STORAGE_PATH=/data
    - VECTOR_DIMENSION=768
    - SUTRA_STORAGE_MODE=sharded
    - SUTRA_NUM_SHARDS=4
    - SUTRA_EMBEDDING_SERVICE_URL=http://embedding-ha:8888
```

#### Protocol Messages

```rust
// Request types (bincode serialization)
pub enum StorageRequest {
    LearnConceptV2 {
        content: String,
        options: LearnOptionsMsg,
    },
    LearnBatch {
        contents: Vec<String>,
        options: LearnOptionsMsg,
    },
    QueryConcept { concept_id: String },
    GetNeighbors { concept_id: String },
    FindPath {
        start_id: String,
        end_id: String,
        max_depth: u32,
    },
    VectorSearch {
        query_vector: Vec<f32>,
        k: u32,
        ef_search: u32,
    },
    GetStats,
    Flush,
    HealthCheck,
}
```

See [TCP Protocol Specification](../../docs/TCP_PROTOCOL_SPECIFICATION.md) for complete details.

---

## Configuration

### Environment Variables

#### Storage Server

| Variable | Default | Description |
|----------|---------|-------------|
| `STORAGE_PATH` | `./storage` | Base directory for storage files |
| `STORAGE_HOST` | `0.0.0.0` | Server bind address |
| `STORAGE_PORT` | `50051` | Server port |
| `VECTOR_DIMENSION` | `768` | Embedding dimension (**MUST match embedding service**) |
| `SUTRA_STORAGE_MODE` | `single` | `single` or `sharded` |
| `SUTRA_NUM_SHARDS` | `4` | Number of shards (4-16 recommended) |
| `RUST_LOG` | `info` | Log level (`debug`, `info`, `warn`, `error`) |

#### Unified Learning Pipeline

| Variable | Default | Description |
|----------|---------|-------------|
| `SUTRA_EMBEDDING_SERVICE_URL` | - | **REQUIRED**: Embedding service endpoint |
| `SUTRA_EMBEDDING_TIMEOUT_SEC` | `30` | HTTP timeout for embedding requests |
| `SUTRA_MIN_ASSOCIATION_CONFIDENCE` | `0.5` | Minimum confidence for extracted associations |
| `SUTRA_MAX_ASSOCIATIONS_PER_CONCEPT` | `10` | Max associations to extract per concept |

#### Self-Monitoring (Optional)

| Variable | Default | Description |
|----------|---------|-------------|
| `STORAGE_NODE_ID` | `storage-{pid}` | Node identifier for events |
| `EVENT_STORAGE` | - | Grid event storage address (e.g., `localhost:50052`) |

---

## Performance Characteristics

### Benchmarked Performance

| Operation | Throughput | Latency | Notes |
|-----------|------------|---------|-------|
| **Write** | 57,412 concepts/sec | 0.02ms | Lock-free write log |
| **Read** | - | <0.01ms | Zero-copy mmap, immutable snapshots |
| **Path Finding (BFS)** | - | ~1ms | 3-hop traversal |
| **Path Finding (Parallel)** | - | 4-8× faster | Rayon-based parallel exploration |
| **Vector Search (HNSW)** | - | <50ms (P50) | USearch SIMD-optimized |
| **HNSW Load (Startup)** | - | 3.5ms | 94× faster than rebuild (1K vectors) |
| **Flush** | - | ~200ms | 1M concepts → disk |

### Memory Usage

- **Base overhead**: ~0.1KB per concept (excluding embeddings)
- **With embeddings**: ~3.2KB per concept (768-d float32)
- **HNSW index**: ~1KB per vector (768-d, M=16)
- **Total for 1M concepts**: ~4.3GB RAM + ~900MB HNSW index

### Scalability

| Concept Count | Mode | Shards | RAM Usage | Disk Usage | Notes |
|---------------|------|--------|-----------|------------|-------|
| < 100K | Single | 1 | ~430MB | ~300MB | Development/small datasets |
| 100K - 1M | Single | 1 | ~4.3GB | ~3GB | Monitor performance |
| 1M - 5M | Sharded | 4 | ~17GB | ~12GB | Production recommended |
| 5M - 10M | Sharded | 8 | ~34GB | ~24GB | High-scale deployment |
| 10M+ | Sharded | 16 | ~68GB | ~48GB | Enterprise scale |

---

## Production Deployment

### Docker Compose (Recommended)

See root `docker-compose-grid.yml` for complete setup.

### Standalone Binary

```bash
# Build release binary
cargo build --release --bin storage-server

# Run
./target/release/storage-server
```

### Health Checks

```bash
# TCP health check (nc)
nc -z localhost 50051 && echo "Healthy"

# Via Docker
docker exec storage-server nc -z localhost 50051
```

---

## Monitoring & Observability

### Grid Events (17 Event Types)

Storage automatically emits events when `EVENT_STORAGE` configured:

- `StorageMetrics` - Real-time concept/edge count, throughput, memory
- `QueryPerformance` - Query depth, latency, confidence tracking
- `EmbeddingLatency` - Batch processing with cache hit rates
- `HnswIndexBuilt/Loaded` - Vector index build/load metrics
- `PathfindingMetrics` - Graph traversal performance
- `ReconciliationComplete` - Background reconciliation tracking

### Adaptive Reconciler Health

```rust
let stats = storage.reconciler_stats();

// Health scoring (0.0-1.0)
if stats.health_score < 0.5 {
    log::warn!("⚠️ Storage health degraded: {}", stats.recommendation);
}

// Predictive alerting
if stats.predicted_queue_depth > stats.queue_capacity * 0.9 {
    log::error!("🚨 Queue approaching capacity!");
}
```

---

## Troubleshooting

### Common Issues

#### 1. **Dimension Mismatch**

```
Error: Dimension mismatch: expected 768, got 384
```

**Solution**: Ensure `VECTOR_DIMENSION` matches embedding service:

```bash
# Storage server
export VECTOR_DIMENSION=768

# Verify embedding service
curl http://localhost:8888/info | jq '.dimension'
# Expected: 768
```

#### 2. **WAL Corruption**

```
Error: Failed to read WAL: invalid MessagePack format
```

**Solution**: Delete WAL and restart:

```bash
rm storage/wal.log
```

#### 3. **HNSW Load Failure**

```
Error: Failed to load HNSW index
```

**Solution**: Rebuild index:

```bash
rm storage/storage.usearch storage/storage.hnsw.meta
```

#### 4. **Out of Memory (OOM)**

```
Error: memory allocation failed
```

**Solution**: Reduce `memory_threshold` or add shards:

```bash
export SUTRA_STORAGE_MODE=sharded
export SUTRA_NUM_SHARDS=8
```

---

## Development

### Building from Source

```bash
# Build library
cargo build --release

# Build storage server
cargo build --release --bin storage-server

# Build with Python bindings
cargo build --release --features python-bindings
```

### Running Tests

```bash
# All tests
cargo test --release

# WAL tests
cargo test --release test_wal

# Sharding tests
cargo test --release sharded_storage
```

---

## Technical Design Documents

- [Storage Architecture](../../docs/storage/ARCHITECTURE.md)
- [2PC Transactions](../../docs/storage/TRANSACTIONS.md)
- [Sharding](../../docs/storage/SHARDING.md)
- [Adaptive Reconciliation](../../docs/storage/ADAPTIVE_RECONCILIATION_ARCHITECTURE.md)
- [USearch HNSW](../../docs/storage/USEARCH_MIGRATION_COMPLETE.md)

---

## License

MIT License

---

**Built with ❤️ by the Sutra AI Team**

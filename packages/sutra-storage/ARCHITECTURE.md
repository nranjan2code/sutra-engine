# Sutra Storage - Next-Generation Graph Storage Architecture

## Design Philosophy

**No Traditional Databases.** We're building a storage system specifically designed for temporal, evolving knowledge graphs with continuous learning. This is not a relational database, document store, or traditional graph database.

## Core Innovations

### 1. **Temporal Log-Structured Storage**
- **Append-only writes**: All mutations are appends (no in-place updates)
- **Time-travel**: Query graph state at any timestamp
- **Compaction**: Background merging of old segments
- **Zero-copy reads**: Memory-mapped file regions

### 2. **Adaptive Memory Layout**
```
Memory Layout:
┌─────────────────────────────────────────┐
│  Segment Header (metadata, timestamps)   │
├─────────────────────────────────────────┤
│  Concept Block (hot concepts in cache)   │
│  - ID (16 bytes MD5)                     │
│  - Strength (f32)                        │
│  - Access count (u32)                    │
│  - Last accessed (u64 timestamp)         │
│  - Content offset (u64)                  │
│  - Embedding offset (u64)                │
├─────────────────────────────────────────┤
│  Association Block (typed edges)         │
│  - Source ID (16 bytes)                  │
│  - Target ID (16 bytes)                  │
│  - Type (u8 enum)                        │
│  - Confidence (f32)                      │
│  - Weight (f32)                          │
│  - Timestamps (created, last_used)       │
├─────────────────────────────────────────┤
│  Vector Block (quantized embeddings)     │
│  - Quantized vectors (Product Quant)     │
│  - HNSW index structures                 │
│  - Distance tables                       │
├─────────────────────────────────────────┤
│  Content Block (variable-length strings) │
│  - UTF-8 text with length prefix         │
└─────────────────────────────────────────┘
```

### 3. **Vector Quantization for Embeddings**
- **Product Quantization**: 384D → 96D with minimal loss
- **SIMD Operations**: AVX2/NEON for fast similarity
- **Approximate Search**: HNSW graph for O(log n) retrieval
- **Memory Efficiency**: 4x reduction in embedding storage

### 4. **Lock-Free Concurrent Access**
- **Read-mostly**: Optimized for concurrent queries
- **Copy-on-Write**: Immutable segments
- **Atomic pointers**: Lock-free index updates
- **Hazard pointers**: Safe memory reclamation

### 5. **Intelligent Indexing**
```rust
// Multi-level index structure
struct GraphIndex {
    // O(1) concept lookup by ID
    concept_map: DashMap<ConceptId, u64>,  // ID -> offset
    
    // O(1) neighbor lookup
    adjacency: DashMap<ConceptId, SmallVec<[ConceptId; 8]>>,
    
    // O(log n) semantic search
    vector_index: HnswIndex,
    
    // O(1) word -> concept mapping
    inverted_index: DashMap<String, SmallVec<[ConceptId; 16]>>,
    
    // O(1) temporal queries
    temporal_index: BTreeMap<Timestamp, SegmentId>,
}
```

## Storage Format Specifications

### Concept Format (Fixed-size: 128 bytes)
```
┌─────────────────────────────────────────┐
│ concept_id: [u8; 16]           (MD5)    │
│ strength: f32                           │
│ confidence: f32                         │
│ access_count: u32                       │
│ created: u64                (timestamp) │
│ last_accessed: u64          (timestamp) │
│ content_offset: u64         (file pos)  │
│ content_length: u32                     │
│ embedding_offset: u64       (file pos)  │
│ source_hash: u32            (optional)  │
│ flags: u32                  (metadata)  │
│ reserved: [u8; 40]          (future)    │
└─────────────────────────────────────────┘
```

### Association Format (Fixed-size: 64 bytes)
```
┌─────────────────────────────────────────┐
│ source_id: [u8; 16]                     │
│ target_id: [u8; 16]                     │
│ assoc_type: u8                          │
│ confidence: f32                         │
│ weight: f32                             │
│ created: u64                            │
│ last_used: u64                          │
│ flags: u8                               │
│ reserved: [u8; 7]                       │
└─────────────────────────────────────────┘
```

### Vector Format (Product Quantized)
```
┌─────────────────────────────────────────┐
│ concept_id: [u8; 16]                    │
│ dimension: u16              (384)       │
│ num_subspaces: u8           (96)        │
│ codebook_size: u8           (256)       │
│ quantized_codes: [u8; 96]   (indices)   │
│ residual: [f16; 8]          (optional)  │
└─────────────────────────────────────────┘
```

## Performance Targets

| Operation | Target | Strategy |
|-----------|--------|----------|
| Concept write | < 10μs | Append-only, batched |
| Concept read | < 1μs | Memory-mapped, cache |
| Neighbor lookup | < 5μs | Adjacency list cache |
| Semantic search | < 1ms | HNSW + quantization |
| Graph traversal | < 100μs | Prefetching paths |
| Compaction | Background | LSM-tree style merge |

## Technology Stack

### Core (Rust)
- **Memory mapping**: `memmap2` for zero-copy I/O
- **Concurrency**: `crossbeam`, `dashmap` for lock-free structures
- **Serialization**: Custom binary format (no overhead)
- **SIMD**: `std::arch` for vectorized operations
- **TCP Protocol**: Custom binary protocol (10-50× faster than gRPC)

### Vector Search
- **Quantization**: Custom PQ implementation
- **HNSW**: Modified `hnswlib` or custom implementation
- **Distance metrics**: Cosine, L2 with SIMD

## API Design

```rust
// Rust API (Direct Library Usage)
pub struct ConcurrentMemory {
    pub fn new(config: ConcurrentConfig) -> Self;
    pub fn learn_concept(
        &self, id: ConceptId, content: Vec<u8>, 
        embedding: Option<Vec<f32>>, strength: f32, confidence: f32
    ) -> Result<u64, WriteLogError>;
    pub fn query_concept(&self, id: &ConceptId) -> Option<ConceptNode>;
    pub fn get_neighbors(&self, id: &ConceptId) -> Vec<(ConceptId, AssociationRecord)>;
    pub fn vector_search(&self, query: &[f32], k: usize, ef: usize) -> Vec<(ConceptId, f32)>;
    pub fn find_path(&self, start: ConceptId, end: ConceptId, max_depth: u8) -> Option<Vec<ConceptId>>;
    pub fn flush(&self) -> Result<()>;
}

// TCP Client API (For Network Usage)
// Use sutra-storage-client-tcp crate for Python/other languages
// See: packages/sutra-storage-client-tcp/
```

## Implementation Status

### ✅ Phase 1: Core Storage (Completed)
- ✅ Basic file format and memory mapping
- ✅ Concept and association storage
- ✅ Advanced indexing (lock-free hash maps)
- ✅ TCP binary protocol (replaces Python bindings)

### ✅ Phase 2: Vector Support (Completed)
- ✅ USearch HNSW (94× faster startup with true mmap)
- ✅ SIMD-optimized distance computations
- ✅ Production HNSW index with persistence
- ✅ Integration with semantic embeddings

### ✅ Phase 3: Concurrency (Completed)
- ✅ Lock-free read paths (immutable snapshots)
- ✅ Concurrent writes with batching (write log)
- ✅ Snapshot isolation (ArcSwap atomic pointers)
- ✅ Crash recovery (WAL with MessagePack)

### ✅ Phase 4: Optimization (Completed)
- ✅ AI-native adaptive reconciliation (EMA-based, 1-100ms dynamic)
- ✅ Horizontal sharding (4-16 shards)
- ✅ Parallel pathfinding (4-8× speedup with Rayon)
- ✅ Performance: 57K writes/sec, <10ms reads

### 🎉 Bonus: Production Features
- ✅ 2PC transactions for cross-shard atomicity
- ✅ Circuit breaker with exponential backoff + jitter
- ✅ Self-monitoring via Grid events
- ✅ Security: TLS 1.3 + HMAC-SHA256 authentication

## Why This Architecture?

1. **Graph-Native**: Structure optimized for graph traversal, not tables/documents
2. **Temporal**: Time is a first-class citizen (decay, evolution)
3. **Zero-Copy**: Memory-mapped files eliminate serialization overhead
4. **Vector-Aware**: Embeddings are not bolted on, they're core to storage
5. **Evolvable**: Continuous learning means continuous writes, optimized for that
6. **Explainable**: Can reconstruct reasoning from storage format
7. **Portable**: Single binary, no database server needed

## Comparison to Traditional Approaches

| Feature | Traditional Graph DB | Our Approach |
|---------|---------------------|--------------|
| Write Pattern | ACID transactions | Append-only log |
| Read Pattern | Query language | Direct memory access |
| Concurrency | Locks/MVCC | Lock-free structures |
| Vectors | External index | Native storage |
| Temporal | Add-on | Built-in |
| Scale-up | More RAM/CPU | More segments |
| Scale-out | Sharding | Partition by time |

This is not a database. This is a **living knowledge substrate**.

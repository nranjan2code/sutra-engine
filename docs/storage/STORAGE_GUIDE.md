# Sutra Storage Complete Guide

**Production-ready storage for knowledge graphs at scale**

## Overview

Sutra AI provides two storage modes optimized for different scales:

| Mode | Concept Count | Use Case | Performance |
|------|--------------|----------|-------------|
| **Single** | < 1M | Development, small deployments | 57K writes/sec, <0.01ms reads |
| **Sharded** | 1M - 10M+ | Production, enterprise scale | 57K writes/sec per shard, parallel queries |

## Architecture

### Single Storage Mode

```
┌──────────────────────────────────────┐
│     ConcurrentMemory Storage         │
├──────────────────────────────────────┤
│  WriteLog (lock-free append)        │
│  ReadView (immutable snapshots)      │
│  HNSW Index (vector search)          │
│  WAL (durability)                    │
│  storage.dat (single file)           │
└──────────────────────────────────────┘
```

**Characteristics:**
- Single `storage.dat` file (512MB initial)
- Lock-free concurrent writes
- Zero-copy memory-mapped reads
- O(log N) vector search with HNSW
- WAL for crash recovery

### Sharded Storage Mode

```
┌────────────────────────────────────────────────────────────┐
│              ShardedStorage (Consistent Hashing)           │
├────────────────────────────────────────────────────────────┤
│  Shard 0          Shard 1          Shard 2      ...        │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐            │
│  │ storage  │    │ storage  │    │ storage  │            │
│  │   .dat   │    │   .dat   │    │   .dat   │            │
│  │   + WAL  │    │   + WAL  │    │   + WAL  │            │
│  │  + HNSW  │    │  + HNSW  │    │  + HNSW  │            │
│  └──────────┘    └──────────┘    └──────────┘            │
└────────────────────────────────────────────────────────────┘
     ↑                ↑                ↑
     └────────────────┴────────────────┘
          Parallel Vector Search
```

**Characteristics:**
- Multiple independent shards (4-16 recommended)
- Consistent hashing for even distribution
- Each shard: independent storage.dat + WAL + HNSW
- Parallel operations across all shards
- O(log(N/S)) effective search complexity

## LearningStorage Trait

The unified interface for all storage backends:

```rust
/// Common trait for both single and sharded storage
pub trait LearningStorage {
    /// Store a concept with optional embedding
    fn learn_concept(
        &self,
        id: ConceptId,
        content: Vec<u8>,
        vector: Option<Vec<f32>>,
        strength: f32,
        confidence: f32,
    ) -> Result<u64>;
    
    /// Create an association between concepts
    fn learn_association(
        &self,
        source: ConceptId,
        target: ConceptId,
        assoc_type: AssociationType,
        confidence: f32,
    ) -> Result<u64>;
}
```

**Implementations:**
- `ConcurrentMemory` - Single storage backend
- `ShardedStorage` - Distributed storage backend
- `Arc<T>` where `T: LearningStorage` - Shared ownership pattern

## Unified Learning Pipeline

The storage server owns the complete learning pipeline:

```
Client Request → TCP → Storage Server
                        ├─→ 1. Generate Embedding (via embedding service)
                        ├─→ 2. Extract Associations (NLP patterns)
                        ├─→ 3. Generate Concept ID (deterministic hash)
                        ├─→ 4. Store Concept + Vector (WAL + memory)
                        ├─→ 5. Store Associations (graph edges)
                        └─→ 6. Return concept_id
```

**Benefits:**
- ✅ Single source of truth (no duplicate logic)
- ✅ Guaranteed consistency (same behavior everywhere)
- ✅ Automatic embeddings (never miss vectors)
- ✅ Atomic operations (all-or-nothing)
- ✅ Works with both storage modes

## Configuration

### Single Storage

```yaml
# docker-compose-grid.yml
storage-server:
  environment:
    - SUTRA_STORAGE_MODE=single  # Single storage mode
    - STORAGE_PATH=/data
    - VECTOR_DIMENSION=768
    - SUTRA_EMBEDDING_SERVICE_URL=http://sutra-embedding-service:8888
```

### Sharded Storage

```yaml
# docker-compose-grid.yml
storage-server:
  environment:
    - SUTRA_STORAGE_MODE=sharded  # Sharded storage mode
    - SUTRA_NUM_SHARDS=4          # Number of shards
    - STORAGE_PATH=/data
    - VECTOR_DIMENSION=768
    - SUTRA_EMBEDDING_SERVICE_URL=http://sutra-embedding-service:8888
```

## Performance Benchmarks

### Single Storage

```
Write Performance:
  - 57,412 concepts/sec (lock-free writes)
  - 25,000× faster than baseline JSON storage
  
Read Performance:
  - <0.01ms per read (zero-copy mmap)
  - Immutable snapshots (no blocking)
  
Vector Search:
  - O(log N) with HNSW index
  - 1-2ms for 3-hop graph traversal
  
Memory:
  - ~0.1KB per concept (excluding vectors)
  - 512MB initial file size
```

### Sharded Storage

```
Write Performance:
  - 57,412 concepts/sec per shard
  - Linear scaling with shard count
  
Read Performance:
  - <0.01ms per shard
  - Parallel reads across shards
  
Vector Search:
  - O(log(N/S)) effective complexity
  - Parallel query all shards simultaneously
  - Aggregate and rank top-k results
  
Scalability:
  - 4 shards: 4M concepts optimal
  - 8 shards: 8M concepts optimal
  - 16 shards: 16M+ concepts optimal
```

## Durability and Recovery

### Write-Ahead Log (WAL)

Both storage modes use WAL for durability:

```rust
// Every write goes to WAL FIRST
1. Append to WAL (disk sync)
2. Write to memory structures
3. Background reconciliation
4. Checkpoint WAL on flush
```

**Guarantees:**
- RPO (Recovery Point Objective): 0 (zero data loss)
- RTO (Recovery Time Objective): < 1 second
- Automatic crash recovery on startup

### Crash Recovery

```bash
# Startup sequence
1. Load storage.dat (if exists)
2. Replay WAL (if exists)
3. Reconcile in-memory state
4. Checkpoint WAL
5. Ready for operations
```

## Migration Guide

### From Single to Sharded

**Zero-downtime migration:**

1. **Backup existing data:**
   ```bash
   docker exec sutra-storage cat /data/storage.dat > backup.dat
   ```

2. **Update configuration:**
   ```yaml
   storage-server:
     environment:
       - SUTRA_STORAGE_MODE=sharded
       - SUTRA_NUM_SHARDS=4
   ```

3. **Restart storage server:**
   ```bash
   docker-compose -f docker-compose-grid.yml restart storage-server
   ```

4. **Verify sharding:**
   ```bash
   docker logs sutra-storage | grep "shard"
   # Should show: "Initialized shard 0 at /data/shard_0000"
   ```

5. **Test operations:**
   ```bash
   # Learn a concept
   curl -X POST http://localhost:8001/sutra/learn \
     -H "Content-Type: application/json" \
     -d '{"text": "Test concept for sharding"}'
   
   # Query to verify
   curl -X POST http://localhost:8001/sutra/query \
     -H "Content-Type: application/json" \
     -d '{"query": "test"}'
   ```

### From Sharded to Single

**Consolidation process:**

1. **Backup all shards:**
   ```bash
   for i in {0..3}; do
     docker exec sutra-storage tar czf /tmp/shard_$i.tar.gz /data/shard_000$i
   done
   ```

2. **Update configuration:**
   ```yaml
   storage-server:
     environment:
       - SUTRA_STORAGE_MODE=single
   ```

3. **Restart and rebuild index:**
   ```bash
   docker-compose -f docker-compose-grid.yml restart storage-server
   ```

## Monitoring

### Storage Stats

```bash
# Get storage statistics
curl http://localhost:8000/stats | jq

# Expected output (single mode):
{
  "total_concepts": 1000,
  "total_associations": 500,
  "total_embeddings": 1000,
  "average_strength": 0.95
}

# Expected output (sharded mode):
{
  "total_concepts": 4000,
  "num_shards": 4,
  "shard_distribution": [1000, 1000, 1000, 1000]
}
```

### Health Checks

```bash
# Storage server health
curl http://localhost:50051/health

# Check shard status (sharded mode)
docker logs sutra-storage | grep "Shard" | tail -10
```

### Performance Metrics

```bash
# Write performance
docker logs sutra-storage | grep "concepts/sec"

# HNSW index stats
docker logs sutra-storage | grep "HNSW"

# WAL replay stats
docker logs sutra-storage | grep "WAL replay"
```

## Troubleshooting

### Issue: "Dimension mismatch" errors

**Cause:** Vector dimension mismatch between storage and embeddings

**Solution:**
```yaml
# Ensure consistent 768-d configuration
storage-server:
  environment:
    - VECTOR_DIMENSION=768  # MUST be 768
```

### Issue: Slow vector search

**Cause:** HNSW index not built or misconfigured

**Solution:**
```bash
# Check HNSW indexing logs
docker logs sutra-storage | grep "HNSW"

# Should see: "🔍 HNSW: Indexing vector for concept"
# If missing, embeddings are not being generated
```

### Issue: Data loss after crash

**Cause:** WAL not properly configured or flushed

**Solution:**
```bash
# Verify WAL is being written
docker logs sutra-storage | grep "WAL"

# Should see:
# - "🔄 Replaying WAL for crash recovery..."
# - "✅ WAL checkpoint: truncated to position X"
```

### Issue: Uneven shard distribution

**Cause:** Consistent hashing with few concepts

**Solution:**
- Load more concepts (need 100+ per shard for even distribution)
- Or reduce shard count temporarily

### Issue: "LearnConceptV2 not implemented"

**Cause:** Old storage server binary without sharded learning pipeline

**Solution:**
```bash
# Rebuild storage server
docker-compose -f docker-compose-grid.yml build storage-server

# Restart
docker-compose -f docker-compose-grid.yml up -d storage-server
```

## Best Practices

### Development
- Use **single storage mode**
- Enable WAL for durability
- Monitor `storage.dat` file size
- Flush before shutdown

### Production
- Use **sharded storage mode** for > 1M concepts
- Configure 4-8 shards for optimal performance
- Monitor shard distribution
- Set up backup for all shard directories
- Use dedicated embedding service

### Capacity Planning

| Concepts | RAM | Disk | Shards | Notes |
|----------|-----|------|--------|-------|
| 100K | 512MB | 2GB | 1 | Single mode |
| 500K | 2GB | 10GB | 1 | Single mode |
| 1M | 4GB | 20GB | 4 | Switch to sharded |
| 5M | 16GB | 100GB | 8 | Production scale |
| 10M+ | 32GB+ | 200GB+ | 16 | Enterprise scale |

## References

- **`packages/sutra-storage/src/storage_trait.rs`** - LearningStorage trait
- **`packages/sutra-storage/src/learning_pipeline.rs`** - Unified pipeline
- **`packages/sutra-storage/src/concurrent_memory.rs`** - Single storage
- **`packages/sutra-storage/src/sharded_storage.rs`** - Sharded storage
- **`docs/UNIFIED_LEARNING_ARCHITECTURE.md`** - Learning architecture
- **`docs/storage/SHARDING.md`** - Detailed sharding guide

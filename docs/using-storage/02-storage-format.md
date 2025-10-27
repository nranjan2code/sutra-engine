# Storage Format

Sutra uses a custom binary format called **SUTRADAT v2** optimized for high-performance AI reasoning workloads. This document explains the persistence layer, memory mapping, and durability guarantees.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    Storage Layer Stack                          │
├─────────────────────────────────────────────────────────────────┤
│  Application Layer                                              │
│  ├─ StorageClient (Python/TCP)                                 │
│  └─ Direct Rust API                                            │
├─────────────────────────────────────────────────────────────────┤
│  Protocol Layer                                                │
│  ├─ Binary TCP Protocol                                        │
│  └─ MessagePack Serialization                                  │
├─────────────────────────────────────────────────────────────────┤
│  Storage Engine (Rust)                                         │
│  ├─ Concurrent Memory Manager                                  │
│  ├─ Write-Ahead Log (WAL)                                      │
│  ├─ Memory-Mapped Store                                        │
│  └─ HNSW Vector Index                                          │
├─────────────────────────────────────────────────────────────────┤
│  File System                                                   │
│  ├─ storage.dat (Binary format)                               │
│  ├─ wal.log (Durability)                                      │
│  └─ vectors.idx (HNSW index)                                  │
└─────────────────────────────────────────────────────────────────┘
```

## SUTRADAT v2 Binary Format

### File Structure

The storage file uses a structured binary format for optimal performance:

```
┌────────────────────────────────────────────────────────────────┐
│  FILE HEADER (64 bytes)                                        │
├────────────────────────────────────────────────────────────────┤
│  Magic:      "SUTRADAT" (8 bytes)                              │
│  Version:    2 (u32, 4 bytes)                                  │
│  Concepts:   count (u32, 4 bytes)                              │
│  Edges:      count (u32, 4 bytes)                              │
│  Vectors:    count (u32, 4 bytes)                              │
│  Reserved:   36 bytes (future extensions)                      │
└────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────┐
│  CONCEPTS SECTION                                              │
├────────────────────────────────────────────────────────────────┤
│  For each concept (128 bytes fixed):                          │
│  ├─ ID:              UUID (16 bytes)                           │
│  ├─ Strength:        f32 (4 bytes)                             │
│  ├─ Confidence:      f32 (4 bytes)                             │
│  ├─ Access Count:    u32 (4 bytes)                             │
│  ├─ Created:         timestamp (8 bytes)                       │
│  ├─ Last Accessed:   timestamp (8 bytes)                       │
│  ├─ Content Offset:  u64 (8 bytes)                             │
│  ├─ Content Length:  u32 (4 bytes)                             │
│  ├─ Embedding Offset: u64 (8 bytes)                            │
│  ├─ Source Hash:     u32 (4 bytes)                             │
│  ├─ Flags:           u32 (4 bytes)                             │
│  └─ Reserved:        56 bytes                                  │
└────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────┐
│  ASSOCIATIONS SECTION                                          │
├────────────────────────────────────────────────────────────────┤
│  For each association (64 bytes fixed):                       │
│  ├─ Source ID:       UUID (16 bytes)                           │
│  ├─ Target ID:       UUID (16 bytes)                           │
│  ├─ Type:            u8 (1 byte)                               │
│  ├─ Confidence:      f32 (4 bytes)                             │
│  ├─ Weight:          f32 (4 bytes)                             │
│  ├─ Created:         timestamp (8 bytes)                       │
│  ├─ Last Used:       timestamp (8 bytes)                       │
│  ├─ Flags:           u8 (1 byte)                               │
│  └─ Reserved:        6 bytes                                   │
└────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────┐
│  CONTENT SECTION                                               │
├────────────────────────────────────────────────────────────────┤
│  Variable-length UTF-8 strings                                │
│  (Referenced by content_offset + content_length)              │
└────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────┐
│  VECTORS SECTION                                               │
├────────────────────────────────────────────────────────────────┤
│  For each vector:                                              │
│  ├─ Concept ID:      UUID (16 bytes)                           │
│  ├─ Dimension:       u32 (4 bytes)                             │
│  └─ Components:      [f32; dimension] (dimension × 4 bytes)    │
└────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────┐
│  FOOTER (Variable size)                                        │
├────────────────────────────────────────────────────────────────┤
│  ├─ Bloom Filter:    Probabilistic concept lookup              │
│  ├─ Offset Index:    Concept ID → file offset mapping          │
│  └─ Checksum:        Data integrity verification               │
└────────────────────────────────────────────────────────────────┘
```

### Key Design Decisions

#### 1. Fixed-Size Records
- **Concepts**: 128 bytes each for predictable memory layout
- **Associations**: 64 bytes each for efficient cache utilization
- **Benefits**: O(1) random access, optimal memory alignment

#### 2. Separate Content Storage
- **Why**: Variable-length content would break fixed-size records
- **How**: Offset + length pointers to content section
- **Benefits**: Minimal memory waste, efficient string storage

#### 3. Footer Indexes
- **Bloom Filter**: Fast negative lookup (concept exists?)
- **Offset Index**: Direct file offset lookup by ConceptId
- **Performance**: Sub-millisecond concept retrieval

## Memory Mapping (mmap)

### Zero-Copy Architecture

Sutra uses memory mapping for maximum performance:

```rust
// Rust implementation example
use memmap2::Mmap;

pub struct MmapStore {
    file: File,
    mmap: Mmap,           // Memory-mapped view of storage.dat
    header: &SegmentHeader, // Direct pointer to file header
    concepts: &[ConceptRecord], // Array view of concept records
    associations: &[AssociationRecord], // Array view of associations
}

impl MmapStore {
    pub fn read_concept(&self, id: ConceptId) -> Option<&ConceptRecord> {
        // O(1) lookup using footer offset index
        let offset = self.find_concept_offset(id)?;
        let record_index = (offset - HEADER_SIZE) / CONCEPT_RECORD_SIZE;
        self.concepts.get(record_index as usize)
    }
    
    pub fn read_content(&self, record: &ConceptRecord) -> &str {
        // Zero-copy string slice from mmap
        let start = record.content_offset as usize;
        let end = start + record.content_length as usize;
        std::str::from_utf8(&self.mmap[start..end]).unwrap()
    }
}
```

### Performance Characteristics

| Operation | Latency | Throughput | Notes |
|-----------|---------|------------|-------|
| Concept read | <0.01ms | ~10M ops/sec | Zero-copy mmap |
| Content read | <0.1ms | ~1M ops/sec | String slice |
| Association lookup | <0.01ms | ~5M ops/sec | Fixed-size array |
| Vector read | <0.5ms | ~100K ops/sec | Large float arrays |

## Write-Ahead Log (WAL)

### Durability Guarantee

All writes go through a Write-Ahead Log before updating the main storage:

```rust
// WAL Operation Types
pub enum Operation {
    WriteConcept {
        id: ConceptId,
        content: Vec<u8>,
        embedding: Option<Vec<f32>>,
        strength: f32,
        confidence: f32,
        timestamp: u64,
    },
    WriteAssociation {
        source: ConceptId,
        target: ConceptId, 
        association_id: u64,
        strength: f32,
        created: u64,
    },
    UpdateConcept {
        id: ConceptId,
        field: UpdateField,
        value: UpdateValue,
    },
}
```

### WAL Workflow

```
1. Application Request
   ├─ learn_concept("New medical protocol...")
   │
2. WAL Append (Synchronous)
   ├─ Serialize operation to WAL
   ├─ fsync() to disk
   ├─ Return sequence number to client ✓
   │
3. Background Processing (Asynchronous)
   ├─ Apply WAL operations to mmap store
   ├─ Update HNSW vector index
   ├─ Rebuild footer indexes
   └─ Advance WAL checkpoint
```

### Recovery Process

On startup after crash:

```rust
pub fn recover_from_wal(&mut self) -> Result<u64> {
    let mut recovered = 0u64;
    
    // Read WAL from last checkpoint
    let operations = self.wal.read_from_checkpoint()?;
    
    for op in operations {
        match op {
            Operation::WriteConcept { id, content, .. } => {
                self.apply_concept_write(id, content)?;
                recovered += 1;
            }
            Operation::WriteAssociation { source, target, .. } => {
                self.apply_association_write(source, target)?; 
                recovered += 1;
            }
            // ... handle other operations
        }
    }
    
    // Rebuild indexes after recovery
    self.rebuild_indexes()?;
    
    Ok(recovered)
}
```

## HNSW Vector Index

### Persistent Vector Search

Sutra maintains a persistent HNSW (Hierarchical Navigable Small Worlds) index:

```
vectors.idx Structure:
├─ Index Header (metadata)
├─ Node Pool (graph nodes)
├─ Edge Lists (connectivity)
└─ Vector Data (embeddings)
```

### Index Performance

| Metric | Value | Notes |
|--------|-------|-------|
| Startup Time | 3.5ms | For 1M vectors (persistent mmap) |
| Query Latency | <1ms | k=10 nearest neighbors |
| Index Size | ~0.1KB per vector | Excluding embeddings |
| Recall@10 | >95% | With ef_search=100 |

### Vector Operations

```python
# Add vectors automatically during learning
concept_id = client.learn_concept_v2(
    content="Heart disease requires lifestyle changes.",
    # Embedding generated automatically
)

# Search similar concepts
results = client.vector_search(
    query_text="cardiovascular health recommendations",
    k=10,
    ef_search=50  # Quality vs speed tradeoff
)

# Results include similarity scores
for result in results:
    print(f"Concept: {result['concept_id']}")
    print(f"Similarity: {result['similarity']:.3f}")
    print(f"Content: {result['content'][:100]}...")
```

## Storage Files

### File Layout

```bash
# Storage directory structure
/data/storage/
├── domain-storage.dat      # Domain knowledge (main file)
├── domain-wal.log          # Domain WAL
├── domain-vectors.idx      # Domain HNSW index
├── user-storage.dat        # Multi-tenant user data
├── user-wal.log            # User data WAL
├── user-vectors.idx        # User data HNSW index
└── locks/
    ├── domain.lock         # Prevent concurrent access
    └── user.lock
```

### File Size Estimation

| Data Type | Size per Item | 1M Items | 10M Items |
|-----------|---------------|----------|-----------|
| Concepts | 128 bytes + content | ~200MB | ~2GB |
| Associations | 64 bytes | ~64MB | ~640MB |
| Vectors (768d) | ~3KB | ~3GB | ~30GB |
| HNSW Index | ~100 bytes | ~100MB | ~1GB |
| **Total** | | **~3.4GB** | **~34GB** |

## Performance Optimization

### 1. Memory Configuration

```bash
# Optimal OS settings
echo 'vm.swappiness=1' >> /etc/sysctl.conf
echo 'vm.vfs_cache_pressure=50' >> /etc/sysctl.conf

# Increase file descriptor limits
ulimit -n 65536

# Use huge pages for large mmaps
echo always > /sys/kernel/mm/transparent_hugepage/enabled
```

### 2. Batch Operations

```python
# Efficient bulk loading
concept_ids = client.learn_batch_v2([
    "Concept 1 content...",
    "Concept 2 content...", 
    # ... up to 1000 per batch
])

# Batch associations
associations = []
for source_id, target_id in concept_pairs:
    associations.append({
        "source_id": source_id,
        "target_id": target_id,
        "assoc_type": AssociationType.Semantic,
        "confidence": 0.8
    })
client.learn_associations_batch(associations)
```

### 3. Storage Maintenance

```python
# Periodic maintenance operations
client.flush()              # Force WAL checkpoint
client.compact()            # Reclaim deleted space  
client.rebuild_indexes()    # Optimize lookup performance
```

## Backup and Recovery

### 1. File-Level Backup

```bash
# Stop storage server gracefully
./sutra-deploy.sh stop storage-server

# Backup storage files
cp -r /data/storage /backup/storage-$(date +%Y%m%d)

# Verify backup integrity
./sutra-storage-tool verify /backup/storage-$(date +%Y%m%d)/*.dat
```

### 2. Incremental Backup

```bash
# Backup only WAL since last backup
./sutra-storage-tool export-wal \
    --since-sequence 1000000 \
    --output /backup/incremental-$(date +%Y%m%d).wal
```

### 3. Recovery from Backup

```bash
# Restore full backup
cp -r /backup/storage-20241027/* /data/storage/

# Apply incremental WAL
./sutra-storage-tool import-wal \
    --input /backup/incremental-20241028.wal \
    --storage /data/storage/domain-storage.dat
```

## Troubleshooting Storage Issues

### Common Problems

#### 1. Corruption Detection

```bash
# Verify file integrity
./sutra-storage-tool verify storage.dat
# Output: Checksum mismatch at offset 0x12345 (expected: abc123, got: def456)

# Repair if possible
./sutra-storage-tool repair storage.dat --backup storage.dat.backup
```

#### 2. Performance Issues

```python
# Check storage statistics
stats = client.get_stats()
print(f"Concepts: {stats['concepts']}")
print(f"Pending writes: {stats['pending']}")
print(f"WAL size: {stats['wal_size_mb']} MB")

# If WAL is large, force checkpoint
if stats['wal_size_mb'] > 1000:
    client.flush()
```

#### 3. Disk Space Management

```bash
# Monitor storage growth
du -sh /data/storage/
# 15G    /data/storage/

# Clean up old WAL segments
./sutra-storage-tool cleanup-wal --keep-days 7
```

## Next Steps

- [**TCP Protocol**](./03-tcp-protocol.md) - Learn the communication interface
- [**Performance Guide**](./08-performance.md) - Advanced optimization techniques
- [**Troubleshooting**](./09-troubleshooting.md) - Solving storage problems

---

*The binary storage format provides the performance foundation for real-time AI reasoning at scale.*
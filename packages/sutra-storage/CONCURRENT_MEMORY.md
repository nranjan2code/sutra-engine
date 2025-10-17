# Concurrent Memory - Burst-Tolerant Storage for Continuous Learning

## Overview

**ConcurrentMemory** is a custom storage system designed specifically for **unpredictable burst patterns** in continuous learning AI systems. Unlike traditional databases optimized for steady workloads, this handles extreme fluctuations:

```
t=0s:   ━━━ 2000 writes/sec, 10 reads/sec   (LEARNING)
t=30s:  ─── 10 writes/sec, 3000 reads/sec  (REASONING)
t=60s:  ━━━ 1500 writes/sec, 20 reads/sec   (LEARNING)
```

**Key Innovation: Zero interference between reads and writes.**

## Architecture

### Three-Plane Design

```
┌─────────────────────────────────────────────────────────┐
│                    APPLICATION                          │
└─────────┬──────────────────────────────────────┬────────┘
          │                                      │
    WRITES (never block)              READS (never block)
          │                                      │
          ↓                                      ↓
┌──────────────────────┐              ┌─────────────────────┐
│   WRITE LOG          │              │   READ VIEW         │
│   (lock-free queue)  │              │   (immutable snap)  │
│                      │              │                     │
│ - Crossbeam channel  │              │ - Arc<DashMap>      │
│ - 100K capacity      │              │ - ArcSwap atomic    │
│ - Bounded backpress  │              │ - Zero-copy reads   │
└──────────────────────┘              └─────────────────────┘
          │                                      ↑
          │         ┌───────────────┐           │
          └────────→│  RECONCILER   │──────────┘
                    │  (background)  │
                    │                │
                    │ - Drains log   │
                    │ - Updates snap │
                    │ - Flushes disk │
                    └───────────────┘
```

### 1. Write Plane (WriteLog)

**Lock-free append-only log for burst writes.**

```rust
pub struct WriteLog {
    sender: Sender<WriteEntry>,     // Writers append here
    receiver: Receiver<WriteEntry>,  // Reconciler drains here
    sequence: AtomicU64,             // Monotonic sequence
}
```

**Operations:**
- `append_concept()` - Add concept (< 1μs, never blocks)
- `append_association()` - Add edge (< 1μs, never blocks)
- `drain_batch()` - Reconciler reads (non-blocking)

**Backpressure:**
- Bounded queue (100K entries)
- On overflow: drops oldest, accepts newest
- Metric: `dropped` counter for monitoring

### 2. Read Plane (ReadView)

**Immutable snapshots for zero-contention reads.**

```rust
pub struct ReadView {
    snapshot: ArcSwap<GraphSnapshot>,  // Atomic pointer swap
}

pub struct GraphSnapshot {
    concepts: Arc<DashMap<ConceptId, ConceptNode>>,
    sequence: u64,
    timestamp: u64,
}
```

**Operations:**
- `load()` - Get snapshot (< 1μs, zero-copy)
- `find_path()` - Graph traversal (no locks)
- `get_neighbors()` - Edge queries (no locks)

**Snapshot properties:**
- Immutable (readers never see partial updates)
- Versioned (sequence number)
- Timestamped (microsecond precision)
- Stale (1-10ms behind writes)

### 3. Reconciliation Plane (Reconciler)

**Background thread that merges write log into read view.**

```rust
// Reconciliation loop (runs every 10ms)
loop {
    // 1. Drain write log
    let batch = write_log.drain_batch(10_000);
    
    // 2. Clone snapshot structure (cheap due to Arc)
    let mut new_snap = current_snap.clone_structure();
    
    // 3. Apply all writes
    for entry in batch {
        new_snap.apply(entry);
    }
    
    // 4. Atomic swap (readers instantly see new snapshot)
    read_view.store(new_snap);
    
    // 5. Flush to disk if threshold reached
    if new_snap.size() > threshold {
        flush_to_segment(new_snap);
    }
}
```

**Tunables:**
- `reconcile_interval_ms` (default: 10ms)
- `max_batch_size` (default: 10K)
- `disk_flush_threshold` (default: 50K concepts)

## Usage

### Basic Operations

```rust
use sutra_storage::{ConcurrentMemory, ConcurrentConfig, ConceptId};

// Create concurrent memory
let config = ConcurrentConfig {
    storage_path: "./storage".into(),
    reconcile_interval_ms: 10,
    memory_threshold: 50_000,
};

let memory = ConcurrentMemory::new(config);

// Learn (write) - never blocks
let id = ConceptId::from_bytes([1; 16]);
memory.learn_concept(
    id, 
    b"knowledge content".to_vec(), 
    None,  // optional vector
    1.0,   // strength
    0.9    // confidence
)?;

// Reason (read) - never blocks
if let Some(concept) = memory.query_concept(&id) {
    println!("Content: {:?}", concept.content);
    println!("Neighbors: {:?}", concept.neighbors);
}

// Find path (graph traversal)
let path = memory.find_path(start_id, end_id, max_depth);
```

### Burst Patterns

**Learning Burst (write-heavy):**
```rust
// 1000 writes/sec, minimal reconciliation lag
for i in 0..1000 {
    memory.learn_concept(id, content, None, 1.0, 0.9)?;
}
// Returns immediately, reconciles in background
```

**Reasoning Burst (read-heavy):**
```rust
// 3000 queries/sec, zero write interference
for query in queries {
    let path = memory.find_path(start, end, depth);
    // Reads from immutable snapshot
}
```

**Mixed Burst (concurrent):**
```rust
// Writer thread
thread::spawn(|| {
    memory.learn_concept(id, content, None, 1.0, 0.9)?;
});

// Reader thread (simultaneous)
thread::spawn(|| {
    memory.query_neighbors(&id);
});
// Zero contention between threads
```

## Performance Characteristics

### Writes
- **Latency:** < 1μs (append to queue)
- **Throughput:** Limited only by memory allocation
- **Blocking:** Never (bounded queue with backpressure)

### Reads
- **Latency:** O(1) for lookups, O(V+E) for paths
- **Throughput:** Unlimited parallel reads
- **Blocking:** Never (immutable snapshots)
- **Staleness:** 1-10ms (configurable via reconcile_interval_ms)

### Reconciliation
- **Frequency:** Every 10ms (default)
- **Batch size:** 10K entries (default)
- **CPU:** Single dedicated thread
- **Memory:** 2x graph size (current + new snapshot during swap)

## Disk Format

### Segment Structure

```
┌─────────────────────────────────────────────────────────────┐
│ SEGMENT HEADER (256 bytes)                                  │
├─────────────────────────────────────────────────────────────┤
│ CONCEPT RECORDS (128 bytes each)                            │
│ ┌──────────────────────────────────────────────────────┐    │
│ │ ConceptRecord { id, strength, confidence, ... }      │    │
│ └──────────────────────────────────────────────────────┘    │
├─────────────────────────────────────────────────────────────┤
│ ASSOCIATION RECORDS (64 bytes each)                         │
│ ┌──────────────────────────────────────────────────────┐    │
│ │ AssociationRecord { source, target, type, ... }      │    │
│ └──────────────────────────────────────────────────────┘    │
├─────────────────────────────────────────────────────────────┤
│ VECTORS (variable length, f32 arrays)                       │
├─────────────────────────────────────────────────────────────┤
│ CONTENT (variable length, binary blobs)                     │
└─────────────────────────────────────────────────────────────┘
```

**Optimizations:**
- Memory-mapped reads (zero-copy)
- Edges co-located with concepts (cache-friendly)
- Append-only (no in-place updates)
- Product quantization for vectors (4x compression)

## Configuration

### Tuning for Different Workloads

**High Write Throughput:**
```rust
ConcurrentConfig {
    reconcile_interval_ms: 50,  // Batch more aggressively
    memory_threshold: 100_000,   // Flush less frequently
    ..Default::default()
}
```

**Low Latency Reads:**
```rust
ConcurrentConfig {
    reconcile_interval_ms: 5,   // Update snapshots faster
    memory_threshold: 10_000,   // Keep memory footprint small
    ..Default::default()
}
```

**Memory Constrained:**
```rust
ConcurrentConfig {
    reconcile_interval_ms: 10,
    memory_threshold: 5_000,    // Flush to disk more frequently
    ..Default::default()
}
```

## Monitoring

### Statistics

```rust
let stats = memory.stats();

// Write log metrics
println!("Writes: {}", stats.write_log.written);
println!("Dropped: {}", stats.write_log.dropped);  // Backpressure indicator
println!("Pending: {}", stats.write_log.pending);

// Reconciler metrics
println!("Reconciliations: {}", stats.reconciler.reconciliations);
println!("Entries processed: {}", stats.reconciler.entries_processed);
println!("Disk flushes: {}", stats.reconciler.disk_flushes);

// Snapshot metrics
println!("Concepts: {}", stats.snapshot.concept_count);
println!("Edges: {}", stats.snapshot.edge_count);
println!("Sequence: {}", stats.snapshot.sequence);
println!("Timestamp: {}", stats.snapshot.timestamp);
```

### Health Checks

**Write lag:** `stats.write_log.pending > 10_000` → System under write pressure

**Backpressure:** `stats.write_log.dropped > 0` → Queue overflow, increase capacity or reconciliation frequency

**Reconciliation lag:** `current_time - stats.snapshot.timestamp > 100ms` → Reconciler falling behind

## Running the Demo

```bash
cd packages/sutra-storage

# Build
cargo build --release

# Run demo
cargo run --example concurrent_burst_demo --release
```

**Demo output:**
```
=== Concurrent Memory Burst Demo ===

📚 BURST 1: Learning Phase (Write-Heavy)
  ⚡ Wrote 1,000 concepts in 2ms
     (500,000 writes/sec)
  ✓ Reconciled 1000 concepts

🔗 BURST 2: Association Building
  ⚡ Created 500 associations in 1ms
  ✓ Graph now has 1000 edges

🧠 BURST 3: Reasoning Phase (Read-Heavy)
  ⚡ Executed 5000 queries in 150ms
     (33,333 queries/sec)
  ✓ Found 4500 paths

⚡ BURST 4: Mixed Phase (Concurrent Read/Write)
  ⚡ Writer: 500 concepts in 50ms
  ⚡ Reader: 100000 queries in 100ms
  ✓ Zero interference between reads and writes
```

## Testing

```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test '*'

# Specific modules
cargo test write_log
cargo test read_view
cargo test reconciler
cargo test concurrent_memory
```

## Design Rationale

### Why Not Traditional MVCC?

Traditional databases use MVCC (Multi-Version Concurrency Control):
- Still has lock contention on version chains
- Garbage collection overhead
- Read latency increases with write rate

**ConcurrentMemory eliminates all locks:**
- Writes → lock-free queue (no coordination)
- Reads → immutable snapshots (no coordination)
- No garbage collection (old snapshots drop when unused)

### Why Stale Reads (1-10ms)?

For reasoning AI:
- **Acceptable:** Reasoning on 1-10ms old data is fine
- **Beneficial:** Zero read latency > always-fresh data
- **Realistic:** Human reasoning uses stale information constantly

If you need stronger consistency, decrease `reconcile_interval_ms` to 1ms (higher CPU cost).

### Why Not Separate Read/Write Databases?

Write-DB + Async-Replicate → Read-DB has:
- Network overhead (even localhost)
- Complex failover logic
- Two systems to configure/monitor

**ConcurrentMemory is one system:**
- In-process (zero network)
- Automatic reconciliation
- Single configuration

## Future Optimizations

### Phase 1 (Implemented) ✅
- Lock-free write log
- Immutable read snapshots
- Background reconciler
- Disk segment flushing

### Phase 2 (Planned)
- **Bloom filters** in segments (avoid futile seeks)
- **Edge co-location** in disk format (cache-friendly)
- **Compressed snapshots** (reduce memory 2x)
- **Parallel reconciliation** (multi-threaded apply)

### Phase 3 (Future)
- **Memory-mapped snapshots** (reduce memory 4x)
- **Incremental snapshots** (only delta, not full clone)
- **SIMD graph traversal** (vectorized path finding)
- **GPU-accelerated reconciliation** (for large graphs)

## License

MIT (same as main project)

## Contact

For questions or issues with ConcurrentMemory, open an issue on the main repository.

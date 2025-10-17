# Sutra Storage Architecture

## Design Philosophy

Sutra Storage is a **custom storage engine** designed for continuous learning AI systems with unpredictable burst patterns. Unlike traditional databases optimized for steady workloads, this storage is purpose-built for:

1. **Unpredictable Bursts** - Write-heavy learning → read-heavy reasoning → mixed phases, flipping every few seconds
2. **Zero Interference** - Concurrent reads and writes without blocking each other
3. **Temporal Memory** - Recent knowledge is "hot" by nature, old knowledge fades
4. **Complete Explainability** - Full audit trail, never delete historical data
5. **Reasoning-First** - Graph traversal and path finding as primary operations

### Core Principles

**Principle 1: Separate Read and Write Planes**
```
Writers → Lock-free Log → [Background Reconciler] → Immutable Snapshots ← Readers
```
- Writers never wait for readers
- Readers never wait for writers
- Reconciliation happens invisibly in background

**Principle 2: Single-File Storage**
```
One file = One system view
├─ OS manages paging automatically (KB → GB scaling)
├─ No fragmentation across multiple files
└─ Human-visible simplicity
```

**Principle 3: Zero-Copy Internally**
```
Write → mmap arena (direct memory)
Read  → pointer into mmap (no deserialization)
Convert → only at system boundaries (Python API, disk sync)
```

**Principle 4: Memory-Adaptive**
```
Small graphs  (< 100MB)  → Pure in-memory
Medium graphs (100MB-1GB) → Memory-mapped, OS decides
Large graphs  (> 1GB)     → Memory-mapped, lazy loading
```

## System Overview

### Two-Layer Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   APPLICATION LAYER                      │
│  (Python API, ReasoningEngine, PathFinder, MPPA)        │
└──────────────────┬────────────────────┬──────────────────┘
                   │                    │
          LEARN (write)          REASON (read)
                   │                    │
                   ↓                    ↓
┌──────────────────────────────────────────────────────────┐
│              CONCURRENT MEMORY LAYER                      │
│  ┌─────────────────┐          ┌─────────────────┐       │
│  │  Write Plane    │          │  Read Plane     │       │
│  │  (WriteLog)     │          │  (ReadView)     │       │
│  │                 │          │                 │       │
│  │  - Lock-free    │          │  - Immutable    │       │
│  │  - Append-only  │          │  - Snapshots    │       │
│  │  - 100K queue   │          │  - Zero-copy    │       │
│  └────────┬────────┘          └────────▲────────┘       │
│           │                            │                 │
│           │     ┌──────────────┐      │                 │
│           └────→│  Reconciler  │──────┘                 │
│                 │  (10ms loop) │                        │
│                 └──────┬───────┘                        │
└────────────────────────┼────────────────────────────────┘
                         │
                         ↓
┌──────────────────────────────────────────────────────────┐
│              STORAGE LAYER (MmapStore)                    │
│                                                           │
│           storage.dat (single memory-mapped file)        │
│  ┌───────────────────────────────────────────────────┐  │
│  │ Header  │ Concepts │ Edges │ Vectors │ Content   │  │
│  │ (256B)  │ (arena)  │(arena)│ (blob)  │ (blob)    │  │
│  └───────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────┐  │
│  │ Footer: Bloom Filter + Offset Index               │  │
│  └───────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

## Component Responsibilities

### 1. Application Layer
- **Input:** Natural language queries, raw data
- **Output:** Reasoning paths, explanations, confidence scores
- **Role:** Business logic, no storage concerns

### 2. Concurrent Memory Layer
- **Input:** Learn/reason operations
- **Output:** Concepts, associations, paths
- **Role:** Burst-tolerant coordination, zero contention

#### 2a. Write Plane (WriteLog)
```
Responsibility: Accept writes at any rate without blocking
```
- Bounded lock-free queue (crossbeam channel)
- Capacity: 100,000 entries
- Backpressure: Drop oldest on overflow (metric tracked)
- Latency: < 1μs per write

#### 2b. Read Plane (ReadView)
```
Responsibility: Serve reads at any rate without blocking
```
- Immutable graph snapshots (Arc-wrapped DashMap)
- Atomic pointer swap (arc-swap)
- Staleness: 1-10ms (configurable)
- Latency: < 1μs per lookup, O(V+E) for paths

#### 2c. Reconciler
```
Responsibility: Merge writes into read view invisibly
```
- Background thread, runs every 10ms (tunable)
- Drains write log (non-blocking)
- Applies batch to new snapshot
- Atomic swap (readers instantly see new data)
- Flushes to disk when threshold reached

### 3. Storage Layer (MmapStore)
- **Input:** Concepts, associations, vectors, content
- **Output:** Persisted file, fast lookups
- **Role:** Durable single-file storage with zero-copy access

## Data Flow

### Write Path
```
1. learn_concept(id, content, vector)
   │
   ↓ (< 1μs, never blocks)
2. WriteLog.append(entry)
   │
   ↓ (queue, non-blocking)
3. [Background: every 10ms]
   Reconciler.drain_batch()
   │
   ↓ (apply to new snapshot)
4. GraphSnapshot.apply(entries)
   │
   ↓ (atomic pointer swap)
5. ReadView.store(new_snapshot)
   │
   ↓ (when threshold reached)
6. MmapStore.flush_to_disk()
```

### Read Path
```
1. query_concept(id) or find_path(start, end)
   │
   ↓ (< 1μs, load pointer)
2. ReadView.load()  → Arc<GraphSnapshot>
   │
   ↓ (zero-copy access)
3. snapshot.get_concept(id)
   │  or
   snapshot.find_path(start, end)
   │
   ↓ (O(1) or O(V+E), no locks)
4. Return result
```

## Consistency Model

### Eventual Consistency (Tunable Lag)

**Write visibility:**
```
t₀: Write submitted     → WriteLog
t₁: Reconciled (10ms)   → ReadView updated
t₂: Persisted (threshold) → Disk synced
```

**Read consistency:**
- Readers see snapshot from t₁ (1-10ms old)
- Within snapshot: perfectly consistent (no partial updates)
- Across snapshots: monotonic reads (sequence numbers)

**Trade-off justification:**
- AI reasoning on 10ms-old data is acceptable
- Zero read latency > always-fresh data
- Human reasoning uses stale information constantly

### Durability Guarantees

**Write-Ahead Log (implicit in WriteLog):**
```
Write → Queue (in-memory, not yet durable)
  ↓
Reconcile → Snapshot (in-memory)
  ↓
Flush → storage.dat (durable)
```

**Crash scenarios:**
1. Crash before reconciliation → Lost (acceptable for learning bursts)
2. Crash after reconciliation → Lost (in-memory snapshot not yet flushed)
3. Crash after flush → Durable (footer indexes persisted)

**Tunability:**
- Decrease `reconcile_interval_ms` → Faster visibility
- Decrease `memory_threshold` → More frequent flushes, more durability

## Scalability Characteristics

### Horizontal (Multi-Instance)
```
Not applicable - single-machine in-process design
```
Reasoning: Latency-critical graph traversals require in-process memory access.

### Vertical (Single-Machine)

**Concepts:**
```
Small:   1K-100K concepts  → ~10MB-1GB memory
Medium:  100K-1M concepts  → ~1GB-10GB memory
Large:   1M-10M concepts   → ~10GB-100GB memory
```

**Scaling strategy:**
```
Memory: OS-managed paging (mmap handles it)
CPU:    Single reconciler thread (< 1% overhead)
Disk:   Single file grows dynamically
```

**Limits:**
- Practical: ~10M concepts (~100GB) on modern hardware
- Theoretical: 2⁶⁴ bytes (file offset limit)

## Performance Model

### Latency

**Write operations:**
```
learn_concept():     < 1μs   (append to queue)
learn_association(): < 1μs   (append to queue)
```

**Read operations:**
```
query_concept():     < 1μs   (HashMap lookup + pointer)
query_neighbors():   O(degree) × 100ns  (adjacent memory)
find_path():         O(V + E) × 100ns  (BFS/DFS)
```

**Reconciliation:**
```
Batch size: 10K entries
Time:       ~1-2ms
Frequency:  10ms (90% idle)
```

### Throughput

**Burst capacity:**
```
Writes:  Limited by memory allocation (~1M/sec)
Reads:   Unlimited parallel (immutable snapshots)
Mixed:   Both simultaneously at full speed
```

**Sustained throughput:**
```
Writes:  100K/sec (reconciler keeps up at 10K/10ms)
Reads:   Millions/sec (no coordination needed)
```

## Design Trade-offs

### Chosen: Speed over Strict Consistency
**Alternative:** ACID transactions, strict linearizability
**Why not:** Would require locks → blocking → burst intolerance

### Chosen: Single File over Sharded Files
**Alternative:** LSM-tree with multiple segments
**Why not:** Fragmentation, compaction overhead, human complexity

### Chosen: In-Memory Index over Disk-Only
**Alternative:** Pure on-disk B-tree
**Why not:** Index is tiny (~24B/concept), lookup speed critical

### Chosen: Stale Reads over Always Fresh
**Alternative:** Read-after-write consistency
**Why not:** Would couple read/write planes, destroy concurrency

### Chosen: Monolithic over Microservices
**Alternative:** Separate read/write services
**Why not:** Network latency unacceptable for graph traversal

## Summary

Sutra Storage achieves **burst tolerance through separation**:
- Write plane and read plane never interact directly
- Reconciler is the only coupling point (invisible)
- Each plane optimized independently for its workload
- Single-file storage provides simplicity and OS-managed scaling

Next: [Memory Layout](./02-memory-layout.md)

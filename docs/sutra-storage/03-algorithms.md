# Core Algorithms

## 1. Concurrent Reconciliation Algorithm

### Problem Statement

**Given:**
- Write log W = [w₀, w₁, ..., wₙ] (entries from concurrent writers)
- Current graph snapshot Gₜ = (V, E) at time t
- Target: next snapshot Gₜ₊₁ incorporating all writes

**Goal:** Apply W to Gₜ atomically without blocking readers.

### Algorithm

```
function RECONCILE(write_log, current_snapshot, interval_ms):
    loop every interval_ms:
        // 1. Drain write log (non-blocking)
        batch ← write_log.drain_batch(MAX_BATCH_SIZE)
        
        if batch.is_empty():
            continue
        
        // 2. Clone snapshot structure (cheap via Arc)
        new_snapshot ← current_snapshot.clone_structure()
        new_snapshot.sequence ← current_snapshot.sequence + 1
        new_snapshot.timestamp ← now()
        
        // 3. Apply all writes
        for entry in batch:
            apply_write(new_snapshot, entry)
        
        // 4. Update statistics
        new_snapshot.update_stats()
        
        // 5. Atomic pointer swap (readers instantly see new snapshot)
        atomic_swap(snapshot_ptr, new_snapshot)
        
        // 6. Check disk flush threshold
        if new_snapshot.concept_count > DISK_THRESHOLD:
            flush_to_disk_async(new_snapshot)
```

**Complexity:**
```
Time:  O(|batch| × α) where α = O(1) average insert time
Space: O(|V| + |E|) for new snapshot
```

**Properties:**
- **Lock-free:** No mutual exclusion primitives
- **Wait-free for readers:** Readers never blocked
- **Bounded staleness:** max latency = interval_ms
- **Monotonic:** sequence numbers strictly increasing

### Write Application

```
function APPLY_WRITE(snapshot, entry):
    match entry:
        AddConcept{id, content, vector, ...}:
            node ← ConceptNode.new(id, content, vector)
            snapshot.concepts[id] ← node
        
        AddAssociation{source, target, record}:
            // Add forward edge
            if snapshot.concepts.contains(source):
                snapshot.concepts[source].add_edge(target, record)
            
            // Add backward edge (bidirectional)
            if snapshot.concepts.contains(target):
                snapshot.concepts[target].add_edge(source, record)
        
        UpdateStrength{id, strength}:
            if snapshot.concepts.contains(id):
                snapshot.concepts[id].strength ← strength
        
        RecordAccess{id, timestamp}:
            if snapshot.concepts.contains(id):
                snapshot.concepts[id].last_accessed ← timestamp
                snapshot.concepts[id].access_count += 1
```

**Idempotency:** Not guaranteed. Last write wins for conflicts.

## 2. Path Finding Algorithm

### Breadth-First Search (BFS)

**Problem:** Find shortest path from start to end in unweighted graph.

**Algorithm:**
```
function FIND_PATH(graph, start, end, max_depth):
    if start == end:
        return [start]
    
    queue ← Queue()
    visited ← HashMap()
    
    queue.enqueue((start, 0))
    visited[start] ← None
    
    while !queue.is_empty():
        (current, depth) ← queue.dequeue()
        
        if depth >= max_depth:
            continue
        
        neighbors ← graph.get_neighbors(current)
        
        for neighbor in neighbors:
            if !visited.contains(neighbor):
                visited[neighbor] ← current
                
                if neighbor == end:
                    // Reconstruct path
                    return reconstruct_path(visited, start, end)
                
                queue.enqueue((neighbor, depth + 1))
    
    return None  // No path found

function RECONSTRUCT_PATH(visited, start, end):
    path ← [end]
    current ← end
    
    while current != start:
        current ← visited[current]
        path.prepend(current)
    
    return path
```

**Complexity:**
```
Time:  O(V + E) where V = vertices, E = edges
Space: O(V) for visited set and queue
```

**Properties:**
- **Optimal:** Returns shortest path (minimum hops)
- **Complete:** Finds path if one exists within max_depth
- **Lock-free:** Reads immutable snapshot

### Confidence-Weighted Best-First Search

**Problem:** Find highest-confidence path (not shortest).

**Algorithm:**
```
function FIND_BEST_PATH(graph, start, end, max_depth):
    // Priority queue: (node, path, cumulative_confidence)
    pq ← MaxPriorityQueue()
    visited ← HashSet()
    
    pq.push((start, [start], 1.0))
    
    while !pq.is_empty():
        (current, path, confidence) ← pq.pop()
        
        if current in visited:
            continue
        
        visited.add(current)
        
        if current == end:
            return (path, confidence)
        
        if path.length >= max_depth:
            continue
        
        neighbors ← graph.get_neighbors_weighted(current)
        
        for (neighbor, edge_confidence) in neighbors:
            if neighbor not in visited:
                new_conf ← confidence × edge_confidence × DECAY
                new_path ← path.append(neighbor)
                pq.push((neighbor, new_path, new_conf))
    
    return None
```

**Complexity:**
```
Time:  O((V + E) × log V) due to priority queue
Space: O(V × D) where D = max_depth (storing paths)
```

**Confidence decay:**
```
conf_path(p) = conf₀ × ∏ᵢ₌₁ⁿ (edge_confᵢ × δ)

where:
    conf₀ = initial confidence (1.0)
    edge_confᵢ = confidence of i-th edge
    δ = decay factor (typically 0.85)
    n = path length
```

## 3. Bloom Filter Operations

### Initialization

**Given:** N concepts, target false positive rate p

**Optimal parameters:**
```
m = -N × ln(p) / (ln(2))²     (bit array size)
k = (m/N) × ln(2)             (hash function count)

For our case:
    p = 0.01 (1% false positive)
    k = 2 (practical choice)
    m = 2N bits (simplified)
```

**Space efficiency:**
```
bits_per_concept = m / N = 2 bits/concept
bytes_per_million = 1,000,000 × 2 / 8 = 250 KB
```

### Insertion

```
function BLOOM_INSERT(bloom, concept_id):
    hash ← FNV1a(concept_id)
    
    h₁ ← hash mod m
    h₂ ← (hash / m) mod m
    
    bloom.set_bit(h₁)
    bloom.set_bit(h₂)
```

**Hash function (FNV-1a):**
```
function FNV1a(data):
    hash ← 0xcbf29ce484222325  (offset basis)
    
    for byte in data:
        hash ← hash XOR byte
        hash ← hash × 0x100000001b3  (FNV prime)
    
    return hash
```

**Properties:**
- **Fast:** ~10 ns per operation
- **Simple:** Two array accesses
- **Deterministic:** Same input → same bits

### Membership Test

```
function BLOOM_CONTAINS(bloom, concept_id):
    hash ← FNV1a(concept_id)
    
    h₁ ← hash mod m
    h₂ ← (hash / m) mod m
    
    if !bloom.get_bit(h₁):
        return DEFINITELY_NOT_PRESENT
    
    if !bloom.get_bit(h₂):
        return DEFINITELY_NOT_PRESENT
    
    return POSSIBLY_PRESENT
```

**False positive rate (actual):**
```
P(false_positive) = (1 - e^(-kN/m))^k

For k=2, m=2N:
    P ≈ (1 - e^(-1))^2 ≈ 0.01 (1%)
```

**Trade-off:**
- 1% of negative queries → unnecessary index lookup
- 99% of negative queries → saved from expensive I/O

## 4. Memory Management Algorithm

### Arena Allocation

**Concept:** Bump-pointer allocator for append-only structures.

```
struct Arena:
    base_offset: u64      // Start of arena
    current: u64          // Current write position
    capacity: u64         // Max size before growth

function ARENA_ALLOC(arena, size, alignment):
    // Align current position
    aligned ← align_up(arena.current, alignment)
    
    // Check capacity
    if aligned + size > arena.capacity:
        grow_file(arena.capacity × 2)
        arena.capacity ← arena.capacity × 2
    
    // Allocate
    offset ← aligned
    arena.current ← aligned + size
    
    return offset
```

**Alignment formula:**
```
align_up(x, a) = ⌈x/a⌉ × a = (x + a - 1) & ~(a - 1)
```

**Properties:**
- **Fast:** O(1) allocation, just pointer bump
- **No fragmentation:** Append-only, no reuse
- **Cache-friendly:** Sequential access pattern

### File Growth Strategy

**Problem:** Grow file without blocking operations.

```
function GROW_FILE(current_size, needed_size):
    // Compute new size (power of 2 for alignment)
    new_size ← max(
        next_power_of_two(needed_size),
        current_size × 2
    )
    
    // Extend file (OS may use sparse allocation)
    ftruncate(fd, new_size)
    
    // Remap memory (may be expensive)
    new_mmap ← mremap(old_mmap, new_size)
    
    return new_mmap

function NEXT_POWER_OF_TWO(n):
    if n == 0:
        return 1
    
    n ← n - 1
    n ← n | (n >> 1)
    n ← n | (n >> 2)
    n ← n | (n >> 4)
    n ← n | (n >> 8)
    n ← n | (n >> 16)
    n ← n | (n >> 32)
    
    return n + 1
```

**Growth sequence:**
```
256 MB → 512 MB → 1 GB → 2 GB → 4 GB → 8 GB → ...

Growth factor: 2× (exponential)
Amortized cost: O(1) per byte
```

## 5. Snapshot Creation Algorithm

### Copy-on-Write (CoW) Snapshot

**Problem:** Create new snapshot without copying entire graph.

```
function CREATE_SNAPSHOT(old_snapshot):
    // Arc-wrapped DashMap is cheap to clone (reference count)
    new_snapshot ← GraphSnapshot {
        concepts: Arc::clone(&old_snapshot.concepts),
        sequence: old_snapshot.sequence + 1,
        timestamp: now(),
        concept_count: old_snapshot.concept_count,
        edge_count: old_snapshot.edge_count,
    }
    
    return new_snapshot
```

**Cost analysis:**
```
Shallow clone: O(1) - just increment Arc refcount
Deep clone:    O(N) - would copy all concepts

We use shallow clone, modifications create new entries.
```

**Memory sharing:**
```
Before write: snapshot₁ → DashMap (refcount = 1)
After clone:  snapshot₁ → DashMap (refcount = 2)
              snapshot₂ ↗

After write:  snapshot₁ → DashMap₁ (refcount = 1)
              snapshot₂ → DashMap₂ (refcount = 1)
                          (with new entry)
```

### Atomic Snapshot Swap

```
function ATOMIC_SWAP(snapshot_ptr, new_snapshot):
    // ArcSwap provides lock-free atomic pointer exchange
    old ← snapshot_ptr.swap(Arc::new(new_snapshot))
    
    // old snapshot still accessible by existing readers
    // will be dropped when last reader releases it
    
    return old
```

**Memory ordering:**
```
Store: atomic with Release semantics
Load:  atomic with Acquire semantics

Guarantees:
    - Readers see consistent snapshot
    - No torn reads/writes
    - Happens-before relationship established
```

## 6. Index Persistence Algorithm

### Footer Write

**Problem:** Write index to end of file atomically.

```
function WRITE_FOOTER(mmap, concept_index):
    // 1. Compute footer position
    footer_start ← content_end
    footer_start ← align_up(footer_start, 8)
    
    // 2. Write Bloom filter
    bloom ← build_bloom_filter(concept_index.keys())
    bloom_off ← footer_start
    mmap.write(bloom_off, bloom)
    bloom_bytes ← bloom.size()
    
    // 3. Write offset index
    index_off ← bloom_off + bloom_bytes
    index_off ← align_up(index_off, 8)
    
    for (concept_id, offset) in concept_index:
        mmap.write(index_off, concept_id.bytes)
        mmap.write(index_off + 16, offset.to_le_bytes)
        index_off += 24
    
    index_count ← concept_index.len()
    
    // 4. Update header (atomic commit point)
    header.bloom_off ← bloom_off
    header.bloom_bytes ← bloom_bytes
    header.index_off ← index_off
    header.index_count ← index_count
    header.epoch ← header.epoch + 1
    
    mmap.write(0, header)
    mmap.flush()
```

**Atomicity guarantee:**
- Header write is last operation
- If crash before header update → old footer still valid
- If crash after header update → new footer valid
- No intermediate inconsistent state

### Footer Read

```
function READ_FOOTER(mmap, header):
    concept_index ← HashMap::new()
    
    if header.index_count == 0:
        return concept_index  // No index yet
    
    index_off ← header.index_off
    
    for i in 0..header.index_count:
        entry_off ← index_off + (i × 24)
        
        concept_id ← mmap.read_bytes(entry_off, 16)
        offset ← mmap.read_u64_le(entry_off + 16)
        
        concept_index[concept_id] ← offset
    
    return concept_index
```

**Complexity:**
```
Time:  O(N) where N = concept_count
Space: O(N) for HashMap
```

## Mathematical Properties

### Staleness Bound

**Theorem:** Maximum read staleness is bounded by reconciliation interval.

**Proof:**
```
Let:
    Δt = reconciliation interval (e.g., 10ms)
    t_write = time of write submission
    t_reconcile = next reconciliation time
    t_read = time of read

Then:
    t_reconcile ≤ t_write + Δt
    staleness = t_read - t_reconcile ≤ Δt

Therefore: max staleness = Δt
```

### Space Complexity

**Total space:**
```
S_total = S_concepts + S_edges + S_vectors + S_content + S_footer

where:
    S_concepts = N × 128 bytes
    S_edges = M × 64 bytes
    S_vectors = N × (4 + d × 4) bytes, d = vector dimension
    S_content = Σᵢ (4 + |cᵢ|) bytes, cᵢ = content of concept i
    S_footer = (N / 4) + (N × 24) bytes
```

**Example (1M concepts, 384-dim vectors):**
```
S_concepts = 1M × 128B = 128 MB
S_edges = 5M × 64B = 320 MB (avg 5 edges/concept)
S_vectors = 1M × (4 + 384×4) = 1.5 GB
S_content = 1M × 50B (avg) = 50 MB
S_footer = 0.25 MB + 24 MB = 24.25 MB

Total ≈ 2 GB
```

### Throughput Model

**Write throughput (sustained):**
```
T_write = min(
    W_app / t_reconcile,      // Application write rate
    C_queue,                  // Queue capacity (100K)
    M_alloc / t_alloc         // Memory allocation rate
)

Example:
    W_app = 50K writes/sec
    t_reconcile = 10ms
    Batch processing = 10K writes/10ms = 1M writes/sec

Therefore: sustained T_write ≈ 50K writes/sec
```

**Read throughput (parallel):**
```
T_read = N_threads × (1 / t_lookup)

where:
    t_lookup ≈ 1μs (hash + pointer deref)
    N_threads = CPU cores

Example (8 cores):
    T_read ≈ 8 × 1M ops/sec = 8M reads/sec
```

## Summary

The algorithms are designed for:
1. **Lock-free operation** - Atomic primitives, no mutexes
2. **Bounded latency** - O(1) or O(log N) operations
3. **Cache efficiency** - Sequential access, aligned data
4. **Space efficiency** - Bloom filters, shared snapshots
5. **Correctness** - Atomic commits, monotonic ordering

Next: [Performance Analysis](./04-performance.md)

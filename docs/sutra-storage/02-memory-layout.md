# Memory Layout

## File Structure Overview

`storage.dat` is a single memory-mapped file with five regions:

```
┌──────────────────────────────────────────────────────────────┐
│ File: storage.dat                                            │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│ [0x000] Header (256 bytes)                                  │
│         ├─ Magic: "SUTRAALL" (8B)                           │
│         ├─ Version: 1 (4B)                                  │
│         ├─ Arena offsets (concept, edge, vector, content)   │
│         ├─ Counts (concept_count, edge_count, etc.)         │
│         ├─ Footer offsets (bloom, index)                    │
│         └─ Epoch counter (monotonic writes)                 │
│                                                              │
├──────────────────────────────────────────────────────────────┤
│ [0x100] Concept Arena (aligned to 128 bytes)                │
│         ├─ ConceptRecord[0] (128B)                          │
│         ├─ ConceptRecord[1] (128B)                          │
│         ├─ ...                                              │
│         └─ ConceptRecord[N-1] (128B)                        │
│                                                              │
├──────────────────────────────────────────────────────────────┤
│ [+64MB] Edge Arena (aligned to 64 bytes)                    │
│         ├─ AssociationRecord[0] (64B)                       │
│         ├─ AssociationRecord[1] (64B)                       │
│         ├─ ...                                              │
│         └─ AssociationRecord[M-1] (64B)                     │
│                                                              │
├──────────────────────────────────────────────────────────────┤
│ [+128MB] Vector Blob (aligned to 4 bytes)                   │
│          ┌──────────────────────┐                           │
│          │ dim: u32 (4B)        │  Vector[0]                │
│          │ data: [f32; dim]     │                           │
│          ├──────────────────────┤                           │
│          │ dim: u32 (4B)        │  Vector[1]                │
│          │ data: [f32; dim]     │                           │
│          └──────────────────────┘                           │
│                                                              │
├──────────────────────────────────────────────────────────────┤
│ [+256MB] Content Blob (aligned to 4 bytes)                  │
│          ┌──────────────────────┐                           │
│          │ len: u32 (4B)        │  Content[0]               │
│          │ data: [u8; len]      │                           │
│          ├──────────────────────┤                           │
│          │ len: u32 (4B)        │  Content[1]               │
│          │ data: [u8; len]      │                           │
│          └──────────────────────┘                           │
│                                                              │
├──────────────────────────────────────────────────────────────┤
│ [EOF-X]  Footer (written on sync, read on open)             │
│          ├─ Bloom Filter (2 bits/concept, aligned 8B)       │
│          └─ Offset Index (24B per concept, aligned 8B)      │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

## Region Details

### 1. Header (256 bytes, offset 0x000)

**Purpose:** Metadata for interpreting the rest of the file.

**Structure (little-endian, packed):**
```
Offset  Size  Field              Description
------  ----  -----              -----------
0x00    8B    magic              "SUTRAALL" (magic bytes)
0x08    4B    version            Format version (currently 1)
0x0C    4B    _pad0              Alignment padding

0x10    8B    concept_off        Concept arena start offset
0x18    8B    edge_off           Edge arena start offset
0x20    8B    vector_off         Vector blob start offset
0x28    8B    content_off        Content blob start offset

0x30    8B    concept_count      Number of concepts written
0x38    8B    edge_count         Number of edges written
0x40    8B    vector_bytes       Total bytes in vector blob
0x48    8B    content_bytes      Total bytes in content blob

0x50    8B    epoch              Write epoch (monotonic)

0x58    8B    bloom_off          Bloom filter start offset
0x60    4B    bloom_bytes        Bloom filter size
0x64    8B    index_off          Offset index start
0x6C    4B    index_count        Number of index entries

0x70    ?     reserved           Reserved for future use
...
0xFF    (end of header at 256 bytes)
```

**Initialization:**
- On new file: compute arena offsets with alignment
- On open: read header, validate magic/version

### 2. Concept Arena (starting at aligned offset)

**Purpose:** Tightly-packed array of concept metadata.

**Alignment:** 128 bytes (for cache line efficiency)

**Structure: ConceptRecord (128 bytes, packed)**
```
Offset  Size  Field              Range/Type
------  ----  -----              ----------
0x00    16B   concept_id         MD5 hash (128-bit UUID)
0x10    4B    strength           float32 ∈ [0.0, 1.0]
0x14    4B    confidence         float32 ∈ [0.0, 1.0]
0x18    4B    access_count       uint32 (usage tracking)
0x1C    8B    created            uint64 (unix timestamp, seconds)
0x24    8B    last_accessed      uint64 (unix timestamp, seconds)
0x2C    8B    content_offset     File offset to content blob
0x34    4B    content_length     Content size in bytes
0x38    8B    embedding_offset   File offset to vector blob
0x40    4B    source_hash        uint32 (checksum)
0x44    4B    flags              uint32 (bitfield for future use)
0x48    32B   reserved1          Reserved
0x68    24B   reserved2          Reserved
------
Total:  128B
```

**Indexing:**
```
concept_offset = concept_arena_base + (index × 128)
```

**Access pattern:**
- Sequential write (append-only)
- Random read (via offset index)

### 3. Edge Arena (starting at aligned offset)

**Purpose:** Association records between concepts.

**Alignment:** 64 bytes

**Structure: AssociationRecord (64 bytes, packed)**
```
Offset  Size  Field              Range/Type
------  ----  -----              ----------
0x00    16B   source_id          ConceptId (16B)
0x10    16B   target_id          ConceptId (16B)
0x20    1B    assoc_type         Enum: 0=Semantic, 1=Causal, etc.
0x21    4B    confidence         float32 ∈ [0.0, 1.0]
0x25    4B    weight             float32 ∈ [0.0, ∞)
0x29    8B    created            uint64 (unix timestamp)
0x31    8B    last_used          uint64 (unix timestamp)
0x39    1B    flags              uint8 bitfield
0x3A    6B    reserved           Padding
------
Total:  64B
```

**Indexing:**
```
edge_offset = edge_arena_base + (index × 64)
```

**Note:** Edges are not co-located with source concepts in disk format (for simplicity). In-memory ReadView co-locates them.

### 4. Vector Blob (variable length, length-prefixed)

**Purpose:** Store embeddings as raw f32 arrays.

**Alignment:** 4 bytes (f32 natural alignment)

**Entry format:**
```
┌────────────────┬──────────────────────────┐
│ dim: u32 (4B)  │ data: [f32; dim] (4×dim) │
└────────────────┴──────────────────────────┘
```

**Example:**
```
Offset    Data
--------  ----
vector_off + 0:  0x0180 0x0000   (dim = 384, little-endian)
             + 4:  f32[0]  f32[1]  ...  f32[383]
             + (4 + 384×4):  0x0180 0x0000   (next vector, dim = 384)
             ...
```

**Indexing:**
- Via `ConceptRecord.embedding_offset`
- Read dim, then read dim × 4 bytes

**Optional optimization (future):**
- Product quantization: replace raw f32 with 8-bit codes
- Compression ratio: ~4×

### 5. Content Blob (variable length, length-prefixed)

**Purpose:** Store arbitrary binary content (text, images, etc.)

**Alignment:** 4 bytes (u32 length prefix)

**Entry format:**
```
┌────────────────┬────────────────────┐
│ len: u32 (4B)  │ data: [u8; len]    │
└────────────────┴────────────────────┘
```

**Example:**
```
Offset         Data
-----------    ----
content_off+0: 0x000B 0x0000  (len = 11, LE)
          + 4: "hello world" (11 bytes UTF-8)
          +15: 0x0005 0x0000  (len = 5)
          +19: "sutra" (5 bytes)
          ...
```

**Indexing:**
- Via `ConceptRecord.content_offset` and `content_length`
- Direct memory access (zero-copy)

### 6. Footer (appended on sync, read on open)

**Purpose:** Fast lookups without scanning arenas.

**Written:** Only on `sync()` call
**Read:** On `open()` to rebuild in-memory indexes

#### 6a. Bloom Filter

**Purpose:** Avoid futile lookups for non-existent concepts.

**Parameters:**
```
k = 2                  (number of hash functions)
m = N × 2 bits         (bit array size, N = concept count)
false positive ≈ 1%    (acceptable for negative results)
```

**Layout:**
```
┌────────────────────────────────────────┐
│ Bit array: [(m + 7) / 8 bytes]        │
└────────────────────────────────────────┘
```

**Hash functions:**
```
h₁ = FNV1a(concept_id) mod m
h₂ = (FNV1a(concept_id) / m) mod m
```

**Membership test:**
```
if bloom[h₁/8] & (1 << h₁%8) == 0:
    return DEFINITELY_NOT_PRESENT
if bloom[h₂/8] & (1 << h₂%8) == 0:
    return DEFINITELY_NOT_PRESENT
return POSSIBLY_PRESENT  (check offset index)
```

**Space overhead:**
```
Bloom size = (N concepts × 2 bits) / 8 = N / 4 bytes
Example: 1M concepts → 250 KB
```

#### 6b. Offset Index

**Purpose:** O(1) lookup from ConceptId → file offset.

**Layout:**
```
┌───────────────────────────────────────────┐
│ Entry[0]: concept_id (16B) | offset (8B) │
│ Entry[1]: concept_id (16B) | offset (8B) │
│ ...                                       │
│ Entry[N-1]: concept_id (16B) | offset (8B)│
└───────────────────────────────────────────┘
```

**Entry structure:**
```
Offset  Size  Field
------  ----  -----
0x00    16B   concept_id    (ConceptId bytes)
0x10    8B    offset        (u64, little-endian)
------
Total:  24B per entry
```

**Indexing:**
```
On open:
    for i in 0..index_count:
        entry_offset = index_off + (i × 24)
        concept_id = read_bytes(entry_offset, 16)
        file_offset = read_u64_le(entry_offset + 16)
        hashmap[concept_id] = file_offset

On lookup:
    file_offset = hashmap.get(concept_id)?
    concept_record = mmap[file_offset..file_offset+128]
```

**Space overhead:**
```
Index size = N concepts × 24 bytes
Example: 1M concepts → 24 MB
```

**Trade-off:**
- 24 MB in-memory index for 1M concepts
- O(1) lookup vs. O(N) scan or O(log N) B-tree
- Acceptable for reasoning-critical workload

## Memory Access Patterns

### Write Path (Append-Only)

```
1. Append content:
   offset = content_off + content_bytes
   write_u32_le(offset, len)
   write_bytes(offset + 4, data)
   content_bytes += 4 + len

2. Append vector:
   offset = vector_off + vector_bytes
   write_u32_le(offset, dim)
   write_f32_array(offset + 4, vector)
   vector_bytes += 4 + (dim × 4)

3. Append concept:
   record.content_offset = content_offset
   record.embedding_offset = vector_offset
   offset = concept_off + (concept_count × 128)
   write_bytes(offset, &record, 128)
   concept_count += 1

4. Update in-memory index:
   hashmap[record.concept_id] = offset
```

### Read Path (Zero-Copy)

```
1. Bloom filter check:
   if !bloom.contains(concept_id):
       return None

2. Index lookup:
   offset = hashmap.get(concept_id)?

3. Direct memory access:
   record_ptr = mmap_base + offset
   record = *(record_ptr as *const ConceptRecord)

4. Load content (if needed):
   content_ptr = mmap_base + record.content_offset + 4
   content_slice = slice(content_ptr, record.content_length)

5. Load vector (if needed):
   vector_ptr = mmap_base + record.embedding_offset + 4
   dim = read_u32_le(record.embedding_offset)
   vector_slice = slice(vector_ptr, dim × 4)
```

## Alignment and Padding

**Why alignment matters:**
- CPU cache lines: 64 bytes
- SIMD instructions: require aligned data
- MMU page size: 4KB (mmap granularity)

**Alignment strategy:**
```
align_up(offset, alignment) = (offset + alignment - 1) & !(alignment - 1)

concept_arena: align to 128B (2× cache line)
edge_arena:    align to 64B (1× cache line)
vector_blob:   align to 4B (f32 size)
content_blob:  align to 4B (u32 length)
footer:        align to 8B (u64 offsets)
```

## Growth Strategy

**Initial allocation:**
```
Header:        256 bytes
Concept arena: 64 MB  (512K concepts)
Edge arena:    64 MB  (1M edges)
Vector blob:   128 MB (varies)
Content blob:  (remaining initial file size)
```

**Growth mechanism:**
```
On write, if offset + size > file_len:
    new_size = max(
        (offset + size).next_power_of_two(),
        file_len × 2
    )
    ftruncate(fd, new_size)
    mremap(mmap, new_size)
```

**Example growth:**
```
Initial: 256 MB
Write overflow → 512 MB
Write overflow → 1 GB
Write overflow → 2 GB
...
```

**OS behavior:**
- Sparse file: only allocated pages consume disk
- Lazy allocation: OS allocates on first write
- Automatic paging: OS moves cold pages to disk

## Summary

The memory layout is designed for:
1. **Zero-copy reads** - Direct pointer access via mmap
2. **Cache efficiency** - Aligned structures, co-located data
3. **Append-only writes** - No in-place updates, bump pointers
4. **OS-managed scaling** - File grows, OS decides what's in RAM
5. **Fast lookups** - Bloom + HashMap for O(1) access

Next: [Algorithms](./03-algorithms.md)

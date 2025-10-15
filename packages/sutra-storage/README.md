# Sutra Storage - Next-Generation Graph Storage

**Status**: 🚧 Foundation Phase - Architecture complete, implementation in progress

## What Is This?

This is NOT a traditional database. This is a **living knowledge substrate** - a storage engine designed specifically for temporal, continuously-learning knowledge graphs.

## Key Innovations

1. **Temporal Log-Structured Storage** - Time is a first-class citizen
2. **Zero-Copy Memory Mapping** - Direct memory access, no serialization overhead
3. **Lock-Free Concurrency** - Optimized for continuous learning workloads
4. **Native Vector Storage** - Embeddings are core, not bolted on
5. **Product Quantization** - 4x memory reduction for vectors

## Current Status

- ✅ Architecture designed (see ARCHITECTURE.md)
- ✅ Type system defined
- 🚧 Core storage implementation
- ⏳ Vector quantization
- ⏳ HNSW index
- ⏳ Python bindings

## Building

```bash
cd packages/sutra-storage
cargo build --release
```

## Python Integration

```python
# Will be available after completion
from sutra_storage import GraphStore

store = GraphStore("./knowledge.sutra")
concept_id = store.write_concept({
    "content": "Photosynthesis converts light to energy",
    "strength": 1.0
})
```

## Performance Targets

- Concept write: < 10μs
- Concept read: < 1μs  
- Semantic search: < 1ms
- Graph traversal: < 100μs

## Implementation Timeline

- Week 1: Core storage with memory mapping ✅
- Week 2: Vector quantization and HNSW
- Week 3: Concurrency and crash recovery
- Week 4: Performance optimization

## Why Rust?

- Memory safety without garbage collection
- Zero-cost abstractions
- SIMD support for vector operations
- Excellent for building storage engines
- Seamless Python integration via PyO3

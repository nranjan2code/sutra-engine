# Architecture Overview

Sutra is an explainable, graph-based AI system with native Rust storage and vector search.

Key changes in this release:
- Native HNSW vector search inside ConcurrentStorage (Rust)
- Python VectorIndex removed; QueryProcessor calls storage.vector_search
- Simplified, consistent learning and query paths

Core components:
- sutra-storage (Rust): ConcurrentStorage with append-only logs, memory-mapped snapshot, native HNSW
- sutra-core (Python): Learning, associations, reasoning (PathFinder, MPPA), QueryProcessor
- sutra-hybrid (Python): Optional embedding orchestration

Data flow:
1) Learn: content -> embeddings -> storage.learn_concept -> persisted
2) Query: text -> embedding -> storage.vector_search -> concepts -> reasoning paths -> MPPA -> answer

Performance:
- k-NN: HNSW O(log N) over in-memory vectors (rebuilt on-demand)
- Learning: append-only writes + periodic flush
- Reasoning: multi-path with confidence decay and consensus

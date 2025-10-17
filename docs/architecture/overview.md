# Architecture Overview

Sutra is an explainable, graph-based AI system with native Rust storage and vector search.

Key changes in this release:
- gRPC-only architecture: all services connect to storage-server
- Thin sutra-api: REST-to-gRPC proxy (no local ReasoningEngine)
- sutra-hybrid: embeddings + orchestration, forwards graph ops via gRPC

Core components:
- storage-server (Rust): ConcurrentMemory + HNSW, gRPC service
- storage-client (Python): gRPC client used by API/Hybrid/Control
- sutra-hybrid (Python): Optional embeddings and orchestration

Data flow:
1) Learn: content → optional embedding (hybrid) → gRPC LearnConcept → persisted
2) Query: text → embedding → gRPC VectorSearch → concept content → explanation built client-side

Performance:
- k-NN: HNSW O(log N) over in-memory vectors (rebuilt on-demand)
- Learning: append-only writes + periodic flush
- Reasoning: multi-path with confidence decay and consensus

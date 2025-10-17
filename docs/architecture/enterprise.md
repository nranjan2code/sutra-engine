# Enterprise Architecture

This document describes how Sutra’s components compose in production, based on the current codebase.

Core services and libraries:
- Storage (Rust, ConcurrentStorage)
  - Append-only write log with background reconciler
  - In-memory read view with memory-mapped snapshot
  - Native HNSW k-NN built on-demand over in-memory vectors
  - PyO3 bindings exposed via RustStorageAdapter
- Core (Python, sutra-core)
  - ReasoningEngine orchestrating QueryProcessor, PathFinder, MPPA
  - QueryProcessor performs semantic retrieval via storage.vector_search
  - Association extraction and graph reasoning over storage
- Hybrid (Python, sutra-hybrid)
  - Embedding orchestration (batching, device selection, prompts)
  - API layer and protocols (endpoints in sutra_hybrid/api/*)

Data flow (production):
1) Learn
   - Client sends content + embedding (or content and embedding produced by hybrid)
   - RustStorageAdapter.store.learn_concept → write log → reconciler → snapshot
   - Vectors auto-indexed in memory for HNSW
2) Query
   - Text → embedding (hybrid) → storage.vector_search → candidate concepts
   - PathFinder finds reasoning paths → MPPA aggregates → answer + confidence

Operational model:
- Process model: one process hosts ConcurrentStorage; reconciler thread handles compaction/snapshot
- Durability: explicit flush via storage.save(); snapshot written to storage path
- Scaling:
  - Vertical scale: larger memory for more in-memory vectors and faster HNSW
  - Horizontal patterns: shard by dataset or tenant at the application layer; each shard has its own storage path
  - Stateless API: sutra-hybrid instances can scale horizontally; each instance attaches to its shard’s storage
- Performance characteristics (from code and demos):
  - Embedding generation depends on model/device; HNSW query is O(log N) over in-memory vectors
  - Reconciliation/flush is periodic or on-demand and does not block reads

Security and compliance:
- At-rest: storage files are written under the configured path; use OS/disk encryption as needed
- In-transit: secure API transport (TLS) handled by the deployment runtime (e.g., reverse proxy)
- Access control: implement at API gateway/service layer (sutra-hybrid)

Reliability and operations:
- Health: expose health/readiness on API (sutra-hybrid) and track storage.stats()
- Backup/restore: snapshot directory can be backed up; restore by pointing storage to the snapshot path
- Observability: structured logging present; stats endpoints available in API; extend as needed for metrics

Configuration (examples):
- Storage path: environment variable SUTRA_STORAGE_PATH (default ./knowledge)
- Vector dimension: set when initializing RustStorageAdapter
- Embeddings: choose model via sutra-hybrid embedding processor; prefer float32 numpy arrays

Deployment guidelines:
- Build Rust extension (maturin develop --release) and install sutra-core
- Run sutra-hybrid API behind a reverse proxy (TLS, auth, rate limits)
- Use per-tenant storage directories for isolation and easy backup

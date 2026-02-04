# Architecture (Concise)

Sutra Engine is a high-performance storage core with two primary planes:

1. **Write Plane**: lock-free append log for continuous learning.
2. **Read Plane**: immutable snapshots for zero-copy reads.

These are reconciled by an adaptive background loop that batches writes into the read snapshot with bounded latency.

The storage server exposes a TCP binary protocol and can run in single-node or sharded mode. Vector search is powered by USearch-backed HNSW with mmap persistence.

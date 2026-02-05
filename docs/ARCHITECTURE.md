# Architecture (Concise)

Sutra Engine is a high-performance storage core with two primary planes:

1. **Write Plane**: lock-free append log for continuous learning.
2. **Read Plane**: immutable snapshots for zero-copy reads.

These are reconciled by an adaptive background loop that batches writes into the read snapshot with bounded latency.

The storage server exposes a **Dual-Protocol Interface**:
- **Binary (TCP)**: High-performance machine-to-machine.
- **Text (NL)**: "Babelfish" human-readable interface.

It features an embedded **Internal Brain** (Candle/Bert) for local vector inference, removing the need for external embedding services. Vector search is powered by USearch-backed HNSW with mmap persistence.

# Architecture

Sutra Engine is a high-performance hybrid storage engine with two primary planes:

1. **Write Plane**: Lock-free append log for continuous ingestion.
2. **Read Plane**: Immutable snapshots for zero-copy reads.

These are reconciled by an adaptive background loop that batches writes into the read snapshot with bounded latency.

The storage server exposes a **Dual-Protocol Interface**:
- **Binary (TCP)**: High-performance machine-to-machine.
- **Text (NL)**: Human-readable interface for development and testing.

It features an **Embedded Inference Engine** (Candle/BERT) for local vector generation, removing the need for external embedding services. Vector search is powered by USearch-backed HNSW with mmap persistence.

## Background Maintenance

The Background Maintenance system provides 7 configurable jobs, all managed by a central `AutonomyManager`:

| Feature | Module | Interval | Purpose |
|---------|--------|----------|---------|
| **Strength Decay** | `decay.rs` | 5s | Exponential strength decay with access-count reinforcement. Prunes records below threshold. |
| **Health Metrics** | `self_monitor.rs` | 10s | Captures engine stats (records, edges, writes, vectors) and stores them internally. Maintains bounded history. |
| **Auto-Association** | `reasoning.rs` | 10s | Samples random records, discovers new edges via vector similarity, detects contradictions between neighbors, strengthens connected pairs. |
| **Trigger System** | `goals.rs` | 5s | Triggers stored as records with `SemanticType::Goal`. Evaluates conditions (record existence, count thresholds, strength checks) and executes actions (notify, insert, associate). |
| **Subscriptions** | `subscriptions.rs` | 500ms | Push notifications when records matching a filter are created. Polls ReadView for snapshot sequence changes. TCP push or log-only mode. |
| **Graph Analysis** | `gap_detector.rs` | 30s | Identifies isolated records, near-miss pairs (similar but unconnected), and incomplete causal chains. Emits gaps through subscription system. |
| **Feedback Processing** | `feedback.rs` | sync | Processes accept/reject signals to adjust record strengths. Supports ranking-based proportional boosts. |

All background loops follow the same pattern: `Arc<AtomicBool>` running flag, `thread::spawn`, `JoinHandle`, `Drop` calls `stop()`. They interact with `ConcurrentMemory` exclusively through its public API.

Controlled by `SUTRA_AUTONOMY` env var (default `true`). Use `AutonomyConfig::disabled()` to turn off all features.

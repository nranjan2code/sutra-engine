# Architecture (Concise)

Sutra Engine is a high-performance storage core with two primary planes:

1. **Write Plane**: lock-free append log for continuous learning.
2. **Read Plane**: immutable snapshots for zero-copy reads.

These are reconciled by an adaptive background loop that batches writes into the read snapshot with bounded latency.

The storage server exposes a **Dual-Protocol Interface**:
- **Binary (TCP)**: High-performance machine-to-machine.
- **Text (NL)**: "Babelfish" human-readable interface.

It features an embedded **Internal Brain** (Candle/Bert) for local vector inference, removing the need for external embedding services. Vector search is powered by USearch-backed HNSW with mmap persistence.

## Autonomy Engine

The Autonomy Engine makes Sutra self-directed through 7 background features, all managed by a central `AutonomyManager`:

| Feature | Module | Interval | Purpose |
|---------|--------|----------|---------|
| **Knowledge Decay** | `decay.rs` | 5s | Exponential strength decay with access-count reinforcement. Prunes concepts below threshold. |
| **Self-Monitoring** | `self_monitor.rs` | 10s | Captures engine health stats (concepts, edges, writes, HNSW vectors) and stores them as `Event` concepts. Maintains bounded history. |
| **Background Reasoning** | `reasoning.rs` | 10s | Samples random concepts, discovers new associations via vector similarity, detects contradictions between neighbors, strengthens connected pairs. |
| **Goal System** | `goals.rs` | 5s | Goals stored as concepts with `SemanticType::Goal`. Evaluates conditions (concept existence, count thresholds, strength checks) and executes actions (notify, learn, associate). |
| **Subscriptions** | `subscriptions.rs` | 500ms | Push notifications when concepts matching a filter are created. Polls ReadView for snapshot sequence changes. TCP push or log-only mode. |
| **Gap Detection** | `gap_detector.rs` | 30s | Identifies isolated concepts, near-miss pairs (similar but unconnected), and incomplete causal chains. Emits gaps through subscription system. |
| **Feedback Integration** | `feedback.rs` | sync | Processes accept/reject signals to adjust concept strengths. Supports ranking-based proportional boosts. |

All background loops follow the same pattern: `Arc<AtomicBool>` running flag, `thread::spawn`, `JoinHandle`, `Drop` calls `stop()`. They interact with `ConcurrentMemory` exclusively through its public API.

Controlled by `SUTRA_AUTONOMY` env var (default `true`). Use `AutonomyConfig::disabled()` to turn off all features.

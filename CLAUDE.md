# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Sutra Engine is a high-performance hybrid storage engine written in Rust. It combines dense vector search with graph relationships for applications requiring both semantic similarity and relational data. The engine supports sub-millisecond search, 50K+ writes/sec, and local ML inference without external API calls.

## Build & Development Commands

```bash
# Build the entire workspace
cargo build --workspace

# Build release (optimized)
cargo build --release --workspace

# Run the storage server (default port 50051)
cargo run --release --bin storage-server

# Run with custom port (e.g., NL interface on 9000)
STORAGE_PORT=9000 cargo run --release --bin storage-server

# Run all workspace tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p sutra-storage
cargo test -p sutra-protocol
cargo test -p sutra-bulk-ingester

# Run a single test by name
cargo test -p sutra-storage --test real_scenarios
cargo test -p sutra-storage test_name_here

# Format and lint
cargo fmt --check
cargo clippy -- -D warnings

# Full CI validation
./scripts/ci-validate.sh
```

## Workspace Structure

Three crates in a Cargo workspace (resolver v2):

- **`crates/storage`** — Core storage engine (~14K LOC). Contains the TCP server binary (`storage-server`), concurrent store, WAL, HNSW vector index, semantic analyzer, local inference (Candle), sharding, auth, and rate limiting.
- **`crates/protocol`** — Custom binary TCP protocol (~940 LOC). Length-prefixed bincode serialization with 16MB max message size. Defines `StorageMessage`/`StorageResponse` enums and wire format helpers.
- **`crates/bulk-ingester`** — HTTP-based bulk data ingestion service. Axum web server with pluggable adapters. Optional Python plugin support via PyO3 (`python-plugins` feature; default is `embedded-only`).

## Architecture

**Dual-plane design:** Write plane (lock-free append via `ConcurrentMemory`) and Read plane (immutable snapshots via `ReadView`). An `AdaptiveReconciler` batches writes into read snapshots with bounded latency (1-100ms adaptive interval).

**Key data flow:** Raw text → TCP server → optional local embedding (Candle/all-MiniLM-L6-v2, 384D vectors) → semantic analysis (10 types) → association extraction → atomic storage (2PC for cross-shard) → WAL persistence → HNSW index update.

**Sharding:** Consistent hashing across configurable shards (default 16). `TransactionCoordinator` handles 2PC for atomic cross-shard writes.

**Vector persistence:** USearch HNSW with mmap for 94x faster startup vs in-memory rebuild.

**Record types (10):** Entity, Event, Rule, Temporal, Negation, Condition, Causal, Quantitative, Definitional, Goal. Rules are configured in `semantics.toml`.

**Background Maintenance:** 7 configurable background jobs managed by `AutonomyManager`: strength decay, health metrics, auto-association discovery, trigger system, subscriptions, graph analysis, and feedback processing. All jobs interact with `ConcurrentMemory` through its public API. Controlled by the `SUTRA_AUTONOMY` env var (default `true`).

**Protocol:** Custom TCP binary (not gRPC). Format: `[4 bytes u32 length][bincode payload]`. All data ingestion routes through the TCP `learn_concept()` path — never bypass this.

## Key Source Files (storage crate)

- `src/bin/storage_server.rs` — Server entry point, env var config, single/sharded mode selection
- `src/tcp_server.rs` — Main TCP server with message handling (largest file)
- `src/concurrent_memory.rs` — Write-plane concurrent store (largest module)
- `src/wal.rs` — Write-Ahead Log for crash recovery
- `src/adaptive_reconciler.rs` — Write-to-read plane batching
- `src/sharded_storage.rs` — Sharding with consistent hashing
- `src/transaction.rs` — 2PC cross-shard transactions
- `src/hnsw_container.rs` — USearch HNSW vector index wrapper
- `src/semantic/analyzer.rs` — Record classification engine
- `src/inference/embedding_engine.rs` — Local Candle-based BERT inference
- `src/nl_parser.rs` — Natural language command parser ("Insert...", "Search for...")
- `src/auth.rs` — HMAC-SHA256 auth, TLS 1.3
- `src/learning_pipeline.rs` — End-to-end record ingestion orchestration
- `src/autonomy/mod.rs` — Background jobs manager (decay, auto-association, triggers, subscriptions, graph analysis, feedback, health metrics)
- `src/autonomy/decay.rs` — Strength decay with access reinforcement
- `src/autonomy/reasoning.rs` — Auto-association discovery + contradiction detection
- `src/autonomy/goals.rs` — Trigger system with conditions/actions
- `src/autonomy/subscriptions.rs` — Push notifications on record changes
- `src/autonomy/gap_detector.rs` — Isolated record and near-miss detection
- `src/autonomy/feedback.rs` — Accept/reject feedback to adjust strengths
- `src/autonomy/self_monitor.rs` — Engine health metrics stored as records

## Environment Variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `STORAGE_PATH` | `/data/storage.dat` | Persistent storage file path |
| `STORAGE_HOST` | `0.0.0.0` | Server bind address |
| `STORAGE_PORT` | `50051` | Server port |
| `VECTOR_DIMENSION` | `384` | Embedding vector dimension |
| `SUTRA_STORAGE_MODE` | `single` | `single` or `sharded` |
| `SUTRA_NUM_SHARDS` | `16` | Number of shards (sharded mode) |
| `SUTRA_SECURE_MODE` | `false` | Enable TLS 1.3 + HMAC auth |
| `RECONCILE_BASE_INTERVAL_MS` | `10` | Adaptive reconciler base interval |
| `MEMORY_THRESHOLD` | `50000` | Write count before forced reconciliation |
| `SUTRA_AUTONOMY` | `true` | Enable/disable background maintenance jobs |

## Testing

Integration tests live in `crates/storage/tests/`:
- `real_scenarios.rs` — Real-world workflows
- `tcp_protocol_real_scenarios.rs` — TCP protocol validation
- `security_scenarios.rs` — Auth/security testing
- `load_scenarios.rs` — Performance under load
- `failure_injection.rs` — Fault injection
- `soak_stability.rs` — Long-running soak tests (controlled by `SUTRA_SOAK_RUN`, `SUTRA_SOAK_MINUTES`, `SUTRA_SOAK_CONCURRENCY` env vars)
- `test_hnsw_persistence.rs` — Vector index persistence
- `namespace_test.rs` — Namespace isolation
- `compat_scenarios.rs` — Backward compatibility

Unit tests are embedded in source files as `#[cfg(test)]` modules.

## Anti-Patterns

- **Never bypass unified ingestion:** All data must go through storage server TCP protocol (`learn_concept()`)
- **No SQL/Cypher/GraphQL:** Architectural policy — only the custom binary TCP protocol
- **Don't assume security:** Default mode has no authentication (development only)
- **Don't add external observability:** Health metrics via internal records is an intentional design choice

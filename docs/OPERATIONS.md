# Operations & Performance Tuning

This guide covers running Sutra Engine in production, managing data, and optimizing performance.

---

## 💾 Persistence & Recovery

Sutra Engine uses a multi-layered persistence strategy:

### 1. Write-Ahead Log (WAL)
Every write operation is first recorded in the WAL. If the engine crashes, it will automatically replay the WAL upon restart to ensure zero data loss.

### 2. Snapshots
The engine periodically flushes the in-memory graph to a binary `storage.dat` file.
- **Manual Flush**: Call the `Flush` request via the API.
- **Auto Flush**: Controlled by the `MEMORY_THRESHOLD` environment variable.

### 3. Backup & Restore
To backup Sutra, simply copy the `STORAGE_PATH` directory.
```bash
# Backup
cp -r /path/to/sutra/data /backups/sutra-$(date +%F)

# Restore
export STORAGE_PATH=/backups/sutra-old-version
./start-engine.sh
```

---

## ⚡ Performance Optimization

### Environment Variables for Tuning

| Variable | Default | Description |
|----------|---------|-------------|
| `MEMORY_THRESHOLD` | `50000` | Number of writes allowed before a mandatory disk flush. Increase for higher throughput, decrease for lower memory usage. |
| `RECONCILE_BASE_INTERVAL_MS` | `10` | Frequency of background graph reconciliation. |
| `VECTOR_DIMENSION` | `768` | Must match your embedding model. Common values: 384, 768, 1536. |

### HNSW Tuning
The engine uses HNSW for vector search. You can tune search quality vs. speed via the `ef_search` parameter in `VectorSearch` requests (default: 128).

---

## 🏗 Sharding & Scaling

For knowledge graphs exceeding 10 million concepts, we recommend **Sharded Mode**.

```bash
export SUTRA_STORAGE_MODE=sharded
export SUTRA_NUM_SHARDS=16
./start-engine.sh
```

In sharded mode, concepts are distributed across independent shards using consistent hashing on the `ConceptID`. This allows for massive parallelization of both search and ingestion.

---

## 🤖 Autonomy Engine

Sutra includes a self-directed **Autonomy Engine** with 7 background features. It is enabled by default and controlled via the `SUTRA_AUTONOMY` environment variable.

```bash
# Disable autonomy (e.g. for benchmarking)
export SUTRA_AUTONOMY=false

# Enable (default)
export SUTRA_AUTONOMY=true
```

### Features

| Feature | Default Interval | Tuning Notes |
|---------|-----------------|--------------|
| Knowledge Decay | 5s | Adjusts concept strengths based on access patterns. Prunes concepts below 0.01 strength. |
| Self-Monitoring | 10s | Stores health snapshots as concepts. Keeps last 1000 snapshots. |
| Background Reasoning | 10s | Discovers associations, detects contradictions. Samples 20 concepts per cycle. |
| Goal Evaluator | 5s | Evaluates goal conditions and triggers actions. |
| Subscriptions | 500ms | Polls ReadView for changes. Push notifications via TCP or log-only. |
| Gap Detector | 30s | Finds isolated concepts and near-miss pairs. |
| Feedback | Synchronous | Processes accept/reject signals from the `ProvideFeedback` API. |

### Monitoring Autonomy

Use `GetAutonomyStats` (or `echo "status" | nc localhost 9000`) to view:
- Active subscriptions and goals count
- Per-feature enabled/disabled status
- Concept/edge/vector counts
- Reconciler health and pending writes

### NL Commands

```bash
echo "status" | nc localhost 9000              # Autonomy stats
echo "set goal: track new concepts" | nc localhost 9000  # Create a goal
echo "list goals" | nc localhost 9000           # List goals
echo "subscribe to Rust" | nc localhost 9000    # Subscribe to filter
```

---

## 🖥 Monitoring

Use the `GetStats` request to monitor engine health.
- **pending_writes**: If this stays high, increase your disk I/O or adjust the reconciler.
- **uptime_seconds**: Tracking engine stability.
- **concept_count**: Monitoring growth.

---

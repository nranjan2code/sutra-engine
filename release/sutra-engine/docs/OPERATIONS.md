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

## 🖥 Monitoring

Use the `GetStats` request to monitor engine health.
- **pending_writes**: If this stays high, increase your disk I/O or adjust the reconciler.
- **uptime_seconds**: Tracking engine stability.
- **concept_count**: Monitoring growth.

---

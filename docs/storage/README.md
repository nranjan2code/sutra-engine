# Sutra Storage Documentation

**Complete technical documentation for Sutra's production storage engine**

> **Current Version**: 2.0.0 (October 2025)  
> **Status**: Production-ready, battle-tested at scale

---

## 📚 Documentation Index

### Getting Started

1. **[STORAGE_GUIDE.md](./STORAGE_GUIDE.md)** - **START HERE**
   - Complete architecture overview
   - Core components deep dive
   - Configuration reference
   - Operations guide
   - Performance characteristics
   - **Audience**: Developers, operators, architects

### Architecture Deep Dives

2. **[HNSW_OPTIMIZATION.md](./HNSW_OPTIMIZATION.md)**
   - USearch migration from hnsw-rs
   - Persistent vector index design
   - 100× faster startup (50ms vs 2-5s)
   - Incremental update strategies

3. **[ADAPTIVE_RECONCILIATION_ARCHITECTURE.md](./ADAPTIVE_RECONCILIATION_ARCHITECTURE.md)**
   - AI-native self-tuning reconciler
   - EMA-based workload prediction
   - Dynamic interval adjustment (1ms-100ms)
   - Health scoring and monitoring

4. **[SHARDING.md](./SHARDING.md)**
   - Horizontal scaling to 10M+ concepts
   - Consistent hashing strategy
   - Two-phase commit for cross-shard atomicity
   - Parallel operations and aggregation

5. **[WAL_MSGPACK_MIGRATION.md](./WAL_MSGPACK_MIGRATION.md)**
   - Write-ahead log implementation
   - MessagePack binary format
   - Crash recovery protocols
   - Transaction support

### Implementation Details

6. **[HNSW_PERSISTENCE_DESIGN.md](./HNSW_PERSISTENCE_DESIGN.md)**
   - USearch `.usearch` file format
   - Memory-mapped loading
   - ID mapping strategies
   - Capacity planning

7. **[USEARCH_MIGRATION_COMPLETE.md](./USEARCH_MIGRATION_COMPLETE.md)**
   - Migration timeline and rationale
   - Lifetime constraint solutions
   - Performance comparisons
   - Lessons learned

8. **[DEEP_CODE_REVIEW.md](./DEEP_CODE_REVIEW.md)**
   - Production-grade code patterns
   - Safety considerations
   - Testing strategies
   - Best practices

### Operations

9. **[PRODUCTION_GRADE_COMPLETE.md](./PRODUCTION_GRADE_COMPLETE.md)**
   - Production readiness checklist
   - Deployment validation
   - Monitoring and alerting
   - Failure modes and recovery

10. **[DEPLOYMENT_CHECKLIST.md](./DEPLOYMENT_CHECKLIST.md)**
    - Pre-deployment validation
    - Configuration verification
    - Health check procedures
    - Rollback strategies

---

## 🏗️ Architecture at a Glance

```
┌─────────────────────────────────────────────────────────────────┐
│                    Sutra Storage Server (Rust)                  │
│                      TCP Binary Protocol (Port 7000)            │
├─────────────────────────────────────────────────────────────────┤
│  Unified Learning Pipeline                                      │
│  Semantic Analysis → Embedding → Associations → Storage         │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌────────────────────────┐ │
│  │ Write Plane │  │ Read Plane  │  │  Vector Plane          │ │
│  │ (WriteLog)  │  │ (ReadView)  │  │  (HnswContainer)       │ │
│  │ Lock-free   │  │ Immutable   │  │  USearch Persistent    │ │
│  └─────────────┘  └─────────────┘  └────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│  Adaptive Reconciler (AI-native self-tuning)                   │
│  EMA smoothing • Predictive backpressure • Health scoring      │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────────┐  │
│  │  WAL (Disk)  │  │  HNSW Index  │  │  Graph Storage     │  │
│  │  wal.log     │  │  .usearch    │  │  storage.dat       │  │
│  └──────────────┘  └──────────────┘  └────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### Key Features

✅ **57K writes/sec** - Lock-free concurrent writes  
✅ **<0.01ms reads** - Zero-copy memory-mapped access  
✅ **Zero data loss** - WAL-based durability  
✅ **100× faster startup** - Persistent HNSW with USearch  
✅ **10M+ concepts** - Horizontal scaling with sharding  
✅ **Self-tuning** - AI-native adaptive reconciliation  
✅ **Semantic understanding** - Built-in domain classification  

---

## 🚀 Quick Start

### Single Storage Mode (< 1M concepts)

```yaml
# docker-compose.yml
storage-server:
  image: sutra/storage-server:2.0.0
  ports:
    - "7000:7000"
  environment:
    - SUTRA_STORAGE_MODE=single
    - STORAGE_PATH=/data
    - VECTOR_DIMENSION=768
    - WAL_FSYNC=true
  volumes:
    - ./data:/data
```

### Sharded Storage Mode (1M-10M+ concepts)

```yaml
storage-server:
  image: sutra/storage-server:2.0.0
  ports:
    - "7000:7000"
  environment:
    - SUTRA_STORAGE_MODE=sharded
    - SUTRA_NUM_SHARDS=16
    - STORAGE_PATH=/data/sharded
    - VECTOR_DIMENSION=768
    - WAL_FSYNC=true
  volumes:
    - ./data:/data
```

---

## 📊 Performance Summary

### Benchmarks (Single Storage)

| Operation | Latency | Throughput |
|-----------|---------|------------|
| **Write concept** | ~10μs | 57,412/sec |
| **Read concept** | <0.01ms | ~100K/sec |
| **Vector search (k=10)** | 2ms | 500 queries/sec |
| **Graph traversal (3-hop)** | 1-2ms | 500-1000/sec |
| **Startup (1M vectors)** | <200ms | N/A |

### Scalability (Sharded Storage)

| Shards | Max Concepts | Write Throughput | RAM Usage |
|--------|--------------|------------------|-----------|
| 4 | 4M | 228K/sec | 4GB |
| 8 | 8M | 458K/sec | 8GB |
| 16 | 16M+ | 915K/sec | 16GB+ |

---

## 🎯 Use Cases

### Medical (Healthcare)

```
Domain: Medical diagnosis and treatment
Types: Rule, Event, Entity, Fact
Context: Diagnosis, treatment, dosage, contraindication
Example: "If patient has diabetes and hypertension, prescribe ACE inhibitor"
```

### Legal (Compliance)

```
Domain: Legal precedents and regulations
Types: Rule, Precedent, Statute, Opinion
Context: Case law, regulatory compliance, contract interpretation
Example: "Regulation requires 30-day notice period for tenant eviction"
```

### Financial (Trading/Risk)

```
Domain: Financial regulations and risk management
Types: Rule, Event, Transaction, Risk
Context: Compliance, fraud detection, portfolio optimization
Example: "Trades exceeding $1M require dual authorization"
```

---

## 🔧 Configuration Examples

### High-Throughput (Batch Ingestion)

```rust
ConcurrentConfig {
    memory_threshold: 100_000,      // Larger batches
    adaptive_reconciler_config: {
        base_interval_ms: 5,        // Fast reconciliation
        max_batch_size: 20_000,     // Large batches
    },
}
// WAL_FSYNC=false (accept risk for speed)
```

### Low-Latency (Real-Time Queries)

```rust
ConcurrentConfig {
    memory_threshold: 10_000,       // Frequent snapshots
    adaptive_reconciler_config: {
        min_interval_ms: 1,         // Immediate sync
        max_batch_size: 5_000,      // Small batches
    },
}
// HnswConfig { ef_search: 40 }
```

### Memory-Constrained

```rust
ConcurrentConfig {
    memory_threshold: 10_000,       // Flush early
    adaptive_reconciler_config: {
        max_batch_size: 1_000,      // Tiny batches
        disk_flush_threshold: 10_000,
    },
}
// HnswConfig { max_elements: 50_000 }
```

---

## 🔍 Monitoring

### Key Metrics

```bash
# Query storage stats
echo '{"type":"GetStats"}' | nc localhost 7000

# Critical metrics:
- concept_count: Total concepts stored
- write_log_pending: Queue depth (warn if >70%)
- reconciler_health_score: 0.0-1.0 (alert if <0.5)
- estimated_lag_ms: Reader staleness (alert if >100ms)
- processing_rate_per_sec: Reconciliation throughput
```

### Health Thresholds

| Metric | Excellent | Good | Fair | Critical |
|--------|-----------|------|------|----------|
| **Health Score** | 0.9-1.0 | 0.7-0.9 | 0.5-0.7 | <0.5 |
| **Queue Utilization** | <50% | 50-70% | 70-85% | >85% |
| **Lag** | <10ms | 10-50ms | 50-100ms | >100ms |
| **Dropped Entries** | 0 | <0.1% | 0.1-1% | >1% |

---

## 🐛 Troubleshooting

### Common Issues

**High write latency**
```
Symptom: Client writes timing out
Diagnosis: Check write_log_pending and reconciler_health_score
Fix: Reduce base_interval_ms or increase max_batch_size
```

**Memory growing unbounded**
```
Symptom: OOM kills, swap thrashing
Diagnosis: Check memory_threshold and write_log_pending
Fix: Lower memory_threshold to force more frequent flushes
```

**Poor vector search quality**
```
Symptom: Irrelevant results in semantic search
Diagnosis: Check HNSW ef_construction and ef_search
Fix: Increase ef_construction (rebuild) or ef_search (query-time)
```

**Slow startup times**
```
Symptom: Container takes minutes to be ready
Diagnosis: Large WAL replay or missing .usearch index
Fix: Flush before shutdown, verify .usearch exists
```

---

## 📖 Further Reading

### External Resources

- [USearch Documentation](https://github.com/unum-cloud/usearch) - Vector index library
- [MessagePack Specification](https://msgpack.org/) - Binary serialization format
- [Two-Phase Commit Protocol](https://en.wikipedia.org/wiki/Two-phase_commit_protocol) - Distributed transactions
- [HNSW Algorithm Paper](https://arxiv.org/abs/1603.09320) - Hierarchical Navigable Small World

### Related Sutra Docs

- [UNIFIED_LEARNING_ARCHITECTURE.md](../UNIFIED_LEARNING_ARCHITECTURE.md) - Pipeline design
- [TCP_PROTOCOL_ARCHITECTURE.md](../TCP_PROTOCOL_ARCHITECTURE.md) - Binary protocol spec
- [SEMANTIC_UNDERSTANDING.md](../SEMANTIC_UNDERSTANDING.md) - Domain classification
- [PRODUCTION_GUIDE.md](../PRODUCTION_GUIDE.md) - Deployment best practices

---

## 🤝 Contributing

### Documentation Updates

When updating storage documentation:

1. **Update STORAGE_GUIDE.md first** - It's the source of truth
2. **Keep code and docs in sync** - Reference actual implementations
3. **Include examples** - Show real usage patterns
4. **Benchmark claims** - Back up performance numbers with data
5. **Version compatibility** - Note which versions are documented

### Code Review Checklist

- [ ] Configuration validation added
- [ ] Error handling comprehensive
- [ ] Metrics/logging included
- [ ] WAL operations idempotent
- [ ] Tests cover edge cases
- [ ] Documentation updated

---

## 📝 Version History

### v2.0.0 (October 2025) - **Current**

- ✅ USearch migration complete (persistent HNSW)
- ✅ Adaptive reconciler with EMA smoothing
- ✅ Semantic understanding module integrated
- ✅ Two-phase commit for sharded storage
- ✅ Production validation at 10M+ concepts

### v1.5.0 (September 2025)

- Sharded storage mode added
- WAL with transaction support
- Unified learning pipeline
- Self-monitoring via Grid events

### v1.0.0 (August 2025)

- Initial production release
- Single storage mode only
- hnsw-rs vector index
- Basic reconciliation

---

## 📧 Support

- **Issues**: GitHub Issues
- **Discussions**: GitHub Discussions
- **Documentation Bugs**: File PR with fixes
- **Performance Questions**: Include metrics and config

---

**Last Updated**: October 27, 2025  
**Maintained By**: Sutra AI Engineering Team  
**License**: Proprietary

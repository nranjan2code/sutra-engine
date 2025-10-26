# Monitoring & Observability Guide

**Comprehensive monitoring for Sutra AI production deployments**

Version: 2.0.0 | Last Updated: 2025-10-23

---

## Overview

Sutra AI provides built-in monitoring through health checks, metrics endpoints, and structured logging. This guide covers production observability strategies.

---

## Health Checks

### Storage Server

```bash
# TCP health check (port check)
nc -zv localhost 50051

# Stats endpoint
curl -s http://localhost:50051/stats | jq .
```

**Response:**
```json
{
  "mode": "sharded",
  "shards": 16,
  "total_concepts": 1000000,
  "total_edges": 5000000,
  "total_vectors": 1000000,
  "write_log_pending": 0,
  "reconciler_running": true
}
```

### API Services

```bash
# Sutra API
curl -f http://localhost:8000/health

# Sutra Hybrid
curl -f http://localhost:8001/ping

# Embedding Service
curl -f http://localhost:8888/health
```

---

## Key Metrics

### Storage Performance

**Metrics to track:**
- Write throughput (concepts/sec)
- Read latency (ms)
- Vector search latency (ms)
- Write log pending count
- Memory usage per shard

**Alerting thresholds:**
- Write throughput < 10K/sec (degraded)
- Read latency > 10ms (slow storage)
- Write log pending > 10K (backpressure)
- Memory usage > 90% (OOM risk)

### Embedding Service

**Metrics:**
- Requests per second
- Latency (p50, p95, p99)
- Success rate
- Cache hit rate

**Thresholds:**
- Success rate < 95% (failing)
- p99 latency > 500ms (slow)
- Cache hit rate < 50% (inefficient)

---

## Logging

### Structured Logging (Rust)

```rust
use tracing::{info, warn, error};

info!("Storage server started on port {}", port);
warn!("High memory usage: {}GB", mem_gb);
error!("Failed to persist storage: {:?}", err);
```

### Log Levels

```bash
# Development
RUST_LOG=debug cargo run

# Production
RUST_LOG=info cargo run

# Troubleshooting
RUST_LOG=trace cargo run
```

---

## Dashboards (Planned)

### Grafana Metrics

**Coming in Phase 2:**
- Request rate graphs
- Latency histograms
- Error rate tracking
- System resource usage

---

## Troubleshooting

### High Latency

**Diagnosis:**
```bash
# Check system load
top

# Check disk I/O
iostat -x 1

# Check network
netstat -s
```

### Memory Issues

```bash
# Memory usage
free -h

# Process memory
ps aux | grep storage-server
```

---

## References

- [Production Requirements](PRODUCTION_REQUIREMENTS.md)
- [Scaling Guide](SCALING_GUIDE.md)
- [Troubleshooting](../TROUBLESHOOTING.md)

---

Last Updated: 2025-10-23 | Version: 2.0.0

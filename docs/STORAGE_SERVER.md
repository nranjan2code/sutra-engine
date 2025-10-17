# Standalone Storage Server

## Overview

The storage system can run in two modes:

1. **Embedded Mode (default)**: Each process has its own `ConcurrentStorage` instance
2. **Server Mode**: Single storage server that all processes connect to

## Architecture

```
┌─────────────────────────────┐
│   Storage Server (Rust)     │
│  - ConcurrentMemory          │  Port 50051
│  - gRPC API                  │  (configurable)
│  - Single storage.dat        │
└─────────────────────────────┘
         ↑ ↑ ↑
         │ │ │ (gRPC clients)
    ┌────┘ │ └────┐
    │      │      │
┌───┴──┐ ┌─┴───┐ ┌┴────┐
│ API  │ │Core │ │ CLI │
└──────┘ └─────┘ └─────┘
```

**Benefits:**
- **Single source of truth**: All processes share same in-memory state
- **No synchronization issues**: Server handles all coordination
- **Hot reload**: Restart clients without losing state
- **Monitoring**: Centralized metrics and health checks
- **Scalability**: Can move to different machine later

## Quick Start

### 1. Build Storage Server

```bash
cd packages/sutra-storage
cargo build --release --features="server"
```

Or for development:
```bash
maturin develop --features="server"
```

### 2. Start Storage Server

```bash
# Using Python wrapper
./packages/sutra-storage/bin/storage-server \
  --host 0.0.0.0 \
  --port 50051 \
  --storage ./knowledge \
  --reconcile-ms 10 \
  --threshold 50000

# Or directly with Rust binary
cargo run --release --bin storage-server -- \
  --host 0.0.0.0 \
  --port 50051 \
  --storage ./knowledge
```

Server output:
```
🚀 Starting Sutra Storage Server
   Host: 0.0.0.0:50051
   Storage: /path/to/knowledge
   Reconciliation: 10ms
   Threshold: 50000 concepts

✅ PRODUCTION STARTUP: Loaded 1000 concepts and 5000 edges from storage.dat
📡 Server listening on 0.0.0.0:50051
```

### 3. Configure Clients

Set environment variables:

```bash
export SUTRA_STORAGE_MODE=server
export SUTRA_STORAGE_SERVER=localhost:50051
```

### 4. Run Your Application

```bash
# API server
cd packages/sutra-api
python -m sutra_api.main

# CLI
cd packages/sutra-cli
python -m sutra_cli.main

# All use the same storage server!
```

## Configuration

### Server Options

| Option | Default | Description |
|--------|---------|-------------|
| `--host` | `0.0.0.0` | Server bind address |
| `--port` | `50051` | gRPC port |
| `--storage` | `./knowledge` | Storage directory path |
| `--reconcile-ms` | `10` | Background reconciliation interval (ms) |
| `--threshold` | `50000` | Memory threshold before disk flush |
| `--vector-dim` | `768` | Vector embedding dimension |

### Client Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `SUTRA_STORAGE_MODE` | `embedded` | Storage mode: `embedded` or `server` |
| `SUTRA_STORAGE_SERVER` | `localhost:50051` | Server address (when mode=server) |

## Comparison: Embedded vs Server

### Embedded Mode (Current)

```python
from sutra_storage import ConcurrentStorage

# Each process has its own instance
storage = ConcurrentStorage("./knowledge")
```

**Pros:**
- Simple deployment
- No network overhead
- Works offline

**Cons:**
- Multiple processes can't share state
- Must reload from disk on startup
- No coordination between instances

### Server Mode (New)

```python
from sutra_storage_client import StorageClient

# All processes connect to same server
storage = StorageClient("localhost:50051")
```

**Pros:**
- Single source of truth
- Shared in-memory state
- Hot reload capability
- Centralized monitoring

**Cons:**
- Network latency (~0.1-0.5ms local)
- Requires running server
- Single point of failure (can add HA later)

## Migration Guide

### Step 1: Install Client

```bash
pip install grpcio grpcio-tools
cd packages/sutra-storage-client
pip install -e .
```

### Step 2: Generate Protobuf Code

```bash
cd packages/sutra-storage
python -m grpc_tools.protoc \
  -I./proto \
  --python_out=../sutra-storage-client/sutra_storage_client \
  --grpc_python_out=../sutra-storage-client/sutra_storage_client \
  proto/storage.proto
```

### Step 3: Update Code (Optional)

Your existing code works unchanged! Just set environment variables:

```python
# Old code - still works!
from sutra_core.storage import RustStorageAdapter

storage = RustStorageAdapter("./knowledge")
# Automatically uses server mode if SUTRA_STORAGE_MODE=server
```

Or explicitly:

```python
# New explicit code
from sutra_core.storage.connection import get_storage_backend

storage = get_storage_backend("./knowledge")
# Returns StorageClient or ConcurrentStorage based on env
```

## Performance

### Embedded Mode
- **Write**: 0.02ms per concept (57K/sec)
- **Read**: <0.01ms (zero-copy mmap)
- **Path finding**: ~1ms (3-hop BFS)

### Server Mode (Local)
- **Write**: ~0.1ms (embedded + network)
- **Read**: ~0.05ms (embedded + network)
- **Path finding**: ~1.1ms (embedded + network)
- **Network overhead**: ~50-100µs per RPC

### Server Mode (Remote - 1ms RTT)
- **Write**: ~1ms (embedded + network)
- **Read**: ~1ms (embedded + network)
- **Batch operations recommended for remote**

## Production Deployment

### Systemd Service

```ini
[Unit]
Description=Sutra Storage Server
After=network.target

[Service]
Type=simple
User=sutra
WorkingDirectory=/opt/sutra
ExecStart=/opt/sutra/bin/storage-server \
  --host 0.0.0.0 \
  --port 50051 \
  --storage /var/lib/sutra/knowledge \
  --reconcile-ms 10 \
  --threshold 50000

Restart=on-failure
RestartSec=5s

[Install]
WantedBy=multi-user.target
```

### Docker

```dockerfile
FROM rust:1.70 as builder
WORKDIR /build
COPY packages/sutra-storage .
RUN cargo build --release --features="server"

FROM debian:bookworm-slim
COPY --from=builder /build/target/release/storage-server /usr/local/bin/
EXPOSE 50051
CMD ["storage-server", "--host", "0.0.0.0", "--port", "50051"]
```

### Health Checks

```bash
# Check if server is running
grpcurl -plaintext localhost:50051 list

# Get stats
grpcurl -plaintext localhost:50051 sutra.storage.StorageService/GetStats
```

## Monitoring

### Metrics to Track

1. **Storage Stats** (via `GetStats` RPC):
   - `concepts`: Total concepts in storage
   - `edges`: Total edges/associations
   - `written`: Total writes since startup
   - `pending`: Pending writes in buffer
   - `reconciliations`: Background sync count

2. **Server Metrics**:
   - RPC latency percentiles (p50, p95, p99)
   - Request rate by operation
   - Error rate by operation
   - Active connections

3. **System Metrics**:
   - Memory usage
   - CPU usage
   - Disk I/O
   - Network throughput

### Example Prometheus Exporter

```python
from prometheus_client import Counter, Histogram, start_http_server

# Define metrics
rpc_requests = Counter('storage_rpc_requests_total', 'Total RPC requests', ['method'])
rpc_latency = Histogram('storage_rpc_latency_seconds', 'RPC latency', ['method'])

# Start metrics server
start_http_server(8000)
```

## Troubleshooting

### Server won't start

```bash
# Check if port is in use
lsof -i :50051

# Check storage permissions
ls -la ./knowledge/

# Check logs
journalctl -u storage-server -f
```

### Client connection errors

```bash
# Verify server is reachable
nc -zv localhost 50051

# Check environment variables
echo $SUTRA_STORAGE_MODE
echo $SUTRA_STORAGE_SERVER

# Test with grpcurl
grpcurl -plaintext localhost:50051 list
```

### Performance issues

1. **High latency**: Check network latency with `ping`
2. **High memory**: Reduce `--threshold` to flush more often
3. **Slow queries**: Check storage stats for pending writes

## Future Enhancements

### Phase 2: High Availability
- Multiple storage replicas with leader election
- Read replicas for query load balancing
- Consensus protocol (Raft) for consistency

### Phase 3: Shared Memory (Zero-copy reads)
- Memory-mapped storage.dat shared between processes
- Server manages writes only
- Clients read directly from mmap

### Phase 4: Distributed Storage
- Sharding across multiple servers
- Consistent hashing for concept distribution
- Cross-shard queries via coordinator

## FAQ

**Q: Can I mix embedded and server mode?**  
A: Yes, but they won't share state. Useful for testing.

**Q: What happens if server crashes?**  
A: Storage is persisted to `storage.dat`. Restart server to reload.

**Q: Can clients on different machines connect?**  
A: Yes, set `SUTRA_STORAGE_SERVER=hostname:50051` with accessible hostname.

**Q: How do I back up the storage?**  
A: Copy `storage.dat` file. Consider sending `Flush` RPC first.

**Q: Can I run multiple servers?**  
A: Not currently - only one server per storage.dat. HA coming in Phase 2.

# Storage Server Quick Start

## TL;DR

✅ All services connect to a standalone storage server over gRPC  
✅ No embedded/in-process storage in API/Hybrid  
✅ Python clients use the included storage-client package

## What You Get

**Before (Embedded):**
```
Process 1 → ConcurrentStorage → storage.dat
Process 2 → ConcurrentStorage → storage.dat (separate instance!)
```

**After (Server):**
```
Process 1 ──┐
Process 2 ──┼─→ Storage Server → storage.dat (single source of truth!)
Process 3 ──┘
```

## Quick Start

### 1) Build storage-server

```toml
# packages/sutra-storage/Cargo.toml
[dependencies]
tonic = "0.10"
prost = "0.12"
tokio = { version = "1", features = ["full"] }

[build-dependencies]
tonic-build = "0.10"
```

### Step 2: Modify Storage Adapter (1 line!)

```python
# packages/sutra-core/sutra_core/storage/rust_adapter.py
# Line 55-66, replace:

# OLD:
try:
    self.store = ConcurrentStorage(
        str(self.storage_path),
        reconcile_interval_ms=10,
        memory_threshold=50000,
    )

# NEW:
from .connection import get_storage_backend
try:
    self.store = get_storage_backend(
        str(self.storage_path),
        reconcile_interval_ms=10,
        memory_threshold=50000,
        vector_dimension=self.vector_dimension,
    )
```

### Step 3: Build & Test (10 min)

```bash
# Build server
cd packages/sutra-storage
cargo build --release --features="server"

# OR with maturin (includes Python bindings)
maturin develop --features="server"

# Python storage client is already included as a package
# (no proto generation needed)

# Start server
./bin/storage-server --storage ./knowledge

# Start API and Hybrid (connect automatically via env)
export SUTRA_STORAGE_SERVER=localhost:50051
uvicorn sutra_api.main:app --host 0.0.0.0 --port 8000
uvicorn sutra_hybrid.api.app:app --host 0.0.0.0 --port 8001
```

## Usage Examples

### Development
```bash
# Run via docker-compose (recommended)
DEPLOY=local VERSION=v2 bash deploy-optimized.sh
```

### Manual run
```bash
# Terminal 1: Start storage server
./packages/sutra-storage/bin/storage-server \
  --host 0.0.0.0 \
  --port 50051 \
  --storage /var/lib/sutra/knowledge

# Terminal 2: Start API (connects automatically)
# no SUTRA_STORAGE_MODE; gRPC is the only mode now
export SUTRA_STORAGE_SERVER=localhost:50051
cd packages/sutra-api
python -m sutra_api.main

# Terminal 3: Start another service (shares same storage!)
export SUTRA_STORAGE_MODE=server
python demo_mass_learning.py
```

### Docker Compose
```yaml
version: '3.8'
services:
  storage:
    build: ./packages/sutra-storage
    ports:
      - "50051:50051"
    volumes:
      - ./knowledge:/data
    command: storage-server --host 0.0.0.0 --storage /data

  api:
    build: ./packages/sutra-api
    ports:
      - "8000:8000"
    environment:
      - SUTRA_STORAGE_MODE=server
      - SUTRA_STORAGE_SERVER=storage:50051
    depends_on:
      - storage
```

## Verification Checklist

### ✅ Health checks
```bash
curl -f http://localhost:8000/health
curl -f http://localhost:8001/ping
```

### ✅ Stats
```bash
# Start server first
./bin/storage-server &

grpcurl -plaintext localhost:50051 list
```

### ✅ Multi-Client Shared State (via gRPC)
```bash
# Terminal 1
export SUTRA_STORAGE_SERVER=localhost:50051
python -c "
from sutra_core import ReasoningEngine
e = ReasoningEngine()
e.learn('Redis is an in-memory database')
"

# Terminal 2
export SUTRA_STORAGE_SERVER=localhost:50051
python -c "
from sutra_core import ReasoningEngine
e = ReasoningEngine()
result = e.query('What is Redis?')
print(result)
"
# Expected: Should return result about Redis from Terminal 1
```

## Troubleshooting

### Server won't start
```bash
# Check if port is in use
lsof -i :50051

# Check storage permissions
ls -la ./knowledge/

# Run with debug logging
RUST_LOG=debug ./bin/storage-server
```

### Client can't connect
```bash
# Verify server is running
nc -zv localhost 50051

# Check environment
echo $SUTRA_STORAGE_MODE
echo $SUTRA_STORAGE_SERVER

# Test with grpcurl
grpcurl -plaintext localhost:50051 list
```

### Still using embedded mode
```bash
# Make sure env vars are set
export SUTRA_STORAGE_MODE=server

# Verify in Python
python -c "
import os
print('Mode:', os.getenv('SUTRA_STORAGE_MODE'))
from sutra_core.storage.connection import get_storage_backend
backend = get_storage_backend('./knowledge')
print('Backend:', type(backend))
# Expected: <class 'sutra_storage_client.client.StorageClient'>
"
```

## Next Steps

1. **Phase 1 (Now):** Implement changes, build, test locally
2. **Phase 2 (Week 1):** Deploy to dev environment, collect metrics
3. **Phase 3 (Week 2):** Parallel testing (embedded + server)
4. **Phase 4 (Month 1):** Production deployment with monitoring
5. **Phase 5 (Month 2+):** Add HA, replication, distributed features

## Key Files Created

```
docs/
  STORAGE_SERVER.md              # Complete guide
  STORAGE_SERVER_COMPATIBILITY.md # Package analysis
  STORAGE_SERVER_QUICKSTART.md   # This file

packages/sutra-storage/
  proto/storage.proto            # gRPC service definition
  src/server.rs                  # Server implementation
  bin/storage-server             # Startup script

packages/sutra-storage-client/
  sutra_storage_client/
    client.py                    # Python gRPC client

packages/sutra-core/
  sutra_core/storage/
    connection.py                # Mode selector
```

## Performance Expectations

| Operation | Embedded | Server (Local) | Server (Remote) |
|-----------|----------|----------------|-----------------|
| Write | 0.02ms | ~0.1ms | ~1ms |
| Read | <0.01ms | ~0.05ms | ~1ms |
| Path Finding | ~1ms | ~1.1ms | ~2ms |

**Local server mode is ~5x slower but still excellent performance!**

## Documentation

- **Full Guide:** `docs/STORAGE_SERVER.md`
- **Compatibility:** `docs/STORAGE_SERVER_COMPATIBILITY.md`
- **This Quick Start:** `docs/STORAGE_SERVER_QUICKSTART.md`

## Questions?

- Architecture questions → See `STORAGE_SERVER.md`
- Package compatibility → See `STORAGE_SERVER_COMPATIBILITY.md`
- Implementation help → See this file
- Issues → Check Troubleshooting section above

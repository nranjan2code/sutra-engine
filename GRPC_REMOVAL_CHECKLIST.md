# gRPC Removal - 100% Completion Checklist

## ✅ COMPLETED

### 1. Protocol Layer
- ✅ Created `sutra-protocol` package
- ✅ Core protocol with bincode serialization
- ✅ Production client with connection pooling
- ✅ Error types (ProtocolError)
- ✅ Logging dependencies added

### 2. Storage Server
- ✅ TCP server implementation (`tcp_server.rs`)
- ✅ Binary entrypoint (`bin/storage_server.rs`)
- ✅ Cargo.toml updated (removed gRPC, added TCP binary)
- ✅ Module exported in lib.rs

### 3. Python Client
- ✅ TCP storage client created (`sutra-storage-client-tcp`)
- ✅ Drop-in API replacement
- ✅ setup.py for installation

## ⏳ IN PROGRESS - CRITICAL

### 4. API Integration
- ✅ dependencies.py updated to use new client
- ⏳ Actually import and test the client
- ⏳ Update requirements.txt

### 5. Build System
- ⏳ Test compilation of storage-server binary
- ⏳ Build Docker images
- ⏳ Update docker-compose.yml

## 🔴 REMAINING - HIGH PRIORITY

### 6. Grid Components
- ⏳ Update grid-master to use sutra-protocol
- ⏳ Update grid-agent to use sutra-protocol
- ⏳ Remove gRPC from Grid Cargo.toml files

### 7. Docker & Deployment
- ⏳ Update storage Dockerfile
- ⏳ Update docker-compose-grid.yml
- ⏳ Update sutra-deploy.sh
- ⏳ Test end-to-end deployment

### 8. Testing
- ⏳ Integration tests
- ⏳ Load tests
- ⏳ Deployment verification

## 📋 DETAILED TASKS

### A. Finish Protocol (5 min)
```bash
# Export error module
cd packages/sutra-protocol/src
# Add: pub mod error; to lib.rs
# Add: pub use error::{ProtocolError, Result}; to lib.rs
```

### B. Update API Requirements (2 min)
```bash
cd packages/sutra-api
# Remove from requirements.txt: grpcio, grpcio-tools, protobuf
# Add: msgpack
# Install: pip install -e ../sutra-storage-client-tcp
```

### C. Test Storage Server Build (5 min)
```bash
cd packages/sutra-storage
cargo build --release --bin storage-server
# Should compile successfully
```

### D. Update Grid Master (15 min)
```bash
cd packages/sutra-grid-master
# Add to Cargo.toml: sutra-protocol = { path = "../sutra-protocol" }
# Remove: tonic, prost, tonic-build
# Update src/main.rs to use TCP listener + sutra_protocol
```

### E. Update Grid Agent (15 min)
```bash
cd packages/sutra-grid-agent
# Add to Cargo.toml: sutra-protocol = { path = "../sutra-protocol" }
# Remove: tonic, prost
# Update src/agent.rs to use GridClient
```

### F. Update Dockerfiles (10 min)
```dockerfile
# packages/sutra-storage/Dockerfile
# Change: RUN cargo build --release --bin storage-server-grpc
# To: RUN cargo build --release --bin storage-server
# Change CMD to run new binary

# packages/sutra-api/Dockerfile
# Remove: RUN pip install grpcio grpcio-tools
# Add: COPY ../sutra-storage-client-tcp and pip install

# Same for sutra-hybrid, grid-master, grid-agent
```

###G. Update docker-compose (10 min)
```yaml
# docker-compose-grid.yml
# storage-server: no changes (same port, same image, just new binary)
# grid-master: no changes (same ports)
# grid-agent: no changes
# sutra-api: update environment variables
# sutra-hybrid: update environment variables
```

### H. Integration Test (10 min)
```bash
# Test local
cd packages/sutra-storage
cargo run --bin storage-server &
sleep 2
cd ../sutra-storage-client-tcp
python3 -c "
from sutra_storage_client import StorageClient
client = StorageClient('localhost:50051')
print(client.health_check())
"
```

## PRIORITY ORDER

1. **Finish protocol exports** (2 min) - critical for compilation
2. **Test storage build** (5 min) - verify server compiles
3. **Update API requirements** (2 min) - verify Python client works
4. **Test local deployment** (10 min) - verify end-to-end
5. **Update Grid components** (30 min) - complete the migration
6. **Update Dockerfiles** (20 min) - production deployment
7. **Update docker-compose** (10 min) - orchestration
8. **Full integration test** (20 min) - verify everything works

## ESTIMATED TIME TO 100%

- High priority tasks: **1.5 hours**
- Testing & verification: **30 minutes**
- **Total: ~2 hours to production-ready**

## COMMANDS TO RUN NOW

```bash
# 1. Finish protocol
cd packages/sutra-protocol/src
echo 'pub mod error;' >> lib.rs
echo 'pub use error::{ProtocolError, Result};' >> lib.rs

# 2. Test storage build
cd ../../sutra-storage
cargo build --release --bin storage-server

# 3. Install Python client
cd ../sutra-storage-client-tcp
pip install -e .

# 4. Test integration
cd ../sutra-storage
cargo run --bin storage-server &
SERVER_PID=$!
sleep 3
python3 -c "from sutra_storage_client import StorageClient; c = StorageClient('localhost:50051'); print(c.health_check())"
kill $SERVER_PID
```

## SUCCESS CRITERIA

✅ Storage server compiles and runs
✅ Python client connects and works
✅ Grid components compile with new protocol
✅ Docker images build successfully
✅ docker-compose up starts all services
✅ End-to-end request (learn → query) works
✅ Performance is 10-50× better than gRPC
✅ Zero data loss during migration

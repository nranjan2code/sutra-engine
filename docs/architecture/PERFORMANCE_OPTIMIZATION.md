# Performance Optimization Guide

**Version 3.0.0** | Last Updated: November 8, 2025

## Overview

This guide documents performance optimizations implemented in Sutra AI v3.0 that achieved **50-70× throughput improvements** and resolved critical bottlenecks in concurrent processing.

---

## Performance Achievements (November 2025)

### Benchmark Results

| Test Scenario | Before | After | Improvement |
|--------------|--------|-------|-------------|
| **Sequential Processing** | 0.13 req/sec (7542ms) | 9.06 req/sec (107ms) | **70× faster** |
| **Thread Concurrent (2)** | 0.13 req/sec (14888ms) | 6.52 req/sec (306ms) | **49× faster** |
| **Async Concurrent (5)** | 0% success (30900ms) | 100% success (649ms) | **∞ improvement** |

### System Status
- ✅ **Scaling Configuration**: Enabled
- ✅ **Concurrent Processing**: Working
- ✅ **System Stability**: 100% success rate across all tests

---

## Critical Fixes Implemented

### 1. Config-Driven Dimension Validation

**Problem**: Hardcoded 768-dimensional embedding validation caused 100% failure when using Matryoshka 256-dim embeddings.

**Root Cause**:
```rust
// packages/sutra-storage/src/embedding_client.rs:139 (BEFORE)
if embedding_response.dimension != 768 {  // ❌ Hardcoded
    return Err(anyhow::anyhow!(
        "Expected 768-dimensional embeddings, got {}",
        embedding_response.dimension
    ));
}
```

**Symptoms**:
- Storage logs: `ERROR: Expected 768-dimensional embeddings, got 256`
- 4 retry attempts × 3-4 seconds = **12-16s per request**
- Async tests: **0% success rate** (30s timeout exceeded)

**Solution**:
```rust
// packages/sutra-storage/src/embedding_client.rs (AFTER)
pub struct EmbeddingConfig {
    // ... other fields
    pub expected_dimension: usize,  // ✅ Config-driven
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            // ... other fields
            expected_dimension: std::env::var("VECTOR_DIMENSION")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(768),  // Default for backward compatibility
        }
    }
}

// Validation now uses config
if embedding_response.dimension as usize != self.config.expected_dimension {
    return Err(anyhow::anyhow!(
        "Expected {}-dimensional embeddings (VECTOR_DIMENSION), got {}",
        self.config.expected_dimension,
        embedding_response.dimension
    ));
}
```

**Impact**: Eliminated 15s retry penalty, enabled 256-dim Matryoshka embeddings (3× faster than 768-dim).

**Files Modified**:
- `packages/sutra-storage/src/embedding_client.rs`

---

### 2. TCP Connection Resilience

**Problem**: TCP client had fragile connection handling with no state validation, causing `'NoneType' object has no attribute 'sendall'` errors.

**Root Cause**:
```python
# packages/sutra-storage-client-tcp/sutra_storage_client/__init__.py (BEFORE)
def _connect(self):
    self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    self.socket.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
    self.socket.connect(self.address)  # ❌ No error handling

def _send_request(self, variant_name: str, data: dict) -> dict:
    # ❌ No connection state check
    self.socket.sendall(struct.pack(">I", len(packed)))  # Crashes if socket=None
```

**Solution**:
```python
# packages/sutra-storage-client-tcp/sutra_storage_client/__init__.py (AFTER)
def _connect(self):
    """Establish TCP connection with error handling"""
    try:
        if self.socket:
            try:
                self.socket.close()
            except:
                pass
        self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.socket.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
        self.socket.settimeout(30.0)  # ✅ 30s timeout
        self.socket.connect(self.address)
    except Exception as e:
        self.socket = None  # ✅ Clear state on failure
        raise ConnectionError(f"Failed to connect to {self.address}: {e}")

def _send_request(self, variant_name: str, data: dict) -> dict:
    if not self.socket:  # ✅ State validation
        raise ConnectionError("Not connected to storage server")
    # ... rest of logic
```

**Impact**: Eliminated connection crashes, enabled proper error reporting.

**Files Modified**:
- `packages/sutra-storage-client-tcp/sutra_storage_client/__init__.py`

---

### 3. Enhanced TCP Server Monitoring

**Problem**: TCP server had no visibility into slow requests or connection lifecycle, making performance debugging difficult.

**Solution**:
```rust
// packages/sutra-storage/src/tcp_server.rs (AFTER)
async fn handle_client(&self, mut stream: TcpStream, peer_addr: SocketAddr) -> std::io::Result<()> {
    eprintln!("Client connected: {}", peer_addr);
    stream.set_nodelay(true)?;
    
    let mut request_count = 0u64;
    
    loop {
        let request_start = std::time::Instant::now();
        
        // ... request handling ...
        
        request_count += 1;
        let request_duration = request_start.elapsed();
        
        // ✅ Log slow requests (> 1s)
        if request_duration.as_millis() > 1000 {
            eprintln!("⚠️  Slow request from {}: {}ms (total: {})", 
                peer_addr, request_duration.as_millis(), request_count);
        }
    }
    
    eprintln!("Client {} disconnected after {} requests", peer_addr, request_count);
}
```

**Impact**: Enabled production performance monitoring, simplified debugging.

**Files Modified**:
- `packages/sutra-storage/src/tcp_server.rs`

---

### 4. Connection Pooling & Keep-Alive

**Problem**: Stress tests created new connections per request, causing TCP overhead and connection exhaustion.

**Solution**:
```python
# scripts/stress_test.py (AFTER)
async def send_async_request(self, session: aiohttp.ClientSession, concept: Dict[str, Any]):
    async with session.post(
        f"{self.base_url}/sutra/learn",
        json=concept,
        timeout=aiohttp.ClientTimeout(total=60),
        headers={"Connection": "keep-alive"}  # ✅ Explicit keep-alive
    ) as response:
        # ...

async def test_concurrent_async(self, num_requests: int, concurrency: int):
    connector = aiohttp.TCPConnector(
        limit=concurrency * 2,
        limit_per_host=concurrency,
        ttl_dns_cache=300,
        force_close=False,  # ✅ Keep connections alive
        enable_cleanup_closed=True
    )
    timeout = aiohttp.ClientTimeout(total=60, connect=10)
    
    async with aiohttp.ClientSession(connector=connector, timeout=timeout) as session:
        # ... use session for all requests
```

**Impact**: Reduced connection overhead, improved concurrent throughput.

**Files Modified**:
- `scripts/stress_test.py`

---

### 5. Environment Variable Consistency

**Problem**: Services had inconsistent dimension configuration - some read `VECTOR_DIMENSION`, others hardcoded values.

**Solution**: Standardized on `VECTOR_DIMENSION` environment variable across all services:

```yaml
# .sutra/compose/production.yml
services:
  storage-server:
    environment:
      - VECTOR_DIMENSION=${MATRYOSHKA_DIM:-768}  # ✅ Consistent
  
  # All services now use SUTRA_VECTOR_DIMENSION
  sutra-hybrid:
    environment:
      - SUTRA_VECTOR_DIMENSION=${MATRYOSHKA_DIM:-768}
```

**Python services automatically read**:
```python
# packages/sutra-hybrid/sutra_hybrid/embeddings/service.py
expected_dim = int(os.getenv("SUTRA_VECTOR_DIMENSION", "768"))
```

**Impact**: Eliminated dimension mismatch errors across service boundaries.

---

## Configuration Requirements

### Environment Variables

All services **MUST** set consistent dimension configuration:

```bash
# For 256-dim Matryoshka (3× faster embeddings)
export VECTOR_DIMENSION=256
export MATRYOSHKA_DIM=256
export SUTRA_VECTOR_DIMENSION=256

# For 768-dim full embeddings (maximum accuracy)
export VECTOR_DIMENSION=768
export MATRYOSHKA_DIM=768
export SUTRA_VECTOR_DIMENSION=768
```

### Docker Compose

```yaml
services:
  storage-server:
    environment:
      - VECTOR_DIMENSION=${MATRYOSHKA_DIM:-768}
      - SUTRA_EMBEDDING_SERVICE_URL=http://sutra-works-embedding-single:8888
      
  sutra-hybrid:
    environment:
      - SUTRA_VECTOR_DIMENSION=${MATRYOSHKA_DIM:-768}
      - SUTRA_EMBEDDING_SERVICE_URL=http://sutra-works-embedding-single:8888
      
  sutra-api:
    environment:
      - SUTRA_STORAGE_SERVER=storage-server:50051
```

### Network Aliases

Services communicate via Docker network aliases:
- Storage server: `storage-server` (NOT `sutra-works-storage`)
- Embedding service: `sutra-works-embedding-single`

---

## Performance Testing

### Running Stress Tests

```bash
# Quick test (10-25 requests per scenario)
python3 scripts/stress_test.py --quick

# Comprehensive test (multiple complexity levels)
python3 scripts/stress_test.py
```

### Expected Results (Post-Optimization)

```
✅ Sequential Baseline (simple)
   Requests:     10/10
   Success Rate: 100.0%
   Throughput:   9.06 req/sec
   Avg Latency:  107ms

✅ Thread Concurrent (2 threads, medium)
   Requests:     20/20
   Success Rate: 100.0%
   Throughput:   6.52 req/sec
   Avg Latency:  306ms

✅ Async Concurrent (5 parallel, medium)
   Requests:     25/25
   Success Rate: 100.0%
   Throughput:   7.69 req/sec
   Avg Latency:  649ms
```

### Performance Indicators

**Healthy System**:
- ✅ Success rate: 100%
- ✅ Sequential latency: <200ms
- ✅ Concurrent throughput: >5 req/sec
- ✅ No dimension mismatch errors in logs

**Degraded System**:
- ⚠️  Success rate: 90-99%
- ⚠️  Sequential latency: 200-500ms
- ⚠️  Occasional connection errors

**Failing System**:
- ❌ Success rate: <90%
- ❌ Sequential latency: >1000ms
- ❌ "Expected 768-dimensional embeddings" errors
- ❌ "NoneType has no attribute sendall" errors

---

## Troubleshooting

### Dimension Mismatch Errors

**Symptom**:
```
ERROR: Expected 768-dimensional embeddings, got 256
```

**Solution**:
1. Check environment variables:
   ```bash
   docker exec sutra-works-storage env | grep DIMENSION
   docker exec sutra-works-hybrid env | grep DIMENSION
   ```

2. Rebuild services with correct config:
   ```bash
   SUTRA_EDITION=simple ./sutra build storage hybrid
   ```

3. Restart with proper environment:
   ```bash
   docker-compose -f .sutra/compose/production.yml \
     --profile simple \
     -e MATRYOSHKA_DIM=256 \
     up -d
   ```

### Connection Failures

**Symptom**:
```
'NoneType' object has no attribute 'sendall'
```

**Solution**:
1. Verify network connectivity:
   ```bash
   docker exec sutra-works-api ping -c 2 storage-server
   ```

2. Check service hostnames in environment:
   ```bash
   docker inspect sutra-works-api | grep STORAGE_SERVER
   # Should show: storage-server:50051
   ```

3. Rebuild TCP client with fixes:
   ```bash
   ./sutra build api hybrid
   docker restart sutra-works-api sutra-works-hybrid
   ```

### Slow Performance

**Symptom**: Latency >1000ms, throughput <1 req/sec

**Solution**:
1. Check storage server logs for slow requests:
   ```bash
   docker logs sutra-works-storage | grep "Slow request"
   ```

2. Verify embedding service is responsive:
   ```bash
   curl -s http://localhost:8888/health
   ```

3. Check resource usage:
   ```bash
   docker stats --no-stream
   ```

---

## Best Practices

### 1. Always Use Environment Variables for Dimensions
❌ **DON'T**: Hardcode dimensions in code
```rust
if embedding_response.dimension != 768 {  // BAD
```

✅ **DO**: Read from environment
```rust
expected_dimension: std::env::var("VECTOR_DIMENSION")
    .ok()
    .and_then(|s| s.parse().ok())
    .unwrap_or(768)
```

### 2. Implement Connection State Validation
❌ **DON'T**: Assume socket is always connected
```python
self.socket.sendall(data)  # BAD - crashes if None
```

✅ **DO**: Validate before use
```python
if not self.socket:
    raise ConnectionError("Not connected")
self.socket.sendall(data)
```

### 3. Use Connection Pooling for Concurrent Requests
❌ **DON'T**: Create new connection per request
```python
for request in requests:
    response = requests.post(url, json=request)  # BAD
```

✅ **DO**: Reuse connections
```python
async with aiohttp.ClientSession() as session:
    for request in requests:
        async with session.post(url, json=request):  # GOOD
```

### 4. Monitor Slow Requests in Production
✅ **DO**: Add timing instrumentation
```rust
let start = std::time::Instant::now();
// ... handle request ...
let duration = start.elapsed();
if duration.as_millis() > 1000 {
    eprintln!("⚠️  Slow request: {}ms", duration.as_millis());
}
```

---

## Related Documentation

- [System Architecture](./SYSTEM_ARCHITECTURE.md) - Overall system design
- [Storage Architecture](../storage/README.md) - Storage engine internals
- [Embedding Service Scaling](./scaling/README.md) - HA embedding configuration
- [Troubleshooting Guide](../guides/troubleshooting.md) - Common issues

---

## Version History

- **v3.0.0** (November 8, 2025): Initial performance optimization documentation
  - 50-70× throughput improvements
  - Config-driven dimension validation
  - TCP connection resilience
  - Connection pooling & monitoring

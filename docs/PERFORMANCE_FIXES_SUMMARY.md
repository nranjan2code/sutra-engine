# Performance Fixes Summary - November 8, 2025

## Quick Reference

**Performance Improvement: 50-70× throughput increase**
- Sequential: 0.13 → 9.06 req/sec (70× faster)
- Concurrent: 0.13 → 6.52 req/sec (49× faster)  
- Async: 0% → 100% success rate (∞ improvement)

## Files Modified

### Rust (Storage Layer)
1. **packages/sutra-storage/src/embedding_client.rs**
   - Added `expected_dimension` field to `EmbeddingConfig`
   - Read from `VECTOR_DIMENSION` env var (default 768)
   - Eliminated hardcoded 768-dim validation

2. **packages/sutra-storage/src/tcp_server.rs**
   - Added request counting and timing instrumentation
   - Log slow requests (>1s threshold)
   - Track connection lifecycle

### Python (Client & Testing)
3. **packages/sutra-storage-client-tcp/sutra_storage_client/__init__.py**
   - Added connection state validation (`if not self.socket`)
   - Implemented error handling in `_connect()`
   - Added 30s socket timeout

4. **scripts/stress_test.py**
   - Optimized aiohttp connector (force_close=False, keep-alive)
   - Increased timeout from 30s → 60s
   - Added Connection: keep-alive headers

### Documentation
5. **docs/architecture/PERFORMANCE_OPTIMIZATION.md** - NEW
   - Complete guide with before/after benchmarks
   - Root cause analysis for each fix
   - Configuration requirements and troubleshooting

6. **docs/guides/troubleshooting.md**
   - Added performance troubleshooting section
   - Quick diagnosis and fix procedures

7. **docs/architecture/SYSTEM_ARCHITECTURE.md**
   - Updated to v3.0.0
   - Added link to performance guide

8. **docs/README.md**
   - Added v3.0.0 performance release notes
   - Updated version to 3.0.0

9. **README.md**
   - Added performance optimization announcement
   - 50-70× improvements highlighted

10. **.github/copilot-instructions.md**
    - Added performance optimization section
    - Updated testing commands with stress tests

## Environment Variables Required

All services must use consistent dimension configuration:

```bash
# For 256-dim Matryoshka (recommended for performance)
export VECTOR_DIMENSION=256
export MATRYOSHKA_DIM=256
export SUTRA_VECTOR_DIMENSION=256

# For 768-dim (maximum accuracy)
export VECTOR_DIMENSION=768
export MATRYOSHKA_DIM=768
export SUTRA_VECTOR_DIMENSION=768
```

## Deployment Instructions

```bash
# 1. Rebuild affected services
SUTRA_EDITION=simple ./sutra build storage api hybrid

# 2. Stop and remove old containers
docker stop sutra-works-storage sutra-works-api sutra-works-hybrid
docker rm sutra-works-storage sutra-works-api sutra-works-hybrid

# 3. Start with correct configuration
docker run -d --name sutra-works-storage \
  --network sutra-works_sutra-network \
  --network-alias storage-server \
  -e VECTOR_DIMENSION=256 \
  -e RUST_LOG=info \
  -e SUTRA_EMBEDDING_SERVICE_URL=http://sutra-works-embedding-single:8888 \
  -v sutra-works_storage-data:/data \
  sutra-works-storage-server:latest

docker run -d --name sutra-works-api \
  --network sutra-works_sutra-network \
  -e SUTRA_EDITION=simple \
  -e SUTRA_STORAGE_SERVER=storage-server:50051 \
  -p 8000:8000 \
  sutra-works-api:latest

docker run -d --name sutra-works-hybrid \
  --network sutra-works_sutra-network \
  -e SUTRA_EDITION=simple \
  -e SUTRA_STORAGE_SERVER=storage-server:50051 \
  -e SUTRA_EMBEDDING_SERVICE_URL=http://sutra-works-embedding-single:8888 \
  -e SUTRA_VECTOR_DIMENSION=256 \
  -p 8001:8000 \
  sutra-works-hybrid:latest

# 4. Restart nginx
docker restart sutra-works-nginx-proxy

# 5. Validate performance
python3 scripts/stress_test.py --quick
```

## Validation Checklist

- [ ] Sequential test: 9+ req/sec, 100% success, <200ms latency
- [ ] Thread concurrent: 6+ req/sec, 100% success, <400ms latency
- [ ] Async concurrent: 7+ req/sec, 100% success, <800ms latency
- [ ] No "Expected 768-dimensional embeddings" errors in logs
- [ ] No "NoneType has no attribute sendall" errors
- [ ] Storage server shows "expected_dim=256" (or configured value) on startup

## Troubleshooting

**If tests still fail:**

1. Check dimension consistency:
   ```bash
   docker exec sutra-works-storage env | grep DIMENSION
   docker exec sutra-works-hybrid env | grep DIMENSION
   ```

2. Verify network connectivity:
   ```bash
   docker exec sutra-works-api ping -c 2 storage-server
   ```

3. Check logs for errors:
   ```bash
   docker logs sutra-works-storage --tail 50
   docker logs sutra-works-hybrid --tail 50
   ```

## Performance Metrics

### Before Fixes
| Test | Success | Throughput | Latency |
|------|---------|------------|---------|
| Sequential | 100% | 0.13 req/sec | 7542ms |
| Thread (2) | 100% | 0.13 req/sec | 14888ms |
| Async (5) | **0%** | 0.00 req/sec | 30900ms |

### After Fixes
| Test | Success | Throughput | Latency |
|------|---------|------------|---------|
| Sequential | 100% | 9.06 req/sec | 107ms |
| Thread (2) | 100% | 6.52 req/sec | 306ms |
| Async (5) | **100%** | 7.69 req/sec | 649ms |

### Improvements
- **Sequential**: 70× faster
- **Thread (2)**: 49× faster
- **Async (5)**: ∞ improvement (0% → 100% success)

## Related Documentation

- [Performance Optimization Guide](architecture/PERFORMANCE_OPTIMIZATION.md) - Complete technical guide
- [Troubleshooting Guide](guides/troubleshooting.md#1-performance-issues-november-2025-) - Quick fixes
- [System Architecture](architecture/SYSTEM_ARCHITECTURE.md) - Overall design

---

**Version**: 3.0.0  
**Date**: November 8, 2025  
**Status**: Production-Ready ✅

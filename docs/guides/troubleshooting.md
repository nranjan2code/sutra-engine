# Sutra AI Troubleshooting Guide

## Quick Diagnostics

### Check System Health
```bash
# All-in-one health check
curl -s http://localhost:8000/stats | jq '{concepts: .total_concepts, embeddings: .total_embeddings, associations: .total_associations}'
```

**Expected:**
- `embeddings` == `concepts` (if not, see "Same Answer Problem" below)
- `associations` > 0 (if concepts > 5, indicates graph building)

---

## Common Issues

### 1. Performance Issues (November 2025) ⚡ NEW

**Symptoms:**
- Very slow learning (>5 seconds per concept)
- Async requests timing out
- "Expected 768-dimensional embeddings, got 256" errors
- `'NoneType' object has no attribute 'sendall'` errors
- 0% success rate on concurrent tests

**Quick Diagnosis:**
```bash
# Run stress test
python3 scripts/stress_test.py --quick

# Expected (healthy system):
# Sequential: ~9 req/sec, 100% success
# Async (5): ~8 req/sec, 100% success

# Check dimension configuration
docker exec sutra-works-storage env | grep VECTOR_DIMENSION
docker exec sutra-works-hybrid env | grep VECTOR_DIMENSION
# Should both show same value (256 or 768)
```

**Root Causes & Fixes:**

**A. Dimension Mismatch (70× slowdown)**
```bash
# Symptom in logs:
docker logs sutra-works-storage 2>&1 | grep "Expected 768"
# Shows: "Expected 768-dimensional embeddings, got 256"

# Fix: Set consistent dimension across all services
export MATRYOSHKA_DIM=256
SUTRA_EDITION=simple ./sutra build storage hybrid
./sutra deploy
```

**B. TCP Connection Errors**
```bash
# Symptom in logs:
docker logs sutra-works-hybrid 2>&1 | grep "NoneType"
# Shows: "'NoneType' object has no attribute 'sendall'"

# Fix: Rebuild with updated TCP client
./sutra build api hybrid
docker restart sutra-works-api sutra-works-hybrid
```

**C. Network Configuration**
```bash
# Verify service can reach storage
docker exec sutra-works-api ping -c 2 storage-server
# Should succeed

# Check environment variables
docker inspect sutra-works-api | grep STORAGE_SERVER
# Should show: storage-server:50051 (NOT sutra-works-storage)
```

**Full Details:** [`docs/architecture/PERFORMANCE_OPTIMIZATION.md`](../architecture/PERFORMANCE_OPTIMIZATION.md)

**Performance Benchmarks:**
- After fixes: 9+ req/sec, <200ms latency, 100% success
- Before fixes: 0.13 req/sec, 7500ms latency, 0% async success

---

### 2. Same Answer for All Questions ⭐ MOST COMMON

**Status:** ✅ **FIXED** (2025-10-19) - Unified learning architecture prevents this bug

**Symptoms:**
- Every query returns identical answer
- Different questions get same response
- System appears to work but is non-functional

**Historical Issue (Pre-2025-10-19):**
- Old architecture: Only Hybrid service generated embeddings
- API and Bulk Ingester learned without embeddings
- Result: Zero embeddings → "same answer" bug

**Current Architecture (Post-2025-10-19):**
- ✅ Storage server owns complete learning pipeline
- ✅ ALL services automatically generate embeddings
- ✅ Bug cannot occur with new architecture

**If Issue Persists (Ollama Problem):**
```bash
# Check embeddings
curl -s http://localhost:8000/stats | jq '.total_embeddings'
# If 0: Ollama not running or not configured

# Fix Ollama
curl http://localhost:11434/api/tags | jq '.models[].name'
# Should include: granite-embedding:30m
# If missing: ollama pull granite-embedding:30m

# Restart storage to reconnect to Ollama
docker restart sutra-storage

# Learn test fact (ANY service works now)
curl -X POST http://localhost:8001/sutra/learn \
  -H "Content-Type: application/json" \
  -d '{"text":"Paris is the capital of France"}'

# Verify
curl -s http://localhost:8000/stats | jq '.total_embeddings'
# Should be > 0
```

**Full Details:** [`docs/UNIFIED_LEARNING_ARCHITECTURE.md`](docs/UNIFIED_LEARNING_ARCHITECTURE.md)

---

### 2. "No embedding processor available"

**Symptoms:**
```
RuntimeError: No embedding processor available
```

**Quick Fix:**
```bash
# Check Ollama
curl http://localhost:11434/api/tags | jq '.models[].name'
# Should include: granite-embedding:30m

# If missing, install:
ollama pull granite-embedding:30m

# Restart hybrid service
docker restart sutra-hybrid

# Verify
docker logs sutra-hybrid 2>&1 | grep "Initialized Ollama"
# Should see: "Initialized OllamaEmbedding with model: granite-embedding:30m"
```

**Full Details:** [`docs/EMBEDDING_TROUBLESHOOTING.md`](../EMBEDDING_TROUBLESHOOTING.md) Section 1

---

### 3. Services Won't Start

**Check logs:**
```bash
sutra logs
# Or specific service:
docker logs sutra-api
docker logs sutra-hybrid
docker logs sutra-storage
```

**Common causes:**
- Port conflicts (change ports in .sutra/compose/production.yml)
- Ollama not running (see Issue #2)
- Build errors (run `sutra clean && sutra deploy`)

---

### 4. "Storage-backed pathfinding failed"

**Symptoms:**
```
WARNING - Storage-backed pathfinding failed: 'TcpStorageAdapter' object has no attribute 'find_paths'
```

**Root Cause:** Missing TCP adapter methods (fixed in latest version)

**Quick Fix:**
```bash
# Pull latest code
git pull origin main

# Restart services (code auto-reloads in containers)
docker restart sutra-api sutra-hybrid sutra-client
```

**Note:** This was fixed on 2025-10-19 with addition of:
- `find_paths()` - Multi-path reasoning
- `get_association()` - Association retrieval
- `get_all_concept_ids()` - Health checks

---

### 5. High Latency / Slow Queries

**Check embedding generation:**
```bash
docker logs sutra-hybrid 2>&1 | grep -i "embedding" | tail -20
```

**Optimize:**
- Ensure Ollama has GPU access (if available)
- Check network latency between services
- Monitor storage stats: `curl http://localhost:8000/stats`

---

### 6. Docker Volume Issues

**Clean all data (⚠️ DESTRUCTIVE):**
```bash
# Stop everything
./sutra-deploy.sh down

# Remove all volumes
docker volume rm sutra-models_storage-data
docker volume rm sutra-models_grid-event-data
docker volume rm sutra-models_agent1-data
docker volume rm sutra-models_agent2-data

# Restart fresh
./sutra-deploy.sh up
```

---

## Verification Checklist

After any fix, run these checks:

```bash
# 1. Services healthy
sutra status

# 2. Embeddings present
curl -s http://localhost:8000/stats | jq '.total_embeddings'
# Should be > 0

# 3. Ollama working
curl -s http://localhost:8001/health | jq '.embedding_model'
# Should show model name

# 4. Learn test
curl -X POST http://localhost:8001/sutra/learn \
  -H "Content-Type: application/json" \
  -d '{"text":"Test fact for verification"}'
# Should return success

# 5. Query test
curl -X POST http://localhost:8001/sutra/query \
  -H "Content-Type: application/json" \
  -d '{"query":"What is a test?"}'
# Should return answer with confidence > 0
```

---

## Emergency Recovery

If system is completely broken:

```bash
# 1. Full cleanup
sutra clean

# 2. Verify Ollama
ollama pull granite-embedding:30m
curl http://localhost:11434/api/tags

# 3. Fresh install
sutra deploy

# 4. Test immediately
curl -X POST http://localhost:8001/sutra/learn -H "Content-Type: application/json" -d '{"text":"Test"}'
curl -s http://localhost:8000/stats | jq
```

---

## Getting Help

1. **Check logs first:** `sutra logs [service-name]`
2. **Review docs:**
   - [`docs/deployment/README.md`](../deployment/README.md) - Deployment guide
   - [`docs/EMBEDDING_TROUBLESHOOTING.md`](../EMBEDDING_TROUBLESHOOTING.md) - Detailed embedding fixes
   - [`WARP.md`](../../WARP.md) - Development reference
3. **Verify prerequisites:**
   - Docker 20.10+
   - Docker Compose 1.29+
   - Ollama with granite-embedding:30m
   - Ports 8000-8002, 9000, 11434, 50051-50052 available

---

## Key Lessons Learned

1. **Embeddings are mandatory** - Without them, system degrades to random retrieval
2. **Always learn via Hybrid** - Use `/sutra/learn` endpoint, not `/learn`
3. **Verify before use** - Check `total_embeddings` matches `total_concepts`
4. **TCP adapter needs all methods** - Ensure `find_paths`, `get_association`, `get_all_concept_ids` exist
5. **Ollama first** - Start Ollama BEFORE learning any data

---

**Last Updated:** 2025-10-19  
**Critical Fix:** Same answer issue resolution (Section 1)

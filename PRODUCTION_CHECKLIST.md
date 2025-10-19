# Sutra AI Production Deployment Checklist

## \u26a0\ufe0f CRITICAL: Complete This Checklist Before ANY Deployment

**Date Applied:** 2025-10-19  
**Reason:** Prevent complete system failures due to missing embedding configuration

---

## Pre-Deployment Verification (MANDATORY)

### \ud83d\udd34 1. Ollama Service (SYSTEM WILL NOT WORK WITHOUT THIS)

- [ ] Ollama service running on host machine
- [ ] Port 11434 accessible
- [ ] `granite-embedding:30m` model loaded and ready
- [ ] Test command: `curl http://localhost:11434/api/tags | grep granite-embedding`
- [ ] Docker container can reach host: `docker exec sutra-hybrid curl http://host.docker.internal:11434/api/tags`

**Failure Impact:** "No embedding processor available" error \u2192 All queries return "I don't have enough knowledge"

---

### \ud83d\udd34 2. Environment Variables (CRITICAL CONFIGURATION)

- [ ] `SUTRA_OLLAMA_URL=http://host.docker.internal:11434` set in hybrid service
- [ ] `SUTRA_STORAGE_SERVER=storage-server:50051` set in all services  
- [ ] `SUTRA_STORAGE_MODE=server` set in distributed services
- [ ] Verify: `docker exec sutra-hybrid env | grep -E "(OLLAMA|STORAGE)"`

**Failure Impact:** Services cannot communicate \u2192 Connection errors and timeouts

---

### \ud83d\udd34 3. Package Versions (TCP PROTOCOL FIXES REQUIRED)

- [ ] `sutra-storage-client-tcp` package includes unit variant fixes
- [ ] Numpy array serialization fixes applied to `vector_search()` method
- [ ] Response parsing fixes for nested list formats
- [ ] Verify: No direct `sutra_storage` imports in distributed services

**Test Fix Applied:**
```bash
# Should work without "wrong msgpack marker" error
docker exec sutra-hybrid python -c "
from sutra_storage_client import StorageClient
client = StorageClient('storage-server:50051')
stats = client.stats()
print('Stats:', stats)
"
```

**Failure Impact:** "wrong msgpack marker" and serialization errors \u2192 All TCP operations fail

---

## Post-Deployment Testing (MANDATORY)

### \ud83d\udd34 Test 1: Basic Connectivity

```bash
# All should return success (200 status)
curl -f http://localhost:8001/ping
curl -f http://localhost:8000/health
curl -f http://localhost:8080/
curl -f http://localhost:9000/
```

### \ud83d\udd34 Test 2: Learning with Embeddings

```bash
curl -X POST http://localhost:8001/sutra/learn \
  -H "Content-Type: application/json" \
  -d '{"text": "Paris is the capital of France"}'
```

**Expected Response:**
```json
{"success": true, "concepts_learned": 1, "associations_created": 0, "message": "Successfully learned 1 concept"}
```

**\u26a0\ufe0f If you get errors here, STOP DEPLOYMENT - system is broken**

### \ud83d\udd34 Test 3: Query with Embeddings (FINAL VERIFICATION)

```bash
curl -X POST http://localhost:8001/sutra/query \
  -H "Content-Type: application/json" \
  -d '{"query": "What is the capital of France?"}'
```

**Expected Response:**
```json
{
  "answer": "Here's what I found: Paris is the capital of France.",
  "confidence": 1.0,
  "confidence_breakdown": {
    "graph_confidence": 1.0,
    "semantic_confidence": 1.0,
    "final_confidence": 1.0
  }
}
```

**\ud83d\udea8 DEPLOYMENT FAILURE INDICATORS:**
- Answer: "I don't have enough knowledge to answer"  
- Confidence: 0.0
- Errors in logs about embeddings or TCP protocol

---

## Emergency Rollback Procedure

If production deployment fails:

### 1. Immediate Diagnostics
```bash
# Check service logs for critical errors
docker logs --tail 20 sutra-hybrid 2>&1 | grep -E "(embedding|TCP|msgpack|numpy)"
docker logs --tail 20 sutra-storage 2>&1 | grep -E "(Client error|Connection|marker)"

# Check Ollama accessibility  
docker exec sutra-hybrid curl -f http://host.docker.internal:11434/api/tags || echo "OLLAMA UNREACHABLE"
```

### 2. Service Restart Sequence
```bash
# Restart in dependency order
docker-compose -f docker-compose-grid.yml restart sutra-storage
sleep 5
docker-compose -f docker-compose-grid.yml restart sutra-hybrid
sleep 5

# Verify fix
curl -X POST http://localhost:8001/sutra/query \
  -H "Content-Type: application/json" \
  -d '{"query": "test query"}' | jq -r '.confidence'
```

### 3. Full Rollback (Last Resort)
```bash
# Stop all services
./sutra-deploy.sh down

# Restore previous working configuration
git checkout <previous-working-commit>

# Redeploy
./sutra-deploy.sh up
```

---

## Sign-off Required

**Deployment Team Lead:** _________________________ Date: _________

**System Architect:** _________________________ Date: _________

**QA Verification:** _________________________ Date: _________

**By signing, you confirm:**
1. All checklist items verified
2. Embedding system working correctly  
3. Production quality answers achieved
4. System ready for live traffic

---

## Historical Context

**Critical Fix Date:** 2025-10-19  
**Issue Resolved:** Complete system failure due to missing embedding configuration  
**Impact:** All queries returned meaningless responses with 0.0 confidence  
**Root Causes:**
1. No Ollama service configured
2. TCP protocol implementation bugs  
3. Numpy array serialization failures
4. Response parsing format mismatches

**Prevention:** This checklist ensures these issues never happen again.
# Sutra AI Production Deployment Checklist

## Overview

This checklist ensures all 10 services in the Sutra AI ecosystem are properly deployed and operational. **ALL items must be verified** before considering deployment complete.

**Last Updated:** 2025-10-20 (Embedding Service Migration)

## Pre-Deployment Verification (MANDATORY)

### 🔴 1. Embedding Service (CRITICAL - SYSTEM WILL NOT WORK WITHOUT THIS)

- [ ] sutra-embedding-service container builds successfully
- [ ] Ubuntu-based Python image (python:3.11-slim)
- [ ] All dependencies in requirements.txt (including einops>=0.6.0)
- [ ] PyTorch CPU version compatible with Ubuntu
- [ ] nomic-ai/nomic-embed-text-v1.5 model accessible via Hugging Face

**Test Build:**
```bash
docker-compose -f docker-compose-grid.yml build sutra-embedding-service
```

**Failure Impact:** Hybrid service will restart continuously → No semantic search capability

---

### 🔴 2. Environment Variables (CRITICAL CONFIGURATION)

- [ ] `SUTRA_EMBEDDING_SERVICE_URL=http://sutra-embedding-service:8888` in hybrid service
- [ ] `VECTOR_DIMENSION=768` in storage server
- [ ] `SUTRA_VECTOR_DIMENSION=768` in hybrid service
- [ ] `SUTRA_STORAGE_SERVER=storage-server:50051` set in all services
- [ ] `SUTRA_STORAGE_MODE=server` set in distributed services

**Verify Configuration:**
```bash
docker-compose -f docker-compose-grid.yml config | grep -E "(EMBEDDING_SERVICE|VECTOR_DIMENSION)"
```

**Failure Impact:** Dimension mismatches → Wrong query results ("tallest mountain" → "Pacific Ocean")

---

### 🔴 3. Service Dependencies (NO LEGACY CONFIGURATIONS)

- [ ] No "temporarily disabled" comments in docker-compose-grid.yml
- [ ] sutra-embedding-service uncommented and active
- [ ] sutra-hybrid depends_on includes sutra-embedding-service
- [ ] All legacy Ollama references removed
- [ ] No fallback embedding configurations present

**Verify Dependencies:**
```bash
grep -E "(# -|temporarily|ollama)" docker-compose-grid.yml || echo "No legacy configs found ✅"
```

**Failure Impact:** Services try to use non-existent dependencies → Connection failures

---

## Deployment Process

### Build Phase
```bash
./sutra-deploy.sh down    # Clean slate
./sutra-deploy.sh build   # Build all images
```

**Validation Checklist:**
- [ ] All 10 services build successfully
- [ ] No build errors reported
- [ ] sutra-embedding-service builds with all dependencies
- [ ] Build artifacts created with correct tags

### Service Startup
```bash
./sutra-deploy.sh up
```

**Initial Status Check:**
- [ ] All 10 containers start without immediate crashes
- [ ] No services show "Restarting" status after 2 minutes
- [ ] Health checks begin showing "health: starting"

---

## Post-Deployment Testing (MANDATORY)

### 🔴 Test 1: Embedding Service Health

```bash
# Wait for service to load model (up to 60 seconds)
curl -s http://localhost:8888/health | jq
```

**Expected Response:**
```json
{
  "status": "healthy",
  "model_loaded": true,
  "dimension": 768,
  "model_name": "nomic-ai/nomic-embed-text-v1.5"
}
```

**🚨 CRITICAL FAILURE INDICATORS:**
- Status: "unhealthy"
- model_loaded: false
- Connection refused errors
- PyTorch compatibility errors in logs

### 🔴 Test 2: Embedding Generation

```bash
curl -X POST http://localhost:8888/embed \
  -H "Content-Type: application/json" \
  -d '{"texts": ["test"], "normalize": true}' | jq '.embeddings[0] | length'
```

**Expected Output:** `768` (embedding dimension)

### 🔴 Test 3: Hybrid Service Connection

```bash
curl -f http://localhost:8001/ping
docker logs sutra-hybrid --tail 10 | grep -i "embedding"
```

**Expected:**
- [ ] Ping returns 200 OK
- [ ] No "PRODUCTION FAILURE" errors about embedding service
- [ ] No "Cannot connect to embedding service" errors
- [ ] Service shows "healthy" status (not restarting)

### 🔴 Test 4: End-to-End Learning with Embeddings

```bash
curl -X POST http://localhost:8001/sutra/learn \
  -H "Content-Type: application/json" \
  -d '{"text": "The Eiffel Tower is in Paris, France."}'
```

**Expected Response:**
```json
{
  "success": true,
  "concepts_learned": 1,
  "message": "Successfully learned 1 concept"
}
```

### 🔴 Test 5: Semantic Query (FINAL VERIFICATION)

```bash
curl -X POST http://localhost:8001/sutra/query \
  -H "Content-Type: application/json" \
  -d '{"query": "Where is the Eiffel Tower?"}'
```

**Expected Response:**
- [ ] Relevant answer mentioning "Paris" or "France"
- [ ] Semantic similarity > 80%
- [ ] Response includes reasoning path
- [ ] No fallback to "I don't have enough knowledge"

**🚨 DEPLOYMENT FAILURE INDICATORS:**
- Answer: "I don't have enough knowledge"
- Similarity scores: 0.0
- Any mention of fallback embeddings (spaCy, TF-IDF)

### 🔴 Test 6: Statistics Validation

```bash
curl -s http://localhost:8000/stats | jq '{total_concepts, total_embeddings, total_associations}'
```

**Expected:**
- [ ] `total_concepts > 0`
- [ ] `total_embeddings > 0` (CRITICAL - must not be zero)
- [ ] `total_embeddings == total_concepts` (all concepts have embeddings)

---

## Complete Service Health Check

### All 10 Services Must Show "Healthy"

```bash
./sutra-deploy.sh status
```

**Required Status for Each:**
- [ ] `sutra-storage`: Up X minutes (healthy)
- [ ] `sutra-grid-events`: Up X minutes (healthy)
- [ ] `sutra-grid-master`: Up X minutes (healthy)
- [ ] `sutra-grid-agent-1`: Up X minutes (healthy)
- [ ] `sutra-grid-agent-2`: Up X minutes (healthy)
- [ ] `sutra-api`: Up X minutes (healthy)
- [ ] `sutra-embedding-service`: Up X minutes (healthy) ⭐ CRITICAL
- [ ] `sutra-hybrid`: Up X minutes (healthy) ⭐ CRITICAL
- [ ] `sutra-control`: Up X minutes (healthy)
- [ ] `sutra-client`: Up X minutes (healthy)

### Service URLs Accessibility

**Test all endpoints:**
- [ ] http://localhost:8000/health (API)
- [ ] http://localhost:8001/ping (Hybrid)
- [ ] http://localhost:8080/ (Client UI)
- [ ] http://localhost:8888/health (Embedding Service) ⭐ CRITICAL
- [ ] http://localhost:9000/health (Control Center)
- [ ] http://localhost:7001/health (Grid Master)

---

## Production Smoke Test

```bash
./scripts/smoke-test-embeddings.sh
```

**All tests must pass:**
- [ ] ✅ Embedding service health check
- [ ] ✅ Model dimension verification (768)
- [ ] ✅ End-to-end semantic search
- [ ] ✅ No fallback embedding warnings
- [ ] ✅ All dependent services operational

---

## Emergency Rollback Procedure

If any critical test fails:

### 1. Immediate Diagnostics
```bash
# Check embedding service logs
docker logs sutra-embedding-service --tail 50

# Check hybrid service connectivity
docker logs sutra-hybrid --tail 20 | grep -E "(embedding|PRODUCTION)"

# Verify embedding service accessibility
docker exec sutra-hybrid curl -f http://sutra-embedding-service:8888/health
```

### 2. Service Recovery Attempt
```bash
# Restart embedding service first
docker-compose -f docker-compose-grid.yml restart sutra-embedding-service
sleep 60  # Wait for model loading

# Restart dependent services
docker-compose -f docker-compose-grid.yml restart sutra-hybrid
sleep 10

# Verify fix
curl -s http://localhost:8888/health | jq '.status'
```

### 3. Full Rollback (Critical Failure)
```bash
# Complete shutdown
./sutra-deploy.sh down
./sutra-deploy.sh clean

# Restore previous working state
git checkout <previous-working-commit>

# Redeploy
./sutra-deploy.sh up

# Verify restoration
./scripts/smoke-test-embeddings.sh
```

---

## Sign-off Requirements

**Technical Lead Sign-off:**
- [ ] All 10 services healthy and operational
- [ ] Embedding service fully functional (768-d, nomic-embed-text-v1.5)
- [ ] End-to-end learning and query pipelines working
- [ ] No legacy configurations remaining
- [ ] Production smoke test passed

**Operations Sign-off:**
- [ ] Resource usage within limits (embedding service < 4GB)
- [ ] All health checks responding correctly
- [ ] Monitoring and alerting configured
- [ ] Rollback procedures verified

**Quality Assurance Sign-off:**
- [ ] Semantic search accuracy > 80%
- [ ] No "same answer for all questions" bug
- [ ] All user interfaces accessible and functional
- [ ] Performance benchmarks met

---

**Deployment Completion Status:** ❌ Not Complete

**Technical Lead:** ________________  **Date:** ________________  
**Operations:** ________________     **Date:** ________________  
**QA:** ________________             **Date:** ________________  

---

## Migration Notes (2025-10-20)

**From Ollama to Dedicated Embedding Service:**
- ✅ Removed all Ollama dependencies
- ✅ Implemented dedicated sutra-embedding-service
- ✅ Fixed PyTorch compatibility with Ubuntu base
- ✅ Added all required dependencies (einops, huggingface_hub)
- ✅ Configured proper service dependencies
- ✅ Updated health checks and monitoring

**Critical Changes Made:**
1. Docker base image: Alpine → Ubuntu (PyTorch compatibility)
2. Model loading: Local path → Hugging Face direct download
3. Dependencies: Added einops, huggingface_hub
4. Service architecture: Ollama integration → Dedicated service
5. Health checks: Extended timeout for model loading (60s)

**No Rollback Available:** Ollama integration completely removed and not supported.
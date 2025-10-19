# Embedding System Troubleshooting Guide

## 🚨 CRITICAL: Embedding Requirements for Production

**The Sutra AI system is COMPLETELY NON-FUNCTIONAL without proper embedding configuration.**

This document covers the critical fixes applied on 2025-10-19 to resolve production-breaking embedding issues.

---

## Common Error Messages and Fixes

### 1. "No embedding processor available"

**Error Context:**
```
RuntimeError: No embedding processor available. 
Vector search requires embedding_processor or nlp_processor.
```

**Root Cause:** ReasoningEngine cannot generate embeddings for queries.

**Fix Checklist:**
- [ ] Ollama service running and accessible
- [ ] `SUTRA_OLLAMA_URL` environment variable set correctly
- [ ] `granite-embedding:30m` model loaded in Ollama
- [ ] OllamaNLPProcessor properly injected into ReasoningEngine

**Verification Commands:**
```bash
# Test Ollama connectivity
curl http://localhost:11434/api/tags

# Test from Docker container
docker exec sutra-hybrid curl http://host.docker.internal:11434/api/tags

# Check environment variable
docker exec sutra-hybrid env | grep OLLAMA
```

---

### 2. "can not serialize 'numpy.ndarray' object"

**Error Context:**
```
TypeError: can not serialize 'numpy.ndarray' object
```

**Root Cause:** TCP client trying to serialize numpy arrays with msgpack.

**Fix:** Always convert numpy arrays to Python lists before TCP transport.

**Code Fix:**
```python
# In sutra-storage-client-tcp package
def vector_search(self, query_vector: List[float], k: int = 10):
    # Convert numpy array to list if needed
    if hasattr(query_vector, 'tolist'):
        query_vector = query_vector.tolist()
    elif hasattr(query_vector, '__iter__'):
        query_vector = [float(x) for x in query_vector]
```

---

### 3. "wrong msgpack marker FixMap(0)"

**Error Context:**
```
Client error: wrong msgpack marker FixMap(0)
```

**Root Cause:** Sending `{"GetStats": {}}` instead of `"GetStats"` for unit variants.

**Fix:** Unit variants (operations with no parameters) must send just the string.

**Code Fix:**
```python
# WRONG - causes msgpack parsing error
def stats(self):
    response = self._send_request("GetStats", {})

# CORRECT - works with Rust enum parser
def stats(self):
    request = "GetStats"
    packed = msgpack.packb(request)
    # ... send packed request directly
```

**Affected Operations:**
- `GetStats`
- `Flush` 
- `HealthCheck`

---

### 4. "Same Answer for All Questions" (Zero Embeddings)

**Status:** ✅ **FIXED** (2025-10-19) - Unified learning architecture prevents this

**Symptoms:**
```
Every query returns the same answer, regardless of the question
Vector search appears to work but returns wrong concepts
Storage stats show: "total_embeddings": 0
```

**Historical Root Cause (Pre-2025-10-19):** 
Concepts were learned without embeddings because only the Hybrid service generated embeddings. API and Bulk Ingester services learned concepts directly without embeddings, causing semantic search to fail.

**Current Architecture (Post-2025-10-19):**
- ✅ Storage server owns the complete learning pipeline
- ✅ Embedding generation happens automatically for ALL services
- ✅ API, Hybrid, Bulk Ingester all use the same unified learning path
- ✅ Bug cannot occur unless Ollama is misconfigured

**Diagnosis:**
```bash
# Check embedding count
curl -s http://localhost:8000/stats | jq '.total_embeddings'
# Expected: > 0 (should match concept count)
# If 0: System is non-functional

# Check if Ollama is working
curl -s http://localhost:8001/health | jq '.embedding_model'
# Expected: "granite-embedding:30m" or similar
# If null/none: Embeddings disabled
```

**Fix:**

1. **Clean old data without embeddings:**
```bash
# Stop services
docker stop sutra-storage sutra-api sutra-hybrid sutra-client

# Remove storage volume
docker rm -f sutra-storage
docker volume rm sutra-models_storage-data

# Restart with fresh storage
docker-compose -f docker-compose-grid.yml up -d storage-server sutra-api sutra-hybrid sutra-client
```

2. **Learn via Hybrid service (has embeddings):**
```bash
# Use /sutra/learn endpoint (NOT /learn)
curl -X POST http://localhost:8001/sutra/learn \
  -H "Content-Type: application/json" \
  -d '{"text": "The Eiffel Tower is in Paris, France"}'

# Learn multiple diverse facts
curl -X POST http://localhost:8001/sutra/learn \
  -H "Content-Type: application/json" \
  -d '{"text": "Tokyo is the capital of Japan"}'
```

3. **Verify embeddings generated:**
```bash
curl -s http://localhost:8000/stats | jq '{concepts: .total_concepts, embeddings: .total_embeddings}'
# Expected: concepts == embeddings
```

4. **Test differentiation:**
```bash
# Query 1
curl -X POST http://localhost:8001/sutra/query \
  -H "Content-Type: application/json" \
  -d '{"query": "Where is the Eiffel Tower?"}' | jq -r '.answer'

# Query 2
curl -X POST http://localhost:8001/sutra/query \
  -H "Content-Type: application/json" \
  -d '{"query": "What is Tokyo?"}' | jq -r '.answer'

# Should return DIFFERENT answers
```

**Prevention (Post-2025-10-19):**
- ✅ Use ANY learning endpoint - all generate embeddings automatically
- ✅ Storage server enforces embedding generation for consistency
- ✅ Monitor `docker logs sutra-storage` for "LearningPipeline" logs
- ✅ Ensure Ollama service is running and accessible to storage server
- ✅ Verify `SUTRA_OLLAMA_URL` environment variable is set correctly

**Key Architectural Change:**
With unified learning (2025-10-19), the storage server is the single source of truth for learning operations. All services delegate to the storage server's learning pipeline, which guarantees embeddings are generated. The old "learn via Hybrid only" workaround is no longer needed.

**Key Lesson:** Without embeddings, the system degrades to random concept retrieval. Embeddings are mandatory for semantic differentiation. The new architecture guarantees this at the storage layer.

**See:** [`docs/UNIFIED_LEARNING_ARCHITECTURE.md`](UNIFIED_LEARNING_ARCHITECTURE.md) for complete design documentation.

---

### 5. "list indices must be integers or slices, not str"

**Error Context:**
```
TypeError: list indices must be integers or slices, not str
```

**Root Cause:** Storage server returns nested lists, not dictionaries.

**Response Formats:**
- **VectorSearch:** `[[['concept_id', score], ['concept_id', score]]]`
- **QueryConcept:** `[found, concept_id, content, strength, confidence]`

**Code Fix:**
```python
# Vector search response parsing
if "VectorSearchOk" in response:
    data = response["VectorSearchOk"]
    if isinstance(data, list) and len(data) > 0 and isinstance(data[0], list):
        results = data[0]  # Unwrap one level
        return [(r[0], r[1]) for r in results if len(r) >= 2]

# Query concept response parsing  
if "QueryConceptOk" in response:
    result = response["QueryConceptOk"]
    if isinstance(result, list) and len(result) >= 5 and result[0]:
        return {
            "id": result[1],
            "content": result[2], 
            "strength": result[3],
            "confidence": result[4],
        }
```

---

## Architecture Compliance Rules

### ✅ CORRECT Architecture

```python
# In hybrid service or any distributed service
from sutra_storage_client import StorageClient

client = StorageClient('storage-server:50051')
result = client.learn_concept(concept_id, content, embedding)
```

### ❌ WRONG Architecture 

```python
# NEVER do this in distributed services
from sutra_storage import RustStorageAdapter

storage = RustStorageAdapter(path)  # Bypasses TCP protocol
```

---

## Production Deployment Checklist

### Pre-Deployment Verification

- [ ] **Ollama Service**
  - [ ] Running on host at port 11434
  - [ ] `granite-embedding:30m` model loaded
  - [ ] Accessible from Docker containers via `host.docker.internal:11434`

- [ ] **Environment Variables**
  - [ ] `SUTRA_OLLAMA_URL=http://host.docker.internal:11434` in hybrid service
  - [ ] `SUTRA_STORAGE_SERVER=storage-server:50051` in all services

- [ ] **Package Versions**
  - [ ] `sutra-storage-client-tcp` with fixed unit variant handling
  - [ ] Numpy array serialization fixes applied
  - [ ] Response parsing fixes for nested lists

- [ ] **Service Communication**
  - [ ] All services use `sutra-storage-client-tcp` package
  - [ ] No direct `sutra_storage` imports in distributed services
  - [ ] TCP binary protocol working correctly

### Post-Deployment Testing

```bash
# Test learning with embeddings
curl -X POST http://localhost:8001/sutra/learn \
  -H "Content-Type: application/json" \
  -d '{"text": "Paris is the capital of France"}'

# Expected: {"success": true, "concepts_learned": 1, ...}

# Test query with embeddings  
curl -X POST http://localhost:8001/sutra/query \
  -H "Content-Type: application/json" \
  -d '{"query": "What is the capital of France?"}'

# Expected: High confidence answer with reasoning paths
```

---

## Emergency Recovery

If embeddings fail in production:

1. **Check Ollama Status:**
   ```bash
   docker exec sutra-hybrid curl -f http://host.docker.internal:11434/api/tags
   ```

2. **Restart Services in Order:**
   ```bash
   docker-compose -f docker-compose-grid.yml restart sutra-storage
   docker-compose -f docker-compose-grid.yml restart sutra-hybrid
   ```

3. **Monitor Logs:**
   ```bash
   docker logs --tail 20 sutra-hybrid
   docker logs --tail 20 sutra-storage
   ```

4. **Verify Fix:**
   ```bash
   curl -X POST http://localhost:8001/sutra/query \
     -H "Content-Type: application/json" \
     -d '{"query": "test"}' | jq -r '.confidence'
   ```

---

## Historical Context

**Date:** 2025-10-19  
**Issue:** Complete system failure due to missing embedding configuration  
**Impact:** All queries returned "I don't have enough knowledge to answer"  
**Resolution:** Fixed TCP protocol issues, embedding integration, and response parsing  

**Key Lesson:** Embeddings are not optional - they are a core requirement for the system to function.
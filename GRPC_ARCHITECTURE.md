# Sutra AI - Lightweight gRPC Architecture

## Current Problem

**API**: 612MB, **Hybrid**: 608MB - **TOO HEAVY!**

These services include the full reasoning stack (graph algorithms, HNSW, ML libraries) but with gRPC architecture, they should be **thin client proxies**.

## Optimal Architecture

```
┌─────────────┐
│   Client    │ (77MB - React/nginx) ✓ GOOD
│  (Browser)  │
└──────┬──────┘
       │ HTTP/REST
       ▼
┌─────────────┐
│     API     │ (Target: 50-80MB)
│  FastAPI    │ - Just REST → gRPC proxy
│   (Python)  │ - NO reasoning engine
└──────┬──────┘ - NO graph algorithms
       │ gRPC   - NO ML libraries
       ▼
┌─────────────┐
│   Storage   │ (24MB - Rust) ✓ PERFECT
│   Server    │ - ALL graph operations
│   (Rust)    │ - Path finding, HNSW
└─────────────┘ - Concurrent storage
```

## What Each Service Should Contain

### Storage Server (Rust) - 24MB ✓
**Role**: Centralized compute engine
- Graph data structure & algorithms
- Path finding (BFS, bidirectional)
- HNSW vector search
- Concurrent storage with lock-free writes
- gRPC server

**Dependencies**: All in Rust (compiled binary)

---

### API Service (Python) - **Target: 50-80MB**
**Role**: REST-to-gRPC proxy  

**What it NEEDS:**
```python
fastapi>=0.104.0          # Web framework
uvicorn>=0.24.0           # ASGI server
pydantic>=2.0.0           # Request/response models
sutra-storage-client      # gRPC client (~5MB)
typing_extensions         # Pydantic dependency
```

**What it DOES NOT NEED:**
- ❌ sutra-core (reasoning engine, graph logic)
- ❌ sutra-hybrid (embeddings, ML)
- ❌ numpy, scikit-learn
- ❌ hnswlib (vector search - that's on server!)

**Endpoints are thin proxies:**
```python
@app.post("/learn")
async def learn(request: LearnRequest, client: StorageClient):
    # Just forward to storage server via gRPC
    result = client.learn_concept(
        request.content,
        strength=request.strength
    )
    return {"concept_id": result.id, "status": "learned"}
```

---

### Hybrid Service (Python) - **Target: 200-300MB**
**Role**: Embedding generation + gRPC forwarding

**What it NEEDS:**
```python
fastapi>=0.104.0
uvicorn>=0.24.0
pydantic>=2.0.0
sutra-storage-client
# Only for embeddings:
sentence-transformers     # ~200MB with models
torch                     # If using neural embeddings
```

**What it DOES NOT NEED:**
- ❌ Full sutra-core
- ❌ hnswlib (server does vector search)
- ❌ Graph algorithms
- ❌ Path finding logic

**Flow:**
1. Generate embeddings locally (if needed)
2. Send to storage server via gRPC
3. Server handles vector search, graph ops

---

### Control Service - 137MB (Can optimize to ~60MB)
**Role**: Management UI + monitoring

**What it NEEDS:**
```python
fastapi
uvicorn
jinja2 templates
websockets (for real-time updates)
sutra-storage-client
```

---

## Migration Plan

### Phase 1: Create Minimal Data Models Package
Extract just data classes from sutra-core:
```python
# sutra-core-models (~1MB)
- Concept
- Association  
- AssociationType enum
- Request/Response models
```

### Phase 2: Rewrite API as Thin Proxy
```python
# New API structure:
sutra-api/
  sutra_api/
    main.py           # FastAPI app
    dependencies.py   # gRPC client injection
    models.py         # Pydantic models
    endpoints/
      learn.py        # POST /learn → gRPC
      query.py        # POST /query → gRPC
      search.py       # POST /search → gRPC
```

### Phase 3: Strip Hybrid to Embeddings Only
If embeddings are generated client-side, keep sentence-transformers.
If server-side, hybrid can be as light as API!

### Phase 4: Rebuild Dockerfiles
```dockerfile
# API Dockerfile (target: 50-80MB)
FROM python:3.11-alpine  # Not slim - alpine!
RUN pip install fastapi uvicorn pydantic
COPY sutra-storage-client ./
RUN pip install ./sutra-storage-client
COPY sutra_api ./sutra_api
```

## Expected Size Reductions

| Service | Current | Target | Savings |
|---------|---------|--------|---------|
| Storage | 24MB    | 24MB   | 0%      |
| API     | 612MB   | 60MB   | **90%** |
| Hybrid  | 608MB   | 250MB  | **59%** |
| Control | 137MB   | 60MB   | **56%** |
| Client  | 77MB    | 77MB   | 0%      |
| **TOTAL** | **1458MB** | **471MB** | **68%** |

## Key Principles

1. **Separation of Concerns**
   - Storage Server: ALL compute
   - API/Hybrid: ONLY I/O translation

2. **No Duplicate Logic**
   - Graph algorithms live ONLY in Rust server
   - Python services don't replicate this

3. **Thin Clients**
   - Python services are stateless proxies
   - All state in centralized storage server

4. **Minimal Dependencies**
   - Only what's needed for HTTP → gRPC translation
   - No "just in case" libraries

## Implementation Checklist

- [ ] Create sutra-core-models (data classes only)
- [ ] Rewrite API to use only storage-client
- [ ] Strip Hybrid to embeddings + storage-client  
- [ ] Update Dockerfiles (use alpine where possible)
- [ ] Remove unused dependencies from requirements
- [ ] Test full stack with thin clients
- [ ] Measure final image sizes
- [ ] Document deployment gains

## Benefits

1. **92% smaller Python services** (612MB → 50MB for API)
2. **Faster builds** (less to compile)
3. **Faster deployments** (smaller images)
4. **Lower memory** (thin proxies use minimal RAM)
5. **Clearer architecture** (single source of truth)
6. **Easier maintenance** (logic in one place)

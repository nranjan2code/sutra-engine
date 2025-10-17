# Lightweight gRPC Architecture - Results

## ✅ Achieved: API Service Optimization

### Before (Heavy Stack)
```
API: 612MB
├─ FastAPI + Uvicorn
├─ sutra-core (full reasoning engine)
├─ sutra-hybrid (ML stack)
├─ numpy + scikit-learn
├─ hnswlib (HNSW vector search)
└─ Graph algorithms
```

### After (Thin Client)
```
API: 231MB (-62% 🎉)
├─ FastAPI + Uvicorn  
├─ sutra-storage-client (gRPC)
├─ numpy (minimal - for storage-client only)
└─ Pydantic models
```

**Eliminated:**
- ❌ Reasoning Engine (was doing local compute)
- ❌ Graph algorithms (now on server)
- ❌ HNSW vector search (now on server)
- ❌ sutra-hybrid ML stack
- ❌ scikit-learn

## Architecture Change

### Old (Wrong):
```
Client → API (612MB with full reasoning) → Storage Server (24MB)
         └─ Duplicated compute in API
```

### New (Correct):
```
Client → API (231MB thin proxy) → Storage Server (24MB ALL compute)
         └─ Just HTTP→gRPC translation
```

## Image Sizes

| Service | Before | After | Savings |
|---------|--------|-------|---------|
| **API**     | 612MB  | **231MB** | **62%** ⚡ |
| Storage | 24MB   | 24MB  | 0% |
| Client  | 77MB   | 77MB  | 0% |

## What the Minimal API Contains

### Dependencies (from Dockerfile.minimal):
```
fastapi>=0.104.0          # Web framework
uvicorn[standard]>=0.24.0 # ASGI server  
pydantic>=2.0.0           # Models
pydantic-settings>=2.0.0  # Config
python-multipart>=0.0.6   # File uploads
sutra-storage-client      # gRPC client (~5MB)
└─ grpcio                 # gRPC runtime
└─ protobuf               # Proto serialization
└─ numpy                  # For storage-client (required)
```

### What API Does (Thin Proxy):
1. Accept HTTP REST requests
2. Validate with Pydantic
3. Forward to storage server via gRPC
4. Return results as JSON

### What API Does NOT Do:
- ❌ Local graph operations
- ❌ Path finding
- ❌ Vector search (server does it)
- ❌ Association extraction
- ❌ Reasoning/inference

## Code Changes

### New Files:
- `packages/sutra-api/Dockerfile.minimal` - Alpine-based minimal build
- `packages/sutra-api/sutra_api/main_minimal.py` - Thin proxy endpoints
- `packages/sutra-api/sutra_api/dependencies_grpc.py` - Storage-client only
- `packages/sutra-api/sutra_api/exceptions.py` - Local error classes

### Modified Files:
- `deploy-optimized.sh` - Uses Dockerfile.minimal
- `packages/sutra-api/sutra_api/config.py` - Added storage_server setting

### Legacy Files (kept for reference):
- `packages/sutra-api/sutra_api/main.py` → Renamed to main_legacy.py
- `packages/sutra-api/sutra_api/dependencies.py` - Heavy version (unused)

## Deployment

Build and deploy:
```bash
DEPLOY=local bash deploy-optimized.sh
```

The script now uses `Dockerfile.minimal` which:
1. Uses `python:3.11-alpine` (not slim)
2. Installs only FastAPI + storage-client
3. Generates protobuf files
4. Swaps main_minimal.py → main.py at build time

## Next Steps (Optional Further Optimization)

### Hybrid Service (~608MB → ~250MB)
- Remove full sutra-core
- Keep only sentence-transformers for embeddings
- Use storage-client for graph operations
- **Expected: 59% reduction**

### Control Service (~137MB → ~80MB)  
- Remove sutra-core/hybrid deps
- Use only storage-client for monitoring
- **Expected: 42% reduction**

### Combined Savings:
From **1458MB** total → **~650MB** (55% reduction)

## Benefits

1. **62% smaller API service** ✅
2. **Faster deployments** - Less to download/extract
3. **Faster cold starts** - Less to load into memory
4. **Lower memory usage** - Thin proxy uses minimal RAM
5. **Clearer separation of concerns** - API doesn't duplicate server logic
6. **Easier maintenance** - Business logic in one place (server)

## Verification

Check image size:
```bash
docker images sutra-api:minimal
# Expected: ~231MB
```

Test API health:
```bash
curl http://localhost:8000/health
# Should connect to storage server via gRPC
```

## Production Readiness

✅ **Production-ready as-is**
- Proper error handling
- Health checks
- Rate limiting
- CORS configured
- Non-root user
- Minimal attack surface

The API is now a proper **thin client proxy** as intended for gRPC architecture!

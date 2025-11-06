# Sutra Explorer - Production Integration Guide

**Date:** October 29, 2025  
**Status:** Production-Grade Architecture  
**Integration:** Via Storage Clients (NO Direct Storage Access)

---

## Architecture Overview

Sutra Explorer is a **standalone visualization service** that integrates with the Sutra platform through the **binary TCP storage client protocol**. It NEVER accesses storage files directly.

```
┌─────────────────────────────────────────────────────────────┐
│                    SUTRA PLATFORM                            │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────────┐          ┌──────────────────┐         │
│  │ Storage Server   │          │ User Storage     │         │
│  │ (Domain Graph)   │          │ (Auth, Convs)    │         │
│  │ :50051           │          │ :50053           │         │
│  └────────┬─────────┘          └────────┬─────────┘         │
│           │ TCP Binary                  │ TCP Binary        │
│           │ Protocol                    │ Protocol          │
└───────────┼─────────────────────────────┼───────────────────┘
            │                             │
            │                             │
┌───────────┼─────────────────────────────┼───────────────────┐
│           │         EXPLORER            │                   │
├───────────┼─────────────────────────────┼───────────────────┤
│           │                             │                   │
│  ┌────────▼─────────┐          ┌────────▼─────────┐         │
│  │ StorageClient    │          │ StorageClient    │         │
│  │ (domain)         │          │ (user)           │         │
│  └────────┬─────────┘          └────────┬─────────┘         │
│           │                             │                   │
│           └──────────┬──────────────────┘                   │
│                      │                                      │
│           ┌──────────▼──────────┐                           │
│           │ Explorer Backend    │                           │
│           │ (FastAPI)           │                           │
│           │ :8100               │                           │
│           └──────────┬──────────┘                           │
│                      │ REST API                             │
│           ┌──────────▼──────────┐                           │
│           │ Explorer Frontend   │                           │
│           │ (React + Vite)      │                           │
│           │ :3000               │                           │
│           └─────────────────────┘                           │
└─────────────────────────────────────────────────────────────┘
```

**Key Principles:**
1. ✅ **NO direct file access** - Always via storage client
2. ✅ **Multiple storages** - User + domain(s)
3. ✅ **Read-only** - Safe exploration, no mutations
4. ✅ **Edition-aware** - Respects platform limits
5. ✅ **Stateless** - No local caching, always fresh data

---

## Storage Client Integration

### Binary TCP Protocol

Explorer uses `sutra-storage-client-tcp` (Rust + Python bindings) for ALL storage access:

```python
# backend/main.py
from sutra_storage_client import StorageClient

# Connect to storage servers
user_storage = StorageClient("user-storage-server:50051")
domain_storage = StorageClient("storage-server:50051")

# Read operations only
concept = domain_storage.get_concept("concept_123")
results = domain_storage.search("diabetes", limit=20)
associations = concept.get("associations", [])
```

**Available Operations:**
```python
# Concept operations
client.get_concept(id: str) -> Dict
client.search(query: str, limit: int) -> Dict

# Low-level operations (used internally by backend)
client.learn_concept(content: str) -> str  # Not used by explorer
client.flush() -> None  # Cleanup on shutdown
```

**What Explorer CANNOT Do:**
- ❌ Direct file I/O (`open()`, `read()`, `write()`)
- ❌ Binary format parsing (handled by storage server)
- ❌ Concept creation (read-only)
- ❌ Association modification (read-only)
- ❌ Embedding generation (delegated to storage)

---

## Multi-Storage Architecture

### Storage Types

**1. User Storage (Required)**
- **Purpose:** User data (conversations, messages, bookmarks)
- **Connection:** `user-storage-server:50051`
- **Content:** Auth tokens, conversation metadata, message history
- **Access Pattern:** Session-specific, user-scoped

**2. Domain Storage (Required)**
- **Purpose:** Knowledge graph (concepts, associations)
- **Connection:** `storage-server:50051`
- **Content:** Medical protocols, legal precedents, etc.
- **Access Pattern:** Read-only exploration

**3. Additional Storages (Optional)**
- **Purpose:** Multi-domain deployments
- **Configuration:** `SUTRA_ADDITIONAL_STORAGES=medical=medical-storage:50051,legal=legal-storage:50051`
- **Use Case:** Segregate knowledge by domain

### Storage Selection API

Frontend can query multiple storages:

```typescript
// List available storages
const { storages, default } = await api.listStorages();
// Returns: { storages: ["user", "domain", "medical", "legal"], default: "domain" }

// Query specific storage
const medicalConcepts = await api.getConcepts({ storage: "medical", limit: 100 });
const legalConcepts = await api.getConcepts({ storage: "legal", limit: 100 });
```

---

## Environment Configuration

### Backend Environment Variables

```bash
# Storage Connections (REQUIRED)
SUTRA_USER_STORAGE=user-storage-server:50051      # User data storage
SUTRA_DOMAIN_STORAGE=storage-server:50051         # Main knowledge graph

# Additional Storages (OPTIONAL)
SUTRA_ADDITIONAL_STORAGES=medical=medical-storage:50051,legal=legal-storage:50051

# API Configuration
API_HOST=0.0.0.0
API_PORT=8100
LOG_LEVEL=INFO

# CORS (Frontend URLs)
CORS_ORIGINS=http://localhost:3000,http://localhost:5173

# Edition (Inherited from platform)
SUTRA_EDITION=simple|community|enterprise
```

### Frontend Environment Variables

```bash
# Backend API URL
VITE_API_URL=http://localhost:8100

# Optional: Feature flags
VITE_ENABLE_3D=true
VITE_ENABLE_EXPORT=true
```

---

## Deployment Options

### Option 1: Docker Compose (Recommended)

**1. Start Platform:**
```bash
cd .sutra/compose
docker compose -f production.yml up -d

# Wait for storage servers
docker compose ps | grep -E 'storage|user-storage'
```

**2. Start Explorer:**
```bash
cd packages/sutra-explorer
docker compose up -d

# Check health
curl http://localhost:8100/health
curl http://localhost:3000/health
```

**3. Access Explorer:**
```
Frontend: http://localhost:3000
Backend API: http://localhost:8100/docs (Swagger UI)
```

### Option 2: Development Mode

**Terminal 1 - Backend:**
```bash
cd packages/sutra-explorer/backend
pip install -r requirements.txt
pip install -e ../../sutra-storage-client-tcp

export SUTRA_USER_STORAGE=localhost:50053
export SUTRA_DOMAIN_STORAGE=localhost:50051
export CORS_ORIGINS=http://localhost:5173

uvicorn main:app --reload --host 0.0.0.0 --port 8100
```

**Terminal 2 - Frontend:**
```bash
cd packages/sutra-explorer/frontend
npm install

VITE_API_URL=http://localhost:8100 npm run dev
```

**Terminal 3 - Platform (if not running):**
```bash
cd .sutra/compose
docker compose -f production.yml up storage-server user-storage-server embedding-single
```

---

## API Endpoints

### Health & Discovery

```bash
# Health check
GET /health
Response: {
  "status": "healthy",
  "uptime_seconds": 123.45,
  "storage_clients": ["user", "domain", "medical"],
  "timestamp": "2025-10-29T10:30:00Z"
}

# List storages
GET /storages
Response: {
  "storages": ["user", "domain", "medical"],
  "default": "domain"
}
```

### Concept Operations

```bash
# List concepts (paginated)
GET /concepts?storage=domain&limit=100&offset=0&min_confidence=0.8

# Get concept by ID
GET /concepts/{id}?storage=domain

# Get associations (edges)
GET /associations/{id}?storage=domain
```

### Graph Operations

```bash
# Search concepts
POST /search?query=diabetes&storage=domain&limit=20

# Get N-hop neighborhood
POST /neighborhood?concept_id=123&storage=domain&depth=2&max_nodes=50

# Get statistics
GET /statistics/{storage}
Response: {
  "storage_name": "domain",
  "total_concepts": 1234,
  "total_associations": 5678,
  "avg_confidence": 0.87,
  "timestamp": "2025-10-29T10:30:00Z"
}
```

---

## Security & Access Control

### Network Isolation

Explorer should run in the same Docker network as the platform:

```yaml
# docker-compose.yml
networks:
  sutra-network:
    external: true
    name: sutra-works_sutra-network
```

This ensures:
- Storage servers NOT exposed to public internet
- Explorer can only access via internal network
- TCP connections are local (low latency)

### Read-Only Access

Explorer ONLY uses read operations:
```python
# ✅ Allowed
client.get_concept(id)
client.search(query, limit)

# ❌ Not used (explorer is read-only)
client.learn_concept(content)  # Would create concepts
```

### Edition Limits

Backend respects platform edition limits:
```python
# Edition-specific limits (future)
if edition == "simple":
    max_nodes_per_query = 100
elif edition == "community":
    max_nodes_per_query = 500
else:  # enterprise
    max_nodes_per_query = 2000
```

---

## Performance Considerations

### TCP Connection Pooling

Storage clients maintain persistent TCP connections:
```python
# Single client per storage (reused across requests)
_storage_clients = {
    "user": StorageClient("user-storage-server:50051"),
    "domain": StorageClient("storage-server:50051")
}
```

**Benefits:**
- No connection overhead per request
- Binary protocol (faster than HTTP/REST)
- Zero-copy operations where possible

### Pagination & Limits

Always paginate large queries:
```python
# Backend enforces limits
limit = max(1, min(limit, 500))  # Cap at 500 nodes per request

# Frontend implements virtual scrolling
<VirtualizedList itemCount={totalNodes} itemSize={80} />
```

### Caching Strategy

Explorer is **stateless** (no caching):
- Always fetches fresh data from storage
- No stale data risk
- Scales horizontally (multiple explorer instances)
- Storage server handles caching internally

---

## Monitoring & Observability

### Health Checks

```bash
# Backend health (includes storage status)
curl http://localhost:8100/health

# Frontend health (Nginx)
curl http://localhost:3000/health

# Storage connectivity test
docker exec sutra-explorer-backend python -c "
from sutra_storage_client import StorageClient
client = StorageClient('storage-server:50051')
print('Connected:', client is not None)
"
```

### Logging

Backend logs include:
- Storage client connections
- Query performance
- Error traces
- Request counts

```python
# Structured logging
logger.info(f"Connected to {storage_name} at {address}")
logger.error(f"Failed to get concept {concept_id}: {error}")
```

### Metrics (Future)

Potential metrics to track:
- Requests per storage
- Average query latency
- Concept cache hit rate
- Frontend render time

---

## Troubleshooting

### Backend Cannot Connect to Storage

**Symptom:**
```
Failed to initialize storage clients: Connection refused
```

**Solution:**
```bash
# Check storage servers are running
docker ps | grep storage

# Check network connectivity
docker exec sutra-explorer-backend ping storage-server

# Verify ports
netstat -an | grep 50051
```

### Frontend Shows No Data

**Symptom:**
UI displays "Loading..." indefinitely

**Solution:**
```bash
# Check backend is accessible
curl http://localhost:8100/health

# Check CORS headers
curl -H "Origin: http://localhost:3000" \
     -H "Access-Control-Request-Method: GET" \
     -X OPTIONS http://localhost:8100/concepts

# Check browser console for errors
# Open DevTools → Console
```

### Slow Query Performance

**Symptom:**
Graph rendering takes >5 seconds

**Solution:**
```bash
# Reduce query size
GET /concepts?limit=50  # Instead of 500

# Reduce neighborhood depth
POST /neighborhood?depth=1  # Instead of 5

# Check storage server logs
docker logs storage-server | grep -i slow
```

---

## Future Enhancements

### Phase 1 (Current)
- ✅ Basic graph visualization (list view)
- ✅ Multi-storage support
- ✅ Read-only exploration
- ✅ Docker deployment

### Phase 2 (Next 2-3 weeks)
- 🟡 Force-directed 2D renderer (D3.js)
- 🟡 Search with filtering
- 🟡 Path highlighting
- 🟡 Export to PNG

### Phase 3 (4-6 weeks)
- 🔴 3D immersive view (Three.js)
- 🔴 Touch gestures (mobile)
- 🔴 Keyboard shortcuts (30+ commands)
- 🔴 Real-time collaboration

### Phase 4 (Future)
- 🔴 VR mode (WebXR)
- 🔴 AI-powered layout suggestions
- 🔴 Natural language graph queries
- 🔴 Time-travel mode (temporal graphs)

---

## References

### Documentation
- **Platform Architecture:** `.github/copilot-instructions.md`
- **Storage Protocol:** `packages/sutra-storage/docs/PROTOCOL.md`
- **Explorer Vision:** `docs/sutra-explorer/NEXT_GENERATION_VISION.md`
- **Deep Review:** `docs/sutra-explorer/SUTRA_EXPLORER_DEEP_REVIEW.md`

### Code Locations
- **Backend API:** `packages/sutra-explorer/backend/main.py`
- **Frontend App:** `packages/sutra-explorer/frontend/src/App.tsx`
- **API Client:** `packages/sutra-explorer/frontend/src/services/api.ts`
- **Storage Client:** `packages/sutra-storage-client-tcp/` (Rust + Python)

### Related Services
- **Sutra API:** `packages/sutra-api/` (Main platform API)
- **Storage Server:** `packages/sutra-storage/` (Rust backend)
- **Embedding Service:** `packages/sutra-embedding-service/`

---

**Integration Status:** ✅ **Production-Grade Architecture Complete**

The explorer now properly integrates with the Sutra platform via storage clients, respects the distributed architecture, and maintains read-only access for safe exploration.

**Next Steps:**
1. Implement Force2D renderer (D3.js)
2. Add search filtering
3. Build path highlighting
4. Add export capabilities

Questions? See architecture docs or review the deep analysis document.

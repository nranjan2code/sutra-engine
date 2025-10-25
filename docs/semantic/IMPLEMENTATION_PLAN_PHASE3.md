# Phase 3 Implementation Plan: Semantic Query Integration

**Date:** 2025-01-26  
**Status:** PLANNING  
**Risk Level:** MEDIUM (Touching production chain)

---

## Architecture Chain (CRITICAL - DO NOT BREAK)

```
┌─────────────────────────────────────────────────────────────────────┐
│                         USER INTERFACES                              │
├─────────────────────────────────────────────────────────────────────┤
│  sutra-control (Admin React)      sutra-client (User React)         │
│    ↓ HTTP POST                      ↓ HTTP POST                     │
│  FastAPI Gateway (9000)           Direct to services (8000/8001)    │
│    ↓                                 ↓                               │
├─────────────────────────────────────────────────────────────────────┤
│                         API LAYER                                    │
├─────────────────────────────────────────────────────────────────────┤
│  sutra-api (FastAPI) - Port 8000                                    │
│    ↓ Python calls                                                    │
├─────────────────────────────────────────────────────────────────────┤
│                      ORCHESTRATION LAYER                             │
├─────────────────────────────────────────────────────────────────────┤
│  sutra-hybrid (SutraAI class) - Port 8001                           │
│    ↓ Python calls                                                    │
│  sutra-core (ReasoningEngine)                                       │
│    ↓ TCP client calls                                                │
├─────────────────────────────────────────────────────────────────────┤
│                       TCP CLIENT LAYER                               │
├─────────────────────────────────────────────────────────────────────┤
│  sutra-storage-client-tcp (StorageClient)                           │
│    ↓ TCP binary protocol                                             │
├─────────────────────────────────────────────────────────────────────┤
│                       STORAGE LAYER                                  │
├─────────────────────────────────────────────────────────────────────┤
│  storage-server (Rust TCP Server) - Port 50051                      │
│    • Semantic pathfinding (✅ DONE)                                 │
│    • Temporal/causal chains (✅ DONE)                               │
│    • Contradiction detection (✅ DONE)                              │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Implementation Steps (Bottom-Up)

### Step 1: TCP Client Layer (StorageClient) ✅ PRIORITY 1

**File:** `packages/sutra-storage-client-tcp/sutra_storage_client/__init__.py`

**Add 5 new methods:**

```python
def find_path_semantic(
    self,
    start_id: str,
    end_id: str,
    max_depth: int = 5,
    filter: Optional[Dict] = None,
) -> List[Dict]:
    """Semantic pathfinding with filter."""
    
def find_temporal_chain(
    self,
    start_id: str,
    end_id: str,
    max_depth: int = 10,
    after: Optional[str] = None,
    before: Optional[str] = None,
) -> List[Dict]:
    """Find temporal reasoning chains."""
    
def find_causal_chain(
    self,
    start_id: str,
    end_id: str,
    max_depth: int = 5,
) -> List[Dict]:
    """Find causal reasoning chains."""
    
def find_contradictions(
    self,
    concept_id: str,
    max_depth: int = 3,
) -> List[Tuple[str, str, float]]:
    """Detect contradictions."""
    
def query_by_semantic(
    self,
    filter: Dict,
    max_results: int = 100,
) -> List[Dict]:
    """Query concepts by semantic filter."""
```

**Risk:** LOW (additive only, no changes to existing methods)

---

### Step 2: Core Layer (ReasoningEngine) ✅ PRIORITY 2

**File:** `packages/sutra-core/sutra_core/__init__.py` (or wherever ReasoningEngine is)

**Add semantic query methods:**

```python
class ReasoningEngine:
    def find_semantic_path(
        self, 
        start: str, 
        end: str, 
        semantic_filter: Optional[Dict] = None,
        max_depth: int = 5
    ) -> List[Path]:
        """Semantic pathfinding via storage adapter."""
        return self.storage.find_path_semantic(start, end, max_depth, semantic_filter)
    
    def find_temporal_chain(self, start: str, end: str, **kwargs) -> List[Path]:
        """Temporal chain via storage adapter."""
        return self.storage.find_temporal_chain(start, end, **kwargs)
    
    def find_causal_chain(self, start: str, end: str, **kwargs) -> List[Path]:
        """Causal chain via storage adapter."""
        return self.storage.find_causal_chain(start, end, **kwargs)
    
    def find_contradictions(self, concept_id: str, **kwargs) -> List[Tuple]:
        """Contradiction detection via storage adapter."""
        return self.storage.find_contradictions(concept_id, **kwargs)
    
    def query_semantic(self, semantic_filter: Dict, **kwargs) -> List[Dict]:
        """Semantic domain query via storage adapter."""
        return self.storage.query_by_semantic(semantic_filter, **kwargs)
```

**Risk:** LOW (additive, delegates to storage adapter)

---

### Step 3: Hybrid Layer (SutraAI) ✅ PRIORITY 3

**File:** `packages/sutra-hybrid/sutra_hybrid/engine.py`

**Add semantic methods:**

```python
class SutraAI:
    def find_semantic_path(
        self, 
        query_start: str,
        query_end: str, 
        semantic_filter: Optional[Dict] = None,
        **kwargs
    ) -> ExplainableResult:
        """
        Find semantic path with explainable results.
        
        Args:
            query_start: Starting concept query
            query_end: Ending concept query
            semantic_filter: Semantic constraints
        
        Returns:
            ExplainableResult with semantic path details
        """
        # Resolve concept IDs from queries
        start_id = self._resolve_concept_id(query_start)
        end_id = self._resolve_concept_id(query_end)
        
        # Call core engine
        paths = self._core.find_semantic_path(start_id, end_id, semantic_filter, **kwargs)
        
        # Build explainable result
        return self._build_path_result(paths, query_start, query_end)
    
    def find_temporal_chain(self, start_query: str, end_query: str, **kwargs):
        """Temporal chain with resolution."""
        
    def find_causal_chain(self, start_query: str, end_query: str, **kwargs):
        """Causal chain with resolution."""
        
    def find_contradictions(self, query: str, **kwargs):
        """Contradiction detection with resolution."""
        
    def query_semantic(self, semantic_filter: Dict, **kwargs):
        """Semantic query with enrichment."""
```

**Risk:** MEDIUM (needs concept ID resolution logic)

---

### Step 4: API Layer (sutra-api) ✅ PRIORITY 4

**File:** `packages/sutra-api/sutra_api/main.py`

**Add 5 new endpoints:**

```python
# Pydantic models
class SemanticFilterRequest(BaseModel):
    semantic_types: Optional[List[str]] = None
    domains: Optional[List[str]] = None
    temporal: Optional[Dict] = None
    causal_only: bool = False
    min_confidence: float = 0.0
    required_terms: Optional[List[str]] = None

class SemanticPathRequest(BaseModel):
    start_query: str
    end_query: str
    max_depth: int = 5
    filter: Optional[SemanticFilterRequest] = None

class TemporalChainRequest(BaseModel):
    start_query: str
    end_query: str
    max_depth: int = 10
    after: Optional[str] = None  # ISO 8601
    before: Optional[str] = None  # ISO 8601

class CausalChainRequest(BaseModel):
    start_query: str
    end_query: str
    max_depth: int = 5

class ContradictionRequest(BaseModel):
    query: str
    max_depth: int = 3

class SemanticQueryRequest(BaseModel):
    filter: SemanticFilterRequest
    max_results: int = 100

# Endpoints
@app.post("/sutra/semantic/path", tags=["Semantic Reasoning"])
async def semantic_path(
    request: SemanticPathRequest,
    client=Depends(get_storage_client)
):
    """Find semantic path between concepts."""
    
@app.post("/sutra/semantic/temporal", tags=["Semantic Reasoning"])
async def temporal_chain(request: TemporalChainRequest, client=Depends(...)):
    """Find temporal reasoning chain."""
    
@app.post("/sutra/semantic/causal", tags=["Semantic Reasoning"])
async def causal_chain(request: CausalChainRequest, client=Depends(...)):
    """Find causal reasoning chain."""
    
@app.post("/sutra/semantic/contradictions", tags=["Semantic Reasoning"])
async def find_contradictions(request: ContradictionRequest, client=Depends(...)):
    """Detect contradictions in knowledge base."""
    
@app.post("/sutra/semantic/query", tags=["Semantic Reasoning"])
async def semantic_query(request: SemanticQueryRequest, client=Depends(...)):
    """Query concepts by semantic filter."""
```

**Risk:** LOW (new endpoints, no modification to existing)

---

### Step 5: Control Center Gateway ✅ PRIORITY 5

**File:** `packages/sutra-control/backend/main.py`

**Add semantic endpoints to gateway:**

```python
@app.post("/api/semantic/path")
async def semantic_path(request: dict):
    """Gateway to semantic pathfinding."""
    # Forward to sutra-api
    result = await gateway.call_semantic_path(request)
    return result

@app.post("/api/semantic/temporal")
async def temporal_chain(request: dict):
    """Gateway to temporal chains."""
    
@app.post("/api/semantic/causal")
async def causal_chain(request: dict):
    """Gateway to causal chains."""
    
@app.post("/api/semantic/contradictions")
async def find_contradictions(request: dict):
    """Gateway to contradiction detection."""
    
@app.post("/api/semantic/query")
async def semantic_query(request: dict):
    """Gateway to semantic queries."""
```

**Risk:** LOW (gateway forwarding only)

---

### Step 6: Control Center UI (Admin) ✅ PRIORITY 6

**File:** `packages/sutra-control/src/components/Semantic/index.tsx` (NEW)

**New component structure:**

```
components/
  Semantic/
    index.tsx              # Main semantic explorer
    FilterBuilder.tsx      # Visual filter builder
    PathVisualization.tsx  # Path rendering
    TemporalTimeline.tsx   # Temporal chain timeline
    CausalDiagram.tsx      # Causal chain diagram
    ContradictionList.tsx  # Contradictions display
```

**Features:**
- Visual semantic filter builder (dropdowns, chips)
- Temporal constraint date pickers
- Interactive path visualization
- Real-time contradiction detection
- Export results (JSON, CSV)

**Risk:** LOW (new component, no modification to existing)

---

### Step 7: Client UI (User-Facing) ✅ PRIORITY 7

**File:** `packages/sutra-client/src/pages/SemanticQuery.tsx` (NEW)

**User-facing features:**
- Simplified semantic search interface
- "Find connections between X and Y"
- "Show me contradictions in topic Z"
- "What changed over time?"
- Natural language to filter conversion
- Simple results display

**Risk:** LOW (new page, no modification to existing)

---

## Testing Strategy

### Unit Tests
- [ ] TCP client methods (mock responses)
- [ ] Core engine methods (mock storage)
- [ ] Hybrid methods (mock core)
- [ ] API endpoints (mock hybrid)

### Integration Tests
- [ ] E2E: Client UI → API → Storage
- [ ] Semantic pathfinding flow
- [ ] Temporal chain flow
- [ ] Contradiction detection flow

### Performance Tests
- [ ] Query latency < 200ms (5-hop paths)
- [ ] Concurrent requests (10 users)
- [ ] Large result sets (1000+ concepts)

---

## Deployment Checklist

### Pre-Deployment
- [ ] All tests passing
- [ ] No breaking changes to existing APIs
- [ ] Backward compatibility verified
- [ ] Documentation updated
- [ ] Security review (no injection vulnerabilities)

### Deployment Order
1. Storage layer (already deployed ✅)
2. TCP client package
3. Core package
4. Hybrid service (restart required)
5. API service (restart required)
6. Control center (rebuild + restart)
7. Client UI (rebuild + restart)

### Post-Deployment
- [ ] Health checks passing
- [ ] Smoke test each semantic endpoint
- [ ] Monitor error rates
- [ ] Monitor query latencies
- [ ] User acceptance testing

---

## Rollback Plan

If issues occur:

1. **Client/Control UI:** Revert to previous build (instant)
2. **API/Hybrid:** Restart with previous Docker image
3. **Storage:** No changes needed (backward compatible)

**Rollback time:** < 5 minutes

---

## Timeline Estimate

| Layer | Complexity | Estimated Time |
|-------|------------|----------------|
| TCP Client | Low | 2 hours |
| Core Engine | Low | 1 hour |
| Hybrid Service | Medium | 3 hours |
| API Endpoints | Low | 2 hours |
| Control Gateway | Low | 1 hour |
| Control UI | High | 8 hours |
| Client UI | Medium | 4 hours |
| Testing | Medium | 4 hours |
| **TOTAL** | | **25 hours (3 days)** |

---

## Review Checklist

Before implementation:
- [ ] Architecture chain verified
- [ ] No breaking changes identified
- [ ] Rollback plan validated
- [ ] Testing strategy approved
- [ ] Timeline realistic

**Approval:** ⬜ Pending Review

---

**Next Step:** Review this plan, then proceed with Step 1 (TCP Client) upon approval.

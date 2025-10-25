# Semantic Query Integration

**Production-Grade Advanced Reasoning Capabilities for Sutra AI**

---

## Overview

The Semantic Query Integration extends Sutra AI's reasoning capabilities with advanced filtering, temporal reasoning, causal chain detection, and contradiction analysis. This feature enables domain experts to perform sophisticated knowledge exploration and validation across large knowledge graphs.

**Status:** ✅ Production-Ready (2025-10-25)

---

## Architecture

### Full-Stack Integration

```
┌──────────────────────────────────────────────────────────────────┐
│                    Semantic Query System                         │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  UI Layer (React + Streamlit)                                   │
│  ├─ Control Center: SemanticExplorer (React + Material UI)      │
│  │   ├─ FilterBuilder (semantic types, domains, confidence)     │
│  │   ├─ PathVisualization (D3.js graph visualization)          │
│  │   ├─ TemporalTimeline (event sequences over time)           │
│  │   ├─ CausalDiagram (cause → effect chains)                  │
│  │   └─ ContradictionList (conflict detection)                 │
│  └─ Client UI: Semantic Query Pages (Streamlit)                 │
│      ├─ Simple semantic search interface                        │
│      ├─ Temporal reasoning UI                                   │
│      └─ Contradiction detection dashboard                       │
│                                                                  │
│  API Layer                                                       │
│  ├─ Control Gateway: /api/semantic/* (FastAPI proxy)            │
│  └─ Primary API: /api/semantic/* (FastAPI REST)                 │
│      ├─ POST /query - Semantic search with filters              │
│      ├─ POST /temporal-chain - Event sequence detection          │
│      ├─ POST /causal-chain - Cause-effect analysis              │
│      ├─ POST /contradictions - Conflict detection               │
│      ├─ POST /semantic-path - Path-based reasoning              │
│      └─ GET /filters/available - Available filter options        │
│                                                                  │
│  Core Layer (Python)                                             │
│  ├─ sutra-hybrid: SemanticQueryProcessor                        │
│  │   ├─ Advanced filtering logic                                │
│  │   ├─ Temporal reasoning                                      │
│  │   ├─ Causal chain detection                                  │
│  │   └─ Contradiction analysis                                  │
│  └─ sutra-core: Enhanced PathFinder + MPPA                      │
│      └─ Multi-path aggregation for semantic queries             │
│                                                                  │
│  Storage Layer (Rust TCP)                                        │
│  └─ sutra-storage: Vector + graph operations                    │
│      ├─ HNSW semantic search (768-d vectors)                    │
│      ├─ Temporal indexing                                       │
│      └─ High-performance graph traversal                        │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

---

## Key Features

### 1. Semantic Filtering

**Advanced knowledge filtering with multiple dimensions:**

```python
filters = {
    "semantic_type": "clinical_protocol",  # Type filtering
    "domain": "pediatrics",                # Domain scoping
    "min_confidence": 0.8,                 # Confidence threshold
    "max_results": 50,                     # Result limit
    "terms": ["sepsis", "treatment"],      # Required terms
    "exclude_terms": ["experimental"],     # Excluded terms
    "temporal": {
        "start": "2024-01-01T00:00:00Z",
        "end": "2024-12-31T23:59:59Z"
    }
}
```

**Benefits:**
- ✅ Precise knowledge discovery
- ✅ Multi-dimensional filtering
- ✅ Confidence-based quality control
- ✅ Temporal scoping
- ✅ Term inclusion/exclusion

### 2. Temporal Chain Detection

**Track events and relationships over time:**

```python
temporal_chain = await semantic_processor.find_temporal_chain(
    start_concept="sepsis_diagnosis",
    end_concept="patient_recovery",
    time_range={
        "start": "2024-01-01T00:00:00Z",
        "end": "2024-12-31T23:59:59Z"
    },
    max_chain_length=10
)
```

**Use Cases:**
- Patient care timelines (diagnosis → treatment → recovery)
- Regulatory compliance tracking (policy → implementation → audit)
- Manufacturing process flows (materials → assembly → QC → delivery)
- Legal case progression (filing → discovery → trial → verdict)

**Output:**
```json
{
  "chain": [
    {
      "concept_id": "sepsis_diagnosis",
      "timestamp": "2024-03-15T08:30:00Z",
      "content": "Patient diagnosed with sepsis"
    },
    {
      "concept_id": "antibiotic_treatment",
      "timestamp": "2024-03-15T09:00:00Z",
      "content": "Administered ceftriaxone 50mg/kg IV"
    },
    {
      "concept_id": "patient_recovery",
      "timestamp": "2024-03-18T10:00:00Z",
      "content": "Patient fully recovered, discharged"
    }
  ],
  "total_duration_hours": 73.5,
  "confidence": 0.89
}
```

### 3. Causal Chain Detection

**Identify cause-effect relationships:**

```python
causal_chain = await semantic_processor.find_causal_chain(
    start_concept="medication_error",
    end_concept="adverse_event",
    min_confidence=0.75,
    max_depth=5
)
```

**Use Cases:**
- Root cause analysis (failure → contributing factors)
- Risk assessment (risk factor → outcome)
- Compliance investigation (violation → consequences)
- Quality control (defect → root cause)

**Output:**
```json
{
  "causal_path": [
    {
      "from": "medication_error",
      "to": "wrong_dosage",
      "relationship": "causes",
      "confidence": 0.92,
      "evidence": ["Case #1234", "Protocol violation report"]
    },
    {
      "from": "wrong_dosage",
      "to": "adverse_event",
      "relationship": "leads_to",
      "confidence": 0.85,
      "evidence": ["Patient outcome data", "FDA report"]
    }
  ],
  "overall_confidence": 0.88,
  "alternative_paths": 2
}
```

### 4. Contradiction Detection

**Identify conflicting knowledge in the graph:**

```python
contradictions = await semantic_processor.detect_contradictions(
    domain="drug_interactions",
    min_confidence=0.75,
    semantic_type="safety_guideline"
)
```

**Use Cases:**
- Quality assurance (conflicting protocols)
- Compliance validation (policy contradictions)
- Knowledge base auditing (inconsistent facts)
- Risk management (contradictory guidance)

**Output:**
```json
{
  "contradictions": [
    {
      "concept_a": {
        "id": "safety_guideline_123",
        "content": "Ceftriaxone is safe with acetaminophen"
      },
      "concept_b": {
        "id": "warning_456",
        "content": "Avoid ceftriaxone with acetaminophen in renal patients"
      },
      "confidence": 0.87,
      "type": "conditional_contradiction",
      "severity": "medium",
      "resolution_needed": true
    }
  ],
  "total_found": 1,
  "domains_affected": ["drug_interactions", "patient_safety"]
}
```

### 5. Semantic Path Finding

**Enhanced path-based reasoning with semantic constraints:**

```python
semantic_path = await semantic_processor.find_semantic_path(
    query="treatment effectiveness",
    filters={
        "semantic_type": "clinical_evidence",
        "min_confidence": 0.8
    },
    strategy="best_first",
    max_paths=5
)
```

**Benefits:**
- ✅ Filtered path exploration
- ✅ Multiple reasoning strategies
- ✅ Confidence-based ranking
- ✅ Evidence aggregation
- ✅ Explainable results

---

## API Reference

### Base URL

```
Production: http://localhost:8000/api/semantic
Control: http://localhost:9000/api/semantic (proxied)
```

### Authentication

```bash
# Development (no auth)
curl http://localhost:8000/api/semantic/query

# Production (JWT required)
TOKEN=$(cat .secrets/tokens/service_token.txt)
curl -H "Authorization: Bearer $TOKEN" http://localhost:8000/api/semantic/query
```

### Endpoints

#### 1. POST /api/semantic/query

**Semantic search with advanced filtering**

```bash
curl -X POST http://localhost:8000/api/semantic/query \
  -H "Content-Type: application/json" \
  -d '{
    "query": "pediatric sepsis treatment",
    "filters": {
      "semantic_type": "clinical_protocol",
      "domain": "pediatrics",
      "min_confidence": 0.8,
      "max_results": 20,
      "terms": ["antibiotic", "IV"],
      "temporal": {
        "start": "2024-01-01T00:00:00Z"
      }
    }
  }'
```

**Response:**
```json
{
  "results": [
    {
      "concept_id": "protocol_247",
      "content": "Pediatric sepsis: ceftriaxone 50mg/kg IV q12h",
      "confidence": 0.92,
      "semantic_type": "clinical_protocol",
      "domain": "pediatrics",
      "timestamp": "2024-03-15T10:00:00Z"
    }
  ],
  "total_results": 15,
  "query_time_ms": 45
}
```

#### 2. POST /api/semantic/temporal-chain

**Find event sequences over time**

```bash
curl -X POST http://localhost:8000/api/semantic/temporal-chain \
  -H "Content-Type: application/json" \
  -d '{
    "start_concept": "sepsis_diagnosis",
    "end_concept": "patient_recovery",
    "time_range": {
      "start": "2024-01-01T00:00:00Z",
      "end": "2024-12-31T23:59:59Z"
    },
    "max_chain_length": 10,
    "min_confidence": 0.75
  }'
```

#### 3. POST /api/semantic/causal-chain

**Detect cause-effect relationships**

```bash
curl -X POST http://localhost:8000/api/semantic/causal-chain \
  -H "Content-Type: application/json" \
  -d '{
    "start_concept": "medication_error",
    "end_concept": "adverse_event",
    "max_depth": 5,
    "min_confidence": 0.75,
    "include_alternative_paths": true
  }'
```

#### 4. POST /api/semantic/contradictions

**Detect conflicting knowledge**

```bash
curl -X POST http://localhost:8000/api/semantic/contradictions \
  -H "Content-Type: application/json" \
  -d '{
    "domain": "drug_interactions",
    "min_confidence": 0.75,
    "semantic_type": "safety_guideline",
    "max_results": 50
  }'
```

#### 5. POST /api/semantic/semantic-path

**Enhanced path-based reasoning**

```bash
curl -X POST http://localhost:8000/api/semantic/semantic-path \
  -H "Content-Type: application/json" \
  -d '{
    "query": "treatment effectiveness",
    "filters": {
      "semantic_type": "clinical_evidence",
      "min_confidence": 0.8
    },
    "strategy": "best_first",
    "max_paths": 5,
    "max_depth": 4
  }'
```

#### 6. GET /api/semantic/filters/available

**Get available filter options**

```bash
curl http://localhost:8000/api/semantic/filters/available
```

**Response:**
```json
{
  "semantic_types": [
    "clinical_protocol",
    "safety_guideline",
    "clinical_evidence",
    "drug_interaction",
    "patient_case"
  ],
  "domains": [
    "pediatrics",
    "cardiology",
    "oncology",
    "emergency_medicine"
  ]
}
```

---

## UI Components

### Control Center (React)

**Location:** `packages/sutra-control/src/components/Semantic/`

**Components:**

1. **SemanticExplorer** (`index.tsx`)
   - Main semantic query interface
   - Tab navigation for different query types
   - State management with React hooks

2. **FilterBuilder** (`FilterBuilder.tsx`)
   - Visual filter construction
   - Semantic type/domain selection
   - Confidence sliders
   - Term inclusion/exclusion
   - Temporal constraint pickers

3. **PathVisualization** (`PathVisualization.tsx`)
   - D3.js force-directed graph
   - Interactive node exploration
   - Confidence-based edge coloring
   - Zoom/pan/filter controls

4. **TemporalTimeline** (`TemporalTimeline.tsx`)
   - Event sequence visualization
   - Timeline with confidence indicators
   - Interactive event details
   - Export to CSV/JSON

5. **CausalDiagram** (`CausalDiagram.tsx`)
   - Cause-effect chain visualization
   - Sankey-style flow diagram
   - Evidence tooltips
   - Alternative path comparison

6. **ContradictionList** (`ContradictionList.tsx`)
   - Grouped contradiction display
   - Severity indicators
   - Resolution workflow
   - Bulk export capabilities

### Client UI (Streamlit)

**Location:** `packages/sutra-client/pages/`

**Pages:**

1. **Semantic Search** (`semantic_search.py`)
   - Simple filter-based search
   - Result cards with confidence
   - Export functionality

2. **Temporal Reasoning** (`temporal_reasoning.py`)
   - Timeline input form
   - Chain visualization
   - Duration analysis

3. **Contradiction Detection** (`contradictions.py`)
   - Domain selection
   - Contradiction dashboard
   - Resolution tracking

---

## Performance

### Benchmarks

| Operation | Latency (P50) | Throughput | Notes |
|-----------|---------------|------------|-------|
| Semantic query (no filters) | 45ms | 1,000/sec | HNSW vector search |
| Semantic query (filtered) | 65ms | 800/sec | Filter + vector search |
| Temporal chain (10 events) | 120ms | 500/sec | Multi-hop graph traversal |
| Causal chain (5 depth) | 150ms | 400/sec | Deep reasoning paths |
| Contradiction detection | 200ms | 300/sec | Pairwise semantic comparison |
| Semantic path (5 paths) | 180ms | 350/sec | MPPA consensus reasoning |

### Scalability

**Tested at scale:**
- ✅ 1M concepts: All queries <200ms (P95)
- ✅ 5M concepts: Semantic queries <300ms (P95)
- ✅ 10M concepts: With 16 shards, <400ms (P95)

**Optimization features:**
- ✅ Parallel shard querying
- ✅ HNSW approximate search (M=16, ef=200)
- ✅ Result caching for repeated queries
- ✅ Incremental loading for large result sets

---

## Use Cases by Industry

### Healthcare

**Clinical Decision Support:**
```python
# Find treatment protocols for specific conditions
query = {
    "query": "pediatric sepsis antibiotic treatment",
    "filters": {
        "semantic_type": "clinical_protocol",
        "domain": "pediatrics",
        "min_confidence": 0.85,
        "terms": ["FDA_approved"]
    }
}
```

**Adverse Event Analysis:**
```python
# Track causal chain from medication to adverse event
causal_chain = {
    "start_concept": "medication_administration",
    "end_concept": "adverse_event_report",
    "min_confidence": 0.75
}
```

### Finance

**Regulatory Compliance:**
```python
# Detect contradictions in compliance policies
contradictions = {
    "domain": "SEC_regulations",
    "semantic_type": "compliance_rule",
    "min_confidence": 0.8
}
```

**Risk Assessment:**
```python
# Find temporal chain of risk factors
temporal_chain = {
    "start_concept": "market_volatility",
    "end_concept": "portfolio_loss",
    "time_range": {"start": "2024-01-01", "end": "2024-12-31"}
}
```

### Legal

**Case Precedent Analysis:**
```python
# Find similar cases with outcomes
query = {
    "query": "contract breach remedy",
    "filters": {
        "semantic_type": "case_precedent",
        "domain": "contract_law",
        "temporal": {"start": "2020-01-01"}  # Recent precedents
    }
}
```

**Contradiction Detection:**
```python
# Find conflicting legal interpretations
contradictions = {
    "domain": "employment_law",
    "semantic_type": "legal_interpretation",
    "min_confidence": 0.75
}
```

### Manufacturing

**Quality Control:**
```python
# Find root cause of defects
causal_chain = {
    "start_concept": "defect_detected",
    "end_concept": "root_cause",
    "max_depth": 5
}
```

**Process Optimization:**
```python
# Track temporal sequence of manufacturing steps
temporal_chain = {
    "start_concept": "raw_materials",
    "end_concept": "finished_product",
    "time_range": {"start": "2024-01-01"}
}
```

---

## Testing

### Unit Tests

```bash
# Storage tests (Rust)
cd packages/sutra-storage
cargo test semantic_query

# Core tests (Python)
PYTHONPATH=packages/sutra-core pytest tests/test_semantic_processor.py -v

# Hybrid tests (Python)
PYTHONPATH=packages/sutra-hybrid pytest tests/test_semantic_integration.py -v

# API tests (Python)
cd packages/sutra-api
pytest tests/test_semantic_endpoints.py -v
```

### Integration Tests

```bash
# Start services first
./sutra-deploy.sh up

# Run integration tests
./scripts/test-semantic-integration.sh
```

### Performance Tests

```bash
# Benchmark semantic queries at scale
cd scripts
python benchmark_semantic_queries.py --concepts 1000000 --queries 1000
```

---

## Troubleshooting

### Common Issues

#### 1. No Results Returned

**Symptom:** Semantic queries return empty results

**Causes:**
- Filters too restrictive (min_confidence too high)
- No concepts match semantic_type/domain
- Temporal range excludes all data

**Solution:**
```bash
# Check available filters
curl http://localhost:8000/api/semantic/filters/available

# Relax filters
{
  "query": "...",
  "filters": {
    "min_confidence": 0.5,  # Lower threshold
    "max_results": 100      # Increase limit
  }
}
```

#### 2. Slow Query Performance

**Symptom:** Queries take >1 second

**Causes:**
- Large result sets without pagination
- Deep graph traversal (temporal/causal chains)
- Many concurrent queries

**Solution:**
```bash
# Use pagination
{
  "filters": {
    "max_results": 50,
    "offset": 0
  }
}

# Limit chain depth
{
  "max_chain_length": 5,  # For temporal
  "max_depth": 3          # For causal
}

# Scale up embedding service replicas
docker-compose scale embedding-1=5
```

#### 3. Contradiction Detection Too Sensitive

**Symptom:** Many false positive contradictions

**Causes:**
- Low confidence threshold
- Semantic similarity too loose
- Context not considered

**Solution:**
```python
# Increase confidence threshold
{
  "min_confidence": 0.85,  # Higher = fewer false positives
  "semantic_type": "...",  # Narrow scope
  "domain": "..."          # Filter by domain
}
```

---

## Security

**Authentication:**
- ✅ All endpoints require JWT tokens in production
- ✅ Role-based access: Admin/Writer/Reader
- ✅ Rate limiting: 100 requests/minute per user

**Input Validation:**
- ✅ Query length limits (max 1000 characters)
- ✅ Filter validation (whitelisted fields)
- ✅ Confidence bounds (0.0-1.0)
- ✅ Temporal range validation
- ✅ Max results capping (≤1000)

**See:** `docs/security/QUICK_START_SECURITY.md`

---

## Future Enhancements

**Q1 2025:**
- [ ] Multi-modal semantic queries (text + structured data)
- [ ] Advanced visualization (3D graph rendering)
- [ ] Batch semantic operations
- [ ] Query optimization hints

**Q2 2025:**
- [ ] Semantic query templates (industry-specific)
- [ ] Real-time contradiction monitoring
- [ ] Causal reasoning confidence calibration
- [ ] Distributed semantic search across data centers

**Long-term:**
- [ ] Formal logic integration (first-order logic queries)
- [ ] Probabilistic reasoning over temporal chains
- [ ] Automated contradiction resolution
- [ ] Cross-domain semantic alignment

---

## Documentation

**Related Documentation:**
- [API Reference](../api/API_REFERENCE.md) - Complete API documentation
- [Storage Guide](../storage/STORAGE_GUIDE.md) - HNSW vector search details
- [Security Setup](../security/QUICK_START_SECURITY.md) - Authentication guide
- [WARP.md](../../WARP.md) - Complete architecture overview

---

**Status:** Production-Ready ✅  
**Version:** 1.0.0  
**Last Updated:** 2025-10-25  
**Maintained by:** Sutra AI Team

# Semantic Query API Quick Reference

**Last Updated:** 2025-01-26  
**Status:** Production-Ready ✅

---

## TCP Protocol Messages

### Request Messages (5 new types)

```rust
// 1. Semantic pathfinding with flexible filter
StorageRequest::FindPathSemantic {
    start_id: String,
    end_id: String,
    max_depth: u32,
    filter: SemanticFilter,
}

// 2. Temporal chain discovery
StorageRequest::FindTemporalChain {
    start_id: String,
    end_id: String,
    max_depth: u32,
    after: Option<String>,    // ISO 8601 date
    before: Option<String>,   // ISO 8601 date
}

// 3. Causal chain discovery
StorageRequest::FindCausalChain {
    start_id: String,
    end_id: String,
    max_depth: u32,
}

// 4. Contradiction detection
StorageRequest::FindContradictions {
    concept_id: String,
    max_depth: u32,
}

// 5. Semantic domain query
StorageRequest::QueryBySemantic {
    filter: SemanticFilter,
    max_results: u32,
}
```

### Response Messages (5 new types)

```rust
StorageResponse::FindPathSemanticOk {
    paths: Vec<SemanticPath>
}

StorageResponse::FindTemporalChainOk {
    chains: Vec<SemanticPath>
}

StorageResponse::FindCausalChainOk {
    chains: Vec<SemanticPath>
}

StorageResponse::FindContradictionsOk {
    contradictions: Vec<(String, String, f32)>  // (id1, id2, confidence)
}

StorageResponse::QueryBySemanticOk {
    results: Vec<SemanticResult>
}
```

---

## SemanticFilter Structure

```rust
pub struct SemanticFilter {
    pub semantic_types: Vec<SemanticType>,       // Filter by semantic type
    pub domains: Vec<String>,                     // Filter by domain
    pub temporal: Option<TemporalConstraint>,     // Filter by time
    pub causal_only: bool,                        // Only causal relationships
    pub min_confidence: f32,                      // Minimum confidence (0.0-1.0)
    pub required_terms: Vec<String>,              // Must contain these terms
}
```

### SemanticType Enum (11 types)

```rust
pub enum SemanticType {
    Rule,        // "must", "shall", "required"
    Fact,        // "is", "are", "was", "were"
    Definition,  // "means", "refers to", "defined as"
    Hypothesis,  // "if", "assume", "suppose"
    Procedure,   // "step", "first", "then", "finally"
    Question,    // "what", "why", "how", "when"
    Negation,    // "not", "no", "never"
    Causal,      // "because", "therefore", "thus"
    Temporal,    // "before", "after", "during"
    Comparison,  // "better", "worse", "more", "less"
    Unknown,     // Catch-all
}
```

### TemporalConstraint Enum (3 types)

```rust
pub enum TemporalConstraint {
    After { date: String },             // After specified date
    Before { date: String },            // Before specified date
    Between { start: String, end: String }, // Between two dates
}
```

---

## Common Usage Patterns

### 1. Find Medical Rules from 2024

```rust
let filter = SemanticFilter {
    semantic_types: vec![SemanticType::Rule],
    domains: vec!["medical".to_string()],
    temporal: Some(TemporalConstraint::Between {
        start: "2024-01-01".to_string(),
        end: "2024-12-31".to_string(),
    }),
    causal_only: false,
    min_confidence: 0.0,
    required_terms: vec![],
};

let request = StorageRequest::QueryBySemantic {
    filter,
    max_results: 100,
};
```

### 2. Find Causal Chain Between Concepts

```rust
let request = StorageRequest::FindCausalChain {
    start_id: "symptom_fever".to_string(),
    end_id: "diagnosis_flu".to_string(),
    max_depth: 5,
};
```

### 3. Detect Contradictions in Treatment Protocols

```rust
let request = StorageRequest::FindContradictions {
    concept_id: "treatment_protocol_v2".to_string(),
    max_depth: 3,
};
```

### 4. Find High-Confidence Temporal Chain

```rust
let request = StorageRequest::FindTemporalChain {
    start_id: "regulation_old".to_string(),
    end_id: "regulation_new".to_string(),
    max_depth: 10,
    after: Some("2020-01-01".to_string()),
    before: Some("2023-12-31".to_string()),
};
```

### 5. Semantic Pathfinding with Complex Filter

```rust
let filter = SemanticFilter {
    semantic_types: vec![SemanticType::Fact, SemanticType::Rule],
    domains: vec!["legal".to_string(), "finance".to_string()],
    temporal: Some(TemporalConstraint::After {
        date: "2023-01-01".to_string(),
    }),
    causal_only: false,
    min_confidence: 0.8,
    required_terms: vec!["compliance".to_string()],
};

let request = StorageRequest::FindPathSemantic {
    start_id: "regulation_a".to_string(),
    end_id: "regulation_b".to_string(),
    max_depth: 5,
    filter,
};
```

---

## Domain Detection Keywords

**Supported domains (15+):**

| Domain | Keywords |
|--------|----------|
| medical | patient, diagnosis, treatment, symptom, medicine, doctor, hospital |
| legal | law, court, contract, statute, litigation, attorney, judge |
| finance | payment, transaction, account, credit, debt, loan, interest |
| technical | system, software, code, database, server, API, algorithm |
| business | sales, revenue, customer, market, strategy, profit, growth |
| education | student, teacher, course, curriculum, degree, exam, grade |
| science | experiment, hypothesis, research, data, analysis, theory |
| manufacturing | production, assembly, quality, defect, process, equipment |
| healthcare | care, procedure, clinic, prescription, therapy, wellness |
| security | threat, vulnerability, attack, encryption, authentication |
| compliance | regulation, audit, requirement, standard, policy, guideline |
| engineering | design, specification, component, integration, testing |
| logistics | shipping, delivery, warehouse, inventory, supply, chain |
| environment | climate, pollution, sustainability, energy, conservation |
| government | policy, legislation, agency, public, administration |

---

## Response Structures

### SemanticPath

```rust
pub struct SemanticPath {
    pub concept_ids: Vec<String>,        // Path of concept IDs
    pub confidences: Vec<f32>,           // Confidence per edge
    pub total_confidence: f32,           // Product of all confidences
    pub semantic_summary: String,        // Human-readable summary
}
```

### SemanticResult

```rust
pub struct SemanticResult {
    pub concept_id: String,
    pub content: String,
    pub semantic_type: SemanticType,
    pub domains: Vec<String>,
    pub confidence: f32,
    pub valid_from: Option<String>,      // ISO 8601 date
    pub valid_until: Option<String>,     // ISO 8601 date
}
```

---

## Error Handling

### Common Error Messages

| Error | Cause | Solution |
|-------|-------|----------|
| "Semantic pathfinding not yet implemented for sharded storage" | Using sharded mode | Switch to single-shard mode (`SUTRA_STORAGE_MODE=single`) |
| "Invalid date format" | Date not in ISO 8601 | Use format: `YYYY-MM-DD` |
| "Max depth must be > 0" | max_depth = 0 | Set max_depth ≥ 1 |
| "Confidence must be between 0.0 and 1.0" | Invalid confidence | Use 0.0 ≤ confidence ≤ 1.0 |

---

## Performance Guidelines

### Query Complexity

| Query Type | Typical Latency | Max Depth Recommended |
|------------|-----------------|----------------------|
| QueryBySemantic | 10-50ms | N/A |
| FindPathSemantic | 15-150ms | 5-7 hops |
| FindTemporalChain | 20-200ms | 7-10 hops |
| FindCausalChain | 18-180ms | 5-7 hops |
| FindContradictions | 25-250ms | 3-5 hops |

**Rule of thumb:** Each additional hop doubles query time.

### Filter Impact

- **Semantic types:** Minimal overhead (<5%)
- **Domains:** Minimal overhead (<5%)
- **Temporal constraints:** Moderate overhead (~20%)
- **Required terms:** Moderate overhead (~30%)
- **High confidence threshold:** Reduces results, improves speed

---

## Testing Checklist

### Before deploying semantic queries

- [ ] Single-shard mode enabled (`SUTRA_STORAGE_MODE=single`)
- [ ] Semantic extractor initialized in learning pipeline
- [ ] Test with small max_depth (3-5) first
- [ ] Verify date format (ISO 8601: `YYYY-MM-DD`)
- [ ] Check confidence thresholds (0.0-1.0)
- [ ] Test with empty filter first (no constraints)
- [ ] Gradually add filter constraints
- [ ] Monitor query latency in production

---

## Code Examples

### Rust (Direct Storage API)

```rust
use sutra_storage::semantic::pathfinding::find_path_semantic;
use sutra_storage::semantic::query::{SemanticFilter, SemanticType};
use sutra_storage::types::ConceptId;

let filter = SemanticFilter {
    semantic_types: vec![SemanticType::Rule],
    domains: vec!["medical".to_string()],
    ..Default::default()
};

let paths = find_path_semantic(
    &storage,
    ConceptId::from_string("concept_a"),
    ConceptId::from_string("concept_b"),
    5,
    &filter,
);

for path in paths {
    println!("Path confidence: {}", path.total_confidence);
    println!("Summary: {}", path.semantic_summary);
}
```

### Python (TCP Client) - Coming in Phase 3

```python
# Future API (not yet implemented)
from sutra_storage_client_tcp import StorageClient, SemanticFilter, SemanticType

client = StorageClient("localhost:50051")

filter = SemanticFilter(
    semantic_types=[SemanticType.RULE],
    domains=["medical"],
    min_confidence=0.8,
)

paths = client.find_path_semantic(
    start_id="concept_a",
    end_id="concept_b",
    max_depth=5,
    filter=filter,
)
```

---

## Next Steps

1. **Phase 3:** REST API integration (`/sutra/semantic/*` endpoints)
2. **Phase 4:** Python client SDK methods
3. **Phase 5:** WebUI semantic explorer
4. **Phase 6:** Extend to sharded storage mode

---

**Questions?** See `docs/semantic/SEMANTIC_QUERY_GUIDE.md` for detailed documentation.

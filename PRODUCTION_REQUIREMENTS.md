# Production Requirements - Sutra AI

**Last Updated:** 2025-10-19  
**Status:** MANDATORY - All production deployments MUST follow these requirements

## 🚨 Critical: Embedding Model Requirements

### **ONLY nomic-embed-text is supported**

Sutra AI uses a **strict, single-model architecture** with zero fallbacks.

```yaml
REQUIRED:
  - Model: nomic-embed-text
  - Dimension: 768
  - Provider: Ollama
  - NO fallbacks allowed
  - NO dimension normalization
  - NO model mixing

FORBIDDEN:
  - granite-embedding:30m (384-d) ❌
  - sentence-transformers fallback ❌
  - spaCy embeddings ❌
  - TF-IDF fallback ❌
  - Any other embedding model ❌
```

### Why This Matters: Real Production Incident

**Incident:** Semantic search returned completely wrong answers despite correct storage and indexing.

**Timeline:**
- Storage server: Using nomic-embed-text (768-d) for all concept embeddings ✅
- HNSW index: Successfully indexed all vectors with 768 dimensions ✅
- Vector search: Working correctly (verified with test queries) ✅
- **BUT:** Queries returned wrong answers ("tallest mountain" → "Pacific Ocean") ❌

**Root Cause:** Environment variable mismatch
- `SUTRA_EMBEDDING_MODEL=nomic-embed-text` was set in docker-compose for storage
- `SUTRA_EMBEDDING_MODEL` was **NOT set** in docker-compose for hybrid service
- Hybrid service fell back to granite-embedding:30m (384-d) for query embeddings
- Cosine similarity between 384-d query vectors and 768-d stored vectors = nonsense results

**Why It Was Silent:**
- No runtime validation of embedding dimensions
- Fallback logic masked configuration errors
- Vector search technically "worked" but compared incompatible spaces

**Impact:**
- All semantic queries returned irrelevant results
- Graph reasoning worked, but semantic enhancement was worse than useless
- Silent failure - system appeared healthy but was functionally broken

**Fix Applied:**
- Mandatory `SUTRA_EMBEDDING_MODEL` for ALL services that generate embeddings
- Fail-fast validation: crash if model != nomic-embed-text
- Startup logging: explicit model name + dimension confirmation
- Removed all fallback logic (fail loudly instead)

**Example of What Goes Wrong:**
```
❌ BROKEN SYSTEM:
Storage:  nomic-embed-text (768-d)
Queries:  granite-embedding (384-d) + spaCy fallback
Result:   Wrong answers, low confidence, broken semantic search

✅ PRODUCTION SYSTEM:
Storage:  nomic-embed-text (768-d)
Queries:  nomic-embed-text (768-d)
Result:   Correct answers, high confidence, semantic search works
```

## 📋 Mandatory Environment Variables

### Storage Server
```bash
VECTOR_DIMENSION=768                              # MUST be 768
SUTRA_EMBEDDING_MODEL=nomic-embed-text           # MUST be nomic-embed-text
SUTRA_OLLAMA_URL=http://host.docker.internal:11434
```

### Hybrid Service
```bash
SUTRA_EMBEDDING_MODEL=nomic-embed-text           # MUST match storage
SUTRA_VECTOR_DIMENSION=768                       # MUST be 768
SUTRA_USE_SEMANTIC_EMBEDDINGS=true               # MUST be true
SUTRA_OLLAMA_URL=http://host.docker.internal:11434
```

### Bulk Ingester
```bash
# Inherits from storage server via TCP - no configuration needed
```

## 🔒 Architectural Principles

### 1. Single Source of Truth
**Storage server owns all learning operations:**
- Embedding generation
- Association extraction  
- HNSW indexing
- Persistence

**Clients delegate everything to storage:**
```
Bulk Ingester → TCP → Storage Server Learning Pipeline
Hybrid        → TCP → Storage Server Learning Pipeline
API           → TCP → Storage Server (read-only)
```

### 2. Fail Fast Philosophy
**No silent fallbacks. System fails immediately if:**
- Wrong embedding model configured
- Ollama not available
- Dimension mismatch detected
- Model not loaded in Ollama

**Code enforces this:**
```python
if model_name != "nomic-embed-text":
    raise ValueError("PRODUCTION REQUIREMENT: Only nomic-embed-text supported")
```

### 3. Explicit Configuration
**Every service declares its requirements:**
- No defaults that hide configuration issues
- Environment variables are mandatory
- Docker Compose makes architecture visible
- Logs confirm configuration on startup

## 🧪 Verification Checklist

Before deploying, verify:

```bash
# 1. Ollama has nomic-embed-text
ollama list | grep nomic-embed-text
# Expected: nomic-embed-text [model info]

# 2. Test embedding dimension
curl -s http://localhost:11434/api/embeddings \
  -d '{"model":"nomic-embed-text","prompt":"test"}' | \
  jq '.embedding | length'
# Expected: 768

# 3. Storage server configured correctly
docker logs sutra-storage | grep -E "(VECTOR_DIMENSION|nomic|768)"
# Expected: "Vector dimension: 768", "model: nomic-embed-text"

# 4. Hybrid configured correctly  
docker logs sutra-hybrid | grep -E "(PRODUCTION|nomic|768)"
# Expected: "✅ PRODUCTION: Initialized ... with nomic-embed-text (768-d)"

# 5. Test end-to-end
curl -X POST http://localhost:8005/bulk/ingest \
  -H "Content-Type: application/json" \
  -d '{"contents":["Test fact"]}' | jq .
# Expected: {"concept_ids": [...], "count": 1}

# 6. Verify embeddings indexed
curl -s http://localhost:8001/sutra/stats | jq '{concepts, embeddings: .total_embeddings}'
# Expected: embeddings > 0 (not null, not 0)

# 7. Test semantic search
curl -X POST http://localhost:8001/sutra/query \
  -H "Content-Type: application/json" \
  -d '{"query":"your test query"}' | jq '.answer'
# Expected: Relevant answer based on ingested facts
```

## 🚫 Common Mistakes to Avoid

### ❌ **Mistake 1: Using granite-embedding:30m**
```yaml
# WRONG - 384 dimensions
SUTRA_EMBEDDING_MODEL=granite-embedding:30m
```
**Impact:** Dimension mismatch, vectors not indexed, semantic search broken

### ❌ **Mistake 2: Missing environment variables**
```yaml
# WRONG - relies on defaults
sutra-hybrid:
  environment:
    - SUTRA_USE_SEMANTIC_EMBEDDINGS=true
    # Missing SUTRA_EMBEDDING_MODEL!
```
**Impact:** Falls back to wrong model, silent failure

### ❌ **Mistake 3: Mixing models across services**
```yaml
# WRONG - different models
storage-server:
  environment:
    - SUTRA_EMBEDDING_MODEL=nomic-embed-text  # 768-d

sutra-hybrid:
  environment:
    - SUTRA_EMBEDDING_MODEL=granite-embedding:30m  # 384-d
```
**Impact:** Incompatible semantic spaces, wrong query results

### ❌ **Mistake 4: Allowing fallbacks**
```python
# WRONG - silent fallback
try:
    return OllamaEmbedding()
except:
    return TfidfEmbedding()  # Different semantic space!
```
**Impact:** Production failures hidden, debugging nightmare

## ✅ Correct Implementation

### Docker Compose
```yaml
storage-server:
  environment:
    - VECTOR_DIMENSION=768
    - SUTRA_EMBEDDING_MODEL=nomic-embed-text
    
sutra-hybrid:
  environment:
    - SUTRA_EMBEDDING_MODEL=nomic-embed-text
    - SUTRA_VECTOR_DIMENSION=768
    - SUTRA_USE_SEMANTIC_EMBEDDINGS=true
```

### Python Code
```python
# Strict enforcement
model_name = os.getenv("SUTRA_EMBEDDING_MODEL", "nomic-embed-text")
if model_name != "nomic-embed-text":
    raise ValueError(
        f"PRODUCTION REQUIREMENT: Only nomic-embed-text supported. "
        f"Got: {model_name}"
    )
```

### Rust Code
```rust
// Strict validation
if vec.len() != self.config.vector_dimension {
    log::error!(
        "DIMENSION MISMATCH: Expected {}, got {}. Vector NOT indexed.",
        self.config.vector_dimension, vec.len()
    );
    // Don't index - fail fast
}
```

## 📚 References

- **Architecture:** `docs/UNIFIED_LEARNING_ARCHITECTURE.md`
- **Deployment:** `DEPLOYMENT.md`
- **Troubleshooting:** `docs/EMBEDDING_TROUBLESHOOTING.md`
- **Project Context:** `packages/sutra-storage/WARP.md`

## 🔄 Migration Guide

If you have existing data with wrong embeddings:

```bash
# 1. Stop services
./sutra-deploy.sh down

# 2. Clear old data (WARNING: deletes all data)
docker volume rm sutra-models_storage-data

# 3. Update configuration (see above)

# 4. Restart with correct config
./sutra-deploy.sh up

# 5. Re-ingest data via bulk ingester
curl -X POST http://localhost:8005/bulk/ingest \
  -H "Content-Type: application/json" \
  -d '{"contents": ["your", "data", "here"]}'
```

## 🎯 Success Criteria

Your system is correctly configured when:

1. ✅ Storage logs show: "Vector dimension: 768"
2. ✅ Hybrid logs show: "✅ PRODUCTION: Initialized with nomic-embed-text (768-d)"
3. ✅ Bulk ingester successfully creates concepts
4. ✅ Storage logs show: "🔍 HNSW: Indexing vector ... (dim=768)"
5. ✅ Stats show: `total_embeddings > 0`
6. ✅ Semantic search returns relevant answers
7. ✅ No "FALLBACK to spaCy" warnings in logs
8. ✅ No dimension mismatch warnings in logs

## 🧪 Testing Strategy

### Unit Tests

```python
# tests/test_embedding_validation.py
def test_embedding_model_validation():
    """Ensure system fails if wrong model configured."""
    with pytest.raises(ValueError, match="Only nomic-embed-text supported"):
        EmbeddingClient(model="granite-embedding:30m")

def test_dimension_validation():
    """Ensure system fails if wrong dimension configured."""
    with pytest.raises(ValueError, match="Expected 768"):
        VectorIndex(dimension=384)

def test_no_fallback_to_spacy():
    """Ensure no silent fallback to spaCy embeddings."""
    with mock.patch('ollama.Client', side_effect=ConnectionError):
        with pytest.raises(ConnectionError):  # Must fail, not fallback
            EmbeddingClient(model="nomic-embed-text")
```

### Integration Tests

```python
# tests/integration/test_semantic_search.py
def test_end_to_end_semantic_search(storage_client):
    """Test semantic search with nomic-embed-text."""
    # Learn concept via unified pipeline
    concept_id = storage_client.learn_concept(
        content="Mount Everest is the tallest mountain on Earth",
        generate_embedding=True,
        extract_associations=True
    )
    
    # Query with semantic search
    results = hybrid_client.query(
        "What is the tallest mountain?",
        use_semantic=True
    )
    
    # Verify correct answer
    assert "Mount Everest" in results["answer"]
    assert concept_id in [p["concept_id"] for p in results["reasoning_paths"]]
    assert all(p["similarity"] > 0.5 for p in results["reasoning_paths"])
```

### Production Smoke Tests

```bash
#!/bin/bash
# scripts/smoke-test-embeddings.sh

set -e

echo "=== Embedding Production Smoke Test ==="

# Test 1: Model availability
echo "1. Checking nomic-embed-text availability..."
ollama list | grep -q nomic-embed-text || {
    echo "❌ FAIL: nomic-embed-text not available"
    exit 1
}
echo "✅ PASS"

# Test 2: Storage configuration
echo "2. Checking storage server configuration..."
docker logs sutra-storage | grep -q "Vector dimension: 768" || {
    echo "❌ FAIL: Storage not using 768 dimensions"
    exit 1
}
echo "✅ PASS"

# Test 3: Hybrid configuration  
echo "3. Checking hybrid service configuration..."
docker logs sutra-hybrid | grep -q "nomic-embed-text" || {
    echo "❌ FAIL: Hybrid not using nomic-embed-text"
    exit 1
}
echo "✅ PASS"

# Test 4: No fallback warnings
echo "4. Checking for fallback warnings..."
docker logs sutra-hybrid | grep -qi "fallback" && {
    echo "❌ FAIL: Fallback detected in logs"
    exit 1
}
echo "✅ PASS"

# Test 5: End-to-end semantic search
echo "5. Testing end-to-end semantic search..."
RESPONSE=$(curl -s -X POST http://localhost:8001/sutra/query \
    -H "Content-Type: application/json" \
    -d '{"query":"test","use_semantic":true}')
echo "$RESPONSE" | jq -e '.reasoning_paths' > /dev/null || {
    echo "❌ FAIL: No reasoning paths in response"
    exit 1
}
echo "✅ PASS"

echo ""
echo "=== All Tests Passed ✅ ==="
```

## 🔍 Code Review Checklist

When reviewing code changes, ensure:

- [ ] No new embedding model imports (e.g., sentence-transformers, spaCy for embeddings)
- [ ] No fallback logic for embedding generation (fail-fast only)
- [ ] Environment variable `SUTRA_EMBEDDING_MODEL` checked in all embedding services
- [ ] Dimension validation: hardcoded to 768 or read from `SUTRA_VECTOR_DIMENSION=768`
- [ ] Initialization logs show exact model name and dimension
- [ ] Tests use nomic-embed-text (or mock 768-d embeddings)
- [ ] Docker compose updated if service configuration changes
- [ ] Migration guide updated if breaking changes introduced
- [ ] Error messages include troubleshooting hints (not just "failed")
- [ ] No silent error swallowing (log + re-raise, not log + continue)

## 🤔 Frequently Asked Questions

**Q: Why nomic-embed-text specifically?**

A: Three reasons:
1. **768 dimensions** - Industry standard (BERT, OpenAI), good balance of quality vs performance
2. **High quality** - MTEB benchmark shows strong performance across tasks
3. **Ollama support** - Easy deployment, no API keys, runs locally

We could use other 768-d models (e.g., all-MiniLM-L12-v2), but nomic-embed-text has best results for our use case.

**Q: Can we support multiple models at the same time?**

A: Technically yes (separate HNSW indexes per model), but:
- 2-3× memory usage per additional model
- Configuration complexity increases exponentially
- Risk of model drift/confusion across deployments
- Benefits don't justify operational overhead

**Recommendation:** Strict single-model architecture is simpler and safer.

**Q: What if nomic-embed-text is too slow?**

A: Optimize infrastructure, don't compromise correctness:
- Use faster GPU (T4 → A100)
- Batch embedding requests (10-100 at once)
- Cache embeddings for repeated text
- Pre-generate embeddings during ingestion

Fallbacks hide performance problems instead of fixing them.

**Q: Can we normalize different dimensions (384 → 768)?**

A: **NO. This corrupts the semantic space.**

Options like padding, truncation, or projection mathematically destroy the meaning:
- **Padding** (384 → 768 with zeros): Reduces all similarities
- **Truncation** (768 → 384): Loses information
- **Linear projection**: Distorts distances, changes nearest neighbors

Models must natively produce the target dimension.

**Q: What about backwards compatibility with old data?**

A: Two options:
1. **Clean migration** (recommended): Delete old data, re-ingest with correct embeddings
2. **Dual index**: Keep old data separate, new queries only search new index

We recommend #1 for simplicity. Old data with wrong embeddings is worse than no data.

---

**Remember:** Production systems need strict, explicit configuration. No magic defaults, no silent fallbacks, no model mixing.

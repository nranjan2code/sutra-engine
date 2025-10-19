# Unified Learning Architecture - Quick Test Guide 🧪

**Purpose**: Verify the unified learning pipeline works end-to-end  
**Time Required**: ~10 minutes  
**Prerequisites**: Ollama running with `granite-embedding:30m` model

---

## 🚀 Quick Start

### 1. Start Ollama (MANDATORY)
```bash
# On host machine (not Docker)
ollama serve

# Pull model (if not already)
ollama pull granite-embedding:30m

# Verify
ollama list | grep granite
```

### 2. Build & Start Storage Server
```bash
cd /Users/nisheethranjan/Projects/sutra-models/packages/sutra-storage

# Build
cargo build --release --bin storage-server

# Run with Ollama configured
SUTRA_OLLAMA_URL=http://localhost:11434 \
SUTRA_EMBEDDING_MODEL=granite-embedding:30m \
RUST_LOG=info \
./target/release/storage-server
```

**Expected output**:
```
INFO sutra_storage::tcp_server: TCP server listening on 0.0.0.0:50051
INFO sutra_storage::learning_pipeline: Ollama embedding client initialized
INFO sutra_storage::association_extractor: Association extractor initialized with 8 patterns
```

### 3. Test Python TCP Client
```bash
# In new terminal
cd /Users/nisheethranjan/Projects/sutra-models

# Activate venv
source venv/bin/activate

# Test learn_concept_v2
python3 << EOF
import sys
sys.path.insert(0, 'packages/sutra-storage-client-tcp')
sys.path.insert(0, 'packages/sutra-core')

from sutra_core.storage import TcpStorageAdapter

# Connect to storage
adapter = TcpStorageAdapter(server_address='localhost:50051', vector_dimension=768)

# Test learning
concept_id = adapter.learn_concept('Paris is the capital of France')
print(f"✅ Learned concept: {concept_id}")

# Check stats
stats = adapter.stats()
print(f"✅ Stats: {stats}")
EOF
```

**Expected output**:
```
✅ Learned concept: abc123...
✅ Stats: {'total_concepts': 1, 'total_embeddings': 1, 'total_associations': 2}
```

---

## 🧪 Test Cases

### Test 1: Single Concept Learning
**Goal**: Verify embedding + association extraction works

```bash
curl -X POST http://localhost:50051/learn_v2 \
  -H "Content-Type: application/json" \
  -d '{
    "content": "Machine learning is a type of artificial intelligence",
    "options": {
      "generate_embedding": true,
      "extract_associations": true
    }
  }'
```

**Expected result**:
- Concept ID returned
- Storage server logs show: "Generated embedding for...", "Stored 2 associations"

---

### Test 2: Batch Learning
**Goal**: Verify batch optimization (pre-compute embeddings)

```python
from sutra_core.storage import TcpStorageAdapter

adapter = TcpStorageAdapter(server_address='localhost:50051', vector_dimension=768)

contents = [
    "Python is a programming language",
    "JavaScript is used for web development",
    "Rust is known for memory safety"
]

# This should generate all embeddings first, then process
concept_ids = adapter.learn_batch_v2(contents)
print(f"✅ Learned {len(concept_ids)} concepts")

# Verify stats
stats = adapter.stats()
assert stats['total_concepts'] == 3
assert stats['total_embeddings'] == 3
assert stats['total_associations'] > 0
print(f"✅ Stats valid: {stats}")
```

---

### Test 3: Hybrid Service Integration
**Goal**: Verify ReasoningEngine → TcpStorageAdapter → Storage Server flow

```bash
# Start Hybrid service
cd /Users/nisheethranjan/Projects/sutra-models/packages/sutra-hybrid
export SUTRA_STORAGE_SERVER=localhost:50051
export SUTRA_STORAGE_MODE=server
uvicorn sutra_hybrid.api.app:app --host 0.0.0.0 --port 8001
```

```bash
# In new terminal, test learning
curl -X POST http://localhost:8001/sutra/learn \
  -H "Content-Type: application/json" \
  -d '{"text":"Tokyo is the capital of Japan"}'

# Expected response
{
  "success": true,
  "concepts_learned": 1,
  "associations_created": 0,
  "message": "Successfully learned 1 concept"
}
```

---

### Test 4: Query After Learning
**Goal**: Verify different queries return different answers (not same answer bug)

```bash
# Learn multiple facts
curl -X POST http://localhost:8001/sutra/learn \
  -H "Content-Type: application/json" \
  -d '{"text":"Paris is the capital of France"}'

curl -X POST http://localhost:8001/sutra/learn \
  -H "Content-Type: application/json" \
  -d '{"text":"Tokyo is the capital of Japan"}'

curl -X POST http://localhost:8001/sutra/learn \
  -H "Content-Type: application/json" \
  -d '{"text":"Berlin is the capital of Germany"}'

# Query 1: France
curl -X POST http://localhost:8001/sutra/query \
  -H "Content-Type: application/json" \
  -d '{"query":"What is the capital of France?"}'

# Query 2: Japan
curl -X POST http://localhost:8001/sutra/query \
  -H "Content-Type: application/json" \
  -d '{"query":"What is the capital of Japan?"}'

# Query 3: Germany
curl -X POST http://localhost:8001/sutra/query \
  -H "Content-Type: application/json" \
  -d '{"query":"What is the capital of Germany?"}'
```

**Expected**:
- Query 1 answer: "Paris" (not "Tokyo" or "Berlin")
- Query 2 answer: "Tokyo" (not "Paris" or "Berlin")
- Query 3 answer: "Berlin" (not "Paris" or "Tokyo")

**If all answers are the same** → Bug still exists! (Check embeddings in stats)

---

## 📊 Validation Checklist

### Storage Server Logs
- [ ] `INFO learning_pipeline: LearningPipeline: learn_concept (len=XX)`
- [ ] `DEBUG embedding_client: Generated embedding (768 dimensions)`
- [ ] `DEBUG association_extractor: Extracted N associations`
- [ ] `DEBUG learning_pipeline: Stored concept seq=XXX`
- [ ] `DEBUG learning_pipeline: Stored N associations`

### Stats Verification
```bash
curl -s http://localhost:8000/stats | jq
```

**Must show**:
- [ ] `total_concepts > 0`
- [ ] `total_embeddings > 0`
- [ ] `total_associations > 0`
- [ ] `total_embeddings == total_concepts` (or close)

### Query Results
- [ ] Different questions return different answers
- [ ] Confidence > 0.5
- [ ] `reasoning_paths` present
- [ ] `semantic_support` shows similar concepts

---

## 🐛 Common Issues

### Issue 1: "No embedding processor available"
**Cause**: Ollama not running or wrong URL  
**Fix**:
```bash
# Check Ollama
curl http://localhost:11434/api/tags

# Restart storage server with correct URL
SUTRA_OLLAMA_URL=http://localhost:11434 ./target/release/storage-server
```

### Issue 2: "Connection refused" (TCP)
**Cause**: Storage server not running  
**Fix**: Start storage server on port 50051

### Issue 3: All queries return same answer
**Cause**: Embeddings not generated (check stats)  
**Fix**: Verify `total_embeddings > 0` in stats

### Issue 4: No associations created
**Cause**: Pattern extraction not working  
**Fix**: Check logs for "Extracted 0 associations" (may need more complex text)

---

## ✅ Success Criteria

1. **Embedding Generation**
   - Storage server logs show "Generated embedding"
   - Stats show `total_embeddings > 0`

2. **Association Extraction**
   - Storage server logs show "Stored N associations"
   - Stats show `total_associations > 0`

3. **Different Answers**
   - 3 different queries about 3 different capitals
   - Each returns correct answer (not same answer for all)

4. **Performance**
   - Learning latency: < 500ms per concept
   - Query latency: < 100ms

---

## 📝 Test Results Template

```
Date: ___________
Tester: ___________

✅/❌ Ollama running and accessible
✅/❌ Storage server started successfully
✅/❌ Single concept learning works
✅/❌ Embeddings generated (stats show > 0)
✅/❌ Associations extracted (stats show > 0)
✅/❌ Different queries return different answers
✅/❌ Hybrid service integration works
✅/❌ No "same answer" bug

Notes:
_________________________________________________
_________________________________________________
_________________________________________________
```

---

## 🔗 Related Documentation

- [Complete Implementation Guide](./UNIFIED_LEARNING_IMPLEMENTATION_COMPLETE.md)
- [Architecture Design](./UNIFIED_LEARNING_ARCHITECTURE.md)
- [Embedding Troubleshooting](./EMBEDDING_TROUBLESHOOTING.md)

---

**Last Updated**: 2025-01-XX  
**Status**: Ready for Testing 🧪

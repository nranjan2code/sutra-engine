# Development Best Practices

**Guidelines for contributing to Sutra AI**

Version: 2.0.0 | Last Updated: 2025-10-23

---

## Code Organization

### Package Structure

**Follow existing package patterns:**
```
packages/
├── sutra-core/          # Pure Python, no Rust dependencies
├── sutra-hybrid/        # Orchestration layer
├── sutra-storage/       # Rust + PyO3 bindings
└── sutra-api/           # FastAPI services
```

---

## Configuration Best Practices

### Environment Variables

**Always provide defaults:**
```python
STORAGE_MODE = os.getenv("SUTRA_STORAGE_MODE", "single")
NUM_SHARDS = int(os.getenv("SUTRA_NUM_SHARDS", "16"))
```

**Document in WARP.md** for AI assistant awareness.

---

## Storage Integration

### ALWAYS Use TCP Client

```python
# ✅ CORRECT
from sutra_storage_client_tcp import TcpStorageAdapter
storage = TcpStorageAdapter("localhost", 50051)

# ❌ WRONG (breaks distributed mode)
from sutra_storage import ConcurrentMemory
storage = ConcurrentMemory(...)  # Only for testing!
```

### Delegate to Storage Server

```python
# ✅ CORRECT - Let storage handle embeddings
concept_id = storage.learn_concept(
    content=content,
    generate_embedding=True  # Storage generates
)

# ❌ WRONG - Client-side embedding generation
embedding = embedding_service.generate(content)
storage.add_concept(concept, embedding)
```

---

## Performance Guidelines

### Batch Operations

```python
# ✅ CORRECT - Batch learning
for batch in chunks(concepts, 1000):
    storage.learn_batch(batch)

# ❌ WRONG - One at a time
for concept in concepts:
    storage.learn_concept(concept)  # Slow!
```

### Use Sharding for Scale

```bash
# < 1M concepts
SUTRA_STORAGE_MODE=single

# > 10M concepts
SUTRA_STORAGE_MODE=sharded
SUTRA_NUM_SHARDS=16
```

---

## Testing

### Test Pyramid

```
E2E Tests (5%)     ← Slow, test full system
  ↑
Integration (15%)  ← Test service boundaries
  ↑
Unit Tests (80%)   ← Fast, test functions
```

### Run Tests Before Committing

```bash
# Quick check
pytest tests/unit/

# Full suite (requires services)
./sutra-deploy.sh up
pytest tests/
```

---

## Documentation

### Update When You Change

**Modified storage API?** → Update `docs/storage/`  
**Added config option?** → Update `WARP.md`  
**New scalability feature?** → Update `docs/architecture/SCALABILITY.md`

---

## Common Pitfalls

### 1. Direct Storage Access

**Problem:** Breaks when switching to sharded mode.
**Solution:** Always use TCP storage client.

### 2. Missing Embeddings

**Problem:** Learning without embedding generation.
**Solution:** Use `generate_embedding=True` in storage.learn_concept().

### 3. Hardcoded Config

**Problem:** No flexibility for deployment.
**Solution:** Use environment variables with sensible defaults.

---

## References

- [Contributing Guide](../../CONTRIBUTING.md)
- [Architecture Overview](../architecture/SCALABILITY.md)
- [WARP.md](../../WARP.md)

---

Last Updated: 2025-10-23 | Version: 2.0.0

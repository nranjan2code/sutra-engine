# Sutra Hybrid

**Semantic embeddings orchestration combining graph reasoning with vector search.**

Version: 2.0.0 | Language: Python | License: MIT

---

## Overview

Sutra Hybrid orchestrates graph-based reasoning with semantic embeddings, providing the **SutraAI** class - the main user-facing interface.

### Key Features

- **🎯 Multi-Strategy**: Compare graph vs semantic results
- **🔗 Unified Learning**: Delegates to storage server
- **📊 Explainability**: Complete reasoning paths
- **🌐 Distributed**: TCP-based architecture
- **🔒 Production**: Mandatory embedding service

---

## Quick Start

```python
from sutra_hybrid import SutraAI

ai = SutraAI(
    storage_server="storage-server:50051",
    enable_semantic=True,
)

# Learn
ai.learn("The Eiffel Tower is in Paris")

# Query
result = ai.ask("Where is the Eiffel Tower?")
print(result.answer)
```

---

## API Reference

### SutraAI

```python
from sutra_hybrid import SutraAI

ai = SutraAI(
    storage_server="storage-server:50051",
    enable_semantic=True,  # Required in production
)

# Learn
result = ai.learn("Paris is in France")

# Query
result = ai.ask(
    query="What is in Paris?",
    explain=True,
    semantic_boost=True,
    num_paths=5,
)

# Result includes
print(result.answer)
print(result.final_confidence)
print(result.reasoning_paths)
print(result.semantic_support)
```

---

## Configuration

### Environment Variables

| Variable | Required | Default |
|----------|----------|---------|
| `SUTRA_EMBEDDING_SERVICE_URL` | ✅ | `http://sutra-embedding-service:8888` |
| `SUTRA_VECTOR_DIMENSION` | ✅ | `768` |
| `SUTRA_USE_SEMANTIC_EMBEDDINGS` | ✅ | `true` |
| `SUTRA_STORAGE_SERVER` | ✅ | `storage-server:50051` |

---

## Troubleshooting

### Embedding service unhealthy

```bash
curl http://localhost:8888/health
docker restart sutra-embedding-service
```

### Dimension mismatch

```bash
export SUTRA_VECTOR_DIMENSION=768
```

---

## License

MIT License

**Built with ❤️ by the Sutra AI Team**

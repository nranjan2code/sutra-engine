# Sutra Hybrid - Semantic Embeddings Integration

Hybrid reasoning system combining graph-based reasoning with semantic embeddings for enhanced understanding and similarity search.

## Features

- **Semantic Similarity Search**: Find conceptually similar knowledge using embeddings
- **Graph + Semantic Reasoning**: Best of both worlds - explainable and semantic
- **Automatic Fallback**: Uses TF-IDF when sentence-transformers unavailable
- **Lightweight**: Optional 22MB model or pure Python TF-IDF
- **CPU-Friendly**: No GPU required

## Installation

### Basic Install
```bash
pip install -e .
```

### With Semantic Embeddings (Recommended)
```bash
pip install -e ".[embeddings]"
```

## Quick Start

### Basic Usage

```python
from sutra_hybrid import HybridAI

# Initialize (automatically uses best available embedding)
ai = HybridAI()

# Learn new knowledge
concept_id = ai.learn("Photosynthesis converts sunlight into chemical energy")

# Semantic search
results = ai.semantic_search("How do plants make energy?", top_k=3)
for concept_id, similarity in results:
    concept = ai.get_concept(concept_id)
    print(f"{similarity:.2f}: {concept.content}")

# Get statistics
stats = ai.get_stats()
print(f"Total concepts: {stats['total_concepts']}")
print(f"Embedding provider: {stats['embedding_provider']}")
```

### Force TF-IDF Mode

```python
# Use TF-IDF instead of semantic embeddings
ai = HybridAI(use_semantic=False)
```

✅ **Persistence (Updated)**: TF-IDF vectorizer state is now fully persisted via a pickled sklearn TfidfVectorizer (embeddings.json + tfidf_vectorizer.pkl). On load, the vectorizer is restored; if restoration fails, it will refit on demand. Semantic embeddings remain recommended for best quality.

## Persistence Format (Updated)

- Concepts and associations are stored as JSON. Association keys are serialized with ":" between source and target IDs. The loader accepts both legacy "|" and current ":" separators.
- Embeddings are stored as JSON arrays; TF-IDF vectorizer state is stored as a separate pickle file tfidf_vectorizer.pkl.

## Architecture

### Embedding Providers

1. **SemanticEmbedding** (Recommended)
   - Uses `sentence-transformers`
   - 384-dimensional embeddings
   - 22MB model size
   - High-quality semantic understanding

2. **TfidfEmbedding** (Fallback)
   - Scikit-learn based
   - 100-dimensional vectors
   - No model downloads
   - Good for keyword matching

### Components

```
sutra-hybrid/
├── core.py              # HybridAI main class
├── embeddings/
│   ├── base.py         # Abstract interface
│   ├── semantic.py     # Sentence-transformers
│   └── tfidf.py        # TF-IDF fallback
└── storage/
    └── persistence.py  # Save/load with embeddings
```

## Performance

| Operation | Time | Notes |
|-----------|------|-------|
| Learn (semantic) | ~50ms | With embedding generation |
| Learn (TF-IDF) | ~5ms | Lightweight fallback |
| Search (1000 concepts) | ~20ms | In-memory cosine similarity |
| Storage | ~100ms | JSON serialization |

## Requirements

### Core Requirements
- sutra-core >= 1.0.0
- numpy >= 1.24.0
- scikit-learn (for TF-IDF)

### Optional Requirements
- sentence-transformers >= 2.2.2 (for semantic embeddings)

## Development

```bash
# Run tests
pytest tests/ -v

# Format code
black sutra_hybrid/
isort sutra_hybrid/

# Type checking
mypy sutra_hybrid/
```

## License

MIT License - see LICENSE file for details.

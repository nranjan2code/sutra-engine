# Quick Start

## Core Package (Graph Reasoning)

### Basic Learning

```python
from sutra_core import AdaptiveLearner, AssociationExtractor, Concept
from collections import defaultdict

# Initialize data structures
concepts = {}
associations = {}
word_to_concepts = defaultdict(set)
concept_neighbors = defaultdict(set)

# Create extractor and learner
extractor = AssociationExtractor(
    concepts, word_to_concepts, concept_neighbors, associations
)
learner = AdaptiveLearner(concepts, extractor)

# Learn knowledge
concept_id = learner.learn_adaptive(
    "Photosynthesis converts sunlight into chemical energy",
    source="biology_textbook"
)

print(f"Created concept: {concept_id}")
print(f"Total concepts: {len(concepts)}")
print(f"Total associations: {len(associations)}")
```

### Accessing Concepts

```python
# Get concept by ID
concept = concepts[concept_id]

print(f"Content: {concept.content}")
print(f"Strength: {concept.strength}")  # 1.0 initially
print(f"Access count: {concept.access_count}")

# Strengthen concept by accessing
concept.access()
print(f"Strength after access: {concept.strength}")  # 1.02
```

### Finding Related Concepts

```python
from sutra_core.utils import extract_words

# Extract words from query
query = "How do plants make energy?"
words = extract_words(query)

# Find concepts containing these words
related_concepts = set()
for word in words:
    if word in word_to_concepts:
        related_concepts.update(word_to_concepts[word])

print(f"Found {len(related_concepts)} related concepts")
```

### Core Maintenance (ReasoningEngine)

```python
from sutra_core import ReasoningEngine

ai = ReasoningEngine()
# ... learn/ask ...

# Quick health snapshot
health = ai.get_health_snapshot()
print(health)

# Decay/prune stale concepts and low-confidence, long-inactive associations
ai.decay_and_prune(
    concept_decay_after_days=14,
    concept_remove_after_days=90,
    min_strength_to_keep=1.0,
    association_remove_after_days=90,
    min_association_confidence_to_keep=0.2,
)
```

## Hybrid Package (With Embeddings)

### Initialize

```python
from sutra_hybrid import HybridAI

# With semantic embeddings (downloads 22MB model)
ai = HybridAI(use_semantic=True, storage_path="./knowledge")

# Or with TF-IDF only (no downloads)
ai = HybridAI(use_semantic=False, storage_path="./knowledge")
```

### Learn Knowledge

```python
# Learn single item
concept_id = ai.learn(
    "Neural networks learn patterns from training data",
    source="ml_textbook"
)

# Learn multiple items
knowledge_items = [
    "Deep learning uses multiple neural network layers",
    "Gradient descent optimizes network weights",
    "Backpropagation calculates error gradients",
]

for item in knowledge_items:
    ai.learn(item)

print(f"Total concepts: {len(ai.concepts)}")
```

### Semantic Search

```python
# Search for similar concepts
results = ai.semantic_search(
    "How do neural networks learn?",
    top_k=5,
    threshold=0.5
)

for concept_id, similarity in results:
    concept = ai.get_concept(concept_id)
    print(f"[{similarity:.3f}] {concept.content}")
```

### Get System Statistics

```python
stats = ai.get_stats()

print(f"Concepts: {stats['total_concepts']}")
print(f"Associations: {stats['total_associations']}")
print(f"Embeddings: {stats['total_embeddings']}")
print(f"Provider: {stats['embedding_provider']}")
print(f"Dimension: {stats['embedding_dimension']}")
print(f"Avg strength: {stats['average_strength']:.2f}")
```

### Persistence

```python
# Save to disk
ai.save()

# Load from disk (automatic on init if data exists)
ai2 = HybridAI(use_semantic=False, storage_path="./knowledge")
print(f"Loaded {len(ai2.concepts)} concepts")
```

## API Package (REST Server)

### Start Server

```bash
# Development mode
python -m sutra_api.main

# Or with uvicorn
uvicorn sutra_api.main:app --reload --port 8000

# Production mode
uvicorn sutra_api.main:app --host 0.0.0.0 --port 8000 --workers 4
```

### Learn via API

```bash
# Single learning
curl -X POST "http://localhost:8000/learn" \
  -H "Content-Type: application/json" \
  -d '{
    "content": "Python is a high-level programming language",
    "source": "documentation"
  }'

# Batch learning
curl -X POST "http://localhost:8000/learn/batch" \
  -H "Content-Type: application/json" \
  -d '{
    "items": [
      {"content": "JavaScript runs in web browsers"},
      {"content": "SQL queries databases"}
    ]
  }'
```

### Search via API

```bash
curl -X POST "http://localhost:8000/semantic-search" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "programming languages",
    "top_k": 5,
    "threshold": 0.5
  }'
```

### Get Concept

```bash
curl "http://localhost:8000/concepts/{concept_id}"
```

### System Management

```bash
# Get statistics
curl "http://localhost:8000/stats"

# Save knowledge
curl -X POST "http://localhost:8000/save"

# Health check
curl "http://localhost:8000/health"

# Reset system (deletes all data)
curl -X DELETE "http://localhost:8000/reset"
```

### Interactive Documentation

Open browser to:
- Swagger UI: http://localhost:8000/docs
- ReDoc: http://localhost:8000/redoc

## Python API Client

```python
import requests

BASE_URL = "http://localhost:8000"

# Learn knowledge
response = requests.post(
    f"{BASE_URL}/learn",
    json={
        "content": "Machine learning finds patterns in data",
        "source": "textbook"
    }
)
data = response.json()
concept_id = data["concept_id"]

# Search
response = requests.post(
    f"{BASE_URL}/semantic-search",
    json={
        "query": "data patterns",
        "top_k": 3,
        "threshold": 0.3
    }
)
results = response.json()

for result in results["results"]:
    print(f"{result['similarity']:.3f}: {result['content']}")
```

## Configuration

### Environment Variables

```bash
# API configuration
export SUTRA_HOST="0.0.0.0"
export SUTRA_PORT=8000
export SUTRA_STORAGE_PATH="./api_knowledge"

# AI configuration
export SUTRA_USE_SEMANTIC_EMBEDDINGS=true

# Logging
export SUTRA_LOG_LEVEL="INFO"
```

### Programmatic Configuration

```python
from sutra_api import settings

# Modify settings before starting
settings.use_semantic_embeddings = False
settings.storage_path = "/data/knowledge"
settings.log_level = "DEBUG"
```

## Performance Tips

1. **Use semantic embeddings for quality**
   ```python
   ai = HybridAI(use_semantic=True)  # Better similarity
   ```

2. **Use TF-IDF for speed**
   ```python
   ai = HybridAI(use_semantic=False)  # Faster, no downloads
   ```

3. **Batch operations when possible**
   ```python
   # Faster than individual learns
   for item in items:
       ai.learn(item)
   ```

4. **Save periodically**
   ```python
   if len(ai.concepts) % 1000 == 0:
       ai.save()
   ```

5. **Monitor memory usage**
   ```python
   stats = ai.get_stats()
   if stats['total_concepts'] > 100000:
       # Consider cleanup or archiving
       pass
   ```

## Next Steps

- [Architecture Overview](architecture/overview.md)
- [Package Documentation](packages/sutra-core.md)
- [API Reference](api/endpoints.md)
- [Detailed Tutorials](tutorials/learning.md)

# Python Client for Sutra Engine

The Sutra Engine provides a high-performance TCP binary protocol. To interface with it from Python, use the `sutra-engine-client` package (or the provided `lib/client.py`).

## Installation

```bash
# Recommended: Using pip (if published)
pip install sutra-engine-client

# Or manually using the provided lib
cp -r lib/client.py your_project/
```

## Basic Usage

### Connecting to the Engine

```python
from sutra_engine_client import SutraClient

# Initialize client (default port 50051)
client = SutraClient(host="localhost", port=50051)
```

### Ingesting Knowledge

The `learn` operation is the primary way to feed the graph.

```python
# Simple ingestion
concept_id = client.learn("The capital of France is Paris.")

# Advanced ingestion with options
concept_id = client.learn(
    "Quantum entanglement is a physical phenomenon.",
    generate_embedding=True,
    extract_associations=True
)
```

### Querying & Reasoning

#### Semantic Search
Find concepts similar to a query.

```python
results = client.search("What is Paris?", limit=5)
for res in results:
    print(f"Match: {res.content} (Confidence: {res.confidence})")
```

#### Graph Navigation
Traverse the graph of associations.

```python
# Get directly related concepts
neighbors = client.get_neighbors(concept_id)
for neighbor in neighbors:
    print(f"Related: {neighbor.content}")
```

## Advanced Configuration

### Connection Pooling
For high-concurrency applications, the client supports pooling.

```python
client = SutraClient("localhost:50051", pool_size=10)
```

### Authentication
If the engine is running in secure mode (`SUTRA_SECURE_MODE=true`), you must provide a token or HMAC secret.

```python
client = SutraClient("localhost:50051", secret="your-hmac-secret")
```

## API Summary

| Method | Description |
|--------|-------------|
| `learn(content, **opts)` | Ingest text and return Concept ID. |
| `get(concept_id)` | Retrieve full concept data. |
| `search(query, limit=5)` | Semantic vector search. |
| `get_neighbors(id)` | Get connected concepts. |
| `delete(id)` | Remove concept from memory. |
| `stats()` | Get engine statistics. |

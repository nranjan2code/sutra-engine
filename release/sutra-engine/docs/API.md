# Sutra Engine API Reference

This document outlines the API capabilities of the Sutra Engine. The engine exposes a high-performance **binary TCP protocol**. While most users will use a wrapper library (e.g., Python `sutra-engine-client`), the underlying capability set is defined here.

## Core Concepts

*   **Concept**: An atomic unit of knowledge (node). Contains content, vector embedding, and metadata.
*   **Association**: A directed link between concepts (edge). Can be Semantic, Temporal, Causal, or Hierarchical.
*   **Memory**: The graph of all concepts and associations.

## Protocol Overview

- **Transport**: TCP
- **Serialization**: Custom MessagePack / Bincode
- **Architecture**: Request/Response
- **Default Port**: 50051

---

## Methods

### 1. `learn_concept`
Ingests a new piece of information. The engine handles embedding generation (if configured), deduplication, and graph insertion.

*   **Input**: `text` (String), `options` (Dict)
*   **Output**: `concept_id` (String)

```json
// Approximate JSON representation of payload
{
  "content": "The Eiffel Tower is located in Paris.",
  "options": {
    "generate_embedding": true,
    "extract_associations": true
  }
}
```

### 2. `get_concept`
Retrieves a single concept by its unique ID.

*   **Input**: `concept_id` (String)
*   **Output**: Concept Object

```json
{
  "id": "abc12345...",
  "content": "The Eiffel Tower is located in Paris.",
  "confidence": 1.0,
  "associations": [...]
}
```

### 3. `search_concepts`
Performs vector-based semantic search or exact text matching.

*   **Input**: `query` (String), `limit` (Int)
*   **Output**: List of matches

### 4. `get_neighbors`
Retrieves directly connected concepts from the graph.

*   **Input**: `concept_id` (String)
*   **Output**: List of Concept Objects

### 5. `delete_concept`
Removes a concept and its incident edges from the graph.

*   **Input**: `concept_id` (String)
*   **Output**: Success (Bool)

### 6. `stats`
Returns system health and storage statistics.

*   **Output**:
    *   `concept_count`
    *   `edge_count`
    *   `vector_index_size`

---

## Error Codes

| Code | Name | Description |
|------|------|-------------|
| `0x00` | `SUCCESS` | Operation completed successfully. |
| `0x01` | `NOT_FOUND` | Concept ID does not exist. |
| `0x02` | `AUTH_FAILED` | Invalid credentials (if secure mode on). |
| `0x03` | `INTERNAL` | Server-side storage error. |

---

## Client Libraries

### Python
Use the `sutra-storage-client-tcp` package (often distributed as `sutra-engine-client`).

```python
from sutra_storage_client_tcp import TcpStorageAdapter
client = TcpStorageAdapter("localhost:50051")
```

### Rust
Use the `sutra-storage-client` crate.

```rust
use sutra_storage_client::StorageClient;
let client = StorageClient::connect("tcp://localhost:50051").await?;
```

# API Reference: Sutra Binary Protocol

Sutra Engine uses a custom binary protocol over TCP for maximum performance. This protocol uses a simple **Length-Prefix** framing followed by **MessagePack** serialization.

---

## 🛰 Protocol Overview

### Framing
Each message (Request and Response) is prefixed with a 4-byte big-endian integer representing the length of the MessagePack payload.

`[4 bytes: Length] [N bytes: MessagePack Payload]`

### Port
Default: `50051`

---

## 📥 Storage Requests

Requests are sent as a MessagePack Map where the key is the variant name.

### 1. `LearnConceptV2`
High-level ingestion with automatic embedding and association extraction.

**Payload:**
```json
{
  "LearnConceptV2": {
    "content": "String",
    "options": {
      "generate_embedding": "Boolean",
      "extract_associations": "Boolean",
      "min_association_confidence": "Float",
      "max_associations_per_concept": "Integer",
      "strength": "Float",
      "confidence": "Float"
    }
  }
}
```

### 2. `TextSearch`
Semantic search using natural language.

**Payload:**
```json
{
  "TextSearch": {
    "query": "String",
    "limit": "Integer"
  }
}
```

### 3. `GetNeighbors`
Get directly associated concepts in the graph.

**Payload:**
```json
{
  "GetNeighbors": {
    "concept_id": "String (Hex)"
  }
}
```

### 4. `GetStats`
Get engine health and performance metrics.

**Payload:**
```json
{
  "GetStats": null
}
```

---

## 📤 Storage Responses

Responses follow the same variant-based map structure.

### 1. `LearnConceptV2Ok`
```json
{
  "LearnConceptV2Ok": {
    "concept_id": "String (Hex)"
  }
}
```

### 2. `TextSearchOk`
```json
{
  "TextSearchOk": {
    "results": [
      ["ConceptID_Hex", "SimilarityScore_Float"],
      ["...", 0.95]
    ]
  }
}
```

### 3. `StatsOk`
```json
{
  "StatsOk": {
    "concepts": "Integer",
    "edges": "Integer",
    "vectors": "Integer",
    "written": "Integer",
    "uptime_seconds": "Integer"
  }
}
```

### 4. `Error`
Returned whenever a request fails.
```json
{
  "Error": {
    "message": "String"
  }
}
```

---

## 🛠 CURL & HTTP Support

While Sutra Engine is optimized for TCP, you can use a simple **HTTP Proxy** or the `sutra-cli` (if installed) to interact via CURL.

### Rest Bridge (Experimental)
If you are running the engine in a containerized environment with the REST sidecar enabled:

**Ingest Knowledge:**
```bash
curl -X POST http://localhost:8080/learn \
  -H "Content-Type: application/json" \
  -d '{"content": "Sutra is fast."}'
```

**Search:**
```bash
curl -X GET "http://localhost:8080/search?q=speed&limit=5"
```

---

## ⚙️ Standard Object Types

### `ConceptID`
A 16-character Hex string (e.g., `50039b83b9425364`).

### `SimilarityScore`
A floating point value between `0.0` and `1.0`. Higher is more similar.

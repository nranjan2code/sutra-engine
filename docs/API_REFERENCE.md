# API Reference: Sutra Binary Protocol

Sutra Engine uses a custom binary protocol over TCP for maximum performance. This protocol uses a simple **Length-Prefix** framing followed by `bincode` serialization.

---

## 🛰 Protocol Overview

### Framing
Each message (Request and Response) is prefixed with a 4-byte big-endian integer representing the length of the `bincode` payload.

`[4 bytes: Length] [N bytes: bincode Payload]`

### Port
Default: `50051`

---

## 📥 Storage Requests

Requests are sent as a `bincode`-serialized enum where the variant name is the tag. Most requests now support a `namespace` parameter for multi-tenant isolation.

### 1. `LearnConceptV2`
High-level ingestion with automatic embedding and association extraction.

**Payload:**
```json
{
  "LearnConceptV2": {
    "namespace": "Option<String>",
    "content": "String",
    "options": {
      "generate_embedding": "Boolean",
      "extract_associations": "Boolean",
      "min_association_confidence": "Float",
      "max_associations_per_concept": "Integer",
      "strength": "Float",
      "confidence": "Float",
      "attributes": "Map<String, String>"
    }
  }
}
```

### 2. `QueryConcept`
Semantic search and metadata retrieval for a specific concept.

**Payload:**
```json
{
  "QueryConcept": {
    "namespace": "Option<String>",
    "concept_id": "String (Hex)"
  }
}
```

### 3. `DeleteConcept`
Permanently remove a concept and its associations from a namespace.

**Payload:**
```json
{
  "DeleteConcept": {
    "namespace": "String",
    "id": "String (Hex)"
  }
}
```

### 4. `ListRecent`
Retrieve the most recently learned concepts in a namespace.

**Payload:**
```json
{
  "ListRecent": {
    "namespace": "String",
    "limit": "Integer"
  }
}
```

### 5. `ClearCollection`
Reset an entire namespace, deleting all concepts and vectors.

**Payload:**
```json
{
  "ClearCollection": {
    "namespace": "String"
  }
}
```

### 6. `GetStats`
Get engine health and performance metrics.

**Payload:**
```json
{
  "GetStats": { "namespace": "Option<String>" }
}
```

### 7. `Flush` & `HealthCheck`
Unit variants sent as simple strings: `"Flush"` and `"HealthCheck"`.

---

## 📤 Storage Responses

### 1. `LearnConceptV2Ok`
```json
{
  "LearnConceptV2Ok": {
    "concept_id": "String (Hex)"
  }
}
```

### 2. `StatsOk`
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

### 3. `FlushOk`
```json
"FlushOk" or { "FlushOk": true }
```

---

## ⚙️ Standard Object Types

### `ConceptID`
A 32-character Hex string (e.g., `dcbf7561e84be97e383322b99bb343e2`), representing a 1128-bit (16-byte) MD5 hash.

### `SimilarityScore`
A floating point value between `0.0` and `1.0`. Higher is more similar.

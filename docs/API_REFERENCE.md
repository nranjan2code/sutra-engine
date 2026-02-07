# API Reference: Sutra Binary Protocol

Sutra Engine uses a custom binary protocol over TCP for maximum performance. This protocol uses a simple **Length-Prefix** framing followed by MessagePack serialization.

---

## 🛰 Protocol Overview

Sutra Storage supports a **Dual-Protocol Interface** on the same port, using protocol sniffing.

### 1. Binary Protocol (Machine-to-Machine)
- **Format**: 4-byte Big-Endian Length + MessagePack Payload.
- **Trigger**: First byte is `0x00` (length < 255) in most cases, or strictly formatted.
- **Port**: Default `50051`.

### 2. Natural Language Protocol
- **Format**: Raw text strings terminated by newline (`\n`).
- **Trigger**: First byte is != `0x00`.
- **Usage**: `nc localhost 9000`
- **Port**: Default `50051` (or `9000` convention).

---

## 📥 Storage Requests

Requests are sent as a MessagePack map where the variant name is the tag. Most requests support a `namespace` parameter for multi-tenant isolation.

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
Retrieve a specific record by ID.

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
Permanently remove a record and its edges from a namespace.

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
Retrieve the most recently ingested records in a namespace.

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
Reset an entire namespace, deleting all records and vectors.

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

### 8. `Subscribe`
Subscribe to push notifications when records matching a filter are created.

**Payload:**
```json
{
  "Subscribe": {
    "filter": {
      "semantic_type": "Option<String>",
      "temporal_after": "Option<i64>",
      "temporal_before": "Option<i64>",
      "has_causal_relation": "Boolean",
      "min_confidence": "Float",
      "required_terms": ["String"]
    },
    "callback_addr": "String (host:port or empty for log-only)"
  }
}
```

### 9. `Unsubscribe`
Remove a subscription by ID.

**Payload:**
```json
{
  "Unsubscribe": {
    "subscription_id": "String"
  }
}
```

### 10. `ListSubscriptions`
List all active subscriptions. Unit variant: `"ListSubscriptions"`.

### 11. `CreateGoal`
Create a trigger with a condition and action.

**Payload:**
```json
{
  "CreateGoal": {
    "namespace": "Option<String>",
    "description": "String",
    "condition": "String (e.g. 'count above 100', 'associations above 50', or content match)",
    "action": "String (e.g. 'notify: message', 'learn: content')",
    "priority": "Integer (0-255)"
  }
}
```

### 12. `ListGoals`
List all triggers, optionally filtered by namespace.

**Payload:**
```json
{
  "ListGoals": {
    "namespace": "Option<String>"
  }
}
```

### 13. `CancelGoal`
Cancel (delete) a trigger by ID.

**Payload:**
```json
{
  "CancelGoal": {
    "namespace": "Option<String>",
    "goal_id": "String"
  }
}
```

### 14. `ProvideFeedback`
Submit accept/reject feedback for search results to adjust record strengths.

**Payload:**
```json
{
  "ProvideFeedback": {
    "namespace": "Option<String>",
    "query_id": "String",
    "result_concept_ids": ["String (Hex)"],
    "accepted": ["Boolean"],
    "ranking": "Option<[Integer]>"
  }
}
```

### 15. `GetAutonomyStats`
Get background job status and statistics. Unit variant: `"GetAutonomyStats"`.

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

### 4. `SubscribeOk`
```json
{
  "SubscribeOk": {
    "subscription_id": "String"
  }
}
```

### 5. `UnsubscribeOk`
```json
"UnsubscribeOk"
```

### 6. `ListSubscriptionsOk`
```json
{
  "ListSubscriptionsOk": {
    "subscriptions": [
      { "id": "String", "filter": { ... }, "callback_addr": "String" }
    ]
  }
}
```

### 7. `CreateGoalOk`
```json
{
  "CreateGoalOk": {
    "goal_id": "String (Hex)"
  }
}
```

### 8. `ListGoalsOk`
```json
{
  "ListGoalsOk": {
    "goals": [
      { "goal_id": "String", "description": "String", "status": "String", "priority": "Integer" }
    ]
  }
}
```

### 9. `CancelGoalOk`
```json
"CancelGoalOk"
```

### 10. `ProvideFeedbackOk`
```json
{
  "ProvideFeedbackOk": {
    "adjustments": "Integer"
  }
}
```

### 11. `AutonomyStatsOk`
```json
{
  "AutonomyStatsOk": {
    "stats": "String (JSON)"
  }
}
```

---

## ⚙️ Standard Object Types

### `ConceptID`
A 32-character Hex string (e.g., `dcbf7561e84be97e383322b99bb343e2`), representing a 128-bit (16-byte) MD5 hash.

### `SimilarityScore`
A floating point value between `0.0` and `1.0`. Higher is more similar.

---

## 🗣️ Natural Language Commands

The following NL commands are available:

| Command | Example | Maps To |
|---------|---------|---------|
| `status` / `engine status` | `echo "status" \| nc localhost 9000` | `GetAutonomyStats` |
| `set goal: <desc>` / `goal: <desc>` | `echo "set goal: track new records" \| nc localhost 9000` | `CreateGoal` |
| `list goals` / `goals` | `echo "list goals" \| nc localhost 9000` | `ListGoals` |
| `subscribe to <term>` / `watch for <term>` | `echo "subscribe to Rust" \| nc localhost 9000` | `Subscribe` |

# Test Results

## 2026-02-04 11:13:58 IST

**Command**
```bash
cargo test --workspace
```

**Summary**
- Status: PASS
- Ignored: `embedding_client::tests::test_generate_embedding` (requires external embedding service)

**Real-Scenario Coverage Added**
- Natural language semantic pipeline (causal classification + embedding)
- Persistence + recovery (attributes + semantic metadata)
- Concurrent read/write mixed load
- TCP protocol end-to-end (MessagePack framing + LearnWithEmbedding + QueryConcept)
- HMAC auth (plaintext) + TLS auth handshake
- WAL truncation handling (partial entry recovery)
- Configurable load test via env vars

**Notes**
- Storage snapshot format updated to version 3 to persist attributes + semantic metadata.

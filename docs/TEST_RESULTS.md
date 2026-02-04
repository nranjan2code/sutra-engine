# Test Results

## 2026-02-04 10:46:58 IST

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

**Notes**
- Storage snapshot format updated to version 3 to persist attributes + semantic metadata.

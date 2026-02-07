# Test Results

## 2026-02-04 11:16:52 IST

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
- Storage format compatibility (v2 snapshot loads under v3 engine)
- Configurable load test via env vars

**Background Maintenance**
- `SemanticType::Goal` (variant 9) added and integrated.
- 8 new modules under `crates/storage/src/autonomy/`.
- 8 new `StorageRequest` and 8 new `StorageResponse` variants for background job features.
- NL parser extended with `status`, `set goal`, `list goals`, `subscribe to` commands.
- Secure TCP server updated with auth categorization for all new request types.

**Notes**
- Storage snapshot format updated to version 3 to persist attributes + semantic metadata.
- Background maintenance enabled by default (`SUTRA_AUTONOMY=true`).

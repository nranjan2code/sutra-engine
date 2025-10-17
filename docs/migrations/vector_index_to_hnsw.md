# Migrations: VectorIndex → Native HNSW

This release removes the Python VectorIndex and moves semantic search into Rust storage.

Breaking changes:
- QueryProcessor no longer accepts vector_index
- ReasoningEngine no longer constructs or rebuilds Python vector index
- All similarity calls route to storage.vector_search(np.ndarray)

What to update:
- Remove any imports/usages of indexing/vector_search.py
- Ensure all query embeddings are np.float32 arrays
- If you had custom vector index logic, port it or call storage.vector_search

Verification:
- Run demos (demo_simple.py) and new tests (pytest)

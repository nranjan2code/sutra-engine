# Storage (sutra-storage)

Native Rust storage with HNSW vector search.

Highlights:
- ConcurrentStorage: single-writer log + reconciler + memory-mapped snapshot
- Vector store: in-memory vectors; HNSW index built dynamically at query time
- Python bindings via PyO3 (PyConcurrentStorage)

Key APIs (Python):
- learn_concept(concept_id, content, embedding: np.ndarray, strength=1.0, confidence=1.0)
- learn_association(source_id, target_id, assoc_type: int, confidence: float)
- query_concept(concept_id) -> dict | None
- get_neighbors(concept_id) -> list[str]
- find_path(start_ids, target_ids, max_depth, num_paths, query) -> List[ReasoningPath]
- vector_search(query_vector: np.ndarray, k=10) -> List[Tuple[str, float]]
- stats() -> Dict
- flush()

Notes:
- query_vector must be np.float32 array
- vector_dimension is configured at adapter initialization

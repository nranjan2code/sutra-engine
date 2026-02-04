"""
Vector indexing and search for fast semantic retrieval.

NOTE: As of the production refactor, vector indexing is handled internally
by ConcurrentStorage using native Rust HNSW. This module is deprecated.

Vector search is now accessed via:
    storage.vector_search(query_embedding, k=10)
"""

__all__ = []

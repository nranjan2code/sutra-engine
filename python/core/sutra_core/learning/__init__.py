"""
Learning components for the Sutra AI system.

This package contains:
- Association extraction and pattern matching
- Adaptive focus learning algorithms
- Knowledge integration strategies
- Batch embedding generation with MPS support
- Parallel association extraction for high performance
- Entity caching for LLM-based extraction (Phase 10)
"""

from .adaptive import AdaptiveLearner
from .associations import AssociationExtractor
from .associations_parallel import ParallelAssociationExtractor
from .entity_cache import EntityCache

# Embeddings (optional, requires torch and sentence-transformers)
try:
    from .embeddings import EmbeddingBatchProcessor, create_embedding_processor
    __all_embeddings__ = ["EmbeddingBatchProcessor", "create_embedding_processor"]
except ImportError:
    __all_embeddings__ = []

__all__ = [
    "AssociationExtractor",
    "ParallelAssociationExtractor",
    "AdaptiveLearner",
    "EntityCache",
] + __all_embeddings__

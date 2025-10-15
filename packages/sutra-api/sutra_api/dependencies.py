"""
Dependency injection for FastAPI endpoints.

Provides shared instances and dependency functions.
"""

import time
from typing import Optional

from sutra_core.graph.concepts import AssociationType
from sutra_core.reasoning import ReasoningEngine
from sutra_hybrid import HybridAI

from .config import settings

# Global instances
_ai_instance: Optional[HybridAI] = None
_reasoner_instance: Optional[ReasoningEngine] = None
_start_time: float = time.time()


def get_ai() -> HybridAI:
    """
    Get or create the HybridAI instance.

    Returns:
        HybridAI instance
    """
    global _ai_instance

    if _ai_instance is None:
        _ai_instance = HybridAI(
            use_semantic=settings.use_semantic_embeddings,
            storage_path=settings.storage_path,
        )

    return _ai_instance


def get_uptime() -> float:
    """
    Get service uptime in seconds.

    Returns:
        Uptime in seconds
    """
    return time.time() - _start_time


def get_reasoner() -> ReasoningEngine:
    """Get or create a ReasoningEngine bound to the HybridAI knowledge."""
    global _reasoner_instance

    ai = get_ai()

    if _reasoner_instance is None:
        # Map configured compositional type to AssociationType
        type_map = {
            "compositional": AssociationType.COMPOSITIONAL,
            "hierarchical": AssociationType.HIERARCHICAL,
            "semantic": AssociationType.SEMANTIC,
            "causal": AssociationType.CAUSAL,
            "temporal": AssociationType.TEMPORAL,
        }
        link_type = type_map.get(
            str(getattr(settings, "compositional_type", "compositional")).lower(),
            AssociationType.COMPOSITIONAL,
        )

        # Create a reasoning engine with caching enabled and optional TTL
        _reasoner_instance = ReasoningEngine(
            enable_caching=True,
            cache_ttl_seconds=settings.cache_ttl_seconds,
            enable_central_links=settings.compositional_links,
            central_link_confidence=settings.compositional_confidence,
            central_link_type=link_type,
        )

    # Bind engine data structures to HybridAI's live state and rebuild indexes
    _reasoner_instance.concepts = ai.concepts
    _reasoner_instance.associations = ai.associations
    _reasoner_instance.concept_neighbors = ai.concept_neighbors
    _reasoner_instance.word_to_concepts = ai.word_to_concepts
    # Rebuild indexes to ensure consistency
    _reasoner_instance._rebuild_indexes()

    return _reasoner_instance


def reset_ai() -> None:
    """Reset the AI instance (for testing or reset operations)."""
    global _ai_instance, _reasoner_instance
    _ai_instance = None
    _reasoner_instance = None

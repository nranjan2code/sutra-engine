"""
Global configuration for associative memory settings.
Centralizes retrieval parameters, working memory, weights, and persistence defaults.
"""
from __future__ import annotations

from typing import Dict
from dataclasses import dataclass, field
from enum import Enum


class MemoryType(Enum):
    """Different types of biological memory with varying retention characteristics"""
    EPHEMERAL = "ephemeral"
    SHORT_TERM = "short_term"
    MEDIUM_TERM = "medium_term"
    LONG_TERM = "long_term"
    CORE_KNOWLEDGE = "core_knowledge"


class AssociationType(Enum):
    """Types of associations between knowledge concepts"""
    SEMANTIC = "semantic"
    TEMPORAL = "temporal"
    CAUSAL = "causal"
    ANALOGICAL = "analogical"
    HIERARCHICAL = "hierarchical"
    CONTRADICTORY = "contradictory"
    CONTEXTUAL = "contextual"


@dataclass
class Settings:
    # Persistence
    BASE_PATH: str = "knowledge_store"
    WORKSPACE_ID: str = "core"
    AUDIT_ENABLED: bool = True

    # Retrieval defaults
    RETRIEVAL_HOPS: int = 1
    RETRIEVAL_ALPHA: float = 0.5
    RETRIEVAL_TOPK_NEIGHBORS: int = 8

    # Working memory
    WORKING_MEMORY_SIZE: int = 256
    WORKING_MEMORY_BOOST: float = 0.1

    # Weighting
    MEMORY_TYPE_WEIGHTS: Dict[MemoryType, float] = field(default_factory=lambda: {
        MemoryType.EPHEMERAL: 0.8,
        MemoryType.SHORT_TERM: 1.0,
        MemoryType.MEDIUM_TERM: 1.1,
        MemoryType.LONG_TERM: 1.25,
        MemoryType.CORE_KNOWLEDGE: 1.5,
    })
    ASSOCIATION_TYPE_WEIGHTS: Dict[AssociationType, float] = field(default_factory=lambda: {
        AssociationType.SEMANTIC: 1.0,
        AssociationType.HIERARCHICAL: 0.9,
        AssociationType.TEMPORAL: 0.7,
        AssociationType.CAUSAL: 1.1,
        AssociationType.ANALOGICAL: 0.9,
        AssociationType.CONTEXTUAL: 0.8,
        AssociationType.CONTRADICTORY: 0.2,
    })


settings = Settings()
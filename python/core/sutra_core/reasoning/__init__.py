"""
Advanced reasoning algorithms for Sutra AI.

This package implements sophisticated AI reasoning capabilities:
- Spreading activation with path-finding
- Multi-hop inference chains
- Confidence propagation and scoring
- Multi-Path Plan Aggregation (MPPA)
- Query processing and answer generation
- Query planning and decomposition
- Contradiction resolution
"""

from .contradictions import (
    ConflictType,
    Contradiction,
    ContradictionResolver,
    ResolutionStrategy,
)
from .engine import ReasoningEngine
from .mppa import MultiPathAggregator
from .paths import PathFinder
from .planner import QueryPlan, QueryPlanner, QueryStep, QueryType
from .query import QueryProcessor

__all__ = [
    "ReasoningEngine",
    "MultiPathAggregator",
    "PathFinder",
    "QueryProcessor",
    "QueryPlanner",
    "QueryPlan",
    "QueryStep",
    "QueryType",
    "ContradictionResolver",
    "Contradiction",
    "ConflictType",
    "ResolutionStrategy",
]

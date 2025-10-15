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

from .engine import ReasoningEngine
from .mppa import MultiPathAggregator
from .paths import PathFinder
from .query import QueryProcessor
from .planner import QueryPlanner, QueryPlan, QueryStep, QueryType
from .contradictions import (
    ContradictionResolver,
    Contradiction,
    ConflictType,
    ResolutionStrategy,
)

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

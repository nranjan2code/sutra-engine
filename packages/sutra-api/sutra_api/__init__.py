"""
Sutra AI API - REST API service for graph-based reasoning.

Provides HTTP endpoints for learning, reasoning, and knowledge management.
"""

__version__ = "1.0.0"

from .config import settings
from .dependencies import get_ai, get_uptime, reset_ai
from .models import (
    AssociationType,
    BatchLearnRequest,
    BatchLearnResponse,
    ConceptDetail,
    ErrorResponse,
    HealthResponse,
    LearnRequest,
    LearnResponse,
    ReasoningPath,
    ReasonRequest,
    ReasonResponse,
    SearchResult,
    SemanticSearchRequest,
    SemanticSearchResponse,
    SystemStats,
)

__all__ = [
    # Version
    "__version__",
    # Config
    "settings",
    # Dependencies
    "get_ai",
    "get_uptime",
    "reset_ai",
    # Models
    "AssociationType",
    "LearnRequest",
    "LearnResponse",
    "BatchLearnRequest",
    "BatchLearnResponse",
    "ReasonRequest",
    "ReasonResponse",
    "ReasoningPath",
    "SemanticSearchRequest",
    "SemanticSearchResponse",
    "SearchResult",
    "ConceptDetail",
    "SystemStats",
    "HealthResponse",
    "ErrorResponse",
]

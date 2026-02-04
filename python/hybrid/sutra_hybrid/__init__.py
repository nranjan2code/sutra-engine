"""
Sutra AI - Explainable Real-Time Learning System.

This is the main user-facing interface for Sutra AI.
Core and Storage are internal implementation details.

Key Features:
- Graph-based explainable reasoning
- Semantic embeddings integration
- OpenAI-compatible API
- Real-time learning without retraining
- Full audit trails for compliance
- Multi-strategy reasoning comparison
"""

# Embeddings (optional)
from .embeddings import EmbeddingProvider, SemanticEmbedding, TfidfEmbedding

# Core engine and results
from .engine import SutraAI
from .results import ExplainableResult, LearnResult, MultiStrategyResult

# API (optional - for server deployment)
try:
    from .api import app, create_app

    _api_available = True
except ImportError:
    _api_available = False
    create_app = None
    app = None

__version__ = "2.0.0"

__all__ = [
    # Core
    "SutraAI",
    "ExplainableResult",
    "LearnResult",
    "MultiStrategyResult",
    # Embeddings
    "EmbeddingProvider",
    "SemanticEmbedding",
    "TfidfEmbedding",
    # API (if available)
    "create_app",
    "app",
]

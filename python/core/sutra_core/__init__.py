"""
Sutra AI Core - Graph-based reasoning system.

This is the core package for the Sutra AI system, providing:
- Concept and Association data structures
- Spreading activation reasoning algorithms
- Multi-Path Plan Aggregation (MPPA)
- Adaptive focus learning
- Real-time knowledge integration
- Advanced NLP processing with spaCy
- HNSW vector indexing for O(log N) semantic search
- Batch operations for efficient bulk learning

The system offers a genuine alternative to LLM limitations:
- 100% explainable reasoning paths
- Real-time learning without retraining
- Unlimited persistent memory
- Ultra-efficient CPU-only operation
- Scalable to 100K+ concepts
"""

from .config import (
    ReasoningEngineConfig,
    ReasoningEngineConfigBuilder,
    development_config,
    minimal_config,
    production_config,
)
from .exceptions import (
    AssociationError,
    ConceptError,
    ConfigurationError,
    LearningError,
    StorageError,
    SutraError,
    ValidationError,
)
from .graph import (
    Association,
    AssociationType,
    Concept,
    ReasoningPath,
    ReasoningStep,
)
from .learning import AdaptiveLearner, AssociationExtractor
from .reasoning import MultiPathAggregator, PathFinder, QueryProcessor, ReasoningEngine
from .utils import (
    calculate_word_overlap,
    clean_text,
    extract_words,
    get_association_patterns,
)
from .validation import Validator

# Vector indexing (optional, requires hnswlib)
try:
    from .indexing import VectorIndex

    __all_vector__ = ["VectorIndex"]
except ImportError:
    __all_vector__ = []

# NLP processing (optional, requires spacy)
try:
    from .utils.nlp import TextProcessor

    __all_nlp__ = ["TextProcessor"]
except ImportError:
    __all_nlp__ = []

__version__ = "1.0.0"

__all__ = (
    [
        # Core data structures
        "Concept",
        "Association",
        "AssociationType",
        "ReasoningStep",
        "ReasoningPath",
        # Learning components
        "AssociationExtractor",
        "AdaptiveLearner",
        # Reasoning engine
        "ReasoningEngine",
        "MultiPathAggregator",
        "PathFinder",
        "QueryProcessor",
        # Configuration
        "ReasoningEngineConfig",
        "ReasoningEngineConfigBuilder",
        "minimal_config",
        "development_config",
        "production_config",
        # Utilities
        "extract_words",
        "get_association_patterns",
        "clean_text",
        "calculate_word_overlap",
        # Validation
        "Validator",
        # Exceptions
        "SutraError",
        "ConceptError",
        "AssociationError",
        "LearningError",
        "ValidationError",
        "StorageError",
        "ConfigurationError",
    ]
    + __all_nlp__
    + __all_vector__
)

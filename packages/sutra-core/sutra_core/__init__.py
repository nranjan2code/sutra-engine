"""
Sutra AI Core - Graph-based reasoning system.

This is the core package for the Sutra AI system, providing:
- Concept and Association data structures
- Spreading activation reasoning algorithms
- Multi-Path Plan Aggregation (MPPA)
- Adaptive focus learning
- Real-time knowledge integration
- Advanced NLP processing with spaCy

The system offers a genuine alternative to LLM limitations:
- 100% explainable reasoning paths
- Real-time learning without retraining
- Unlimited persistent memory
- Ultra-efficient CPU-only operation
"""

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

# NLP processing (optional, requires spacy)
try:
    from .utils.nlp import TextProcessor
    
    __all_nlp__ = ["TextProcessor"]
except ImportError:
    __all_nlp__ = []

__version__ = "1.0.0"

__all__ = [
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
] + __all_nlp__

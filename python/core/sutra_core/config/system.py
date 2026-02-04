"""
System Constants and Type Mappings - Single Source of Truth

Eliminates duplication of:
- AssociationType mappings (previously in 3 locations)
- Vector dimensions
- Protocol constants
"""

import logging
from dataclasses import dataclass
from typing import Dict

# Import from graph layer for enum definition
from ..graph.concepts import AssociationType

logger = logging.getLogger(__name__)


# ============================================================================
# Association Type Mapping - Single Source of Truth
# ============================================================================

# This mapping is the ONLY place where Python AssociationType maps to Rust u8
# Previously duplicated in:
# - tcp_adapter.py:162-169
# - tcp_adapter.py:190-197
# - Rust: packages/sutra-storage/src/types.rs:66-72

ASSOCIATION_TYPE_MAP: Dict[AssociationType, int] = {
    AssociationType.SEMANTIC: 0,
    AssociationType.CAUSAL: 1,
    AssociationType.TEMPORAL: 2,
    AssociationType.HIERARCHICAL: 3,
    AssociationType.COMPOSITIONAL: 4,
}

# Reverse mapping for deserialization
ASSOCIATION_INT_TO_TYPE: Dict[int, AssociationType] = {
    v: k for k, v in ASSOCIATION_TYPE_MAP.items()
}


def association_type_to_int(assoc_type: AssociationType) -> int:
    """
    Convert Python AssociationType to Rust protocol integer.

    Args:
        assoc_type: AssociationType enum value

    Returns:
        Integer representation (0-4)

    Raises:
        ValueError: If unknown association type
    """
    try:
        # Handle both enum and string values
        if isinstance(assoc_type, str):
            assoc_type = AssociationType(assoc_type)
        return ASSOCIATION_TYPE_MAP[assoc_type]
    except KeyError:
        raise ValueError(f"Unknown association type: {assoc_type}")


def int_to_association_type(value: int) -> AssociationType:
    """
    Convert Rust protocol integer to Python AssociationType.

    Args:
        value: Integer value (0-4)

    Returns:
        AssociationType enum

    Raises:
        ValueError: If unknown integer value
    """
    try:
        return ASSOCIATION_INT_TO_TYPE[value]
    except KeyError:
        raise ValueError(f"Unknown association type integer: {value}")


# ============================================================================
# System Constants
# ============================================================================

@dataclass(frozen=True)
class SystemConfig:
    """
    System-wide constants and defaults.

    Single source of truth for hardcoded values previously scattered across:
    - engine.py (vector_dim=768)
    - tcp_adapter.py (vector_dimension=768)
    - learning_pipeline.rs (embedding model)
    """

    # Vector dimensions (matching embedding models)
    VECTOR_DIM_NOMIC: int = 768  # nomic-embed-text-v1.5
    VECTOR_DIM_EMBEDDINGGEMMA: int = 768  # google/embeddinggemma-300m
    VECTOR_DIM_DEFAULT: int = 768  # Default for HNSW index

    # Protocol constants
    TCP_DEFAULT_PORT: int = 50051
    TCP_TIMEOUT_SECONDS: int = 30
    TCP_MAX_RETRIES: int = 3
    TCP_RETRY_BACKOFF_BASE: float = 0.5  # Exponential backoff: 0.5s, 1s, 2s

    # Learning defaults
    DEFAULT_STRENGTH: float = 1.0
    DEFAULT_CONFIDENCE: float = 1.0
    MIN_ASSOCIATION_CONFIDENCE: float = 0.5
    MAX_ASSOCIATIONS_PER_CONCEPT: int = 10

    # Batch processing
    DEFAULT_BATCH_SIZE: int = 50
    MEMORY_CLEANUP_INTERVAL: int = 5  # Batches between GC

    # Caching
    DEFAULT_CACHE_SIZE: int = 1000
    DEFAULT_CACHE_TTL_SECONDS: int = 300  # 5 minutes

    # Circuit breaker (for embedding service)
    CIRCUIT_BREAKER_FAILURE_THRESHOLD: int = 5
    CIRCUIT_BREAKER_TIMEOUT_SECONDS: int = 60
    CIRCUIT_BREAKER_HALF_OPEN_REQUESTS: int = 3


# Global instance
SYSTEM_CONFIG = SystemConfig()


def get_vector_dimension(model_name: str) -> int:
    """
    Get vector dimension for a given embedding model.

    Args:
        model_name: Embedding model name

    Returns:
        Vector dimension
    """
    dimension_map = {
        "nomic-embed-text-v1.5": SYSTEM_CONFIG.VECTOR_DIM_NOMIC,
        "google/embeddinggemma-300m": SYSTEM_CONFIG.VECTOR_DIM_EMBEDDINGGEMMA,
    }

    return dimension_map.get(model_name, SYSTEM_CONFIG.VECTOR_DIM_DEFAULT)


logger.debug(f"System config initialized: default_vector_dim={SYSTEM_CONFIG.VECTOR_DIM_DEFAULT}")

"""
Centralized Configuration System for Sutra AI

Single source of truth for all configuration:
- Edition specifications (Simple/Community/Enterprise)
- Storage connection parameters
- Model defaults (embeddings, NLG)
- System constants (vector dimensions, etc.)

Philosophy: Configuration should be injectable, validated, and edition-aware.
"""

from .edition import (
    Edition,
    EditionSpec,
    EDITION_SPECS,
    get_edition_spec,
    validate_edition_consistency,
)
from .storage import StorageConfig, create_storage_config
from .system import SystemConfig, ASSOCIATION_TYPE_MAP
from .reasoning import ReasoningEngineConfig, production_config

__all__ = [
    # Edition management
    "Edition",
    "EditionSpec",
    "EDITION_SPECS",
    "get_edition_spec",
    "validate_edition_consistency",
    # Storage configuration
    "StorageConfig",
    "create_storage_config",
    # System constants
    "SystemConfig",
    "ASSOCIATION_TYPE_MAP",
    # Reasoning engine config
    "ReasoningEngineConfig",
    "production_config",
]

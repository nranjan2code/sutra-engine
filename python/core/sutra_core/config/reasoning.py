"""
Reasoning Engine Configuration Builder

Provides fluent API for configuring ReasoningEngine with sensible defaults.
"""

import logging
from dataclasses import dataclass, field
from typing import Optional

from ..graph.concepts import AssociationType
from .edition import get_edition_spec
from .storage import create_storage_config, StorageConfig
from .system import SYSTEM_CONFIG

logger = logging.getLogger(__name__)


@dataclass
class ReasoningEngineConfig:
    """
    Comprehensive configuration for ReasoningEngine.

    Replaces scattered parameters in engine.py __init__().
    """

    # Storage (injectable configuration)
    storage_config: StorageConfig = field(default_factory=create_storage_config)

    # Backward compatibility
    storage_path: str = "./knowledge"  # Unused, kept for API compatibility

    # Caching
    enable_caching: bool = True
    max_cache_size: int = SYSTEM_CONFIG.DEFAULT_CACHE_SIZE
    cache_ttl_seconds: Optional[float] = SYSTEM_CONFIG.DEFAULT_CACHE_TTL_SECONDS

    # Central links (compositional associations)
    enable_central_links: bool = True
    central_link_confidence: float = 0.6
    central_link_type: AssociationType = AssociationType.COMPOSITIONAL

    # Batch embeddings
    enable_batch_embeddings: bool = True
    embedding_model: Optional[str] = None  # Auto-detected from edition
    mps_batch_threshold: int = 32

    # Parallel associations
    enable_parallel_associations: bool = True
    association_workers: int = 4

    # Entity cache
    enable_entity_cache: bool = False

    def validate(self) -> None:
        """Validate configuration."""
        if self.max_cache_size <= 0:
            raise ValueError(f"max_cache_size must be > 0, got {self.max_cache_size}")

        if not (0.0 <= self.central_link_confidence <= 1.0):
            raise ValueError(
                f"central_link_confidence must be in [0, 1], "
                f"got {self.central_link_confidence}"
            )

        if self.association_workers <= 0:
            raise ValueError(
                f"association_workers must be > 0, got {self.association_workers}"
            )

        # Validate storage config
        self.storage_config.validate()

        logger.debug("Reasoning engine config validated")

    @classmethod
    def builder(cls) -> "ReasoningEngineConfigBuilder":
        """Create a fluent configuration builder."""
        return ReasoningEngineConfigBuilder()


class ReasoningEngineConfigBuilder:
    """
    Fluent API for building ReasoningEngineConfig.

    Example:
        >>> config = (
        ...     ReasoningEngineConfig.builder()
        ...     .with_caching(max_size=500, ttl_seconds=600)
        ...     .with_storage("localhost:50051", edition="enterprise")
        ...     .with_parallel_associations(workers=8)
        ...     .build()
        ... )
    """

    def __init__(self):
        self._config = ReasoningEngineConfig()

    def with_storage(
        self,
        server_address: Optional[str] = None,
        edition: Optional[str] = None,
    ) -> "ReasoningEngineConfigBuilder":
        """Configure storage connection."""
        self._config.storage_config = create_storage_config(
            server_address=server_address,
            edition_override=edition,
        )
        return self

    def with_caching(
        self,
        enabled: bool = True,
        max_size: int = 1000,
        ttl_seconds: Optional[float] = None,
    ) -> "ReasoningEngineConfigBuilder":
        """Configure query caching."""
        self._config.enable_caching = enabled
        self._config.max_cache_size = max_size
        self._config.cache_ttl_seconds = ttl_seconds
        return self

    def with_central_links(
        self,
        enabled: bool = True,
        confidence: float = 0.6,
        link_type: AssociationType = AssociationType.COMPOSITIONAL,
    ) -> "ReasoningEngineConfigBuilder":
        """Configure central link extraction."""
        self._config.enable_central_links = enabled
        self._config.central_link_confidence = confidence
        self._config.central_link_type = link_type
        return self

    def with_batch_embeddings(
        self,
        enabled: bool = True,
        model: Optional[str] = None,
        mps_threshold: int = 32,
    ) -> "ReasoningEngineConfigBuilder":
        """Configure batch embedding generation."""
        self._config.enable_batch_embeddings = enabled
        self._config.embedding_model = model
        self._config.mps_batch_threshold = mps_threshold
        return self

    def with_parallel_associations(
        self,
        enabled: bool = True,
        workers: int = 4,
    ) -> "ReasoningEngineConfigBuilder":
        """Configure parallel association extraction."""
        self._config.enable_parallel_associations = enabled
        self._config.association_workers = workers
        return self

    def with_entity_cache(
        self, enabled: bool = True
    ) -> "ReasoningEngineConfigBuilder":
        """Configure entity extraction caching."""
        self._config.enable_entity_cache = enabled
        return self

    def build(self) -> ReasoningEngineConfig:
        """Build and validate configuration."""
        self._config.validate()
        return self._config


def production_config() -> ReasoningEngineConfig:
    """
    Production-grade configuration with optimal settings.

    Returns:
        ReasoningEngineConfig optimized for production deployment
    """
    edition_spec = get_edition_spec()

    return (
        ReasoningEngineConfig.builder()
        .with_storage(edition=edition_spec.edition.value)
        .with_caching(enabled=True, max_size=1000, ttl_seconds=300)
        .with_batch_embeddings(enabled=True, mps_threshold=32)
        .with_parallel_associations(enabled=True, workers=edition_spec.ingest_workers)
        .with_central_links(enabled=True, confidence=0.6)
        .build()
    )

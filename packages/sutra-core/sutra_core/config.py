"""
Configuration management for Sutra AI Reasoning Engine.

Provides a builder pattern for creating ReasoningEngine instances
with validated configuration instead of passing 17+ parameters.
"""

from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

from .exceptions import ConfigurationError
from .graph.concepts import AssociationType


@dataclass
class ReasoningEngineConfig:
    """
    Configuration for ReasoningEngine with validation.

    Use ReasoningEngineConfig.builder() to create instances with
    a fluent API and automatic validation.

    Example:
        >>> config = (ReasoningEngineConfig.builder()
        ...     .with_storage_path("./my_knowledge")
        ...     .with_caching(enabled=True, max_size=500, ttl_seconds=300)
        ...     .with_vector_index(enabled=True, dimension=768)
        ...     .build())
        >>> engine = ReasoningEngine.from_config(config)
    """

    # Storage configuration
    storage_path: str = "./knowledge"
    use_rust_storage: bool = True

    # Cache configuration
    enable_caching: bool = True
    max_cache_size: int = 1000
    cache_ttl_seconds: Optional[float] = None

    # Learning configuration
    enable_central_links: bool = True
    central_link_confidence: float = 0.6
    central_link_type: AssociationType = AssociationType.COMPOSITIONAL

    # Vector index configuration
    enable_vector_index: bool = True
    vector_index_dimension: Optional[int] = None

    # Batch processing configuration
    enable_batch_embeddings: bool = True
    embedding_model: str = "google/embeddinggemma-300m"  # 768-dim, state-of-the-art
    mps_batch_threshold: int = 32

    # Parallel processing configuration
    enable_parallel_associations: bool = True
    association_workers: int = 4

    # Entity extraction configuration
    enable_entity_cache: bool = False

    def validate(self) -> None:
        """
        Validate configuration values.

        Raises:
            ConfigurationError: If configuration is invalid
        """
        # Validate storage path
        if not self.storage_path or not self.storage_path.strip():
            raise ConfigurationError("storage_path cannot be empty")

        # Validate cache configuration
        if self.max_cache_size < 0:
            raise ConfigurationError("max_cache_size must be non-negative")

        if self.cache_ttl_seconds is not None and self.cache_ttl_seconds < 0:
            raise ConfigurationError("cache_ttl_seconds must be non-negative or None")

        # Validate learning configuration
        if not (0.0 <= self.central_link_confidence <= 1.0):
            raise ConfigurationError(
                "central_link_confidence must be between 0.0 and 1.0"
            )

        # Validate vector index configuration
        if self.vector_index_dimension is not None and self.vector_index_dimension <= 0:
            raise ConfigurationError("vector_index_dimension must be positive or None")

        # Validate batch processing configuration
        if self.mps_batch_threshold < 1:
            raise ConfigurationError("mps_batch_threshold must be positive")

        # Validate parallel processing configuration
        if self.association_workers < 1:
            raise ConfigurationError("association_workers must be positive")

        # Validate model name
        if not self.embedding_model or not self.embedding_model.strip():
            raise ConfigurationError("embedding_model cannot be empty")

    @staticmethod
    def builder() -> "ReasoningEngineConfigBuilder":
        """
        Create a configuration builder.

        Returns:
            ReasoningEngineConfigBuilder instance
        """
        return ReasoningEngineConfigBuilder()

    def to_dict(self) -> dict:
        """Convert configuration to dictionary."""
        return {
            "storage_path": self.storage_path,
            "use_rust_storage": self.use_rust_storage,
            "enable_caching": self.enable_caching,
            "max_cache_size": self.max_cache_size,
            "cache_ttl_seconds": self.cache_ttl_seconds,
            "enable_central_links": self.enable_central_links,
            "central_link_confidence": self.central_link_confidence,
            "central_link_type": self.central_link_type.value,
            "enable_vector_index": self.enable_vector_index,
            "vector_index_dimension": self.vector_index_dimension,
            "enable_batch_embeddings": self.enable_batch_embeddings,
            "embedding_model": self.embedding_model,
            "mps_batch_threshold": self.mps_batch_threshold,
            "enable_parallel_associations": self.enable_parallel_associations,
            "association_workers": self.association_workers,
            "enable_entity_cache": self.enable_entity_cache,
        }


class ReasoningEngineConfigBuilder:
    """
    Fluent builder for ReasoningEngineConfig.

    Example:
        >>> config = (ReasoningEngineConfig.builder()
        ...     .with_storage_path("./knowledge")
        ...     .with_caching(enabled=True)
        ...     .build())
    """

    def __init__(self):
        """Initialize builder with default values."""
        self._config = ReasoningEngineConfig()

    def with_storage_path(
        self, path: str, use_rust: bool = True
    ) -> "ReasoningEngineConfigBuilder":
        """
        Set storage configuration.

        Args:
            path: Path to storage directory
            use_rust: Whether to use Rust storage backend

        Returns:
            Self for method chaining
        """
        self._config.storage_path = path
        self._config.use_rust_storage = use_rust
        return self

    def with_caching(
        self,
        enabled: bool = True,
        max_size: int = 1000,
        ttl_seconds: Optional[float] = None,
    ) -> "ReasoningEngineConfigBuilder":
        """
        Set cache configuration.

        Args:
            enabled: Enable query result caching
            max_size: Maximum number of cached results
            ttl_seconds: Time-to-live for cache entries (None = no expiration)

        Returns:
            Self for method chaining
        """
        self._config.enable_caching = enabled
        self._config.max_cache_size = max_size
        self._config.cache_ttl_seconds = ttl_seconds
        return self

    def with_learning(
        self,
        enable_central_links: bool = True,
        central_link_confidence: float = 0.6,
        central_link_type: AssociationType = AssociationType.COMPOSITIONAL,
    ) -> "ReasoningEngineConfigBuilder":
        """
        Set learning configuration.

        Args:
            enable_central_links: Create links from central concept to extracted phrases
            central_link_confidence: Confidence for central links (0.0 - 1.0)
            central_link_type: Association type for central links

        Returns:
            Self for method chaining
        """
        self._config.enable_central_links = enable_central_links
        self._config.central_link_confidence = central_link_confidence
        self._config.central_link_type = central_link_type
        return self

    def with_vector_index(
        self, enabled: bool = True, dimension: Optional[int] = None
    ) -> "ReasoningEngineConfigBuilder":
        """
        Set vector index configuration.

        Args:
            enabled: Enable HNSW vector indexing for O(log N) search
            dimension: Embedding dimension (None = auto-detect)

        Returns:
            Self for method chaining
        """
        self._config.enable_vector_index = enabled
        self._config.vector_index_dimension = dimension
        return self

    def with_batch_embeddings(
        self,
        enabled: bool = True,
        model: str = "google/embeddinggemma-300m",  # 768-dim, state-of-the-art
        mps_threshold: int = 64,
    ) -> "ReasoningEngineConfigBuilder":
        """
        Set batch embedding configuration.

        Args:
            enabled: Enable batch embedding generation with MPS support
            model: Sentence-transformer model name
            mps_threshold: Minimum batch size to use MPS (Apple Silicon GPU)

        Returns:
            Self for method chaining
        """
        self._config.enable_batch_embeddings = enabled
        self._config.embedding_model = model
        self._config.mps_batch_threshold = mps_threshold
        return self

    def with_parallel_associations(
        self, enabled: bool = True, workers: int = 4
    ) -> "ReasoningEngineConfigBuilder":
        """
        Set parallel association extraction configuration.

        Args:
            enabled: Enable parallel association extraction
            workers: Number of worker processes

        Returns:
            Self for method chaining
        """
        self._config.enable_parallel_associations = enabled
        self._config.association_workers = workers
        return self

    def with_entity_cache(
        self, enabled: bool = False
    ) -> "ReasoningEngineConfigBuilder":
        """
        Set entity cache configuration.

        Args:
            enabled: Enable cached entity extraction with background LLM service

        Returns:
            Self for method chaining
        """
        self._config.enable_entity_cache = enabled
        return self

    def build(self) -> ReasoningEngineConfig:
        """
        Build and validate configuration.

        Returns:
            Validated ReasoningEngineConfig

        Raises:
            ConfigurationError: If configuration is invalid
        """
        self._config.validate()
        return self._config


# Predefined configurations for common use cases


def minimal_config() -> ReasoningEngineConfig:
    """
    Minimal configuration for testing or low-resource environments.

    - No vector index
    - Small cache
    - Sequential processing
    - No entity cache
    """
    return (
        ReasoningEngineConfig.builder()
        .with_vector_index(enabled=False)
        .with_caching(enabled=True, max_size=100)
        .with_batch_embeddings(enabled=False)
        .with_parallel_associations(enabled=False)
        .with_entity_cache(enabled=False)
        .build()
    )


def production_config(storage_path: str = "./knowledge") -> ReasoningEngineConfig:
    """
    Production configuration with all optimizations enabled.

    - Rust storage
    - Vector indexing
    - Large cache with TTL
    - Batch embeddings
    - Parallel associations
    """
    return (
        ReasoningEngineConfig.builder()
        .with_storage_path(storage_path, use_rust=True)
        .with_caching(enabled=True, max_size=5000, ttl_seconds=3600)
        .with_vector_index(enabled=True, dimension=768)  # EmbeddingGemma dimension
        .with_batch_embeddings(
            enabled=True
        )  # Uses google/embeddinggemma-300m by default
        .with_parallel_associations(enabled=True, workers=8)
        .build()
    )


def development_config() -> ReasoningEngineConfig:
    """
    Development configuration with fast iteration.

    - In-memory storage (no Rust)
    - Small cache
    - Vector indexing enabled
    - Batch embeddings
    - Fewer parallel workers
    """
    return (
        ReasoningEngineConfig.builder()
        .with_storage_path("./dev_knowledge", use_rust=False)
        .with_caching(enabled=True, max_size=500)
        .with_vector_index(enabled=True)
        .with_batch_embeddings(enabled=True)
        .with_parallel_associations(enabled=True, workers=2)
        .build()
    )

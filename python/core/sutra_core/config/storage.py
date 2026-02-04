"""
Storage Configuration - Injectable, Edition-Aware

Eliminates hardcoded storage addresses and parameters in:
- reasoning/engine.py:158 (hardcoded server_address)
- reasoning/engine.py:157 (hardcoded vector_dimension=768)
"""

import os
import logging
from dataclasses import dataclass
from typing import Optional

from .edition import Edition, get_edition_spec
from .system import SYSTEM_CONFIG, get_vector_dimension

logger = logging.getLogger(__name__)


@dataclass
class StorageConfig:
    """
    Storage connection configuration.

    Injectable configuration object replacing hardcoded values.
    """

    # Connection
    server_address: str
    timeout_seconds: int
    max_retries: int

    # Vector search
    vector_dimension: int

    # Edition-aware configuration
    edition: Edition

    # Circuit breaker settings
    circuit_breaker_enabled: bool = True
    circuit_breaker_failure_threshold: int = SYSTEM_CONFIG.CIRCUIT_BREAKER_FAILURE_THRESHOLD
    circuit_breaker_timeout_seconds: int = SYSTEM_CONFIG.CIRCUIT_BREAKER_TIMEOUT_SECONDS

    def validate(self) -> None:
        """Validate storage configuration."""
        if not self.server_address:
            raise ValueError("Storage server address is required")

        if self.vector_dimension not in [768, 384, 1536]:
            logger.warning(
                f"Unusual vector dimension: {self.vector_dimension}. "
                f"Common values: 768, 384, 1536"
            )

        if self.timeout_seconds <= 0:
            raise ValueError(f"timeout_seconds must be > 0, got {self.timeout_seconds}")

        logger.debug(f"Storage config validated: {self.server_address}")


def create_storage_config(
    server_address: Optional[str] = None,
    edition_override: Optional[str] = None,
    vector_dimension: Optional[int] = None,
    timeout_seconds: Optional[int] = None,
    max_retries: Optional[int] = None,
) -> StorageConfig:
    """
    Create storage configuration from environment and edition spec.

    Args:
        server_address: Override default server address (from SUTRA_STORAGE_SERVER)
        edition_override: Override edition (from SUTRA_EDITION)
        vector_dimension: Override vector dimension (auto-detected from model)
        timeout_seconds: Override timeout (default from SYSTEM_CONFIG)
        max_retries: Override max retries (default from SYSTEM_CONFIG)

    Returns:
        Validated StorageConfig instance

    Example:
        >>> # Use environment defaults
        >>> config = create_storage_config()
        >>>
        >>> # Explicit configuration
        >>> config = create_storage_config(
        ...     server_address="localhost:50051",
        ...     edition_override="enterprise"
        ... )
    """
    # Get edition spec (validates edition)
    edition_spec = get_edition_spec(edition_override)

    # Resolve server address
    resolved_server_address = (
        server_address
        or os.getenv("SUTRA_STORAGE_SERVER")
        or f"storage-server:{SYSTEM_CONFIG.TCP_DEFAULT_PORT}"
    )

    # Resolve vector dimension from edition's embedding model
    if vector_dimension is None:
        vector_dimension = get_vector_dimension(edition_spec.embedding_model)

    # Create config
    config = StorageConfig(
        server_address=resolved_server_address,
        timeout_seconds=timeout_seconds or SYSTEM_CONFIG.TCP_TIMEOUT_SECONDS,
        max_retries=max_retries or SYSTEM_CONFIG.TCP_MAX_RETRIES,
        vector_dimension=vector_dimension,
        edition=edition_spec.edition,
    )

    config.validate()

    logger.info(
        f"Storage config: {config.server_address} "
        f"(dim={config.vector_dimension}, edition={config.edition.value})"
    )

    return config

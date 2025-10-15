"""
Configuration settings for the Sutra API service.

Manages environment variables and application settings.
"""

import os
from typing import Optional

from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    """API configuration settings."""

    # API Metadata
    api_title: str = "Sutra AI API"
    api_version: str = "1.0.0"
    api_description: str = "REST API for Sutra AI graph-based reasoning system"

    # Reasoning cache configuration
    # Set via env var SUTRA_CACHE_TTL_SECONDS (e.g., 300 for 5 minutes)
    cache_ttl_seconds: Optional[float] = None

    # Server Configuration
    host: str = "0.0.0.0"
    port: int = 8000
    reload: bool = False
    workers: int = 1

    # Storage Configuration
    storage_path: str = "./api_knowledge"
    auto_save: bool = True
    save_interval_seconds: int = 300  # 5 minutes

    # AI Configuration
    use_semantic_embeddings: bool = True
    max_concepts: Optional[int] = None  # No limit by default

    # Learning/linking configuration
    compositional_links: bool = True
    compositional_confidence: float = 0.6
    compositional_type: str = "compositional"

    # CORS Configuration
    allow_origins: list = ["*"]
    allow_credentials: bool = True
    allow_methods: list = ["*"]
    allow_headers: list = ["*"]

    # Logging Configuration
    log_level: str = "INFO"
    log_format: str = "%(asctime)s - %(name)s - %(levelname)s - %(message)s"

    # Rate Limiting (requests per minute)
    rate_limit_learn: int = 60
    rate_limit_reason: int = 30
    rate_limit_search: int = 100

    class Config:
        """Pydantic configuration."""

        env_prefix = "SUTRA_"
        case_sensitive = False


# Global settings instance
settings = Settings()

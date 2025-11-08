"""
Configuration settings for the Sutra API service.

Manages environment variables and application settings.
"""

import os
from typing import Optional

from pydantic_settings import BaseSettings

# Import feature flags for edition-aware configuration
try:
    from sutra_core.feature_flags import (
        Edition,
        EditionLimits,
        detect_edition,
        get_edition_limits,
    )
    FEATURE_FLAGS_AVAILABLE = True
except ImportError:
    FEATURE_FLAGS_AVAILABLE = False
    # Fallback to simple edition
    class Edition:
        SIMPLE = "simple"
        COMMUNITY = "community"
        ENTERPRISE = "enterprise"


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

    # Storage Configuration (Custom Binary Protocol)
    storage_server: str = "storage-server:50051"  # Domain storage server address
    user_storage_server: str = "user-storage-server:50051"  # User storage server address
    auto_save: bool = True
    save_interval_seconds: int = 300  # 5 minutes

    # Authentication Configuration
    jwt_secret_key: str = "INSECURE_DEFAULT_SECRET_CHANGE_IN_PRODUCTION"
    jwt_algorithm: str = "HS256"
    jwt_expiration_hours: int = 24  # Token expires in 24 hours
    jwt_refresh_expiration_days: int = 7  # Refresh token expires in 7 days

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

    # Edition Configuration
    edition: str = "simple"  # simple, community, enterprise
    license_key: Optional[str] = None

    # Rate Limiting (requests per minute) - DISABLED FOR NGINX-ONLY CONTROL
    # Set to very high values so nginx handles all rate limiting
    rate_limit_learn: int = 10000    # Effectively unlimited
    rate_limit_reason: int = 10000   # Effectively unlimited
    rate_limit_search: int = 10000   # Effectively unlimited

    class Config:
        """Pydantic configuration."""

        env_prefix = "SUTRA_"
        case_sensitive = False

    def get_edition_limits(self):
        """Get edition-specific limits."""
        if FEATURE_FLAGS_AVAILABLE:
            try:
                # Detect edition from environment
                edition_enum, limits = detect_edition(
                    license_key=self.license_key,
                    edition_override=self.edition
                )
                return limits
            except Exception as e:
                # Fallback to simple edition on error - create proper instance
                from dataclasses import make_dataclass
                Limits = make_dataclass('Limits', [
                    ('learn_per_min', int),
                    ('reason_per_min', int),
                    ('max_concepts', int),
                    ('max_dataset_gb', int),
                    ('ingest_workers', int),
                ])
                return Limits(
                    learn_per_min=10,
                    reason_per_min=50,
                    max_concepts=100_000,
                    max_dataset_gb=1,
                    ingest_workers=2,
                )
        else:
            # Feature flags not available - create proper instance
            from dataclasses import make_dataclass
            Limits = make_dataclass('Limits', [
                ('learn_per_min', int),
                ('reason_per_min', int),
                ('max_concepts', int),
                ('max_dataset_gb', int),
                ('ingest_workers', int),
            ])
            return Limits(
                learn_per_min=self.rate_limit_learn,
                reason_per_min=self.rate_limit_reason,
                max_concepts=self.max_concepts or 100_000,
                max_dataset_gb=1,
                ingest_workers=2,
            )


# Global settings instance
settings = Settings()

# Apply edition-based rate limits at startup
if FEATURE_FLAGS_AVAILABLE:
    limits = settings.get_edition_limits()
    settings.rate_limit_learn = limits.learn_per_min
    settings.rate_limit_reason = limits.reason_per_min
    if limits.max_concepts:
        settings.max_concepts = limits.max_concepts

"""
Sutra ML Base - Foundation for AI/ML Services

Scalable, edition-aware foundation for Sutra's ML services providing:
- Unified model loading (embedding, generative, multimodal)
- Edition-aware resource allocation and feature gating
- Production-grade monitoring, health checks, and observability
- FastAPI service scaffolding with consistent APIs
- Efficient caching, batching, and performance optimization
- Security, authentication, and access control

Architecture Philosophy:
- Edition-First: Simple/Community/Enterprise feature gating at core
- Zero-Copy: Minimize memory allocations and data movement
- Observable: Rich metrics, tracing, and health monitoring
- Composable: Mix-and-match components for different service types
- Backwards Compatible: Stable APIs across Sutra ecosystem versions
"""

from .edition import EditionManager, Edition, get_edition_config
from .model_loader import ModelLoader, ModelType, LoaderConfig
from .service_base import BaseMlService, ServiceConfig, HealthStatus
from .metrics import MetricsCollector, ServiceMetrics
from .cache import CacheManager, CacheConfig
from .security import SecurityManager, AuthConfig
from .utils import setup_environment, setup_logging

__version__ = "2.0.0"
__author__ = "Sutra AI Team"

# Public API exports
__all__ = [
    # Core Edition System
    "EditionManager", 
    "Edition",
    "get_edition_config",
    
    # Model Management
    "ModelLoader",
    "ModelType", 
    "LoaderConfig",
    
    # Service Framework
    "BaseMlService",
    "ServiceConfig",
    "HealthStatus",
    
    # Observability
    "MetricsCollector",
    "ServiceMetrics",
    
    # Performance
    "CacheManager",
    "CacheConfig",
    
    # Security
    "SecurityManager",
    "AuthConfig",
    
    # Utilities
    "setup_environment",
    "setup_logging",
]
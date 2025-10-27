"""
Edition Management for Sutra ML Services

Extends core edition system with ML-specific features and resource allocation.
Provides edition-aware model selection, performance tuning, and feature gating.
"""

import os
import logging
from enum import Enum
from dataclasses import dataclass
from typing import Dict, Any, Optional, List

logger = logging.getLogger(__name__)


class Edition(Enum):
    """Sutra AI editions with ML-specific extensions"""
    SIMPLE = "simple"        # Single small model, basic features
    COMMUNITY = "community"  # Multiple models, enhanced performance  
    ENTERPRISE = "enterprise"  # Large models, advanced features, HA


@dataclass
class MlEditionConfig:
    """ML-specific configuration per edition"""
    # Model Selection
    max_model_size_gb: float
    concurrent_models: int
    model_cache_gb: float
    
    # Performance
    max_batch_size: int
    max_sequence_length: int
    workers: int
    
    # Features
    custom_models: bool
    model_fine_tuning: bool
    advanced_caching: bool
    multi_gpu: bool
    
    # Resource Limits
    max_memory_gb: float
    max_cpu_cores: int
    
    # Edition Display
    display_name: str
    description: str


# Edition-specific configurations
ML_EDITION_CONFIGS: Dict[Edition, MlEditionConfig] = {
    Edition.SIMPLE: MlEditionConfig(
        # Conservative resource allocation
        max_model_size_gb=2.0,
        concurrent_models=1,
        model_cache_gb=4.0,
        
        # Basic performance
        max_batch_size=32,
        max_sequence_length=512,
        workers=1,
        
        # Limited features  
        custom_models=False,
        model_fine_tuning=False,
        advanced_caching=False,
        multi_gpu=False,
        
        # Resource constraints
        max_memory_gb=8.0,
        max_cpu_cores=4,
        
        display_name="Simple Edition",
        description="Single-tenant, basic ML capabilities"
    ),
    
    Edition.COMMUNITY: MlEditionConfig(
        # Enhanced resource allocation
        max_model_size_gb=8.0,
        concurrent_models=2,
        model_cache_gb=16.0,
        
        # Better performance
        max_batch_size=64,
        max_sequence_length=1024,
        workers=2,
        
        # More features
        custom_models=True,
        model_fine_tuning=False,
        advanced_caching=True,
        multi_gpu=False,
        
        # More resources
        max_memory_gb=32.0,
        max_cpu_cores=8,
        
        display_name="Community Edition",
        description="Enhanced ML with custom models"
    ),
    
    Edition.ENTERPRISE: MlEditionConfig(
        # Unlimited resource allocation
        max_model_size_gb=50.0,
        concurrent_models=10,
        model_cache_gb=100.0,
        
        # Maximum performance  
        max_batch_size=256,
        max_sequence_length=4096,
        workers=4,
        
        # All features
        custom_models=True,
        model_fine_tuning=True,
        advanced_caching=True,
        multi_gpu=True,
        
        # Unlimited resources
        max_memory_gb=256.0,
        max_cpu_cores=64,
        
        display_name="Enterprise Edition",
        description="Full ML capabilities with HA and fine-tuning"
    )
}


class EditionManager:
    """Manages edition-aware ML service behavior"""
    
    def __init__(self, edition: Optional[Edition] = None):
        """Initialize edition manager
        
        Args:
            edition: Explicit edition, or auto-detect from SUTRA_EDITION env
        """
        if edition is None:
            edition_str = os.getenv("SUTRA_EDITION", "simple").lower()
            try:
                edition = Edition(edition_str)
            except ValueError:
                logger.warning(f"Invalid SUTRA_EDITION '{edition_str}', defaulting to Simple")
                edition = Edition.SIMPLE
        
        self.edition = edition
        self.config = ML_EDITION_CONFIGS[edition]
        
        logger.info(f"Initialized ML Edition: {self.config.display_name}")
    
    def get_model_size_limit(self) -> float:
        """Get maximum model size in GB"""
        return self.config.max_model_size_gb
    
    def get_batch_size_limit(self) -> int:
        """Get maximum batch size"""
        return self.config.max_batch_size
    
    def can_load_model(self, model_size_gb: float) -> bool:
        """Check if model size is within edition limits"""
        return model_size_gb <= self.config.max_model_size_gb
    
    def get_worker_count(self) -> int:
        """Get recommended worker count for this edition"""
        return self.config.workers
    
    def supports_custom_models(self) -> bool:
        """Check if custom model loading is supported"""
        return self.config.custom_models
    
    def supports_fine_tuning(self) -> bool:
        """Check if model fine-tuning is supported"""
        return self.config.model_fine_tuning
    
    def supports_advanced_caching(self) -> bool:
        """Check if advanced caching features are available"""
        return self.config.advanced_caching
    
    def supports_multi_gpu(self) -> bool:
        """Check if multi-GPU support is available"""
        return self.config.multi_gpu
    
    def get_cache_size_gb(self) -> float:
        """Get model cache size limit in GB"""
        return self.config.model_cache_gb
    
    def get_sequence_length_limit(self) -> int:
        """Get maximum sequence length"""
        return self.config.max_sequence_length
    
    def get_edition_info(self) -> Dict[str, Any]:
        """Get full edition configuration info"""
        return {
            "edition": self.edition.value,
            "display_name": self.config.display_name,
            "description": self.config.description,
            "max_model_size_gb": self.config.max_model_size_gb,
            "concurrent_models": self.config.concurrent_models,
            "max_batch_size": self.config.max_batch_size,
            "workers": self.config.workers,
            "features": {
                "custom_models": self.config.custom_models,
                "model_fine_tuning": self.config.model_fine_tuning,
                "advanced_caching": self.config.advanced_caching,
                "multi_gpu": self.config.multi_gpu
            }
        }


def get_edition_config(edition: Optional[Edition] = None) -> MlEditionConfig:
    """Get ML edition configuration
    
    Args:
        edition: Edition to get config for, or auto-detect from env
        
    Returns:
        MlEditionConfig for the specified edition
    """
    if edition is None:
        edition_str = os.getenv("SUTRA_EDITION", "simple").lower()
        try:
            edition = Edition(edition_str)
        except ValueError:
            edition = Edition.SIMPLE
    
    return ML_EDITION_CONFIGS[edition]


def get_current_edition() -> Edition:
    """Get current edition from environment"""
    edition_str = os.getenv("SUTRA_EDITION", "simple").lower()
    try:
        return Edition(edition_str)
    except ValueError:
        logger.warning(f"Invalid SUTRA_EDITION '{edition_str}', defaulting to Simple")
        return Edition.SIMPLE
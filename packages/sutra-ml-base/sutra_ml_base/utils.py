"""
Utility Functions for Sutra ML Services

Common utilities for environment setup, logging configuration,
model management, and service initialization.
"""

import os
import logging
import sys
from pathlib import Path
from typing import Dict, Any, Optional


def setup_environment():
    """Setup environment variables for ML services"""
    # HuggingFace cache directories
    cache_dir = "/tmp/.cache/huggingface"
    os.environ.setdefault("TRANSFORMERS_CACHE", cache_dir)
    os.environ.setdefault("HF_HOME", cache_dir)
    os.environ.setdefault("TORCH_HOME", "/tmp/.cache/torch")
    
    # Create cache directories
    Path(cache_dir).mkdir(parents=True, exist_ok=True)
    Path("/tmp/.cache/torch").mkdir(parents=True, exist_ok=True)
    
    # Disable tokenizers warnings
    os.environ.setdefault("TOKENIZERS_PARALLELISM", "false")
    
    # Set reasonable defaults
    os.environ.setdefault("SUTRA_EDITION", "simple")
    os.environ.setdefault("SUTRA_VERSION", "2.0.0")


def setup_logging(
    service_name: str,
    level: str = "INFO",
    format_style: str = "json",
    log_file: Optional[str] = None
) -> logging.Logger:
    """Setup logging configuration for ML services
    
    Args:
        service_name: Name of the service for logger
        level: Logging level
        format_style: "json" or "standard"
        log_file: Optional log file path
        
    Returns:
        Configured logger instance
    """
    # Configure root logger
    logging.basicConfig(
        level=getattr(logging, level.upper()),
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
        if format_style == "standard" else None,
        handlers=[
            logging.StreamHandler(sys.stdout)
        ]
    )
    
    # Get service logger
    logger = logging.getLogger(service_name)
    
    # Add file handler if specified
    if log_file:
        file_handler = logging.FileHandler(log_file)
        file_handler.setFormatter(
            logging.Formatter('%(asctime)s - %(name)s - %(levelname)s - %(message)s')
        )
        logger.addHandler(file_handler)
    
    logger.info(f"Logging initialized for {service_name} (level: {level})")
    return logger


def get_model_info(model_name: str) -> Dict[str, Any]:
    """Get information about a model from its name
    
    Args:
        model_name: HuggingFace model name or local path
        
    Returns:
        Dictionary with model information
    """
    info = {
        "name": model_name,
        "type": "unknown",
        "size_category": "unknown",
        "provider": "unknown"
    }
    
    # Determine model type from name
    if "embed" in model_name.lower():
        info["type"] = "embedding"
    elif any(x in model_name.lower() for x in ["gemma", "phi", "llama", "gpt"]):
        info["type"] = "generative"
    elif "chat" in model_name.lower():
        info["type"] = "chat"
    
    # Determine size category
    if any(x in model_name.lower() for x in ["270m", "300m"]):
        info["size_category"] = "small"
    elif any(x in model_name.lower() for x in ["1b", "2b"]):
        info["size_category"] = "medium"
    elif any(x in model_name.lower() for x in ["7b", "13b"]):
        info["size_category"] = "large"
    
    # Determine provider
    if model_name.startswith("google/"):
        info["provider"] = "google"
    elif model_name.startswith("microsoft/"):
        info["provider"] = "microsoft"
    elif model_name.startswith("nomic-ai/"):
        info["provider"] = "nomic"
    elif "/" in model_name:
        info["provider"] = model_name.split("/")[0]
    
    return info


def validate_model_compatibility(model_name: str, edition: str) -> bool:
    """Validate if model is compatible with edition
    
    Args:
        model_name: Model name to validate
        edition: Sutra edition (simple/community/enterprise)
        
    Returns:
        True if compatible, False otherwise
    """
    model_info = get_model_info(model_name)
    
    # Edition compatibility rules
    if edition == "simple":
        # Only small models allowed
        return model_info["size_category"] in ["small", "unknown"]
    
    elif edition == "community":
        # Small and medium models allowed
        return model_info["size_category"] in ["small", "medium", "unknown"]
    
    elif edition == "enterprise":
        # All models allowed
        return True
    
    return False


def get_device_info() -> Dict[str, Any]:
    """Get information about available compute devices
    
    Returns:
        Dictionary with device information
    """
    device_info = {
        "cpu_available": True,
        "cuda_available": False,
        "mps_available": False,
        "cuda_devices": 0,
        "recommended_device": "cpu"
    }
    
    try:
        import torch
        device_info["cuda_available"] = torch.cuda.is_available()
        device_info["cuda_devices"] = torch.cuda.device_count() if device_info["cuda_available"] else 0
        
        # Check for Apple Silicon MPS
        if hasattr(torch.backends, 'mps'):
            device_info["mps_available"] = torch.backends.mps.is_available()
        
        # Recommend best device
        if device_info["cuda_available"]:
            device_info["recommended_device"] = "cuda"
        elif device_info["mps_available"]:
            device_info["recommended_device"] = "mps"
        
    except ImportError:
        pass  # PyTorch not available
    
    return device_info
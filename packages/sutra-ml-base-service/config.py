# Sutra ML-Base Service Production Configuration

import os
from typing import Dict, Any

# ================================
# Production Configuration
# ================================

class ProductionConfig:
    """Production-grade configuration management"""
    
    # Service Configuration
    SERVICE_NAME = "sutra-ml-base-service"
    SERVICE_VERSION = "2.0.0"
    
    # Network Configuration
    HOST = os.getenv("HOST", "0.0.0.0")
    PORT = int(os.getenv("PORT", "8887"))
    WORKERS = int(os.getenv("WORKERS", "1"))
    
    # ML Configuration
    MODEL_CACHE_DIR = os.getenv("MODEL_CACHE_DIR", "/app/models")
    TORCH_NUM_THREADS = int(os.getenv("TORCH_NUM_THREADS", "4"))
    MAX_MEMORY_GB = float(os.getenv("MAX_MEMORY_GB", "8.0"))
    
    # Inference Configuration
    DEFAULT_BATCH_SIZE = int(os.getenv("DEFAULT_BATCH_SIZE", "16"))
    MAX_BATCH_SIZE = int(os.getenv("MAX_BATCH_SIZE", "64"))
    INFERENCE_TIMEOUT = float(os.getenv("INFERENCE_TIMEOUT", "30.0"))
    
    # Cache Configuration
    CACHE_ENABLED = os.getenv("CACHE_ENABLED", "true").lower() == "true"
    CACHE_TTL_EMBEDDING = int(os.getenv("CACHE_TTL_EMBEDDING", "7200"))  # 2 hours
    CACHE_TTL_GENERATION = int(os.getenv("CACHE_TTL_GENERATION", "3600"))  # 1 hour
    CACHE_MAX_MEMORY_MB = int(os.getenv("CACHE_MAX_MEMORY_MB", "2048"))
    
    # Health Check Configuration
    HEALTH_CHECK_INTERVAL = int(os.getenv("HEALTH_CHECK_INTERVAL", "30"))
    HEALTH_CHECK_TIMEOUT = int(os.getenv("HEALTH_CHECK_TIMEOUT", "10"))
    
    # Logging Configuration
    LOG_LEVEL = os.getenv("LOG_LEVEL", "INFO")
    LOG_FORMAT = os.getenv("LOG_FORMAT", "json")  # json or text
    
    # Security Configuration
    API_KEY_REQUIRED = os.getenv("API_KEY_REQUIRED", "false").lower() == "true"
    API_KEY_HEADER = os.getenv("API_KEY_HEADER", "X-API-Key")
    ALLOWED_ORIGINS = os.getenv("ALLOWED_ORIGINS", "*").split(",")
    
    # Rate Limiting
    RATE_LIMIT_ENABLED = os.getenv("RATE_LIMIT_ENABLED", "true").lower() == "true"
    RATE_LIMIT_RPM = int(os.getenv("RATE_LIMIT_RPM", "1000"))  # Requests per minute
    
    # Model Preloading
    PRELOAD_MODELS = os.getenv("PRELOAD_MODELS", "").split(",") if os.getenv("PRELOAD_MODELS") else []
    
    # Edition Configuration
    SUTRA_EDITION = os.getenv("SUTRA_EDITION", "simple")
    
    @classmethod
    def get_edition_limits(cls) -> Dict[str, Any]:
        """Get edition-specific limits"""
        edition = cls.SUTRA_EDITION.lower()
        
        if edition == "simple":
            return {
                "max_models": 2,
                "max_batch_size": 8,
                "max_memory_gb": 4.0,
                "cache_enabled": False,
                "custom_models": False,
                "rate_limit_rpm": 100
            }
        elif edition == "community":
            return {
                "max_models": 4,
                "max_batch_size": 32,
                "max_memory_gb": 8.0,
                "cache_enabled": True,
                "custom_models": True,
                "rate_limit_rpm": 1000
            }
        else:  # enterprise
            return {
                "max_models": 10,
                "max_batch_size": 128,
                "max_memory_gb": 32.0,
                "cache_enabled": True,
                "custom_models": True,
                "rate_limit_rpm": -1  # Unlimited
            }
    
    @classmethod
    def get_default_models(cls) -> Dict[str, Dict[str, str]]:
        """Get default models for edition"""
        edition = cls.SUTRA_EDITION.lower()
        
        if edition == "simple":
            return {
                "embedding-default": {
                    "name": "sentence-transformers/all-MiniLM-L6-v2",
                    "type": "embedding"
                },
                "nlg-default": {
                    "name": "microsoft/DialoGPT-small",
                    "type": "nlg"
                }
            }
        elif edition == "community":
            return {
                "embedding-nomic": {
                    "name": "nomic-ai/nomic-embed-text-v1.5",
                    "type": "embedding"
                },
                "nlg-gemma": {
                    "name": "google/gemma-2-2b-it",
                    "type": "nlg"
                }
            }
        else:  # enterprise
            return {
                "embedding-nomic": {
                    "name": "nomic-ai/nomic-embed-text-v1.5", 
                    "type": "embedding"
                },
                "embedding-e5": {
                    "name": "intfloat/e5-large-v2",
                    "type": "embedding"
                },
                "nlg-llama": {
                    "name": "meta-llama/Llama-2-7b-chat-hf",
                    "type": "nlg"
                }
            }
    
    @classmethod
    def validate_config(cls) -> None:
        """Validate configuration at startup"""
        errors = []
        
        # Validate required environment
        if cls.PORT < 1 or cls.PORT > 65535:
            errors.append(f"Invalid PORT: {cls.PORT}")
        
        if cls.MAX_MEMORY_GB < 0.5:
            errors.append(f"MAX_MEMORY_GB too low: {cls.MAX_MEMORY_GB}")
        
        if cls.DEFAULT_BATCH_SIZE < 1 or cls.DEFAULT_BATCH_SIZE > cls.MAX_BATCH_SIZE:
            errors.append(f"Invalid batch size configuration")
        
        # Check edition limits
        limits = cls.get_edition_limits()
        if cls.MAX_MEMORY_GB > limits["max_memory_gb"]:
            errors.append(f"Memory limit {cls.MAX_MEMORY_GB}GB exceeds edition limit {limits['max_memory_gb']}GB")
        
        if errors:
            raise ValueError(f"Configuration errors: {', '.join(errors)}")
        
        print(f"✅ Configuration validated for {cls.SUTRA_EDITION} edition")


# Prometheus metrics configuration
PROMETHEUS_METRICS = {
    "ml_requests_total": {
        "type": "counter",
        "description": "Total ML requests processed",
        "labels": ["model_id", "request_type", "status"]
    },
    "ml_request_duration_seconds": {
        "type": "histogram",
        "description": "ML request duration",
        "labels": ["model_id", "request_type"],
        "buckets": [0.01, 0.05, 0.1, 0.5, 1.0, 2.0, 5.0, 10.0]
    },
    "ml_model_memory_bytes": {
        "type": "gauge",
        "description": "Memory usage by model",
        "labels": ["model_id", "model_type"]
    },
    "ml_cache_hits_total": {
        "type": "counter",
        "description": "Cache hit count",
        "labels": ["cache_type"]
    },
    "ml_models_loaded": {
        "type": "gauge",
        "description": "Number of loaded models"
    }
}
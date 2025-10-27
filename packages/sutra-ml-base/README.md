# Sutra ML Base Foundation

**Scalable, Edition-Aware ML Service Framework for Sutra AI**

## Overview

Sutra ML Base provides a unified foundation for all ML services in the Sutra ecosystem, featuring:

### 🏗️ **Core Architecture**
- **Edition-Aware**: Automatic resource allocation and feature gating based on Simple/Community/Enterprise editions
- **Model Agnostic**: Universal model loading for embeddings, generative LLMs, and multimodal models  
- **Performance Optimized**: Zero-copy operations, efficient caching, and batch processing
- **Observable**: Production-grade metrics, health checks, and distributed tracing
- **Secure**: Built-in authentication, rate limiting, and request validation

### 🎯 **Key Features**

#### **Unified Model Loading**
```python
from sutra_ml_base import ModelLoader, LoaderConfig, ModelType

# Load embedding model
config = LoaderConfig(
    model_name="nomic-ai/nomic-embed-text-v1.5",
    model_type=ModelType.EMBEDDING,
    device="auto"
)
model, tokenizer, loader = ModelLoader.load_model(config)
```

#### **Edition-Aware Resource Management**
```python
from sutra_ml_base import EditionManager

manager = EditionManager()  # Auto-detects from SUTRA_EDITION env

# Edition limits enforced automatically
max_batch = manager.get_batch_size_limit()      # 32/64/256 based on edition
can_custom = manager.supports_custom_models()   # False/True/True
cache_size = manager.get_cache_size_gb()        # 4/16/100 based on edition
```

#### **FastAPI Service Scaffolding**
```python
from sutra_ml_base import BaseMlService, ServiceConfig

class MyMlService(BaseMlService):
    async def load_model(self) -> bool:
        # Your model loading logic
        return True
    
    async def process_request(self, request) -> Any:
        # Your request processing logic
        return response

config = ServiceConfig(service_name="my-service", port=8888)
service = MyMlService(config)
service.run()  # Automatic health checks, metrics, auth
```

### 📊 **Performance & Observability**

#### **Built-in Metrics**
- Request throughput and latency tracking
- Error rate monitoring and alerting  
- Resource usage (CPU, memory, GPU) tracking
- Per-endpoint performance analytics
- Prometheus-compatible metrics export

#### **Production Health Checks**
- Model loading status validation
- Memory usage monitoring  
- Response time tracking
- Edition compliance verification
- Automatic degraded mode detection

### 🔒 **Enterprise Security**

#### **Authentication & Authorization**
- API key validation with constant-time comparison
- JWT token support with configurable expiry
- Role-based access control (RBAC)
- Edition-based feature gating

#### **Rate Limiting & Protection**
- Per-client rate limiting (1000/min default)
- Burst protection and DDoS mitigation
- Request size validation
- Input sanitization and validation

### 🚀 **Edition System**

| Feature | Simple | Community | Enterprise |
|---------|---------|-----------|------------|
| **Max Model Size** | 2GB | 8GB | 50GB |
| **Concurrent Models** | 1 | 2 | 10 |
| **Batch Size** | 32 | 64 | 256 |
| **Cache Size** | 4GB | 16GB | 100GB |
| **Custom Models** | ❌ | ✅ | ✅ |
| **Multi-GPU** | ❌ | ❌ | ✅ |
| **Advanced Caching** | ❌ | ✅ | ✅ |

## Quick Start

### Installation

```bash
# Basic installation
pip install -e packages/sutra-ml-base

# With optimizations
pip install -e "packages/sutra-ml-base[optimized]"

# Development setup
pip install -e "packages/sutra-ml-base[dev]"
```

### Environment Setup

```bash
# Set edition (affects resource limits and features)
export SUTRA_EDITION=community  # simple|community|enterprise

# HuggingFace token for gated models
export HF_TOKEN=your_hf_token_here

# Optional: Custom cache location
export TRANSFORMERS_CACHE=/path/to/cache
```

### Basic Service Example

```python
#!/usr/bin/env python3
"""
Example ML service using Sutra ML Base
"""

from sutra_ml_base import (
    BaseMlService, ServiceConfig, 
    ModelLoader, LoaderConfig, ModelType,
    setup_environment, setup_logging
)

class EmbeddingService(BaseMlService):
    async def load_model(self) -> bool:
        """Load embedding model on startup"""
        config = LoaderConfig(
            model_name="nomic-ai/nomic-embed-text-v1.5",
            model_type=ModelType.EMBEDDING
        )
        
        try:
            self.model, self.tokenizer, self.loader = ModelLoader.load_model(config)
            self.set_model_loaded(self.loader.get_model_info())
            return True
        except Exception as e:
            logger.error(f"Model loading failed: {e}")
            return False
    
    async def process_request(self, request):
        """Process embedding request"""
        # Your embedding logic here
        pass

    def get_service_info(self):
        return {
            "description": "High-performance embedding service",
            "supported_models": ["nomic-embed", "sentence-transformers"]
        }

if __name__ == "__main__":
    # Setup environment and logging
    setup_environment()
    logger = setup_logging("embedding-service")
    
    # Create and run service
    config = ServiceConfig(
        service_name="embedding-service",
        port=8888,
        enable_metrics=True
    )
    
    service = EmbeddingService(config)
    service.run()
```

### Docker Integration

```dockerfile
FROM python:3.11-slim

# Install sutra-ml-base
COPY packages/sutra-ml-base /tmp/sutra-ml-base
RUN pip install -e /tmp/sutra-ml-base[optimized]

# Your service code
COPY your_service.py /app/
WORKDIR /app

# Run with edition support
ENV SUTRA_EDITION=community
CMD ["python", "your_service.py"]
```

## API Reference

### Core Classes

#### **EditionManager**
Manages edition-aware features and resource limits.

```python
manager = EditionManager()
manager.get_batch_size_limit()      # Edition-specific batch size
manager.supports_custom_models()    # Feature availability
manager.can_load_model(size_gb)     # Model size validation
```

#### **ModelLoader**  
Universal model loading with caching and validation.

```python
config = LoaderConfig(
    model_name="model-name",
    model_type=ModelType.EMBEDDING,
    device="auto",
    max_memory_gb=4.0
)
model, tokenizer, loader = ModelLoader.load_model(config)
```

#### **BaseMlService**
FastAPI service framework with built-in observability.

```python
class MyService(BaseMlService):
    async def load_model(self) -> bool: ...
    async def process_request(self, request) -> Any: ...
    def get_service_info(self) -> Dict[str, Any]: ...
```

### Standard Endpoints

All services automatically get:

- **GET /** - Service information and status
- **GET /health** - Comprehensive health check  
- **GET /info** - Detailed service capabilities
- **GET /metrics** - Performance metrics and stats

### Configuration

#### **ServiceConfig**
```python
ServiceConfig(
    service_name="my-service",
    port=8888,
    workers=1,
    enable_metrics=True,
    require_auth=False,
    rate_limit_per_minute=1000
)
```

#### **LoaderConfig**
```python  
LoaderConfig(
    model_name="model-name",
    model_type=ModelType.EMBEDDING,
    device="auto",
    torch_dtype="float32",
    load_in_8bit=False,
    cache_dir="/tmp/.cache/huggingface"
)
```

## Development

### Testing

```bash
# Run tests
pytest packages/sutra-ml-base/tests/

# With coverage
pytest --cov=sutra_ml_base packages/sutra-ml-base/tests/

# Integration tests
pytest packages/sutra-ml-base/tests/integration/
```

### Code Quality

```bash
# Format code
black packages/sutra-ml-base/
isort packages/sutra-ml-base/

# Lint
flake8 packages/sutra-ml-base/
mypy packages/sutra-ml-base/
```

## Architecture

### Package Structure
```
sutra-ml-base/
├── sutra_ml_base/
│   ├── __init__.py          # Public API exports
│   ├── edition.py           # Edition management
│   ├── model_loader.py      # Universal model loading  
│   ├── service_base.py      # FastAPI service framework
│   ├── metrics.py           # Observability and monitoring
│   ├── cache.py             # Performance caching
│   ├── security.py          # Authentication and protection
│   └── utils.py             # Common utilities
├── tests/                   # Test suite
├── examples/               # Example services  
└── docs/                   # Documentation
```

### Design Principles

1. **Edition-First**: All features consider Simple/Community/Enterprise tiers
2. **Zero-Copy**: Minimize memory allocations and data movement  
3. **Observable**: Rich metrics and health monitoring built-in
4. **Composable**: Mix-and-match components for different service types
5. **Backwards Compatible**: Stable APIs across Sutra ecosystem versions
6. **Performance**: Sub-10ms request overhead, efficient caching
7. **Security**: Authentication, rate limiting, input validation by default

## License

MIT License - see LICENSE file for details.

**Built with ❤️ by the Sutra AI Team**
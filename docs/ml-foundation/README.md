# Sutra ML Foundation Architecture

## Overview

The **Sutra ML Foundation** (`sutra-ml-base`) is a comprehensive, edition-aware platform that provides the unified architecture for all ML services in the Sutra ecosystem. It eliminates code duplication, ensures consistency, and enables rapid development of new ML services.

## Architecture Principles

### 1. Edition-First Design
Every component is designed with Sutra's three-tier edition system as a first-class citizen:
- **Simple Edition**: Basic models, limited resources, essential features
- **Community Edition**: Better models, moderate resources, enhanced features
- **Enterprise Edition**: Best models, full resources, advanced features

### 2. Unified Service Pattern
All ML services inherit from `BaseMlService`, providing:
- Standardized FastAPI applications
- Consistent health, metrics, and info endpoints
- Automatic model loading and management
- Edition-aware resource limits
- Built-in caching and security

### 3. Zero Duplication
Common ML patterns are implemented once in the foundation:
- Model loading and validation
- Caching strategies
- Metrics collection
- Security management
- Configuration handling

## Core Components

### BaseMlService
The heart of all ML services, providing:

```python
from sutra_ml_base import BaseMlService, ServiceConfig

class MyMlService(BaseMlService):
    def __init__(self, config: ServiceConfig):
        super().__init__(config)
        
    async def load_model(self) -> bool:
        # Your model loading logic
        pass
        
    def _setup_service_routes(self):
        # Your service-specific endpoints
        pass
```

**Key Features:**
- Automatic FastAPI app creation
- Health monitoring and status management
- Built-in metrics collection
- Edition-aware initialization
- Graceful error handling

### EditionManager
Manages resource limits and features across editions:

```python
# Automatically configured based on SUTRA_EDITION environment
edition_manager = EditionManager()

# Edition-specific limits
batch_size = edition_manager.get_batch_size_limit()
cache_size = edition_manager.get_cache_size_gb()
model_limit = edition_manager.get_model_size_limit()

# Feature availability
if edition_manager.supports_advanced_caching():
    # Enable premium caching features
    pass
```

**Edition Specifications:**
| Feature | Simple | Community | Enterprise |
|---------|--------|-----------|------------|
| Batch Size | 32 | 64 | 128 |
| Cache Size | 128MB | 256MB | 512MB |
| Model Size | 1GB | 4GB | 16GB |
| Advanced Features | Basic | Enhanced | Full |

### ModelLoader
Universal model loading with edition awareness:

```python
from sutra_ml_base import ModelLoader, LoaderConfig, ModelType

config = LoaderConfig(
    model_name="microsoft/DialoGPT-medium",
    model_type=ModelType.GENERATIVE,
    device="auto",
    max_memory_gb=edition_manager.get_model_size_limit()
)

model, tokenizer, loader = ModelLoader.load_model(config, edition_manager)
```

**Features:**
- Automatic device selection (CPU/GPU)
- Memory limit enforcement
- Model verification and validation
- Caching with persistent storage
- Edition-appropriate model selection

### CacheManager
High-performance caching with edition limits:

```python
cache_config = CacheConfig(
    max_memory_mb=edition_manager.get_cache_size_gb() * 1024,
    max_items=10000,
    default_ttl_seconds=3600,
    persistent=edition_manager.supports_advanced_caching()
)

cache = CacheManager(cache_config)

# Cache usage
cache_key = cache.cache_key("operation", "input", "model", "params")
result = cache.get(cache_key)
if result is None:
    result = expensive_operation()
    cache.set(cache_key, result, ttl_seconds=1800)
```

### MetricsCollector
Standardized metrics across all services:

```python
metrics = MetricsCollector("my-service")

with metrics.timer("inference"):
    result = model.generate(input)

metrics.increment("requests_total")
metrics.gauge("model_memory_mb", model_memory)

# Get metrics for /metrics endpoint
stats = metrics.get_stats()
```

**Collected Metrics:**
- Request counts and rates
- Processing times and latencies
- Memory usage and model stats
- Error rates and types
- Cache hit/miss ratios

### SecurityManager
Authentication and authorization hooks:

```python
security = SecurityManager(edition_manager)

# Rate limiting (edition-aware)
@security.rate_limit()
async def my_endpoint():
    pass

# Authentication (enterprise+)
@security.require_auth()
async def premium_endpoint():
    pass
```

## Service Development Pattern

### 1. Create Service Class
```python
from sutra_ml_base import BaseMlService

class MyMlService(BaseMlService):
    def __init__(self, config: ServiceConfig):
        super().__init__(config)
        # Initialize service-specific components
        
    async def load_model(self) -> bool:
        # Load your ML model
        return True
        
    def get_service_info(self) -> Dict[str, Any]:
        # Return service-specific information
        return {"description": "My ML service"}
        
    def _setup_service_routes(self):
        # Add service-specific endpoints
        @self.app.post("/predict")
        async def predict(request: MyRequest):
            return await self.process_request(request)
```

### 2. Configure Service
```python
config = ServiceConfig(
    service_name="my-ml-service",
    service_version="2.0.0",
    port=8890,
    enable_metrics=True
)

service = MyMlService(config)
```

### 3. Run Service
```python
if __name__ == "__main__":
    service.run()
```

## Standardized Endpoints

Every ML service automatically gets:

- `GET /health` - Health status and model information
- `GET /info` - Service capabilities and limits  
- `GET /metrics` - Performance metrics and statistics
- `POST /shutdown` - Graceful shutdown (if enabled)

Plus service-specific endpoints defined in `_setup_service_routes()`.

## Configuration

### Environment Variables
```bash
# Edition configuration
SUTRA_EDITION=enterprise

# Service configuration  
PORT=8888
LOG_LEVEL=INFO
WORKERS=1

# ML-specific configuration
ML_DEVICE=auto
ML_CACHE_DIR=/tmp/.cache
ML_MODEL_VERIFICATION=true
```

### Service Config
```python
config = ServiceConfig(
    service_name="my-service",
    service_version="2.0.0",
    port=8888,
    workers=1,
    enable_metrics=True,
    enable_security=True,
    log_level="INFO"
)
```

## Best Practices

### 1. Edition Awareness
Always check edition capabilities:
```python
if self.edition_manager.supports_custom_models():
    # Allow custom model loading
    pass
else:
    # Restrict to default models
    pass
```

### 2. Resource Management
Respect edition limits:
```python
max_batch = min(request.batch_size, self.edition_manager.get_batch_size_limit())
```

### 3. Error Handling
Use consistent error patterns:
```python
try:
    result = process_request()
except Exception as e:
    logger.error(f"Processing failed: {e}")
    raise HTTPException(status_code=500, detail="Processing failed")
```

### 4. Metrics
Track important operations:
```python
with self.metrics.timer("model_inference"):
    result = self.model.predict(input)
    
self.metrics.increment("successful_predictions")
```

## Performance Characteristics

### Startup Time
- **Foundation Load**: ~500ms
- **Model Load**: Varies by model size and edition
- **Service Ready**: Typically 2-30 seconds depending on model

### Memory Usage
- **Foundation Overhead**: ~50MB
- **Per Service**: +model memory + cache allocation
- **Edition Limits**: Automatically enforced

### Throughput
- **Request Routing**: <1ms overhead
- **Caching**: 99%+ hit rate for repeated requests
- **Metrics**: Minimal performance impact

## Migration Guide

### From Legacy Services
1. **Replace imports**: Use `sutra_ml_base` components
2. **Inherit from BaseMlService**: Replace manual FastAPI setup
3. **Use EditionManager**: Replace hardcoded limits
4. **Implement standard methods**: `load_model()`, `get_service_info()`, etc.
5. **Update Docker**: Use shared ML foundation builds

### Example Migration
**Before:**
```python
app = FastAPI(title="My Service")
model = AutoModel.from_pretrained("model-name")

@app.post("/predict")
async def predict(request):
    return model(request.input)
```

**After:**
```python
class MyService(BaseMlService):
    async def load_model(self):
        config = LoaderConfig(model_name="model-name")
        self.model, _, _ = ModelLoader.load_model(config, self.edition_manager)
        return True
        
    def _setup_service_routes(self):
        @self.app.post("/predict")
        async def predict(request):
            return self.model(request.input)
```

## Troubleshooting

### Common Issues

**Import Errors:**
```bash
pip install -e packages/sutra-ml-base
```

**Model Loading Fails:**
- Check edition limits with `edition_manager.get_model_size_limit()`
- Verify model name and availability
- Check disk space and permissions

**Cache Issues:**
- Verify edition supports caching
- Check cache directory permissions
- Monitor cache memory usage

**Performance Issues:**
- Check batch size against edition limits
- Monitor metrics endpoint for bottlenecks
- Verify appropriate device selection (CPU/GPU)

### Debug Mode
```bash
LOG_LEVEL=DEBUG python main.py
```

Enables detailed logging of:
- Model loading progress
- Cache operations
- Request processing
- Edition limit checks

---

*Sutra ML Foundation v2.0.0*  
*World-Class ML Service Architecture*
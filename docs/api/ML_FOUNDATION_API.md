# ML Foundation API Reference

**Version:** 2.0.0  
**Last Updated:** October 27, 2025  
**Status:** ✅ **COMPLETE**

## Overview

The ML Foundation (`sutra-ml-base`) provides a standardized base class for all ML services in the Sutra ecosystem. All ML services inherit from `BaseMlService` and automatically expose consistent health, metrics, and information endpoints with edition-aware features.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    sutra-ml-base                            │
│                   (ML Foundation)                           │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌────────┐ │
│  │BaseMlService│ │EditionMgr   │ │ModelLoader  │ │Cache   │ │
│  │             │ │             │ │             │ │Manager │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └────────┘ │
├─────────────────────────────────────────────────────────────┤
│        Standardized Endpoints for All ML Services          │
└─────────────────────────────────────────────────────────────┘
                             │
              ┌──────────────┼──────────────┐
              ▼              ▼              ▼
    ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
    │ Embedding    │ │    NLG       │ │   Future     │
    │  Service     │ │  Service     │ │ ML Services  │
    │   :8889      │ │   :8890      │ │     :889X    │
    └──────────────┘ └──────────────┘ └──────────────┘
```

## Standard Endpoints

All ML services inheriting from `BaseMlService` automatically provide these endpoints:

### Health Endpoints

#### `GET /health`
Basic health check for service availability.

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2025-10-27T10:30:00Z",
  "service": "sutra-embedding-service",
  "version": "2.0.0"
}
```

#### `GET /health/detailed`
Comprehensive health check including dependencies and resource usage.

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2025-10-27T10:30:00Z",
  "service": "sutra-embedding-service",
  "version": "2.0.0",
  "edition": "enterprise",
  "checks": {
    "model_loaded": true,
    "cache_operational": true,
    "memory_usage_mb": 2048,
    "gpu_available": true,
    "disk_space_gb": 45.2
  },
  "dependencies": {
    "storage_server": "healthy",
    "embedding_model": "loaded"
  },
  "uptime_seconds": 3600
}
```

### Metrics Endpoints

#### `GET /metrics`
Prometheus-compatible metrics for monitoring and alerting.

**Response:**
```text
# HELP sutra_requests_total Total number of requests
# TYPE sutra_requests_total counter
sutra_requests_total{service="embedding",edition="enterprise"} 1542

# HELP sutra_request_duration_seconds Request duration in seconds
# TYPE sutra_request_duration_seconds histogram
sutra_request_duration_seconds_bucket{le="0.005"} 890
sutra_request_duration_seconds_bucket{le="0.01"} 1234
sutra_request_duration_seconds_bucket{le="0.025"} 1456
sutra_request_duration_seconds_bucket{le="+Inf"} 1542

# HELP sutra_cache_hits_total Cache hits
# TYPE sutra_cache_hits_total counter
sutra_cache_hits_total{service="embedding"} 456

# HELP sutra_memory_usage_bytes Current memory usage
# TYPE sutra_memory_usage_bytes gauge
sutra_memory_usage_bytes{service="embedding"} 2147483648
```

### Information Endpoints

#### `GET /info`
Service information including edition limits and capabilities.

**Response:**
```json
{
  "service": "sutra-embedding-service",
  "version": "2.0.0",
  "edition": "enterprise",
  "model": "nomic-embed-text-v1.5",
  "vector_dimension": 768,
  "limits": {
    "max_batch_size": 100,
    "max_text_length": 8192,
    "rate_limit_per_minute": 5000,
    "max_concurrent_requests": 50
  },
  "features": {
    "batch_processing": true,
    "caching": true,
    "gpu_acceleration": true,
    "custom_models": true
  },
  "endpoints": [
    "/generate",
    "/batch",
    "/health",
    "/metrics",
    "/info"
  ]
}
```

## Edition-Aware Features

All ML Foundation services automatically scale based on the detected edition:

### Resource Limits by Edition

| Feature | Simple | Community | Enterprise |
|---------|--------|-----------|------------|
| **Batch Size** | 10 | 50 | 100 |
| **Text Length** | 2048 | 4096 | 8192 |
| **Rate Limit/min** | 100 | 1000 | 5000 |
| **Concurrent Requests** | 5 | 20 | 50 |
| **Cache Size** | 100MB | 500MB | 2GB |
| **GPU Support** | ❌ | ✅ | ✅ |
| **Custom Models** | ❌ | ❌ | ✅ |

### Edition Detection

Services automatically detect the current edition from:
1. `SUTRA_EDITION` environment variable
2. License validation via `SUTRA_LICENSE_KEY`
3. Fallback to "simple" edition if no license

```bash
# Set edition explicitly
export SUTRA_EDITION=enterprise
export SUTRA_LICENSE_KEY=your-license-key
export SUTRA_LICENSE_SECRET=your-secret

# Start service (automatically detects edition)
python -m sutra_embedding_service.main
```

## Error Handling

### Standard Error Responses

All ML Foundation services return consistent error formats:

```json
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Request rate limit exceeded for current edition",
    "details": {
      "current_edition": "community",
      "limit_per_minute": 1000,
      "retry_after_seconds": 60
    }
  },
  "timestamp": "2025-10-27T10:30:00Z",
  "request_id": "req_123456789"
}
```

### Common Error Codes

| Code | Description | HTTP Status |
|------|-------------|-------------|
| `INVALID_INPUT` | Malformed request data | 400 |
| `TEXT_TOO_LONG` | Input exceeds length limit | 400 |
| `RATE_LIMIT_EXCEEDED` | Too many requests | 429 |
| `MODEL_NOT_LOADED` | ML model unavailable | 503 |
| `INTERNAL_ERROR` | Unexpected service error | 500 |

## Authentication & Security

### Development Mode (Default)
- No authentication required
- Services bind to localhost only
- Suitable for development and testing

### Production Mode
Enable with `SUTRA_SECURE_MODE=true`:

- **TLS 1.3 encryption** for all connections
- **JWT authentication** with service-specific keys
- **RBAC** (Role-Based Access Control)
- **Request signing** with HMAC-SHA256

```bash
# Enable secure mode
export SUTRA_SECURE_MODE=true
export SUTRA_JWT_SECRET=your-jwt-secret
export SUTRA_TLS_CERT=/path/to/cert.pem
export SUTRA_TLS_KEY=/path/to/key.pem
```

## Integration Examples

### Python Client

```python
from sutra_client import SutraClient

# Initialize client
client = SutraClient(base_url="http://localhost:8889")

# Health check
health = await client.get("/health")
print(f"Service status: {health['status']}")

# Get service info
info = await client.get("/info")
print(f"Edition: {info['edition']}")
print(f"Rate limit: {info['limits']['rate_limit_per_minute']}/min")
```

### cURL Examples

```bash
# Health check
curl http://localhost:8889/health

# Detailed health with dependencies
curl http://localhost:8889/health/detailed

# Prometheus metrics
curl http://localhost:8889/metrics

# Service information
curl http://localhost:8889/info
```

### Docker Health Checks

```yaml
services:
  embedding-service:
    image: sutra/embedding-service:2.0.0
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8889/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 60s
```

## Monitoring & Observability

### Prometheus Integration

All ML Foundation services expose Prometheus metrics on `/metrics`:

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'sutra-ml-services'
    static_configs:
      - targets: 
        - 'embedding-service:8889'
        - 'nlg-service:8890'
    scrape_interval: 30s
    metrics_path: '/metrics'
```

### Grafana Dashboards

Standard metrics available for visualization:
- **Request rate** and **latency percentiles**
- **Error rates** by service and endpoint
- **Cache hit/miss ratios**
- **Resource utilization** (CPU, memory, GPU)
- **Edition compliance** and **rate limiting**

### Alerting Rules

```yaml
# Example Prometheus alerting rules
groups:
  - name: sutra-ml-services
    rules:
      - alert: MLServiceDown
        expr: up{job="sutra-ml-services"} == 0
        for: 1m
        
      - alert: HighErrorRate
        expr: rate(sutra_requests_total{status="error"}[5m]) > 0.1
        for: 2m
        
      - alert: RateLimitHit
        expr: increase(sutra_rate_limit_exceeded_total[1m]) > 0
        for: 1m
```

## Development Guide

### Creating Custom ML Services

Extend `BaseMlService` for consistent behavior:

```python
from sutra_ml_base import BaseMlService
from fastapi import FastAPI
import asyncio

class CustomMLService(BaseMlService):
    def __init__(self):
        super().__init__(
            service_name="custom-ml-service",
            service_port=8891,
            model_path="models/custom-model"
        )
        
    async def initialize_model(self):
        """Override to load your specific model"""
        self.model = await self.model_loader.load_model(
            self.config.model_path
        )
        
    def create_service_routes(self, app: FastAPI):
        """Add your custom endpoints"""
        
        @app.post("/custom-endpoint")
        async def custom_endpoint(data: dict):
            # Edition-aware processing
            limits = self.edition_manager.get_limits()
            
            # Use built-in caching
            cached = await self.cache_manager.get(f"custom:{data}")
            if cached:
                return cached
                
            # Process with your model
            result = await self.process_with_model(data)
            
            # Cache result
            await self.cache_manager.set(
                f"custom:{data}", 
                result, 
                ttl=limits.cache_ttl
            )
            
            return result

# Start service
if __name__ == "__main__":
    service = CustomMLService()
    asyncio.run(service.start())
```

### Testing ML Services

```python
import pytest
from httpx import AsyncClient
from your_service import CustomMLService

@pytest.mark.asyncio
async def test_service_health():
    service = CustomMLService()
    await service.initialize()
    
    async with AsyncClient(
        app=service.app, 
        base_url="http://test"
    ) as client:
        
        # Test health endpoint
        response = await client.get("/health")
        assert response.status_code == 200
        assert response.json()["status"] == "healthy"
        
        # Test metrics endpoint
        response = await client.get("/metrics")
        assert response.status_code == 200
        assert "sutra_requests_total" in response.text
```

## Migration Guide

### From Legacy Services

If migrating from pre-ML Foundation services:

1. **Update imports:**
   ```python
   # Old
   from fastapi import FastAPI
   
   # New  
   from sutra_ml_base import BaseMlService
   ```

2. **Inherit from BaseMlService:**
   ```python
   # Old
   app = FastAPI()
   
   # New
   class MyService(BaseMlService):
       def __init__(self):
           super().__init__(service_name="my-service")
   ```

3. **Remove manual endpoint definitions:**
   ```python
   # Remove these - automatically provided by BaseMlService
   # @app.get("/health")
   # @app.get("/metrics") 
   # @app.get("/info")
   ```

4. **Use edition-aware features:**
   ```python
   # Access edition limits
   limits = self.edition_manager.get_limits()
   
   # Use built-in caching
   await self.cache_manager.set("key", value)
   
   # Automatic metrics collection
   # (no manual instrumentation needed)
   ```

## Best Practices

### Performance
- **Use async/await** for all I/O operations
- **Enable caching** for expensive computations  
- **Batch requests** when possible
- **Monitor metrics** regularly for bottlenecks

### Security
- **Always validate inputs** before processing
- **Use secure mode** in production environments
- **Rotate JWT secrets** regularly
- **Monitor for rate limiting violations**

### Reliability
- **Implement proper error handling** for all endpoints
- **Use circuit breakers** for external dependencies
- **Configure appropriate timeouts**
- **Test failure scenarios** thoroughly

### Monitoring
- **Set up Prometheus scraping** for all services
- **Create Grafana dashboards** for visualization
- **Configure alerting rules** for critical issues
- **Monitor edition compliance** and usage patterns

---

## Changelog

### 2.0.0 (2025-10-27)
- ✅ Initial ML Foundation API documentation
- ✅ Complete endpoint reference for all ML services
- ✅ Edition-aware feature documentation
- ✅ Security and monitoring guidelines
- ✅ Development and migration guides
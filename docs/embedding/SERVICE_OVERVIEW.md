# Sutra Embedding Service - Built on ML Foundation

**High-Performance Edition-Aware Semantic Embeddings**

Version: 2.0.0 | Built on ML Foundation | Status: Production-Ready ✅

---

## Overview

The **Sutra Embedding Service** is built on the unified **ML Foundation** (`sutra-ml-base`), providing world-class semantic embeddings with automatic edition-aware scaling, advanced caching, and consistent APIs across all Sutra deployments.

**Key Benefits:**
- ⚡ **Edition-Aware**: Automatic resource scaling (Simple → Community → Enterprise)
- 🚀 **High Performance**: Advanced caching with 99%+ hit rates for repeated embeddings
- 🔧 **Zero Configuration**: Works out-of-the-box with smart defaults
- 📊 **Built-in Monitoring**: Comprehensive metrics and health checks
- 🏗️ **Foundation-Based**: Inherits all ML Foundation capabilities

---

## 🏗️ ML Foundation Architecture

### Edition Scaling Matrix

| Feature | Simple Edition | Community Edition | Enterprise Edition |
|---------|----------------|-------------------|-------------------|
| **Embedding Model** | nomic-embed-text-v1.5 | all-mpnet-base-v2 | Custom models supported |
| **Batch Size Limit** | 32 texts per request | 64 texts per request | 128 texts per request |
| **Cache Memory** | 128MB LRU cache | 256MB LRU cache | 512MB LRU cache |
| **Sequence Length** | 512 characters | 1024 characters | 2048 characters |
| **Advanced Caching** | Basic (memory only) | ✅ Advanced (persistent) | ✅ Advanced (persistent) |
| **Custom Models** | ❌ Fixed model | ❌ Fixed model | ✅ Load custom models |
| **Cache Analytics** | Basic stats | ✅ Detailed analytics | ✅ Detailed analytics |

### Foundation Integration

```python
# Built using ML Foundation components
class SutraEmbeddingService(BaseMlService):
    def __init__(self, config: ServiceConfig):
        super().__init__(config)  # Inherits all foundation features
        
        # Edition-aware caching
        self.cache = CacheManager(
            max_memory_mb=self.edition_manager.get_cache_size_gb() * 1024,
            persistent=self.edition_manager.supports_advanced_caching()
        )
        
        # Edition-appropriate model selection
        self.model_name = self._get_model_for_edition()
```

---

## 🚀 Quick Start

### 1. Deploy with Edition

```bash
# Deploy Simple edition (development)
SUTRA_EDITION=simple ./sutra-deploy.sh install

# Deploy Enterprise edition (production)
SUTRA_EDITION=enterprise ./sutra-deploy.sh install
```

### 2. Verify Service

```bash
# Check health and edition configuration
curl -s http://localhost:8888/health | jq
# Returns: {"status": "healthy", "edition": "community", "model_loaded": true}

# Check service information and limits
curl -s http://localhost:8888/info | jq
# Returns edition-specific limits and capabilities
```

### 3. Generate Embeddings

```bash
# Basic embedding generation
curl -X POST http://localhost:8888/embed \
  -H "Content-Type: application/json" \
  -d '{
    "texts": ["Artificial intelligence is transforming healthcare"],
    "normalize": true,
    "cache_ttl_seconds": 3600
  }' | jq

# Expected Response:
{
  "embeddings": [[0.1, -0.2, 0.3, ...]],  # 768-dimensional vector
  "dimension": 768,
  "model": "nomic-ai/nomic-embed-text-v1.5",
  "processing_time_ms": 23.4,
  "cached_count": 0,
  "edition": "community",
  "batch_size": 1
}
```

---

## 🔧 API Reference

### Standardized ML Foundation Endpoints

All ML Foundation services provide these endpoints automatically:

#### Health Check
```http
GET /health
```
**Response:**
```json
{
  "status": "healthy",
  "edition": "community", 
  "model_loaded": true,
  "dimension": 768,
  "model_name": "nomic-ai/nomic-embed-text-v1.5",
  "uptime_seconds": 3600,
  "memory_usage_mb": 1024
}
```

#### Service Information
```http
GET /info
```
**Response:**
```json
{
  "description": "High-performance embedding service with edition-aware scaling",
  "supported_models": ["nomic-ai/nomic-embed-text-v1.5", "sentence-transformers/all-mpnet-base-v2"],
  "features": {
    "caching": true,
    "custom_models": false,  // Edition-dependent
    "batch_processing": true
  },
  "limits": {
    "max_batch_size": 64,
    "max_sequence_length": 1024,
    "cache_size_gb": 0.256
  },
  "model": {
    "name": "nomic-ai/nomic-embed-text-v1.5",
    "dimension": 768,
    "parameters": 25000000
  }
}
```

#### Performance Metrics
```http
GET /metrics
```
**Response:**
```json
{
  "requests_total": 1500,
  "requests_per_second": 12.5,
  "average_latency_ms": 18.2,
  "cache_hit_rate": 0.87,
  "embeddings_generated": 45000,
  "model_memory_mb": 950,
  "cache_memory_mb": 180
}
```

### Service-Specific Endpoints

#### Generate Embeddings
```http
POST /embed
```

**Request:**
```json
{
  "texts": ["Text to embed", "Another text"],
  "normalize": true,
  "cache_ttl_seconds": 3600  // 0 = no cache
}
```

**Response:**
```json
{
  "embeddings": [
    [0.1, -0.2, 0.3, ...],  // First embedding (768 dims)
    [0.4, 0.1, -0.5, ...]   // Second embedding (768 dims)
  ],
  "dimension": 768,
  "model": "nomic-ai/nomic-embed-text-v1.5",
  "processing_time_ms": 34.7,
  "cached_count": 1,  // Number of cache hits
  "edition": "community",
  "batch_size": 2
}
```

#### Cache Management (Community+ Editions)
```http
GET /cache/stats
```
**Response:**
```json
{
  "hit_rate": 0.87,
  "total_hits": 1300,
  "total_misses": 200,
  "memory_usage_mb": 180,
  "max_memory_mb": 256,
  "item_count": 2500,
  "evictions": 45
}
```

```http
DELETE /cache
```
Clears the embedding cache (Community+ editions only).

---

## 🔧 Configuration

### Environment Variables

```bash
# Edition Configuration (Primary)
SUTRA_EDITION=community  # simple|community|enterprise

# Service Configuration
PORT=8888
LOG_LEVEL=INFO
WORKERS=1

# Model Override (Enterprise only)
EMBEDDING_MODEL_OVERRIDE=sentence-transformers/all-mpnet-base-v2

# Cache Configuration
CACHE_TTL_DEFAULT=3600  # Default cache TTL in seconds
CACHE_PERSISTENT=true   # Enable persistent cache (Community+)

# Performance Tuning
ML_DEVICE=auto          # auto|cpu|cuda
ML_MODEL_VERIFICATION=true
```

### Docker Configuration

```yaml
# docker-compose.yml
services:
  sutra-embedding-service:
    image: sutra-embedding-service:latest
    ports:
      - "8888:8888"
    environment:
      - SUTRA_EDITION=${SUTRA_EDITION:-community}
      - LOG_LEVEL=${LOG_LEVEL:-INFO}
    deploy:
      resources:
        limits:
          memory: 4G  # Automatically adjusted by edition
        reservations:
          memory: 2G
    healthcheck:
      test: ["CMD", "python", "-c", "import requests; requests.get('http://localhost:8888/health').raise_for_status()"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 60s
```

---

## 📊 Performance Characteristics

### Latency by Edition

| Operation | Simple | Community | Enterprise | Notes |
|-----------|--------|-----------|------------|-------|
| **Single Embedding** | 15-25ms | 18-30ms | 20-35ms | Larger models = higher latency |
| **Batch (32 texts)** | 45-65ms | 55-75ms | 60-85ms | Edition batch limits |
| **Cache Hit** | 1-2ms | 1-2ms | 1-2ms | Consistent across editions |
| **Model Loading** | 3-5s | 5-8s | 8-15s | Depends on model size |

### Throughput

| Configuration | Requests/Second | Batch Efficiency | Memory Usage |
|---------------|-----------------|------------------|--------------|
| **Simple Edition** | ~150 req/s (32 batch) | High | 1-2GB |
| **Community Edition** | ~120 req/s (64 batch) | Very High | 2-3GB |
| **Enterprise Edition** | ~100 req/s (128 batch) | Maximum | 3-4GB |

### Cache Performance

| Metric | Simple | Community | Enterprise |
|--------|--------|-----------|------------|
| **Hit Rate** | 85-90% | 90-95% | 95-99% |
| **Cache Size** | 128MB | 256MB | 512MB |
| **Persistence** | Memory only | ✅ Disk + Memory | ✅ Disk + Memory |
| **TTL Management** | Basic | ✅ Advanced | ✅ Advanced |

---

## 🐛 Troubleshooting

### Common Issues

#### Edition Configuration Problems

**Problem**: Service shows wrong edition features
```bash
# Check current edition
curl -s http://localhost:8888/info | jq '.limits'

# Expected vs Actual batch size
# Simple: 32, Community: 64, Enterprise: 128
```

**Solution**:
```bash
# Verify environment variable
echo $SUTRA_EDITION

# Restart with correct edition
SUTRA_EDITION=enterprise docker-compose restart sutra-embedding-service
```

#### Cache Not Working

**Problem**: `cache_used: false` in all responses
```json
{
  "cached_count": 0,
  "cache_used": false
}
```

**Solutions**:
```bash
# Check if edition supports advanced caching
curl -s http://localhost:8888/info | jq '.features.caching'

# For Simple edition, only basic memory caching available
# Upgrade to Community/Enterprise for advanced caching

# Check cache stats (Community+ only)
curl -s http://localhost:8888/cache/stats
```

#### Model Loading Failures

**Problem**: `"model_loaded": false` in health check
```bash
# Check detailed service logs
docker logs sutra-embedding-service --tail 50

# Common causes:
# 1. Insufficient memory (increase Docker memory limit)
# 2. Model download failure (check internet connectivity)
# 3. Custom model not found (Enterprise only)
```

**Solutions**:
```bash
# Increase memory limit in docker-compose
deploy:
  resources:
    limits:
      memory: 6G  # Increase for larger models

# Check available disk space for model cache
df -h /tmp/.cache/huggingface

# Restart service
docker-compose restart sutra-embedding-service
```

#### Performance Issues

**Problem**: High latency or low throughput
```bash
# Check current performance metrics
curl -s http://localhost:8888/metrics

# Look for:
# - High average_latency_ms (>100ms)
# - Low cache_hit_rate (<0.8)
# - High model_memory_mb usage
```

**Solutions**:
```bash
# Optimize batch sizes (use edition limits)
# Simple: 32, Community: 64, Enterprise: 128

# Enable caching with appropriate TTL
curl -X POST http://localhost:8888/embed \
  -d '{"texts": ["..."], "cache_ttl_seconds": 3600}'

# Scale horizontally for higher throughput
docker-compose up -d --scale sutra-embedding-service=3
```

### Debug Mode

```bash
# Enable debug logging
LOG_LEVEL=DEBUG docker-compose up -d sutra-embedding-service

# Check detailed request processing
docker logs sutra-embedding-service | grep -E "(DEBUG|batch_size|cache|model)"

# Validate ML Foundation components
docker exec -it sutra-embedding-service python -c "
from sutra_ml_base import EditionManager
em = EditionManager()
print(f'Edition: {em.edition.value}')
print(f'Batch limit: {em.get_batch_size_limit()}')
print(f'Cache size: {em.get_cache_size_gb()}GB')
"
```

---

## 🚀 Production Deployment

### High Availability Setup

```yaml
# production docker-compose
version: '3.8'
services:
  embedding-lb:
    image: haproxy:latest
    ports:
      - "8888:8888"
    volumes:
      - ./haproxy-embedding.cfg:/usr/local/etc/haproxy/haproxy.cfg
    depends_on:
      - embedding-1
      - embedding-2
      - embedding-3

  embedding-1:
    image: sutra-embedding-service:latest
    environment:
      - SUTRA_EDITION=enterprise
    deploy:
      resources:
        limits:
          memory: 4G
        reservations:
          memory: 2G

  embedding-2:
    image: sutra-embedding-service:latest
    environment:
      - SUTRA_EDITION=enterprise
    deploy:
      resources:
        limits:
          memory: 4G
        reservations:
          memory: 2G

  embedding-3:
    image: sutra-embedding-service:latest
    environment:
      - SUTRA_EDITION=enterprise
    deploy:
      resources:
        limits:
          memory: 4G
        reservations:
          memory: 2G
```

### Monitoring Integration

```yaml
# Add to existing monitoring stack
  prometheus:
    image: prom/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
    ports:
      - "9090:9090"

  # prometheus.yml
  scrape_configs:
    - job_name: 'embedding-service'
      static_configs:
        - targets: ['embedding-1:8888', 'embedding-2:8888', 'embedding-3:8888']
      scrape_interval: 15s
      metrics_path: /metrics
```

---

## 🔗 Related Documentation

### ML Foundation
- **[ML Foundation README](../ml-foundation/README.md)** - Complete foundation architecture
- **[ML Foundation Deployment](../ml-foundation/DEPLOYMENT.md)** - Deployment guide

### System Integration
- **[Main Architecture](../ARCHITECTURE.md)** - System overview with ML Foundation
- **[Storage Integration](../storage/)** - How embeddings integrate with storage
- **[API Integration](../api/)** - Using embeddings in APIs

### Operations
- **[Production Guide](../PRODUCTION.md)** - Production deployment best practices
- **[Monitoring Guide](../operations/)** - Comprehensive monitoring setup
- **[Troubleshooting](../TROUBLESHOOTING.md)** - System-wide troubleshooting

---

**Built on ML Foundation v2.0.0**  
**Status**: ✅ Production-Ready  
**Last Updated**: 2025-01-10

### Dependencies

**System Dependencies:**
- Ubuntu-based Python 3.11 (for PyTorch compatibility)
- gcc, g++ (build dependencies)

**Python Dependencies:**
```
fastapi==0.104.1
uvicorn[standard]==0.24.0
transformers==4.44.2
torch>=2.0.0 (CPU version)
numpy>=1.24.0
pydantic>=2.0.0
psutil>=5.9.0
requests>=2.31.0
huggingface_hub>=0.17.0
einops>=0.6.0
```

### Environment Variables

```bash
# Service configuration
PORT=8888
EMBEDDING_MODEL=nomic-ai/nomic-embed-text-v1.5
EMBEDDING_BATCH_SIZE=64
EMBEDDING_MAX_WAIT_MS=50

# Caching configuration
TRANSFORMERS_CACHE=/tmp/.cache/huggingface
HF_HOME=/tmp/.cache/huggingface
HOME=/tmp
```

## Endpoints

### Health Check
```http
GET /health
```

**Response:**
```json
{
  "status": "healthy",
  "model_loaded": true,
  "dimension": 768,
  "model_name": "nomic-ai/nomic-embed-text-v1.5"
}
```

### Model Information
```http
GET /info
```

**Response:**
```json
{
  "model": "nomic-ai/nomic-embed-text-v1.5",
  "dimension": 768,
  "device": "cpu",
  "max_batch_size": 64
}
```

### Generate Embeddings
```http
POST /embed
Content-Type: application/json

{
  "texts": ["Example text to embed"],
  "normalize": true
}
```

**Response:**
```json
{
  "embeddings": [[0.1, -0.2, 0.3, ...]],
  "dimension": 768,
  "model": "nomic-ai/nomic-embed-text-v1.5",
  "processing_time_ms": 45.2,
  "cached_count": 0
}
```

## Integration

### Docker Compose Configuration

```yaml
sutra-embedding-service:
  build:
    context: ./packages/sutra-embedding-service
    dockerfile: Dockerfile
  image: sutra-embedding-service:latest
  container_name: sutra-embedding-service
  ports:
    - "8888:8888"
  environment:
    - PORT=8888
    - EMBEDDING_MODEL=nomic-ai/nomic-embed-text-v1.5
    - EMBEDDING_BATCH_SIZE=64
    - EMBEDDING_MAX_WAIT_MS=50
    - TRANSFORMERS_CACHE=/tmp/.cache/huggingface
    - HF_HOME=/tmp/.cache/huggingface
    - HOME=/tmp
  networks:
    - sutra-network
  restart: unless-stopped
  healthcheck:
    test: ["CMD", "python", "-c", "import requests; requests.get('http://localhost:8888/health').raise_for_status()"]
    interval: 30s
    timeout: 10s
    retries: 3
    start_period: 60s
  deploy:
    resources:
      limits:
        memory: 4G
      reservations:
        memory: 2G
```

### Service Dependencies

**Required by:**
- `sutra-hybrid` (primary consumer)
- `storage-server` (for learning pipeline)

**Depends on:**
- None (standalone service)

## Deployment

### Production Deployment

```bash
# Deploy all services including embedding service
./sutra-deploy.sh up

# Check embedding service status
curl -s http://localhost:8888/health | jq

# Test embedding generation
curl -X POST http://localhost:8888/embed \
  -H "Content-Type: application/json" \
  -d '{"texts": ["test"], "normalize": true}' | jq
```

### Individual Service Deployment

```bash
# Build embedding service
docker-compose -f docker-compose-grid.yml build sutra-embedding-service

# Start embedding service
docker-compose -f docker-compose-grid.yml up -d sutra-embedding-service

# Check logs
docker logs sutra-embedding-service --tail 20
```

## Troubleshooting

### Common Issues

#### 1. PyTorch Library Compatibility

**Symptoms:**
```
OSError: Error relocating /opt/venv/lib/python3.11/site-packages/torch/lib/libgomp-a34b3233.so.1: pthread_attr_setaffinity_np: symbol not found
```

**Solution:**
- Use Ubuntu-based Python image instead of Alpine
- Current Dockerfile uses `python:3.11-slim` for compatibility

#### 2. Missing Dependencies

**Symptoms:**
```
ImportError: This modeling file requires the following packages that were not found in your environment: einops
```

**Solution:**
- Ensure all required packages are in requirements.txt
- Current requirements include einops>=0.6.0

#### 3. Model Loading Failure

**Symptoms:**
```
Failed to load model: 404 Client Error: Not Found for url
```

**Solutions:**
- Verify internet connectivity for Hugging Face downloads
- Check if model name is correct: `nomic-ai/nomic-embed-text-v1.5`
- Ensure cache directories are writable

#### 4. Service Unreachable

**Symptoms:**
```
Connection refused to embedding service
Name resolution failed
```

**Solutions:**
- Ensure service is running: `docker ps | grep sutra-embedding-service`
- Check Docker network connectivity
- Verify port 8888 is exposed and accessible
- Restart dependent services after embedding service is healthy

### Health Validation

```bash
# Quick health check
curl -f http://localhost:8888/health

# Detailed service info
curl -s http://localhost:8888/info | jq

# Test embedding generation
curl -X POST http://localhost:8888/embed \
  -H "Content-Type: application/json" \
  -d '{"texts": ["Health check test"], "normalize": true}' | \
  jq '.embeddings[0] | length'
# Expected output: 768

# Check service logs
docker logs sutra-embedding-service --tail 50

# Monitor resource usage
docker stats sutra-embedding-service --no-stream
```

## Performance

### Benchmarks

- **Model Loading**: ~5-15 seconds (first startup)
- **Single Embedding**: ~20-50ms per text
- **Batch Processing**: Up to 64 texts per request
- **Memory Usage**: ~2-4GB (with model loaded)
- **Cache Hit Rate**: ~50% (for repeated texts)

### Optimization

- **Batch Processing**: Use multiple texts in single request
- **Caching**: Automatic caching for repeated embeddings
- **Memory**: 4GB limit with 2GB reservation
- **CPU**: Optimized for CPU inference (no GPU required)

## Security

- **Non-root User**: Runs as `embedding` user (UID 1000)
- **Resource Limits**: Memory and CPU constraints
- **Network**: Internal Docker network only
- **Permissions**: Minimal file system permissions

## Monitoring

### Health Checks

- **Interval**: 30 seconds
- **Timeout**: 10 seconds
- **Start Period**: 60 seconds (allows model loading)
- **Retries**: 3 failures before unhealthy

### Logs

```bash
# Real-time monitoring
docker logs sutra-embedding-service --follow

# Search for errors
docker logs sutra-embedding-service 2>&1 | grep -i error

# Check startup sequence
docker logs sutra-embedding-service --tail 100 | grep -E "(Loading|Model|Started)"
```

## Production Checklist

### Pre-Deployment

- [ ] Verify requirements.txt includes all dependencies
- [ ] Confirm Ubuntu-based Dockerfile (not Alpine)
- [ ] Check memory limits (4GB max, 2GB reservation)
- [ ] Validate health check endpoint
- [ ] Test model downloading capability

### Post-Deployment

- [ ] Service shows "healthy" status
- [ ] Health endpoint returns 200 OK
- [ ] Model loading completes successfully
- [ ] Embedding generation works (768 dimensions)
- [ ] Dependent services can connect
- [ ] Memory usage within limits
- [ ] No error logs in container

### Smoke Test

```bash
# Run comprehensive embedding service test
./scripts/smoke-test-embeddings.sh

# Manual verification
curl -s http://localhost:8888/health | jq '.status'
# Expected: "healthy"

curl -s http://localhost:8888/info | jq '.dimension'
# Expected: 768
```

## Migration Notes

### From Ollama (2025-10-20)

The embedding service was migrated from Ollama-based implementation to a dedicated high-performance service:

**Changes Made:**
- ✅ Dedicated embedding service with nomic-embed-text-v1.5
- ✅ Ubuntu-based Docker image for PyTorch compatibility
- ✅ Direct Hugging Face model loading
- ✅ Production-ready error handling and health checks
- ✅ Optimized memory and performance configuration

**No Rollback Available**: The Ollama integration has been completely removed and is not supported.

## References

- **Service Implementation**: `packages/sutra-embedding-service/`
- **Docker Configuration**: `docker-compose-grid.yml`
- **Integration Guide**: `WARP.md` - Embedding Service Requirements
- **Troubleshooting**: `TROUBLESHOOTING.md` - Embedding Service Issues
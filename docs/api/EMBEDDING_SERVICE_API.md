# Embedding Service API

**Service:** `sutra-embedding-service`  
**Port:** 8889  
**Version:** 2.0.0  
**Last Updated:** October 27, 2025

## Overview

The Sutra Embedding Service generates high-quality semantic embeddings using state-of-the-art models. Built on the ML Foundation (`sutra-ml-base`), it provides edition-aware scaling, intelligent caching, and production-ready reliability.

**Key Features:**
- **nomic-embed-text-v1.5** model (768-dimensional vectors)
- **Edition-aware batch processing** (10-100 texts per batch)
- **Intelligent caching** with LRU+TTL eviction
- **GPU acceleration** (Community/Enterprise editions)
- **Automatic rate limiting** based on edition

## Base URL

```
http://localhost:8889  # Development
https://embeddings.yourdomain.com  # Production
```

## Authentication

### Development Mode (Default)
No authentication required - service runs on localhost only.

### Production Mode
JWT authentication required when `SUTRA_SECURE_MODE=true`:

```bash
# Include JWT token in requests
curl -H "Authorization: Bearer ${JWT_TOKEN}" \
     http://localhost:8889/generate
```

## Endpoints

### Core Embedding Endpoints

#### `POST /generate`
Generate embeddings for a single text input.

**Request:**
```json
{
  "text": "Your text to embed goes here",
  "normalize": true
}
```

**Parameters:**
- `text` (string, required): Input text to embed (max length varies by edition)
- `normalize` (boolean, optional): Whether to normalize vector to unit length (default: true)

**Response:**
```json
{
  "embedding": [0.1234, -0.5678, 0.9012, ...],
  "dimension": 768,
  "model": "nomic-embed-text-v1.5",
  "text_length": 42,
  "processing_time_ms": 15.7,
  "cached": false
}
```

**Edition Limits:**
| Edition | Max Text Length | Rate Limit/min |
|---------|----------------|----------------|
| Simple | 2,048 chars | 100 |
| Community | 4,096 chars | 1,000 |
| Enterprise | 8,192 chars | 5,000 |

#### `POST /batch`
Generate embeddings for multiple texts in a single request.

**Request:**
```json
{
  "texts": [
    "First text to embed",
    "Second text to embed",
    "Third text to embed"
  ],
  "normalize": true
}
```

**Parameters:**
- `texts` (array[string], required): List of texts to embed
- `normalize` (boolean, optional): Whether to normalize vectors (default: true)

**Response:**
```json
{
  "embeddings": [
    [0.1234, -0.5678, 0.9012, ...],
    [0.2345, -0.6789, 0.8901, ...],
    [0.3456, -0.7890, 0.7890, ...]
  ],
  "dimension": 768,
  "model": "nomic-embed-text-v1.5",
  "batch_size": 3,
  "processing_time_ms": 45.2,
  "cache_hits": 1,
  "cache_misses": 2
}
```

**Edition Limits:**
| Edition | Max Batch Size | Concurrent Batches |
|---------|----------------|-------------------|
| Simple | 10 texts | 2 |
| Community | 50 texts | 10 |
| Enterprise | 100 texts | 25 |

### Standard ML Foundation Endpoints

The embedding service inherits these endpoints from `BaseMlService`:

#### `GET /health`
Service health check.

#### `GET /health/detailed` 
Detailed health including model status and dependencies.

#### `GET /metrics`
Prometheus metrics for monitoring.

#### `GET /info`
Service information and edition limits.

**See [ML Foundation API](./ML_FOUNDATION_API.md) for complete documentation of standard endpoints.**

## Edition-Specific Features

### Simple Edition
- **Text processing:** Up to 2,048 characters
- **Batch size:** Maximum 10 texts
- **Caching:** 100MB cache
- **GPU:** Not available
- **Rate limiting:** 100 requests/minute

```bash
export SUTRA_EDITION=simple
python -m sutra_embedding_service.main
```

### Community Edition  
- **Text processing:** Up to 4,096 characters
- **Batch size:** Maximum 50 texts
- **Caching:** 500MB cache with intelligent prefetching
- **GPU:** CUDA acceleration available
- **Rate limiting:** 1,000 requests/minute

```bash
export SUTRA_EDITION=community
python -m sutra_embedding_service.main
```

### Enterprise Edition
- **Text processing:** Up to 8,192 characters  
- **Batch size:** Maximum 100 texts
- **Caching:** 2GB cache with clustering-based prefetching
- **GPU:** Multi-GPU support with automatic sharding
- **Rate limiting:** 5,000 requests/minute
- **Custom models:** Load proprietary embedding models

```bash
export SUTRA_EDITION=enterprise
export SUTRA_LICENSE_KEY=your-enterprise-key
python -m sutra_embedding_service.main
```

## Error Handling

### Error Response Format

```json
{
  "error": {
    "code": "TEXT_TOO_LONG",
    "message": "Input text exceeds maximum length for current edition",
    "details": {
      "text_length": 5000,
      "max_length": 4096,
      "current_edition": "community"
    }
  },
  "timestamp": "2025-10-27T10:30:00Z",
  "request_id": "embed_123456789"
}
```

### Error Codes

| Code | Description | HTTP Status | Solution |
|------|-------------|-------------|----------|
| `INVALID_TEXT` | Empty or invalid text input | 400 | Provide valid text content |
| `TEXT_TOO_LONG` | Text exceeds edition limit | 400 | Shorten text or upgrade edition |
| `BATCH_TOO_LARGE` | Batch size exceeds limit | 400 | Reduce batch size or upgrade |
| `RATE_LIMIT_EXCEEDED` | Too many requests | 429 | Wait or upgrade edition |
| `MODEL_NOT_LOADED` | Embedding model unavailable | 503 | Check service logs |
| `GPU_OUT_OF_MEMORY` | Insufficient GPU memory | 503 | Reduce batch size |

## Performance Optimization

### Caching Strategy

The service implements intelligent caching based on text similarity:

```python
# Cache key generation includes normalized text
cache_key = f"embed:{hash(normalized_text)}:{model_version}"

# Automatic cache warming for similar texts
if similarity_score > 0.95:
    preload_similar_embeddings()
```

**Cache Performance by Edition:**
- **Simple:** LRU eviction, 100MB limit
- **Community:** LRU+TTL, similarity clustering
- **Enterprise:** Advanced clustering with ML-driven prefetching

### Batch Processing

Optimize performance by batching requests:

```python
# Efficient batching
texts = ["text1", "text2", "text3", ...]
response = await client.post("/batch", json={"texts": texts})

# Avoid individual requests
# for text in texts:  # ❌ Inefficient
#     await client.post("/generate", json={"text": text})
```

**Batch Size Recommendations:**
- **Simple Edition:** 5-10 texts per batch
- **Community Edition:** 20-30 texts per batch  
- **Enterprise Edition:** 50-75 texts per batch

### GPU Utilization

Enable GPU acceleration for faster processing:

```bash
# Check GPU availability
nvidia-smi

# Enable GPU in service
export CUDA_VISIBLE_DEVICES=0
export SUTRA_USE_GPU=true
python -m sutra_embedding_service.main
```

## Integration Examples

### Python Client

```python
import asyncio
import aiohttp
from typing import List

class EmbeddingClient:
    def __init__(self, base_url: str = "http://localhost:8889"):
        self.base_url = base_url
        
    async def generate_embedding(self, text: str) -> List[float]:
        """Generate embedding for single text"""
        async with aiohttp.ClientSession() as session:
            async with session.post(
                f"{self.base_url}/generate",
                json={"text": text, "normalize": True}
            ) as response:
                result = await response.json()
                return result["embedding"]
    
    async def generate_batch(self, texts: List[str]) -> List[List[float]]:
        """Generate embeddings for multiple texts"""
        async with aiohttp.ClientSession() as session:
            async with session.post(
                f"{self.base_url}/batch",
                json={"texts": texts, "normalize": True}
            ) as response:
                result = await response.json()
                return result["embeddings"]

# Usage example
async def main():
    client = EmbeddingClient()
    
    # Single embedding
    embedding = await client.generate_embedding("Hello world")
    print(f"Dimension: {len(embedding)}")
    
    # Batch embeddings
    texts = ["Hello", "World", "AI is amazing"]
    embeddings = await client.generate_batch(texts)
    print(f"Generated {len(embeddings)} embeddings")

asyncio.run(main())
```

### cURL Examples

```bash
# Single embedding
curl -X POST http://localhost:8889/generate \
  -H "Content-Type: application/json" \
  -d '{"text": "Your text here", "normalize": true}'

# Batch embeddings
curl -X POST http://localhost:8889/batch \
  -H "Content-Type: application/json" \
  -d '{
    "texts": ["First text", "Second text", "Third text"],
    "normalize": true
  }'

# Check service info
curl http://localhost:8889/info

# Health check
curl http://localhost:8889/health/detailed
```

### JavaScript/Node.js

```javascript
const axios = require('axios');

class EmbeddingClient {
    constructor(baseUrl = 'http://localhost:8889') {
        this.baseUrl = baseUrl;
    }
    
    async generateEmbedding(text) {
        try {
            const response = await axios.post(`${this.baseUrl}/generate`, {
                text: text,
                normalize: true
            });
            return response.data.embedding;
        } catch (error) {
            console.error('Embedding generation failed:', error.response.data);
            throw error;
        }
    }
    
    async generateBatch(texts) {
        const response = await axios.post(`${this.baseUrl}/batch`, {
            texts: texts,
            normalize: true
        });
        return response.data.embeddings;
    }
}

// Usage
const client = new EmbeddingClient();

client.generateEmbedding("Hello world")
    .then(embedding => console.log(`Generated ${embedding.length}D vector`))
    .catch(error => console.error(error));
```

## Monitoring & Observability

### Key Metrics

Monitor these Prometheus metrics for optimal performance:

```text
# Request metrics
sutra_embedding_requests_total{status="success|error"}
sutra_embedding_request_duration_seconds
sutra_embedding_batch_size_histogram

# Cache metrics  
sutra_embedding_cache_hits_total
sutra_embedding_cache_misses_total
sutra_embedding_cache_evictions_total

# Resource metrics
sutra_embedding_gpu_utilization_percent
sutra_embedding_memory_usage_bytes
sutra_embedding_model_load_time_seconds
```

### Grafana Dashboard

Example dashboard queries:

```promql
# Request rate
rate(sutra_embedding_requests_total[5m])

# Cache hit ratio
rate(sutra_embedding_cache_hits_total[5m]) / 
(rate(sutra_embedding_cache_hits_total[5m]) + rate(sutra_embedding_cache_misses_total[5m]))

# 95th percentile latency
histogram_quantile(0.95, sutra_embedding_request_duration_seconds)

# GPU utilization
sutra_embedding_gpu_utilization_percent
```

### Alerts

```yaml
# High error rate
- alert: EmbeddingServiceHighErrorRate
  expr: rate(sutra_embedding_requests_total{status="error"}[5m]) > 0.05
  for: 2m
  
# GPU memory high
- alert: EmbeddingServiceGPUMemoryHigh  
  expr: sutra_embedding_gpu_memory_usage_percent > 90
  for: 1m
  
# Cache miss rate high
- alert: EmbeddingServiceLowCacheHitRate
  expr: rate(sutra_embedding_cache_hits_total[5m]) / rate(sutra_embedding_requests_total[5m]) < 0.7
  for: 5m
```

## Deployment

### Docker

```dockerfile
# Use official ML Foundation image
FROM sutra/ml-base:2.0.0

# Copy embedding service code
COPY packages/sutra-embedding-service/ /app/
WORKDIR /app

# Install dependencies
RUN pip install -e .

# Expose port
EXPOSE 8889

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s \
  CMD curl -f http://localhost:8889/health || exit 1

# Start service
CMD ["python", "-m", "sutra_embedding_service.main"]
```

### Docker Compose

```yaml
version: '3.8'
services:
  embedding-service:
    image: sutra/embedding-service:2.0.0
    ports:
      - "8889:8889"
    environment:
      - SUTRA_EDITION=${SUTRA_EDITION:-simple}
      - SUTRA_LICENSE_KEY=${SUTRA_LICENSE_KEY}
      - CUDA_VISIBLE_DEVICES=0
    volumes:
      - ./models:/app/models
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8889/health"]
      interval: 30s
      timeout: 10s
      retries: 3
    deploy:
      resources:
        reservations:
          devices:
            - driver: nvidia
              count: 1
              capabilities: [gpu]
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: embedding-service
spec:
  replicas: 3
  selector:
    matchLabels:
      app: embedding-service
  template:
    metadata:
      labels:
        app: embedding-service
    spec:
      containers:
      - name: embedding-service
        image: sutra/embedding-service:2.0.0
        ports:
        - containerPort: 8889
        env:
        - name: SUTRA_EDITION
          value: "enterprise"
        - name: SUTRA_LICENSE_KEY
          valueFrom:
            secretKeyRef:
              name: sutra-license
              key: license-key
        resources:
          requests:
            memory: "2Gi"
            nvidia.com/gpu: 1
          limits:
            memory: "4Gi"
            nvidia.com/gpu: 1
        livenessProbe:
          httpGet:
            path: /health
            port: 8889
          initialDelaySeconds: 60
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /health/detailed
            port: 8889
          initialDelaySeconds: 30
          periodSeconds: 10
---
apiVersion: v1
kind: Service
metadata:
  name: embedding-service
spec:
  selector:
    app: embedding-service
  ports:
  - protocol: TCP
    port: 8889
    targetPort: 8889
  type: ClusterIP
```

## Troubleshooting

### Common Issues

#### Model Loading Errors
```bash
# Check model files
ls -la models/nomic-embed-text-v1.5/

# Check GPU availability
nvidia-smi
python -c "import torch; print(torch.cuda.is_available())"

# Check logs
docker logs sutra-embedding-service
```

#### Performance Issues
```bash
# Monitor resource usage
docker stats sutra-embedding-service

# Check cache hit rates
curl http://localhost:8889/metrics | grep cache

# Profile batch processing
curl -X POST http://localhost:8889/batch \
  -H "Content-Type: application/json" \
  -d '{"texts": ["'$(head -c 1000 /dev/urandom | base64)'"]}' \
  -w "Time: %{time_total}s\n"
```

#### Rate Limiting
```bash
# Check current limits
curl http://localhost:8889/info | jq '.limits'

# Monitor rate limit metrics
curl http://localhost:8889/metrics | grep rate_limit

# Upgrade edition if needed
export SUTRA_EDITION=enterprise
```

### Debug Mode

Enable detailed logging for troubleshooting:

```bash
export SUTRA_LOG_LEVEL=DEBUG
export SUTRA_LOG_FORMAT=json
python -m sutra_embedding_service.main
```

### Support

For additional support:
- **Documentation:** [docs/embedding/](../embedding/)
- **GitHub Issues:** [Embedding Service Issues](https://github.com/your-repo/issues)
- **Community:** [Sutra AI Discord](https://discord.gg/sutra-ai)

---

## Changelog

### 2.0.0 (2025-10-27)
- ✅ Complete rewrite using ML Foundation
- ✅ Edition-aware batch processing and rate limiting  
- ✅ Advanced caching with similarity clustering
- ✅ GPU acceleration support
- ✅ Production-ready monitoring and health checks
- ✅ Comprehensive API documentation
# ML-Base Service API Design

## Overview
The ML-Base Service provides centralized ML inference capabilities for all Sutra services, enabling horizontal scaling and resource efficiency.

## Core Endpoints

### 1. Model Management

#### GET /models
List available models
```json
{
  "models": [
    {
      "id": "embedding-nomic-v1.5",
      "type": "embedding", 
      "name": "nomic-ai/nomic-embed-text-v1.5",
      "status": "loaded",
      "memory_mb": 1200,
      "dimension": 768,
      "max_batch_size": 32,
      "instances": 2
    },
    {
      "id": "nlg-dialogpt-large",
      "type": "nlg",
      "name": "microsoft/DialoGPT-large", 
      "status": "loading",
      "memory_mb": 800,
      "max_tokens": 512,
      "instances": 1
    }
  ]
}
```

#### POST /models/{model_id}/load
Load a specific model
```json
{
  "instances": 2,
  "device": "cpu",
  "memory_limit_gb": 4
}
```

#### DELETE /models/{model_id}
Unload a model

### 2. Embedding Inference

#### POST /embed
Generate embeddings
```json
{
  "model_id": "embedding-nomic-v1.5",
  "texts": ["text1", "text2"],
  "normalize": true,
  "batch_size": 16
}
```

Response:
```json
{
  "embeddings": [[0.1, 0.2, ...], [0.3, 0.4, ...]],
  "dimension": 768,
  "processing_time_ms": 45,
  "model_used": "embedding-nomic-v1.5",
  "cache_hit": false
}
```

### 3. NLG Inference

#### POST /generate
Generate text
```json
{
  "model_id": "nlg-dialogpt-large",
  "prompt": "FACTS: Paris is in France\nQUESTION: Where is Paris?",
  "max_tokens": 150,
  "temperature": 0.3,
  "stop_sequences": ["\n", "QUESTION:"]
}
```

Response:
```json
{
  "text": "Paris is located in France.",
  "tokens_generated": 6,
  "processing_time_ms": 234,
  "model_used": "nlg-dialogpt-large",
  "grounding_applied": true
}
```

### 4. Batch Processing

#### POST /batch/embed
Process multiple embedding requests
```json
{
  "requests": [
    {"id": "req1", "texts": ["text1"], "model_id": "embedding-nomic-v1.5"},
    {"id": "req2", "texts": ["text2"], "model_id": "embedding-nomic-v1.5"}
  ]
}
```

#### POST /batch/generate
Process multiple generation requests

### 5. Streaming (Enterprise)

#### POST /stream/generate
Server-sent events for real-time generation
```
Content-Type: text/event-stream

data: {"token": "Paris", "position": 0}
data: {"token": " is", "position": 1}
data: {"done": true, "total_tokens": 6}
```

### 6. Health & Metrics

#### GET /health
Service health status
```json
{
  "status": "healthy",
  "models_loaded": 2,
  "total_memory_gb": 2.1,
  "average_latency_ms": 67,
  "requests_per_second": 45
}
```

#### GET /metrics
Detailed metrics
```json
{
  "uptime_seconds": 3600,
  "total_requests": 15432,
  "requests_by_type": {
    "embedding": 12000,
    "generation": 3432
  },
  "model_utilization": {
    "embedding-nomic-v1.5": 0.75,
    "nlg-dialogpt-large": 0.23
  },
  "cache_stats": {
    "hit_rate": 0.34,
    "size_mb": 256
  }
}
```

## Service Configuration

### Edition-Based Features

#### Simple Edition
- 1 embedding model, 1 NLG model
- CPU-only inference
- Basic caching (in-memory)
- Max 8 batch size

#### Community Edition  
- 2 embedding models, 2 NLG models
- CPU + GPU support
- Advanced caching (persistent)
- Max 32 batch size
- Load balancing

#### Enterprise Edition
- Custom models supported
- Multi-GPU scaling
- Advanced batch processing
- Streaming inference
- Model versioning
- A/B testing support

### Horizontal Scaling

#### Auto-scaling Triggers
- CPU usage > 80% for 2 minutes
- Average latency > 500ms
- Queue depth > 100 requests
- Memory usage > 85%

#### Scaling Strategies
1. **Model-level scaling**: Add instances of specific models
2. **Service-level scaling**: Add entire ML-Base replicas
3. **Resource-aware**: Scale based on GPU/CPU availability

## Client Libraries

### Python Client
```python
from sutra_ml_client import MLBaseClient

client = MLBaseClient("http://ml-base:8887")

# Embeddings
embeddings = await client.embed(
    texts=["hello world"], 
    model_id="embedding-nomic-v1.5"
)

# Generation
response = await client.generate(
    prompt="FACTS: ...\nQUESTION: ...", 
    model_id="nlg-dialogpt-large",
    max_tokens=150
)
```

### REST Client (for other languages)
Standard HTTP client with async support

## Security & Authentication

### API Keys (Production)
```
Authorization: Bearer sutra_ml_key_enterprise_abc123
```

### Rate Limiting
- Simple: 100 requests/minute
- Community: 1000 requests/minute  
- Enterprise: Unlimited

### Model Access Control
- Simple: Pre-defined models only
- Community: Select custom models
- Enterprise: Full custom model support

## Monitoring & Observability

### Metrics Collection
- Request latency histograms
- Model utilization rates
- Memory/GPU usage
- Error rates by model
- Cache hit rates

### Alerting
- Model load failures
- High latency (>1s)
- Memory exhaustion
- GPU errors

### Logging
- Structured JSON logs
- Request/response tracing
- Model performance metrics
- Error stack traces

## Migration Strategy

### Phase 1: Build ML-Base Service
1. Implement core inference APIs
2. Add model management
3. Basic health/metrics

### Phase 2: Refactor Clients
1. Create lightweight embedding client
2. Create lightweight NLG client
3. Update sutra-hybrid integration

### Phase 3: Advanced Features
1. Batch processing
2. Streaming (enterprise)
3. Auto-scaling
4. Advanced caching

### Phase 4: Optimization
1. GPU optimization
2. Model quantization
3. Inference acceleration
4. Custom model support

This design enables true horizontal scaling while maintaining API compatibility and adding powerful new capabilities for enterprise deployments.
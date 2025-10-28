# Embedding Service Architecture

**Official Embedding Provider for Sutra AI v2.0+**

Last Updated: October 28, 2025

---

## Executive Summary

Sutra AI uses a **single, dedicated embedding service** for all semantic operations:

- **Service**: `sutra-embedding-service`
- **Model**: `nomic-embed-text-v1.5` (Nomic AI)
- **Dimensions**: 768 (fixed)
- **Purpose**: Generate semantic embeddings for concept similarity matching
- **Architecture**: HA-ready with load balancing

**CRITICAL**: All Ollama and granite-embedding references in legacy documentation are **DEPRECATED** and should be ignored.

---

## Service Configuration

### Environment Variables

```bash
# Storage Server & Learning Pipeline
SUTRA_EMBEDDING_SERVICE_URL=http://sutra-embedding-service:8888  # REQUIRED
SUTRA_EMBEDDING_TIMEOUT_SEC=30  # Optional, default: 30
VECTOR_DIMENSION=768  # REQUIRED - must match model output

# Hybrid Service (same config)
SUTRA_EMBEDDING_SERVICE_URL=http://sutra-embedding-service:8888
SUTRA_USE_SEMANTIC_EMBEDDINGS=true
```

### Docker Compose Configuration

```yaml
services:
  sutra-embedding-service:
    image: sutra-embedding-service:latest
    ports:
      - "8888:8888"
    environment:
      - MODEL_NAME=nomic-ai/nomic-embed-text-v1.5
      - VECTOR_DIMENSION=768
      - BATCH_SIZE=32
      - CACHE_SIZE_MB=500
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8888/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 60s

  storage-server:
    image: sutra-storage-server:latest
    environment:
      - SUTRA_EMBEDDING_SERVICE_URL=http://sutra-embedding-service:8888
      - VECTOR_DIMENSION=768
    depends_on:
      sutra-embedding-service:
        condition: service_healthy
```

---

## Architecture Overview

### Unified Learning Flow

```
User Content (Text)
    ↓
TCP Storage Client
    ├─ learn_concept(content, options)
    └─ TCP Message → Storage Server
                         ↓
        ┌───────────────────────────────┐
        │ Learning Pipeline             │
        │ (storage-server internal)     │
        ├───────────────────────────────┤
        │ Step 1: Generate Embedding    │
        │   ↓ HTTP POST                 │
        │   → sutra-embedding-service   │
        │      (nomic-embed-text-v1.5)  │
        │   ← 768-d vector              │
        │                               │
        │ Step 2: Extract Associations  │
        │   (Rust-based NLP patterns)   │
        │                               │
        │ Step 3: Atomic Storage        │
        │   • Concept + embedding       │
        │   • HNSW vector index         │
        │   • Write-Ahead Log (WAL)     │
        │   • Association graph edges   │
        └───────────────────────────────┘
                         ↓
            Return concept_id to client
```

### Key Design Principles

1. **Single Source of Truth**: Storage server owns embedding generation
2. **Service Isolation**: Embedding service is stateless and horizontally scalable
3. **Caching**: 500MB LRU cache in embedding service (50% hit rate typical)
4. **Batch Optimization**: Automatic batching for bulk learning operations
5. **Retry Logic**: 3 retries with exponential backoff on failure

---

## Embedding Client (Rust)

**Location**: `packages/sutra-storage/src/embedding_client.rs`

### Features

- HTTP client with connection pooling
- Automatic retry with exponential backoff
- Batch embedding generation
- Health check monitoring
- Comprehensive error handling

### API

```rust
use sutra_storage::embedding_client::EmbeddingClient;

// Create client with defaults (reads SUTRA_EMBEDDING_SERVICE_URL)
let client = EmbeddingClient::with_defaults()?;

// Generate single embedding
let embedding: Vec<f32> = client.generate("text", true).await?;
assert_eq!(embedding.len(), 768);

// Generate batch
let texts = vec!["text1".to_string(), "text2".to_string()];
let embeddings: Vec<Option<Vec<f32>>> = client.generate_batch(&texts, true).await;

// Health check
if client.health_check().await? {
    println!("Embedding service is healthy");
}
```

### Configuration

```rust
use sutra_storage::embedding_client::EmbeddingConfig;

let config = EmbeddingConfig {
    service_url: "http://sutra-embedding-service:8888".to_string(),
    timeout_secs: 30,
    max_retries: 3,
    retry_delay_ms: 500,
};

let client = EmbeddingClient::new(config)?;
```

---

## Embedding Service Specifications

### Model: nomic-embed-text-v1.5

**Publisher**: Nomic AI  
**Architecture**: BERT-based encoder  
**Context Length**: 8192 tokens  
**Output**: 768-dimensional dense vectors  
**Normalization**: L2-normalized by default  

**Performance**:
- Single embedding: ~10-30ms
- Batch of 32: ~200-500ms
- Cache hit: <1ms

### API Endpoints

#### POST /embed

Generate embeddings for one or more texts.

**Request**:
```json
{
  "texts": ["example text", "another example"],
  "normalize": true
}
```

**Response**:
```json
{
  "embeddings": [
    [0.123, -0.456, ...],  // 768 dimensions
    [0.789, -0.012, ...]
  ],
  "dimension": 768,
  "model": "nomic-ai/nomic-embed-text-v1.5",
  "processing_time_ms": 45.2,
  "cached_count": 1
}
```

#### GET /health

Check service health and model status.

**Response**:
```json
{
  "status": "healthy",
  "model": "nomic-ai/nomic-embed-text-v1.5",
  "model_loaded": true,
  "dimension": 768,
  "uptime_seconds": 3600
}
```

#### GET /info

Get service information.

**Response**:
```json
{
  "model": "nomic-ai/nomic-embed-text-v1.5",
  "dimension": 768,
  "context_length": 8192,
  "cache_size_mb": 500,
  "cache_hit_rate": 0.52,
  "total_requests": 10523
}
```

---

## High Availability Setup

### Architecture

```
┌───────────────────────────────────────────┐
│           HAProxy Load Balancer           │
│         (embedding-ha:8888)               │
│                                           │
│  Algorithm: least-connection              │
│  Health Check: Every 2s                   │
│  Failover: <3s detection                  │
└──────────┬────────┬────────┬──────────────┘
           │        │        │
    ┌──────┴──┐  ┌─┴──────┐ │
    │         │  │        │ │
    v         v  v        v v
┌─────────┐ ┌─────────┐ ┌─────────┐
│ embed-1 │ │ embed-2 │ │ embed-3 │
│  :8889  │ │  :8890  │ │  :8891  │
└─────────┘ └─────────┘ └─────────┘
```

### Docker Compose (HA)

```yaml
services:
  embedding-ha:
    image: haproxy:2.8-alpine
    ports:
      - "8888:8888"
      - "8404:8404"  # Stats dashboard
    volumes:
      - ./haproxy.cfg:/usr/local/etc/haproxy/haproxy.cfg:ro
    depends_on:
      - embedding-1
      - embedding-2
      - embedding-3

  embedding-1:
    image: sutra-embedding-service:latest
    environment:
      - PORT=8889
      - MODEL_NAME=nomic-ai/nomic-embed-text-v1.5
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8889/health"]

  embedding-2:
    image: sutra-embedding-service:latest
    environment:
      - PORT=8890
      - MODEL_NAME=nomic-ai/nomic-embed-text-v1.5
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8890/health"]

  embedding-3:
    image: sutra-embedding-service:latest
    environment:
      - PORT=8891
      - MODEL_NAME=nomic-ai/nomic-embed-text-v1.5
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8891/health"]
```

---

## Verification & Testing

### Smoke Test

```bash
#!/bin/bash
# Test embedding service end-to-end

echo "Testing embedding service..."

# Health check
curl -s http://localhost:8888/health | jq '.status, .model_loaded'

# Generate test embedding
curl -s -X POST http://localhost:8888/embed \
  -H "Content-Type: application/json" \
  -d '{"texts": ["test"], "normalize": true}' | \
  jq '.embeddings[0] | length, .[0:5]'

# Expected output:
# 768
# [0.123, -0.456, 0.789, -0.012, 0.345]
```

### Integration Test

```bash
# Storage server integration test
cd /Users/nisheethranjan/Projects/sutra-models

# Start embedding service
docker-compose up -d sutra-embedding-service

# Wait for health
sleep 10

# Run storage tests that require embeddings
cd packages/sutra-storage
cargo test --features integration-tests test_learning_pipeline

# Verify vector dimension
cargo test test_vector_dimension_consistency
```

---

## Migration Notes

### Deprecated Providers

**🚫 No Longer Supported**:
- Ollama integration (`SUTRA_OLLAMA_URL`)
- granite-embedding model (384-d)
- sentence-transformers fallback
- spaCy embeddings
- TF-IDF fallback

### Why These Were Removed

1. **Inconsistent Dimensions**: granite-embedding produced 384-d vectors, nomic produces 768-d
2. **Wrong Query Results**: Mixing dimensions caused incorrect similarity matching
3. **Complexity**: Multiple providers increased testing surface and maintenance burden
4. **Production Incident**: Real bug where 384-d query vectors + 768-d storage caused wrong answers

### Migration Path (If You Have Old Data)

If you have concepts stored with 384-d embeddings:

1. **Clean rebuild** (recommended):
   ```bash
   docker-compose down
   docker volume rm sutra-models_storage-data
   docker-compose up -d
   # Re-learn all concepts
   ```

2. **Selective migration** (advanced):
   - Export concepts without embeddings
   - Delete old storage
   - Re-import and let system generate new 768-d embeddings

---

## Troubleshooting

### "No embedding processor available"

**Cause**: Storage server can't reach embedding service.

**Fix**:
```bash
# Check service is running
docker ps | grep embedding

# Check health
curl http://localhost:8888/health

# Check storage server can reach it
docker exec sutra-storage curl http://sutra-embedding-service:8888/health

# Verify environment variable
docker exec sutra-storage env | grep SUTRA_EMBEDDING_SERVICE_URL
```

### "Dimension mismatch: expected 768, got 384"

**Cause**: Old data with 384-d embeddings (granite-embedding).

**Fix**: Clean rebuild (see Migration Path above).

### "Embedding service unavailable"

**Cause**: Service is down or overloaded.

**Fix**:
```bash
# Check service logs
docker logs sutra-embedding-service --tail 50

# Restart service
docker-compose restart sutra-embedding-service

# Check resource usage
docker stats sutra-embedding-service
```

### Slow Embedding Generation (>1s)

**Possible causes**:
1. Cold start (first request loads model ~30s)
2. Large batch size (>100 texts)
3. CPU-only mode (GPU recommended for production)
4. Memory pressure

**Fix**:
```bash
# Check if GPU is available
docker exec sutra-embedding-service nvidia-smi

# Increase memory limit
docker-compose up -d --scale sutra-embedding-service=3

# Enable GPU
# Add to docker-compose.yml:
#   deploy:
#     resources:
#       reservations:
#         devices:
#           - driver: nvidia
#             count: 1
#             capabilities: [gpu]
```

---

## Performance Optimization

### Caching Strategy

The embedding service uses LRU cache with the following defaults:

- **Cache size**: 500MB (configurable via `CACHE_SIZE_MB`)
- **Eviction**: Least recently used
- **Hit rate**: 40-60% typical for production workloads
- **Persistence**: Memory-only (cleared on restart)

### Batch Optimization

For bulk learning operations:

```python
# Good: Single batch request
concepts = ["text1", "text2", "text3", ...]  # 1000 concepts
result = storage.learn_batch(concepts)

# Bad: Individual requests
for concept in concepts:
    storage.learn_concept(concept)  # 1000 separate HTTP calls
```

### Resource Allocation

**Simple Edition** (1 replica):
- CPU: 2 cores
- Memory: 4GB
- Storage: 2GB (model + cache)

**Community Edition** (1 replica):
- CPU: 4 cores
- Memory: 8GB
- Storage: 4GB

**Enterprise Edition** (3 replicas + HA):
- CPU: 4 cores per replica
- Memory: 8GB per replica
- Storage: 4GB per replica
- Load balancer: HAProxy

---

## Security Considerations

### Network Isolation

In production deployments:

```yaml
networks:
  internal:
    internal: true  # No external access

services:
  sutra-embedding-service:
    networks:
      - internal  # Only accessible to other services
    # No exposed ports to host
```

### Authentication

The embedding service itself has no authentication (by design):
- Should only be accessible on internal network
- Storage server validates all requests before calling embedding service
- Use SUTRA_SECURE_MODE=true for storage server auth

### Rate Limiting

Implement rate limiting at API gateway level:

```yaml
# API Gateway (nginx/traefik)
rate_limit:
  embedding_service:
    requests_per_minute: 1000
    burst: 100
```

---

## Monitoring & Observability

### Metrics

Embedding service exposes Prometheus metrics at `/metrics`:

```
# Total requests
embedding_requests_total{status="success|failure"}

# Request duration
embedding_request_duration_seconds{quantile="0.5|0.95|0.99"}

# Cache performance
embedding_cache_hits_total
embedding_cache_misses_total
embedding_cache_hit_rate

# Model info
embedding_model_info{model="nomic-ai/nomic-embed-text-v1.5", dimension="768"}
```

### Logging

**Health check logs**:
```json
{
  "level": "INFO",
  "timestamp": "2025-10-28T10:30:00Z",
  "message": "Health check passed",
  "model_loaded": true,
  "uptime_seconds": 3600
}
```

**Error logs**:
```json
{
  "level": "ERROR",
  "timestamp": "2025-10-28T10:31:00Z",
  "message": "Model inference failed",
  "text_length": 10000,
  "error": "OutOfMemoryError"
}
```

---

## Reference Implementation

Complete example: `packages/sutra-storage/src/embedding_client.rs`

**Key files**:
- `embedding_client.rs` - HTTP client implementation
- `learning_pipeline.rs` - Integration with storage
- `hnsw_container.rs` - Vector index (768-d)
- `semantic_extractor.rs` - Association extraction

---

## FAQ

**Q: Can I use a different embedding model?**  
A: Not recommended. The system is optimized for 768-d vectors. Changing models requires:
- Rebuilding all storage data
- Updating VECTOR_DIMENSION everywhere
- Retraining association extraction patterns

**Q: Why nomic-embed-text-v1.5?**  
A: Balance of quality, speed, and open-source licensing. Benchmarks show comparable performance to OpenAI ada-002 at 10× lower cost.

**Q: Can I run without embeddings?**  
A: Yes! Set `generate_embedding=false` in LearnOptions. Graph reasoning will work, but semantic similarity matching will be disabled.

**Q: How do I upgrade the model?**  
A: Deploy new embedding service with updated model, then rebuild storage. Not recommended in-place.

**Q: What about multilingual support?**  
A: nomic-embed-text-v1.5 supports 100+ languages. For domain-specific multilingual, consider fine-tuning.

---

## Changelog

**v2.0.0 (October 2025)**:
- Standardized on sutra-embedding-service
- Removed Ollama integration
- Fixed 384-d → 768-d dimension mismatch
- Added HA setup with HAProxy
- Comprehensive documentation

**v1.x (Deprecated)**:
- Multiple embedding providers
- Fallback logic (caused bugs)
- Inconsistent dimensions

---

**For More Information**:
- **Embedding Service**: `packages/sutra-embedding-service/README.md`
- **Learning Pipeline**: `packages/sutra-storage/src/learning_pipeline.rs`
- **Production Guide**: `docs/PRODUCTION_GUIDE.md`
- **Troubleshooting**: `docs/EMBEDDING_TROUBLESHOOTING.md`

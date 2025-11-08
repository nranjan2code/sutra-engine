# Embedding Service Scaling Strategy

## 🎯 Sutra Philosophy

**All caching uses Sutra Storage** - This document provides the comprehensive strategy using our own storage engine for caching, not Redis or other external dependencies. See [EMBEDDING_SCALING_SUTRA_NATIVE.md](./EMBEDDING_SCALING_SUTRA_NATIVE.md) for the recommended Sutra-native implementation.

---

## Executive Summary

**Current Bottleneck**: Embedding service is the performance limiter (60s timeouts, 0.14 concepts/sec)  
**Target Scale**: 1,000+ concurrent users with high throughput  
**Strategy**: Multi-tier optimization without breaking existing architecture  
**Monitoring**: Grid events (not Prometheus) - see [DevOps Self-Monitoring](../sutra-platform-review/DEVOPS_SELF_MONITORING.md)

---

## Current Architecture Analysis

### 🔴 Bottleneck Identified

From financial intelligence case study (November 2025):
```
Performance Results:
- Processing Speed: 0.14 concepts/sec
- Timeout Configuration: 60 seconds (increased from 30s)
- Concurrency: Limited to 2 workers (optimal for stability)
- Success Rate: 100% (but slow)

Root Cause: Single ML-Base service processing all embedding requests sequentially
```

### Current Architecture

```
Client → API (8000) → Storage Server (50051) → ML-Base Service (8887) → GPU/CPU
         ↓                                            ↓
    Nginx Proxy                             Embedding-Single (8888)
                                                   (Lightweight Proxy)
```

**Layers:**
1. **Nginx**: Load balancer and reverse proxy
2. **Sutra API**: REST endpoints
3. **Storage Server**: Graph storage + unified learning pipeline
4. **Embedding-Single**: Lightweight client (256MB RAM)
5. **ML-Base Service**: Heavy model hosting (6GB RAM limit)

---

## Scaling Strategy: 5-Tier Optimization

### Tier 1: Intelligent Caching (Quick Win - 70% reduction)

**Impact**: Reduces 70% of embedding calls for repeated content  
**Implementation Time**: 2-3 days  
**Architecture Change**: Minimal

#### Current State
```python
# .sutra/compose/production.yml
embedding-single:
  environment:
    - EMBEDDING_CACHE_SIZE=1000        # Too small
    - EMBEDDING_CACHE_TTL=3600         # 1 hour
```

#### Optimization
```python
# Multi-tier caching strategy

# 1. In-Memory Cache (Embedding Service Layer)
EMBEDDING_CACHE_SIZE=50000           # 50K entries (~400MB)
EMBEDDING_CACHE_TTL=86400            # 24 hours
CACHE_STRATEGY=lru                   # LRU eviction

# 2. Sutra Storage Cache Shard (Shared Across Replicas)
SUTRA_CACHE_ENABLED=true
SUTRA_CACHE_HOST=storage-cache-shard  # Dedicated Sutra Storage shard
SUTRA_CACHE_PORT=50052
SUTRA_CACHE_TTL=604800               # 7 days (via concept metadata)

# 3. Storage-Layer Cache (Persistent)
VECTOR_INDEX_CACHE=true              # USearch HNSW cache
MMAP_VECTORS=true                    # Memory-mapped vectors
```

**Cache Hit Rate Targets:**
- Financial Data: 60-70% (repeated company queries)
- User Queries: 40-50% (similar questions)
- System Concepts: 80-90% (internal metadata)

**Implementation:**
```python
# packages/sutra-embedding-service/cache_manager.py

from sutra_storage_client_tcp import StorageClient, ConceptMetadata
import hashlib
from typing import Optional, List
from datetime import datetime, timedelta

class MultiTierCache:
    def __init__(self):
        # L1: In-memory LRU cache
        self.l1_cache = LRUCache(max_size=50000)
        
        # L2: Sutra Storage cache shard (dedicated for caching)
        self.sutra_cache = StorageClient(
            host='storage-cache-shard',
            port=6379,
            decode_responses=False
        )
        
        # L3: Persistent vector index (handled by storage)
    
    def get(self, text: str, model: str) -> Optional[List[float]]:
        cache_key = self._cache_key(text, model)
        
        # L1 check (fastest)
        result = self.l1_cache.get(cache_key)
        if result:
            return result
        
        # L2 check (Sutra Storage cache shard)
        concept = self.sutra_cache.get_concept_by_id(cache_key)
        if concept and concept.embedding:
            embedding = concept.embedding
            self.l1_cache.set(cache_key, embedding)  # Promote to L1
            return embedding
        
        return None  # Cache miss
    
    def set(self, text: str, model: str, embedding: List[float], ttl: int = 86400):
        cache_key = self._cache_key(text, model)
        
        # Store in both layers
        self.l1_cache.set(cache_key, embedding)
        
        # Store in Sutra cache shard with TTL metadata
        metadata = ConceptMetadata(
            concept_type="embedding_cache",
            expires_at=(datetime.now() + timedelta(seconds=ttl)).isoformat()
        )
        self.sutra_cache.store_concept(
            concept_id=cache_key,
            content=text[:200],
            embedding=embedding,
            metadata=metadata
        )
    
    def _cache_key(self, text: str, model: str) -> str:
        return f"emb:{model}:{hashlib.sha256(text.encode()).hexdigest()[:16]}"
```

---

### Tier 2: Batch Processing Optimization (30% improvement)

**Impact**: 30% throughput increase via efficient batching  
**Implementation Time**: 3-4 days  
**Architecture Change**: Internal only

#### Current Limitation
```python
# Edition-based batch limits (too conservative)
EDITION_LIMITS = {
    "simple": {"max_batch_size": 8},
    "community": {"max_batch_size": 32},
    "enterprise": {"max_batch_size": 128}
}
```

#### Optimization: Dynamic Batching
```python
class DynamicBatcher:
    """
    Accumulates requests and processes them in optimal batches
    """
    def __init__(self, max_batch_size: int = 128, max_wait_ms: int = 50):
        self.max_batch_size = max_batch_size
        self.max_wait_ms = max_wait_ms
        self.pending_requests = []
        self.batch_lock = asyncio.Lock()
    
    async def process_with_batching(self, text: str) -> List[float]:
        # Add request to batch
        future = asyncio.Future()
        async with self.batch_lock:
            self.pending_requests.append((text, future))
            
            # Trigger batch if full or timeout
            should_process = (
                len(self.pending_requests) >= self.max_batch_size or
                self._batch_age_ms() >= self.max_wait_ms
            )
        
        if should_process:
            await self._process_batch()
        
        return await future
    
    async def _process_batch(self):
        async with self.batch_lock:
            if not self.pending_requests:
                return
            
            # Extract batch
            batch = self.pending_requests[:self.max_batch_size]
            self.pending_requests = self.pending_requests[self.max_batch_size:]
            
            texts = [req[0] for req in batch]
            futures = [req[1] for req in batch]
        
        # Process batch (single GPU call)
        embeddings = await self.ml_base_client.embed_batch(texts)
        
        # Resolve futures
        for future, embedding in zip(futures, embeddings):
            future.set_result(embedding)
```

**Performance Improvement:**
- Sequential: 8 requests × 200ms = 1600ms total
- Batched: 1 batch × 250ms = 250ms total (6.4x faster)

---

### Tier 3: Horizontal Scaling (5-10x capacity)

**Impact**: 5-10x throughput via ML-Base replicas  
**Implementation Time**: 1 week  
**Architecture Change**: Moderate (add replicas + LB)

#### Current: Single ML-Base Service
```yaml
# .sutra/compose/production.yml
ml-base-service:
  container_name: sutra-works-ml-base
  deploy:
    resources:
      limits:
        memory: 6G
```

#### Optimized: ML-Base Cluster
```yaml
# .sutra/compose/production.yml

# ML-Base Load Balancer (HAProxy)
ml-base-lb:
  image: haproxy:2.9-alpine
  container_name: sutra-works-ml-base-lb
  ports:
    - "8887:8887"
  volumes:
    - ./haproxy/ml-base-lb.cfg:/usr/local/etc/haproxy/haproxy.cfg:ro
  networks:
    - sutra-network
  depends_on:
    - ml-base-1
    - ml-base-2
    - ml-base-3

# ML-Base Replicas (CPU-optimized)
ml-base-1:
  build:
    context: ../../packages/sutra-ml-base-service
  image: sutra-works-ml-base-service:${SUTRA_VERSION}
  container_name: sutra-works-ml-base-1
  environment:
    - INSTANCE_ID=ml-base-1
    - ML_BASE_PORT=8887
    - ML_BASE_MAX_BATCH_SIZE=64
    - WORKER_THREADS=4
  deploy:
    resources:
      limits:
        memory: 6G
      reservations:
        memory: 4G
  networks:
    - sutra-network

ml-base-2:
  extends: ml-base-1
  container_name: sutra-works-ml-base-2
  environment:
    - INSTANCE_ID=ml-base-2

ml-base-3:
  extends: ml-base-1
  container_name: sutra-works-ml-base-3
  environment:
    - INSTANCE_ID=ml-base-3
```

**HAProxy Configuration:**
```
# haproxy/ml-base-lb.cfg
global
    maxconn 4096

defaults
    mode http
    timeout connect 5s
    timeout client 60s
    timeout server 60s

frontend ml_base_frontend
    bind *:8887
    default_backend ml_base_backend

backend ml_base_backend
    balance leastconn              # Route to least busy instance
    option httpchk GET /health     # Health check endpoint
    
    server ml-base-1 ml-base-1:8887 check inter 10s fall 3 rise 2
    server ml-base-2 ml-base-2:8887 check inter 10s fall 3 rise 2
    server ml-base-3 ml-base-3:8887 check inter 10s fall 3 rise 2
```

**Capacity Calculation:**
- Single instance: ~0.14 concepts/sec
- 3 replicas: ~0.42 concepts/sec (3x)
- With caching (70% hit rate): ~1.4 concepts/sec (10x effective)

---

### Tier 4: GPU Acceleration (10-50x for inference)

**Impact**: 10-50x faster inference per request  
**Implementation Time**: 2-3 weeks (hardware + integration)  
**Architecture Change**: Infrastructure upgrade

#### GPU-Enabled ML-Base Service

```yaml
# .sutra/compose/production-gpu.yml

ml-base-gpu:
  build:
    context: ../../packages/sutra-ml-base-service
    dockerfile: Dockerfile.gpu
  image: sutra-works-ml-base-service-gpu:${SUTRA_VERSION}
  runtime: nvidia                    # Requires NVIDIA Docker runtime
  environment:
    - CUDA_VISIBLE_DEVICES=0         # GPU device ID
    - ML_BASE_DEVICE=cuda
    - ML_BASE_DTYPE=float16          # Half precision for speed
    - ML_BASE_BATCH_SIZE=256         # Larger batches on GPU
    - TENSOR_PARALLEL=1              # Single GPU for now
  deploy:
    resources:
      reservations:
        devices:
          - driver: nvidia
            count: 1
            capabilities: [gpu]
```

**Performance Comparison:**
```
Single Request Latency:
- CPU (Intel Xeon): ~2000ms
- GPU (NVIDIA T4): ~50ms (40x faster)
- GPU (NVIDIA A100): ~15ms (133x faster)

Batch Processing (128 texts):
- CPU: ~8000ms
- GPU (T4): ~200ms (40x faster)
- GPU (A100): ~80ms (100x faster)
```

**Cloud Provider Options:**
```bash
# AWS EC2 GPU Instances
g4dn.xlarge    # 1x T4 GPU, $0.526/hour  → Cost-effective
g5.xlarge      # 1x A10G GPU, $1.006/hour → Balanced
p3.2xlarge     # 1x V100 GPU, $3.06/hour  → High-performance

# Google Cloud Platform
n1-standard-4 + T4  # $0.35/hour GPU + $0.19/hour compute
n1-standard-4 + A100 # $2.93/hour GPU + $0.19/hour compute

# Monthly Cost Estimate (24/7 operation)
T4:   $379/month   (cost-effective for 1K users)
A10G: $725/month   (balanced performance)
A100: $2,120/month (enterprise scale)
```

---

### Tier 5: Model Optimization (2-5x efficiency)

**Impact**: 2-5x faster inference via model compression  
**Implementation Time**: 3-4 weeks (experimentation required)  
**Architecture Change**: Model selection only

#### Optimization Techniques

**1. Model Quantization (INT8)**
```python
# Reduce model size by 4x, speed up by 2-3x
from transformers import AutoModel
import torch

model = AutoModel.from_pretrained("nomic-ai/nomic-embed-text-v1.5")

# Dynamic quantization (CPU)
quantized_model = torch.quantization.quantize_dynamic(
    model,
    {torch.nn.Linear},
    dtype=torch.qint8
)

# Static quantization (GPU - more complex)
# Requires calibration dataset
```

**Performance:**
- FP32 (current): 768MB model, 2000ms/request
- INT8 quantized: 192MB model, 800ms/request (2.5x faster)
- FP16 (GPU only): 384MB model, 100ms/request on GPU

**2. Distillation (Smaller Models)**
```python
# Use distilled versions of embedding models
MODELS = {
    "large": "nomic-ai/nomic-embed-text-v1.5",        # 768 dim, 137M params
    "medium": "sentence-transformers/all-MiniLM-L6-v2", # 384 dim, 22M params (6x faster)
    "small": "sentence-transformers/all-MiniLM-L3-v2",  # 384 dim, 11M params (12x faster)
}

# Quality vs. Performance trade-off
# - Large: 100% accuracy, 2000ms
# - Medium: 95% accuracy, 300ms (good for most use cases)
# - Small: 90% accuracy, 150ms (for high-volume, low-criticality)
```

**3. ONNX Runtime Optimization**
```python
# Convert PyTorch model to ONNX for optimized inference
import onnx
from optimum.onnxruntime import ORTModelForFeatureExtraction

# Export to ONNX
model = ORTModelForFeatureExtraction.from_pretrained(
    "nomic-ai/nomic-embed-text-v1.5",
    export=True,
    provider="CUDAExecutionProvider"  # or CPUExecutionProvider
)

# Typically 1.5-3x faster than PyTorch
```

---

## Complete Scaling Architecture

### Production-Ready Architecture for 1,000+ Users

```
                               ┌─────────────────────┐
                               │   Nginx Proxy       │
                               │   (Load Balancer)   │
                               └──────────┬──────────┘
                                          │
                    ┌─────────────────────┼─────────────────────┐
                    │                     │                     │
              ┌─────▼─────┐         ┌────▼────┐          ┌────▼────┐
              │ Sutra API │         │ Sutra   │          │ Sutra   │
              │ Replica 1 │         │ API-2   │          │ API-3   │
              └─────┬─────┘         └────┬────┘          └────┬────┘
                    │                    │                     │
                    └────────────────────┼─────────────────────┘
                                         │
                              ┌──────────▼───────────┐
                              │  Storage Cluster     │
                              │  (4-16 shards)       │
                              └──────────┬───────────┘
                                         │
                              ┌──────────▼───────────┐
                              │   Sutra Cache      │
                              │   Storage Shard    │
                              │   (HNSW + WAL)     │
                              │   (500MB L2 cache)   │
                              └──────────┬───────────┘
                                         │
                              ┌──────────▼───────────┐
                              │  ML-Base HAProxy     │
                              │  (Smart routing)     │
                              └──────────┬───────────┘
                                         │
                    ┌────────────────────┼────────────────────┐
                    │                    │                    │
              ┌─────▼─────┐        ┌────▼────┐         ┌────▼────┐
              │ ML-Base-1 │        │ML-Base-2│         │ML-Base-3│
              │ GPU (T4)  │        │GPU (T4) │         │GPU (T4) │
              │ INT8 Model│        │INT8 Mdl │         │INT8 Mdl │
              └───────────┘        └─────────┘         └─────────┘
```

---

## Implementation Roadmap

### Phase 1: Quick Wins (Week 1-2) - 5x improvement
```
✓ Multi-tier caching (Sutra Storage + in-memory)
✓ Dynamic batching optimization
✓ Batch size tuning per edition
✓ Connection pooling improvements

Expected: 0.14 → 0.7 concepts/sec
```

### Phase 2: Horizontal Scaling (Week 3-4) - 10x total
```
✓ Deploy 3 ML-Base replicas
✓ HAProxy load balancer setup
✓ Health check automation
✓ Monitoring & alerting

Expected: 0.7 → 1.4 concepts/sec
```

### Phase 3: GPU Acceleration (Week 5-8) - 50x total
```
✓ GPU-enabled ML-Base service
✓ CUDA optimization
✓ FP16 inference
✓ Larger batch processing

Expected: 1.4 → 7.0 concepts/sec
```

### Phase 4: Model Optimization (Week 9-12) - 100x total
```
✓ INT8 quantization
✓ ONNX runtime integration
✓ Model distillation (optional)
✓ A/B testing for accuracy

Expected: 7.0 → 14.0 concepts/sec
```

---

## Cost-Benefit Analysis

### Current State (Baseline)
```
Infrastructure:
- 1x ML-Base service (CPU): $150/month
- Storage + API: $200/month
Total: $350/month

Capacity:
- 0.14 concepts/sec = 12,096 concepts/day
- ~360,000 concepts/month
- Estimated: 100-200 active users
```

### Target State (1,000 Users)
```
Estimated Load:
- 1,000 users × 50 queries/day = 50,000 queries/day
- 50,000 queries × 3 concepts avg = 150,000 concepts/day
- Required: 1.74 concepts/sec average (12x current)
- Peak load (3x average): 5.2 concepts/sec

Infrastructure (Recommended):
- 3x GPU ML-Base (T4): $1,137/month ($379 each)
- Sutra cache shard: $30/month (dedicated storage instance)
- Storage cluster (4 shards): $400/month
- API replicas (3x): $300/month
Total: $1,887/month

Capacity Achieved:
- 14 concepts/sec sustained (100x baseline)
- 42 concepts/sec peak (with burst)
- 1.2M concepts/day
- Supports 3,000+ users comfortably
```

**ROI**: $1,537/month investment → 10x capacity increase

---

## Monitoring & Metrics

### Key Performance Indicators

```python
# Real-time monitoring dashboard

METRICS = {
    "throughput": {
        "concepts_per_second": 14.0,
        "requests_per_second": 50.0,
        "target": ">5.0 concepts/sec"
    },
    "latency": {
        "p50": "50ms",
        "p95": "200ms",
        "p99": "500ms",
        "target": "<200ms p95"
    },
    "cache": {
        "l1_hit_rate": 0.85,  # 85% in-memory hits
        "l2_hit_rate": 0.12,  # 12% Sutra cache shard hits
        "total_hit_rate": 0.97,  # 97% cached!
        "target": ">70%"
    },
    "resource_utilization": {
        "gpu_usage": 0.75,    # 75% GPU utilization
        "memory_usage": 0.68,  # 68% RAM usage
        "cpu_usage": 0.45     # 45% CPU usage
    },
    "availability": {
        "uptime": 0.9998,     # 99.98% uptime
        "error_rate": 0.0001, # 0.01% errors
        "target": ">99.9%"
    }
}
```

### Alerts & Thresholds (Sutra Grid Events)

**Sutra monitors itself using Grid events - no external tools needed!**

```python
# Natural language queries via Sutra reasoning engine
# No Prometheus, Grafana, or Datadog required

# Latency monitoring
"Show embedding requests with P95 latency > 500ms in last 5 minutes"
"Which embeddings are slowest today?"

# Cache monitoring  
"Show cache hit rate for last hour"
"Alert when cache hit rate drops below 50%"

# Capacity monitoring
"Show GPU utilization for ml-base replicas"
"Which GPU is overloaded?"

# Availability monitoring
"Show ML-Base service health checks"
"What caused ml-base-2 to go offline?"
```

**Grid Event Types for Embedding Service:**
- `EmbeddingLatency` - Track generation time per request
- `CacheHitRate` - L1/L2 cache performance  
- `MLBaseHealth` - Service health checks
- `BatchProcessing` - Batch size and efficiency
- `DimensionConfig` - Track dimension changes
- `ModelLoading` - Model initialization events

**See:** `docs/sutra-platform-review/DEVOPS_SELF_MONITORING.md` for complete self-monitoring architecture.

---

## Architecture Principles Preserved

### ✅ No Breaking Changes

1. **TCP Binary Protocol**: Unchanged (Storage ↔ Services)
2. **Unified Learning Pipeline**: Remains in Storage Server
3. **Edition System**: All optimizations respect edition limits
4. **Security Model**: TLS/HMAC preserved in production mode
5. **Docker Compose**: Same deployment model, just more replicas
6. **API Compatibility**: All existing endpoints work unchanged

### ✅ Backward Compatibility

```python
# Old clients continue to work
response = requests.post("http://localhost:8080/api/learn", json=concept)

# New optimizations are transparent:
# - Caching happens automatically
# - Batching happens behind the scenes
# - GPU acceleration is invisible to API
# - Load balancing is handled by HAProxy
```

---

## Testing Strategy

### Load Testing

```bash
# Load test script for embedding service
python3 scripts/load_test_embedding.py \
  --target-rps 10 \
  --duration 300 \
  --concurrent-users 100
```

```python
#!/usr/bin/env python3
"""Load test for embedding service scaling validation"""

import asyncio
import aiohttp
import time
from statistics import mean, median

async def load_test(target_rps: int, duration: int):
    results = []
    start_time = time.time()
    
    async with aiohttp.ClientSession() as session:
        while time.time() - start_time < duration:
            # Generate batch of requests
            tasks = []
            for _ in range(target_rps):
                task = generate_embedding_request(session)
                tasks.append(task)
            
            # Execute batch
            batch_results = await asyncio.gather(*tasks, return_exceptions=True)
            results.extend([r for r in batch_results if not isinstance(r, Exception)])
            
            # Rate limiting
            await asyncio.sleep(1.0)
    
    # Report statistics
    latencies = [r['latency'] for r in results]
    print(f"Total requests: {len(results)}")
    print(f"Achieved RPS: {len(results)/duration:.2f}")
    print(f"P50 latency: {median(latencies):.2f}ms")
    print(f"P95 latency: {sorted(latencies)[int(len(latencies)*0.95)]:.2f}ms")
    print(f"Error rate: {sum(1 for r in results if r['error'])/len(results)*100:.2f}%")
```

### Gradual Rollout

```
Week 1-2: Phase 1 (Caching) → 10% traffic
Week 3-4: Phase 2 (Horizontal) → 25% traffic
Week 5-6: Phase 3 (GPU) → 50% traffic
Week 7-8: Phase 4 (Full rollout) → 100% traffic
```

---

## Conclusion

This 5-tier scaling strategy provides a **clear path from 0.14 to 14+ concepts/sec** (100x improvement) while:

✅ Preserving existing architecture  
✅ Maintaining backward compatibility  
✅ Respecting edition-based limits  
✅ Providing incremental improvements  
✅ Supporting 1,000+ concurrent users  

**Next Steps:**
1. Review and approve strategy
2. Begin Phase 1 implementation (caching)
3. Set up monitoring infrastructure
4. Plan GPU hardware acquisition
5. Execute phased rollout with validation

---

*Document Version: 1.0*  
*Last Updated: November 8, 2025*  
*Owner: Sutra AI Engineering Team*

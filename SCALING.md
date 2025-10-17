# Sutra AI - Production Scaling Strategy

## 🎯 Service Analysis & Scaling Recommendations

### Architecture Overview
```
┌─────────────────────────────────────────────────────────────┐
│                    SCALING ZONES                             │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────┐  ┌──────────────┐                         │
│  │ CDN/Edge     │  │  K8s HPA     │                         │
│  │ (Serverless) │  │ (Horizontal) │                         │
│  └──────┬───────┘  └──────┬───────┘                         │
│         │                 │                                  │
│    ┌────▼─────┐      ┌───▼────┐  ┌─────────┐               │
│    │  Client  │      │  API   │  │ Hybrid  │               │
│    │  77 MB   │      │ 703 MB │  │ 653 MB  │               │
│    │ Scale∞   │      │ 3-50x  │  │ 2-10x   │               │
│    └──────────┘      └───┬────┘  └────┬────┘               │
│                          │             │                     │
│                          └─────┬───────┘                     │
│                                │                             │
│                         ┌──────▼────────┐                    │
│                         │   Storage     │  ← STATEFUL        │
│                         │    24 MB      │                    │
│                         │  replicas=1   │                    │
│                         └───────────────┘                    │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

## 📊 Scaling Matrix

| Service | Size | Type | K8s Scaling | Serverless | Why |
|---------|------|------|-------------|------------|-----|
| **sutra-client** | 77MB | Static | ✅ 2-20 | ✅ YES | CDN-ready, stateless |
| **sutra-api** | 703MB | REST | ✅ 3-50 | ❌ NO | Heavy, persistent gRPC |
| **sutra-hybrid** | 653MB | ML | ✅ 2-10 | ❌ NO | CPU-bound, memory-heavy |
| **sutra-control** | 147MB | Admin | ⚠️ 1-2 | ❌ NO | Low traffic, not critical |
| **storage-server** | 24MB | gRPC | ❌ 1 only | ❌ NO | Stateful, shared memory |

## 🚀 Horizontal Pod Autoscaling (HPA)

### 1. Client - Edge/CDN Strategy (BEST)

**Option A: Kubernetes HPA**
```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: sutra-client-hpa
  namespace: sutra-ai
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: sutra-client
  minReplicas: 2
  maxReplicas: 20
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Pods
    pods:
      metric:
        name: requests_per_second
      target:
        type: AverageValue
        averageValue: "100"
```

**Option B: Serverless (RECOMMENDED)**
```bash
# Deploy to CDN
aws s3 sync dist/ s3://sutra-client-prod
aws cloudfront create-invalidation --distribution-id XXX

# Or Netlify/Vercel/Cloudflare Pages
# Cost: $0/month for reasonable traffic
# Latency: <50ms global
# Scaling: Unlimited
```

### 2. API - Compute Scaling

**HPA Configuration:**
```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: sutra-api-hpa
  namespace: sutra-ai
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: sutra-api
  minReplicas: 3
  maxReplicas: 50
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 30
      policies:
      - type: Percent
        value: 100
        periodSeconds: 30
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Percent
        value: 50
        periodSeconds: 120
```

**Why NOT serverless:**
- 703MB image (Lambda limit: 250MB unzipped)
- Persistent gRPC connection to storage
- Cold start would be 5-10 seconds
- Need connection pooling

**Scaling triggers:**
- CPU > 70% → add pod
- Memory > 80% → add pod
- P95 latency > 500ms → add pod
- Request rate > 100 req/s/pod → add pod

### 3. Hybrid - Memory-Bound Scaling

**HPA Configuration:**
```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: sutra-hybrid-hpa
  namespace: sutra-ai
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: sutra-hybrid
  minReplicas: 2
  maxReplicas: 10  # Limited by memory cost
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 60  # Lower due to CPU-intensive
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 75
```

**Why limited scaling:**
- Each pod needs 1-4Gi RAM (numpy/scikit-learn)
- CPU-intensive (vector embeddings)
- Cost scales linearly with replicas
- Recommend max 10 pods unless heavy traffic

**Optimization strategies:**
1. **Model caching**: Cache embeddings
2. **Batch processing**: Queue requests
3. **GPU acceleration**: If vector search is bottleneck
4. **Separate concerns**: 
   - Light API pod (routing)
   - Heavy compute pod (ML)

### 4. Control - Minimal Scaling

```yaml
# Simple 2-replica for HA, no HPA needed
replicas: 2
resources:
  requests:
    memory: 256Mi
    cpu: 250m
  limits:
    memory: 1Gi
    cpu: 1000m
```

**Why minimal:**
- Admin/monitoring UI
- Low traffic (<10 req/min)
- Not critical path
- 1-2 replicas sufficient

### 5. Storage - StatefulSet (NO scaling yet)

**Current: Single replica**
```yaml
apiVersion: apps/v1
kind: StatefulSet  # Not Deployment!
metadata:
  name: storage-server
spec:
  replicas: 1  # CANNOT scale without distributed consensus
  serviceName: storage-server
  volumeClaimTemplates:
  - metadata:
      name: storage-data
    spec:
      accessModes: [ "ReadWriteOnce" ]
      resources:
        requests:
          storage: 50Gi
```

**Why NO scaling:**
- Uses memory-mapped file (single process)
- Lock-free concurrency assumes single writer
- Would need:
  - Raft/Paxos consensus
  - Distributed lock manager
  - Replication protocol
  - Leader election

**Future multi-replica:**
```rust
// Need to implement:
1. Consensus protocol (etcd/Consul/custom)
2. Write-ahead log replication
3. Leader election
4. Follower read-replicas
5. Quorum writes (W + R > N)
```

**Recommended now:**
- Single replica with PVC backup
- Vertical scaling: More CPU/RAM
- Regular snapshots to S3
- Fast failover (< 30s)

## 💰 Cost Optimization

### Serverless Options Analysis

**Client (77MB):**
```
Option 1: Kubernetes (current)
- Cost: $5-20/month (2-20 pods)
- Latency: 50-200ms
- Maintenance: Medium

Option 2: CDN (recommended)
- Cost: $0-5/month (CloudFlare/Netlify)
- Latency: 10-50ms (edge)
- Maintenance: Zero
- Winner: SERVERLESS ✅
```

**API (703MB):**
```
Lambda: ❌ Too heavy, cold starts
Fargate: ✅ Good fit (0.5-4 vCPU)
K8s: ✅ Best control

Recommendation: K8s HPA
- Fargate if no cluster
- Lambda: Never (wrong fit)
```

**Hybrid (653MB + ML):**
```
Lambda: ❌ Memory limits, cold start
SageMaker: ⚠️ Overkill for this
K8s: ✅ Best for ML workloads

Recommendation: K8s with GPU nodes (optional)
```

## 📈 Traffic-Based Scaling Strategy

### Low Traffic (<100 req/s)
```yaml
Client: 2 pods (or CDN)
API: 3 pods
Hybrid: 2 pods
Control: 1 pod
Storage: 1 pod
Total cost: ~$50-100/month
```

### Medium Traffic (100-1000 req/s)
```yaml
Client: CDN (unlimited) or 5 pods
API: 10 pods (HPA)
Hybrid: 5 pods (HPA)
Control: 2 pods
Storage: 1 pod + read replicas
Total cost: ~$200-500/month
```

### High Traffic (>1000 req/s)
```yaml
Client: CDN (global)
API: 20-50 pods (HPA)
Hybrid: 10 pods (HPA) + GPU
Control: 2 pods
Storage: 1 primary + sharding strategy
Total cost: ~$1000-5000/month
```

## 🎯 Immediate Recommendations

### Phase 1: Fix & Stabilize (Now)
1. ✅ Fix pydantic-settings dependency
2. ✅ Verify all containers healthy
3. ✅ Add HPA for API/Hybrid
4. ✅ Move client to CDN/edge

### Phase 2: Production Ready (Week 1)
1. Configure HPA with metrics
2. Set up monitoring (Prometheus)
3. Add circuit breakers
4. Implement rate limiting
5. Configure backups

### Phase 3: Scale (Month 1)
1. Optimize API response caching
2. Add Redis for session/cache
3. Implement request queuing
4. Consider read replicas for storage
5. Add regional deployments

## 🔧 Quick Fixes Needed

**Missing dependencies:**
```bash
# Add to requirements:
pydantic-settings>=2.0.0
grpcio>=1.60.0
grpcio-tools>=1.60.0
```

**Health check improvements:**
```python
# Add storage connectivity check
@app.get("/health")
async def health_check():
    try:
        # Check storage connection
        await storage_client.ping()
        return {"status": "healthy"}
    except:
        raise HTTPException(503, "storage unavailable")
```

## 📝 Serverless Summary

| Can Go Serverless | Cannot Go Serverless |
|-------------------|---------------------|
| ✅ Client (CDN) | ❌ API (too heavy) |
| ✅ Static assets | ❌ Hybrid (ML/memory) |
| ✅ Edge functions | ❌ Storage (stateful) |
| | ❌ Control (WebSocket) |

**Best strategy:** Hybrid cloud-native + serverless
- Client: CDN/Edge
- APIs: Kubernetes with HPA
- Storage: Kubernetes StatefulSet
- Cache: Redis/ElastiCache
- Jobs: Lambda for async tasks

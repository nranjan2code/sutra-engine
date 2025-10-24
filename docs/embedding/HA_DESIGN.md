# High Availability Embedding Service Design

**Multi-replica architecture for 99.99% availability**

Version: 2.0.0 | Status: Designed (Not Implemented) | Last Updated: 2025-10-23

---

## Overview

This document describes the planned high availability (HA) architecture for the embedding service. **Status: Design complete, implementation planned for Phase 2 (Q1 2026).**

**Goal:** Eliminate single point of failure in embedding generation with automatic failover.

---

## Current Architecture (Single Instance)

```
Storage Server
    ↓ HTTP
Embedding Service (port 8888)
    ↓
nomic-embed-text-v1.5 (768-d)
```

**Problem:** If embedding service fails, the entire system cannot learn new concepts.

---

## Proposed HA Architecture

```
┌────────────────────────────────────────────────────┐
│            Load Balancer / Service Registry         │
│          (Round-robin + Health Checks)              │
└─────────┬──────────┬──────────┬───────────────────┘
          │          │          │
     ┌────┴───┐ ┌────┴───┐ ┌────┴───┐
     │ Embed  │ │ Embed  │ │ Embed  │
     │ Svc 1  │ │ Svc 2  │ │ Svc 3  │
     │ :8888  │ │ :8889  │ │ :8890  │
     └────────┘ └────────┘ └────────┘
        ✅         ✅          ❌
```

**Each replica:**
- Independent process/container
- Same model (nomic-embed-text-v1.5)
- Same configuration (768 dimensions)
- Health check endpoint
- Automatic failover on failure

---

## Implementation Components

### 1. Service Registry

**Options:**
- **Consul** (recommended): Built-in health checks, DNS-based load balancing
- **etcd**: Lightweight, Kubernetes-native
- **Custom**: Simple HTTP registry

**Example (Consul):**
```bash
# Register service
curl -X PUT http://consul:8500/v1/agent/service/register \
  -d '{
    "ID": "embedding-1",
    "Name": "embedding-service",
    "Address": "localhost",
    "Port": 8888,
    "Check": {
      "HTTP": "http://localhost:8888/health",
      "Interval": "30s"
    }
  }'
```

### 2. Client-Side Load Balancing

**Storage server modification:**
```rust
struct EmbeddingServicePool {
    services: Vec<EmbeddingServiceClient>,
    health_checker: HealthChecker,
}

impl EmbeddingServicePool {
    fn get_healthy_service(&self) -> Option<&EmbeddingServiceClient> {
        self.services
            .iter()
            .filter(|s| self.health_checker.is_healthy(s))
            .choose(&mut rand::thread_rng())  // Random selection
    }
    
    fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let max_retries = 3;
        
        for attempt in 0..max_retries {
            if let Some(service) = self.get_healthy_service() {
                match service.embed(text) {
                    Ok(embedding) => return Ok(embedding),
                    Err(e) => {
                        warn!("Embedding failed on attempt {}: {}", attempt, e);
                        self.health_checker.mark_unhealthy(service);
                    }
                }
            }
        }
        
        Err(EmbeddingError::AllServicesDown)
    }
}
```

### 3. Health Checks

**Endpoint:**
```
GET /health
```

**Response:**
```json
{
  "status": "healthy",
  "model": "nomic-embed-text-v1.5",
  "dimension": 768,
  "uptime_seconds": 3600,
  "requests_processed": 100000,
  "avg_latency_ms": 25,
  "success_rate": 0.999
}
```

**Health criteria:**
- Response time < 100ms
- Success rate > 95%
- Memory usage < 90%

---

## Deployment

### Docker Compose Example

```yaml
version: '3.8'
services:
  # Embedding Service - Replica 1
  embedding-service-1:
    image: sutra-embedding-service:latest
    ports:
      - "8888:8888"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8888/health"]
      interval: 30s
      timeout: 5s
      retries: 3
    deploy:
      resources:
        limits:
          memory: 2G
  
  # Embedding Service - Replica 2
  embedding-service-2:
    image: sutra-embedding-service:latest
    ports:
      - "8889:8888"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8888/health"]
      interval: 30s
      timeout: 5s
      retries: 3
    deploy:
      resources:
        limits:
          memory: 2G
  
  # Embedding Service - Replica 3
  embedding-service-3:
    image: sutra-embedding-service:latest
    ports:
      - "8890:8888"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8888/health"]
      interval: 30s
      timeout: 5s
      retries: 3
    deploy:
      resources:
        limits:
          memory: 2G
  
  # Storage Server (configured with multiple embedding services)
  storage-server:
    image: sutra-storage-server:latest
    ports:
      - "50051:50051"
    environment:
      - EMBEDDING_SERVICE_URLS=http://embedding-service-1:8888,http://embedding-service-2:8888,http://embedding-service-3:8888
      - EMBEDDING_SERVICE_HEALTH_CHECK_INTERVAL=30
```

---

## Capacity Planning

| Load (req/s) | Replicas Needed | Total Throughput | Cost/Month (AWS) |
|--------------|-----------------|------------------|------------------|
| 100 | 2 (1 + 1 failover) | 200 req/s | $100 |
| 1000 | 5 (4 + 1 failover) | 2000 req/s | $500 |
| 10000 | 20 (16 + 4 failover) | 20000 req/s | $2000 |

**Formula:**
- Active replicas = ceil(target_throughput / 50 req/s)
- Failover replicas = max(1, active_replicas * 0.25)
- Total replicas = active + failover

---

## Availability Calculation

**Single service:** 99% uptime  
**Two replicas:** 1 - (0.01)² = **99.99% uptime**  
**Three replicas:** 1 - (0.01)³ = **99.9999% uptime** (5 nines!)

**SLA Target:** 99.99% (4 nines) = 52 minutes downtime/year

---

## Failure Scenarios

### Scenario 1: One Replica Fails

**System Response:**
1. Health check detects failure (30s)
2. Load balancer removes from pool
3. Requests route to healthy replicas
4. **Zero downtime**

### Scenario 2: All Replicas Fail

**System Response:**
1. Storage server retries 3 times
2. Returns error to client
3. Alert triggered
4. Manual intervention required

**Mitigation:** Run 3+ replicas in different availability zones.

---

## Future Enhancements

### Phase 3 (Q2 2026)
- **Auto-scaling**: Scale replicas based on load
- **Geographic distribution**: Multi-region deployment
- **CDN integration**: Cache embeddings at edge

### Phase 4 (Q3 2026)
- **Model versioning**: A/B test different embedding models
- **Request batching**: Batch multiple requests for efficiency
- **GPU acceleration**: Optional GPU support for faster embedding

---

## Implementation Checklist

- [ ] Add service registry (Consul/etcd)
- [ ] Implement client-side load balancing in storage server
- [ ] Add health check endpoint to embedding service
- [ ] Update Docker Compose for multi-replica deployment
- [ ] Add monitoring and alerting
- [ ] Document failover testing procedures
- [ ] Update WARP.md with HA configuration

---

## References

- [Scalability Architecture](../architecture/SCALABILITY.md)
- [Embedding Service Overview](SERVICE_OVERVIEW.md)
- [Monitoring Guide](../operations/MONITORING.md)

---

**Status:** Design complete, awaiting implementation in Phase 2 (Q1 2026).

Last Updated: 2025-10-23 | Version: 2.0.0

# Production Deployment Guide

**Complete guide for deploying Sutra AI in production**

---

## Prerequisites

- Docker 20.10+
- Docker Compose 2.0+
- 16GB+ RAM
- 50GB+ disk space

---

## Quick Deploy

```bash
# Deploy all services
./sutra-deploy.sh up

# Validate deployment
./sutra-deploy.sh status
./scripts/smoke-test-embeddings.sh
```

---

## Production Requirements

### ⚠️ CRITICAL: Embedding Configuration

**MUST** configure before deployment:

```yaml
# docker-compose-grid.yml
storage-server:
  environment:
    - VECTOR_DIMENSION=768  # MUST be 768
    - SUTRA_EMBEDDING_SERVICE_URL=http://embedding-ha:8888

sutra-hybrid:
  environment:
    - SUTRA_VECTOR_DIMENSION=768  # MUST match
    - SUTRA_EMBEDDING_SERVICE_URL=http://embedding-ha:8888
```

See [Production Requirements](../operations/PRODUCTION_REQUIREMENTS.md) for complete details.

---

## Service Architecture

```
10 Services:
- Storage Server (port 50051)
- Embedding HA (3 replicas + HAProxy, port 8888)
- API (port 8000)
- Hybrid (port 8001)
- Control Center (port 9000)
- Client UI (port 8080)
- Grid Master (ports 7001-7002)
- Grid Agents (ports 8003-8004)
- Grid Event Storage (port 50052)
```

---

## Monitoring

```bash
# Check service status
docker-compose -f docker-compose-grid.yml ps

# View logs
docker logs storage-server --tail 50
docker logs sutra-api --tail 50

# Check health
curl http://localhost:8000/health
curl http://localhost:8001/ping
```

---

## Scaling

### Horizontal Scaling (Sharded Storage)

```yaml
storage-server:
  environment:
    - SUTRA_STORAGE_MODE=sharded
    - SUTRA_NUM_SHARDS=8  # 4-16 recommended
```

### High Availability (Embedding Service)

Already configured with 3 replicas + HAProxy load balancer.

---

## Troubleshooting

See [Troubleshooting Guide](../../TROUBLESHOOTING.md) for common issues.

---

## License

MIT License

**Built with ❤️ by the Sutra AI Team**

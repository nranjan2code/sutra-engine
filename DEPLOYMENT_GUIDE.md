# Sutra AI Deployment Guide

**Single source of truth for production deployment**

## Quick Start

```bash
# 1. Clone repository
git clone https://github.com/sutra-ai/sutra-models.git
cd sutra-models

# 2. Deploy complete system
./sutra-deploy.sh up

# 3. Verify health
./sutra-deploy.sh status

# 4. Access services
open http://localhost:9000  # Control Center
open http://localhost:8080  # Client UI
```

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Docker Network (sutra-network)               │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │               Storage Layer (Sharded)                    │  │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │  │
│  │  │ Storage      │  │ Grid Event   │  │ Embedding    │  │  │
│  │  │ Server       │  │ Storage      │  │ Service      │  │  │
│  │  │ (4 shards)   │  │ (50052)      │  │ (8888)       │  │  │
│  │  │ (50051)      │  │              │  │ nomic-v1.5   │  │  │
│  │  └──────────────┘  └──────────────┘  └──────────────┘  │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    API Layer                              │  │
│  │  ┌──────────────┐  ┌──────────────┐                     │  │
│  │  │ Sutra API    │  │ Sutra Hybrid │                     │  │
│  │  │ (8000)       │  │ (8001)       │                     │  │
│  │  │ REST         │  │ Semantic AI  │                     │  │
│  │  └──────────────┘  └──────────────┘                     │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    UI Layer                               │  │
│  │  ┌──────────────┐  ┌──────────────┐                     │  │
│  │  │ Control      │  │ Client UI    │                     │  │
│  │  │ Center       │  │ (8080)       │                     │  │
│  │  │ (9000)       │  │ Streamlit    │                     │  │
│  │  └──────────────┘  └──────────────┘                     │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │              Grid Infrastructure                          │  │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │  │
│  │  │ Grid Master  │  │ Grid Agent 1 │  │ Grid Agent 2 │  │  │
│  │  │ (7001-7002)  │  │ (8003)       │  │ (8004)       │  │  │
│  │  └──────────────┘  └──────────────┘  └──────────────┘  │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Prerequisites

### System Requirements

- **OS**: Linux, macOS, or Windows with WSL2
- **Docker**: 20.10+ with Docker Compose V2
- **RAM**: 8GB minimum, 16GB recommended
- **Disk**: 20GB free space
- **CPU**: 4 cores minimum, 8 cores recommended

### Software Dependencies

```bash
# Check Docker version
docker --version  # Should be 20.10+
docker compose version  # Should be v2.x

# Check available resources
docker system info | grep -E "(CPUs|Memory)"
```

## Configuration

### Environment Variables

**File**: `docker-compose-grid.yml`

```yaml
# Storage Configuration (CRITICAL)
storage-server:
  environment:
    # Storage Mode
    - SUTRA_STORAGE_MODE=sharded    # single or sharded
    - SUTRA_NUM_SHARDS=4            # Number of shards (if sharded)
    
    # Vector Configuration
    - VECTOR_DIMENSION=768           # MUST be 768 (nomic-embed-text)
    
    # Embedding Service
    - SUTRA_EMBEDDING_SERVICE_URL=http://sutra-embedding-service:8888
    
    # Performance Tuning
    - RECONCILE_INTERVAL_MS=10
    - MEMORY_THRESHOLD=50000
```

### Storage Modes

#### Development (Single Storage)

```yaml
storage-server:
  environment:
    - SUTRA_STORAGE_MODE=single
    - STORAGE_PATH=/data
```

**Use when:**
- Development environment
- < 1M concepts
- Single developer testing

#### Production (Sharded Storage)

```yaml
storage-server:
  environment:
    - SUTRA_STORAGE_MODE=sharded
    - SUTRA_NUM_SHARDS=4
    - STORAGE_PATH=/data
```

**Use when:**
- Production deployment
- > 1M concepts
- Need horizontal scalability

## Deployment

### Single Command Deployment

```bash
# Deploy all services
./sutra-deploy.sh up

# Services will start in dependency order:
# 1. Storage Layer (storage-server, grid-event-storage, embedding-service)
# 2. Grid Infrastructure (grid-master, grid-agents)
# 3. API Layer (sutra-api, sutra-hybrid)
# 4. UI Layer (control-center, client-ui)
```

### Step-by-Step Deployment

```bash
# 1. Build all Docker images
./sutra-deploy.sh build

# 2. Start infrastructure services
docker-compose -f docker-compose-grid.yml up -d \
  storage-server \
  grid-event-storage \
  sutra-embedding-service

# 3. Wait for storage to be ready
sleep 10
docker logs sutra-storage | grep "listening"

# 4. Start application services
docker-compose -f docker-compose-grid.yml up -d \
  sutra-api \
  sutra-hybrid \
  sutra-client \
  sutra-control

# 5. Start grid services
docker-compose -f docker-compose-grid.yml up -d \
  grid-master \
  grid-agent-1 \
  grid-agent-2

# 6. Verify all services
./sutra-deploy.sh status
```

### Verification

```bash
# Check all services are running
docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"

# Expected: 10 services in "Up" status

# Test storage server
curl -s http://localhost:50051/health
# Expected: {"status": "healthy"}

# Test embedding service
curl -s http://localhost:8888/health | jq '.status'
# Expected: "healthy"

# Test API
curl -s http://localhost:8000/health
# Expected: {"status": "healthy"}

# Test complete workflow
curl -X POST http://localhost:8001/sutra/learn \
  -H "Content-Type: application/json" \
  -d '{"text": "Test deployment successful"}'

curl -X POST http://localhost:8001/sutra/query \
  -H "Content-Type: application/json" \
  -d '{"query": "test deployment"}'
```

## Management

### Service Control

```bash
# Stop all services
./sutra-deploy.sh down

# Restart services
./sutra-deploy.sh restart

# Restart specific service
docker-compose -f docker-compose-grid.yml restart sutra-api

# View logs
./sutra-deploy.sh logs [service-name]

# Follow logs in real-time
docker-compose -f docker-compose-grid.yml logs -f sutra-hybrid
```

### Maintenance

```bash
# Interactive maintenance menu
./sutra-deploy.sh maintenance

# Manual operations:
# 1. Flush storage
curl -X POST http://localhost:50051/flush

# 2. Check storage stats
curl -s http://localhost:8000/stats | jq

# 3. Backup storage data
docker exec sutra-storage tar czf /tmp/backup.tar.gz /data

# 4. Copy backup to host
docker cp sutra-storage:/tmp/backup.tar.gz ./backup-$(date +%Y%m%d).tar.gz
```

### Monitoring

```bash
# System health
./sutra-deploy.sh validate

# Resource usage
docker stats --no-stream

# Storage performance
curl -s http://localhost:8000/stats | jq '.total_concepts, .total_embeddings'

# Embedding service metrics
curl -s http://localhost:8888/metrics | jq '.success_rate, .cache_hit_rate'

# Grid status
curl -s http://localhost:9000/api/grid/status | jq
```

## Scaling

### Vertical Scaling (More Resources)

```yaml
# docker-compose-grid.yml
storage-server:
  deploy:
    resources:
      limits:
        cpus: '4.0'
        memory: 16G
      reservations:
        cpus: '2.0'
        memory: 8G
```

### Horizontal Scaling (More Shards)

```yaml
# Update shard count
storage-server:
  environment:
    - SUTRA_NUM_SHARDS=8  # Increase from 4 to 8
```

```bash
# Restart storage server
docker-compose -f docker-compose-grid.yml restart storage-server

# Verify sharding
docker logs sutra-storage | grep "shard"
# Should see 8 shards initialized
```

### Load Balancing (Multiple API Instances)

```bash
# Add more API replicas
docker-compose -f docker-compose-grid.yml up -d --scale sutra-api=3

# Configure reverse proxy (nginx, traefik, etc.)
```

## Backup and Recovery

### Backup Strategy

```bash
# Full system backup
./scripts/backup-system.sh

# Storage data only
docker exec sutra-storage tar czf /tmp/storage-backup.tar.gz /data

# Export to host
docker cp sutra-storage:/tmp/storage-backup.tar.gz ./backups/

# Verify backup
tar tzf ./backups/storage-backup.tar.gz | head
```

### Recovery

```bash
# Stop storage server
docker-compose -f docker-compose-grid.yml stop storage-server

# Restore data
docker cp ./backups/storage-backup.tar.gz sutra-storage:/tmp/
docker exec sutra-storage tar xzf /tmp/storage-backup.tar.gz -C /

# Restart storage server
docker-compose -f docker-compose-grid.yml start storage-server

# Verify recovery
curl -s http://localhost:8000/stats
```

## Troubleshooting

### Common Issues

#### Services Not Starting

```bash
# Check Docker daemon
docker info

# Check for port conflicts
lsof -i :8000 -i :8001 -i :50051

# Check logs
docker-compose -f docker-compose-grid.yml logs --tail=50
```

#### Storage Server Errors

```bash
# Check storage mode
docker logs sutra-storage | grep "Storage mode"

# Verify embedding service
curl http://localhost:8888/health

# Check WAL replay
docker logs sutra-storage | grep "WAL"
```

#### Performance Issues

```bash
# Check shard distribution
curl -s http://localhost:8000/stats

# Monitor resource usage
docker stats

# Check HNSW indexing
docker logs sutra-storage | grep "HNSW"
```

### Getting Help

1. Check logs: `./sutra-deploy.sh logs`
2. Run validation: `./sutra-deploy.sh validate`
3. Review documentation: `docs/` directory
4. Check WARP.md for development guidance

## Production Checklist

- [ ] Storage mode configured (sharded for > 1M concepts)
- [ ] Embedding service healthy (768-d vectors)
- [ ] WAL enabled and verified
- [ ] Backup strategy implemented
- [ ] Monitoring configured
- [ ] Resource limits set
- [ ] Health checks passing
- [ ] End-to-end test successful

## Service URLs

After deployment, services are available at:

| Service | URL | Purpose |
|---------|-----|---------|
| Control Center | http://localhost:9000 | Management UI |
| Client UI | http://localhost:8080 | Interactive interface |
| Sutra API | http://localhost:8000 | REST API |
| Sutra Hybrid | http://localhost:8001 | Semantic AI |
| Embedding Service | http://localhost:8888 | Vector generation |
| Grid Master | http://localhost:7001 | Grid coordination |
| Storage Server | localhost:50051 | TCP binary protocol |

## References

- **WARP.md** - Development guide
- **docs/storage/STORAGE_GUIDE.md** - Storage architecture
- **docs/SCALABILITY.md** - Scaling guide
- **docker-compose-grid.yml** - Service configuration
- **./sutra-deploy.sh** - Deployment script

# Docker Naming Conventions

**Version:** 1.0
**Date:** 2025-01-06
**Status:** Standardized

---

## Overview

All Sutra AI Docker images and containers use the `sutra-works-` prefix to:
- Avoid naming conflicts with other deployments
- Clearly identify Sutra components
- Enable easy filtering and management
- Maintain consistency across environments

---

## Naming Standard

### Format

```
Container Name: sutra-works-<component-name>
Image Name:     sutra-works-<component-name>:<version>
```

### Rules

1. **Prefix**: All custom images and containers MUST use `sutra-works-` prefix
2. **Component Name**: Descriptive, kebab-case name (e.g., `storage`, `api`, `grid-master`)
3. **Version Tag**: Use `${SUTRA_VERSION:-latest}` for consistent versioning
4. **External Images**: Third-party images (HAProxy, nginx base) keep original names

---

## Complete Service Inventory

### Infrastructure Services

| Service ID | Container Name | Image Name |
|-----------|----------------|------------|
| nginx-proxy | `sutra-works-nginx-proxy` | `sutra-works-nginx-proxy:${SUTRA_VERSION}` |

### Storage Layer

| Service ID | Container Name | Image Name |
|-----------|----------------|------------|
| storage-server | `sutra-works-storage` | `sutra-works-storage-server:${SUTRA_VERSION}` |
| grid-event-storage | `sutra-works-grid-events` | `sutra-works-storage-server:${SUTRA_VERSION}` |
| user-storage-server | `sutra-works-user-storage` | `sutra-works-storage-server:${SUTRA_VERSION}` |

### Grid Infrastructure (Enterprise)

| Service ID | Container Name | Image Name |
|-----------|----------------|------------|
| grid-master | `sutra-works-grid-master` | `sutra-works-grid-master:${SUTRA_VERSION}` |
| grid-agent-1 | `sutra-works-grid-agent-1` | `sutra-works-grid-agent:${SUTRA_VERSION}` |
| grid-agent-2 | `sutra-works-grid-agent-2` | `sutra-works-grid-agent:${SUTRA_VERSION}` |

### API Layer

| Service ID | Container Name | Image Name |
|-----------|----------------|------------|
| sutra-api | `sutra-works-api` | `sutra-works-api:${SUTRA_VERSION}` |
| sutra-hybrid | `sutra-works-hybrid` | `sutra-works-hybrid:${SUTRA_VERSION}` |

### ML Services

| Service ID | Container Name | Image Name |
|-----------|----------------|------------|
| ml-base-service | `sutra-works-ml-base` | `sutra-works-ml-base-service:${SUTRA_VERSION}` |
| embedding-single | `sutra-works-embedding-single` | `sutra-works-embedding-service:${SUTRA_VERSION}` |
| embedding-1 | `sutra-works-embedding-1` | `sutra-works-embedding-service:latest` |
| embedding-2 | `sutra-works-embedding-2` | `sutra-works-embedding-service:latest` |
| embedding-3 | `sutra-works-embedding-3` | `sutra-works-embedding-service:latest` |
| embedding-ha | `sutra-works-embedding-ha` | `haproxy:2.8-alpine` (external) |
| nlg-single | `sutra-works-nlg-single` | `sutra-works-nlg-service:${SUTRA_VERSION}` |
| nlg-ha-1 | `sutra-works-nlg-1` | `sutra-works-nlg-service:${SUTRA_VERSION}` |
| nlg-2 | `sutra-works-nlg-2` | `sutra-works-nlg-service:latest` |
| nlg-3 | `sutra-works-nlg-3` | `sutra-works-nlg-service:latest` |
| nlg-ha | `sutra-works-nlg-ha` | `haproxy:2.8-alpine` (external) |

### Web Interfaces

| Service ID | Container Name | Image Name |
|-----------|----------------|------------|
| sutra-control | `sutra-works-control` | `sutra-works-control:${SUTRA_VERSION}` |
| sutra-client | `sutra-works-client` | `sutra-works-client:${SUTRA_VERSION}` |
| sutra-explorer-backend | `sutra-works-explorer-backend` | `sutra-works-explorer-backend:${SUTRA_VERSION}` |
| sutra-explorer-frontend | `sutra-works-explorer-frontend` | `sutra-works-explorer-frontend:${SUTRA_VERSION}` |

### Bulk Ingestion

| Service ID | Container Name | Image Name |
|-----------|----------------|------------|
| sutra-bulk-ingester | `sutra-works-bulk-ingester` | `sutra-works-bulk-ingester:${SUTRA_VERSION}` |

---

## Usage Examples

### Docker Commands

```bash
# List all Sutra containers
docker ps -a --filter "name=sutra-works-"

# Stop all Sutra containers
docker stop $(docker ps -q --filter "name=sutra-works-")

# Remove all Sutra containers
docker rm $(docker ps -aq --filter "name=sutra-works-")

# List all Sutra images
docker images | grep sutra-works-

# Remove all Sutra images
docker rmi $(docker images --format "{{.Repository}}:{{.Tag}}" | grep sutra-works-)

# View logs for specific service
docker logs sutra-works-api
docker logs sutra-works-storage
docker logs sutra-works-nginx-proxy

# Execute command in container
docker exec -it sutra-works-api bash
docker exec sutra-works-nginx-proxy nginx -t
```

### Docker Compose Commands

```bash
# Start specific service
docker-compose -f .sutra/compose/production.yml up -d sutra-api

# Restart service
docker-compose -f .sutra/compose/production.yml restart sutra-api

# View logs
docker-compose -f .sutra/compose/production.yml logs -f sutra-api

# Scale services
docker-compose -f .sutra/compose/production.yml up -d --scale grid-agent-1=3
```

### Filtering and Monitoring

```bash
# Check resource usage for all Sutra containers
docker stats $(docker ps --format "{{.Names}}" | grep sutra-works-)

# Export container list
docker ps --filter "name=sutra-works-" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" > sutra-containers.txt

# Count running services
docker ps --filter "name=sutra-works-" | wc -l

# Find containers using specific image
docker ps -a --filter "ancestor=sutra-works-storage-server:latest"
```

---

## Version Management

### Version Tags

All images support these version formats:

```bash
# Latest (rolling)
sutra-works-api:latest

# Specific version
sutra-works-api:3.0.0
sutra-works-api:3.0.1

# Environment variable (recommended)
sutra-works-api:${SUTRA_VERSION:-latest}
```

### Setting Version

```bash
# Set version for deployment
export SUTRA_VERSION=3.0.0

# Deploy with specific version
docker-compose -f .sutra/compose/production.yml up -d

# Verify deployed version
docker images | grep sutra-works- | grep 3.0.0
```

---

## Troubleshooting

### Issue: Container name conflict

```bash
# Error: The container name "/sutra-works-api" is already in use

# Solution 1: Remove old container
docker rm sutra-works-api

# Solution 2: Stop and remove
docker stop sutra-works-api && docker rm sutra-works-api

# Solution 3: Force remove
docker rm -f sutra-works-api
```

### Issue: Image tag mismatch

```bash
# List all tags for an image
docker images sutra-works-api

# Remove specific tag
docker rmi sutra-works-api:old-version

# Retag image
docker tag sutra-works-api:old-version sutra-works-api:3.0.0
```

### Issue: Finding orphaned containers

```bash
# Find stopped Sutra containers
docker ps -a --filter "name=sutra-works-" --filter "status=exited"

# Clean up stopped containers
docker container prune --filter "label=com.docker.compose.project=sutra"
```

---

## Best Practices

### ✅ DO

1. **Always use the prefix**: Never create containers without `sutra-works-` prefix
2. **Use version tags**: Specify `${SUTRA_VERSION}` instead of hardcoding versions
3. **Consistent naming**: Use kebab-case for component names (e.g., `grid-master`, not `gridMaster`)
4. **Meaningful names**: Container names should clearly indicate their purpose
5. **Document custom names**: If you add new services, update this document

### ❌ DON'T

1. **Mix naming conventions**: Don't use `sutra-` without `works-` prefix
2. **Use numbers only**: `sutra-works-1` is unclear; use `sutra-works-worker-1` instead
3. **Exceed length limits**: Docker names limited to 63 characters
4. **Use special characters**: Stick to alphanumeric and hyphens
5. **Rename running containers**: Stop and recreate instead

---

## Migration Guide

### From Old Naming Convention

If you're migrating from containers using `sutra-` prefix:

```bash
# 1. List old containers
docker ps -a --filter "name=sutra-" | grep -v "sutra-works-"

# 2. Stop old containers
docker stop sutra-api sutra-storage sutra-hybrid

# 3. Remove old containers
docker rm sutra-api sutra-storage sutra-hybrid

# 4. Pull/build new images with correct names
docker-compose -f .sutra/compose/production.yml build

# 5. Start with new naming
docker-compose -f .sutra/compose/production.yml up -d

# 6. Verify new names
docker ps --filter "name=sutra-works-"
```

### Updating Scripts

If you have scripts referencing old names:

```bash
# Old
docker logs sutra-api
docker exec sutra-storage curl localhost:50051

# New
docker logs sutra-works-api
docker exec sutra-works-storage curl localhost:50051
```

Use find-replace in your scripts:
```bash
# In all shell scripts
find scripts/ -type f -name "*.sh" -exec sed -i 's/sutra-api/sutra-works-api/g' {} +
find scripts/ -type f -name "*.sh" -exec sed -i 's/sutra-storage/sutra-works-storage/g' {} +
```

---

## Integration with Other Tools

### Kubernetes

```yaml
# Deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: sutra-works-api
spec:
  template:
    metadata:
      labels:
        app: sutra-works-api
    spec:
      containers:
      - name: api
        image: sutra-works-api:${SUTRA_VERSION}
```

### Docker Swarm

```yaml
version: "3.8"
services:
  sutra-api:
    image: sutra-works-api:${SUTRA_VERSION}
    container_name: sutra-works-api
    deploy:
      replicas: 3
      labels:
        - "com.sutra.service=api"
```

### Monitoring (Prometheus)

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'sutra-services'
    docker_sd_configs:
      - host: unix:///var/run/docker.sock
    relabel_configs:
      - source_labels: [__meta_docker_container_name]
        regex: 'sutra-works-.*'
        action: keep
```

---

## Quick Reference Card

```
┌─────────────────────────────────────────────────────────────┐
│                 Sutra Works Naming Convention               │
├─────────────────────────────────────────────────────────────┤
│ Format:       sutra-works-<component>                       │
│ Version Tag:  ${SUTRA_VERSION:-latest}                      │
│                                                              │
│ Common Commands:                                             │
│   List:   docker ps --filter "name=sutra-works-"            │
│   Logs:   docker logs sutra-works-<component>               │
│   Shell:  docker exec -it sutra-works-<component> bash      │
│   Stop:   docker stop sutra-works-<component>               │
│                                                              │
│ Key Services:                                                │
│   - sutra-works-nginx-proxy  (Entry point)                  │
│   - sutra-works-api          (REST API)                     │
│   - sutra-works-storage      (Core storage)                 │
│   - sutra-works-hybrid       (Semantic + NLG)               │
│   - sutra-works-client       (Web UI)                       │
└─────────────────────────────────────────────────────────────┘
```

---

## Validation Script

Use this script to verify naming conventions:

```bash
#!/bin/bash
# validate-naming.sh

echo "Validating Sutra Works naming conventions..."

# Check all containers have correct prefix
INVALID_CONTAINERS=$(docker ps -a --format "{{.Names}}" | grep "^sutra-" | grep -v "^sutra-works-" || true)

if [ -z "$INVALID_CONTAINERS" ]; then
    echo "✓ All containers use correct naming convention"
else
    echo "✗ Found containers with incorrect naming:"
    echo "$INVALID_CONTAINERS"
    exit 1
fi

# Check all images have correct prefix
INVALID_IMAGES=$(docker images --format "{{.Repository}}" | grep "^sutra-" | grep -v "^sutra-works-" || true)

if [ -z "$INVALID_IMAGES" ]; then
    echo "✓ All images use correct naming convention"
else
    echo "✗ Found images with incorrect naming:"
    echo "$INVALID_IMAGES"
    exit 1
fi

echo "✓ All naming conventions validated successfully"
```

---

## Support & References

- **Main Deployment Guide**: [PRODUCTION_DEPLOYMENT_GUIDE.md](./PRODUCTION_DEPLOYMENT_GUIDE.md)
- **Network Security**: [NETWORK_SECURITY.md](./NETWORK_SECURITY.md)
- **CLAUDE.md**: Project-wide conventions and standards

For issues or questions:
- GitHub Issues: https://github.com/nranjan2code/sutra-memory/issues

**Last Updated:** 2025-01-06

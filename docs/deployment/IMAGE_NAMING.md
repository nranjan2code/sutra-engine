# Docker Image Naming Convention

## Overview

This deployment uses the `sutra-works-` prefix for all Docker images to avoid conflicts with other Sutra deployments on the same system.

## Image Names

All images in this deployment are prefixed with `sutra-works-`:

### Core Services
- `sutra-works-storage-server:latest` - Storage engine (Rust)
- `sutra-works-ml-base:latest` - ML foundation library
- `sutra-works-ml-base-service:latest` - ML inference service
- `sutra-works-embedding-service:latest` - Embedding generation service
- `sutra-works-nlg-service:latest` - Natural language generation service

### Application Services
- `sutra-works-api:latest` - Main REST API (FastAPI)
- `sutra-works-hybrid:latest` - Semantic orchestration service
- `sutra-works-bulk-ingester:latest` - High-throughput data ingestion
- `sutra-works-control:latest` - React control center
- `sutra-works-client:latest` - Streamlit UI

## Container Names

Container names use the `sutra-` prefix (without "works") for consistency:

- `sutra-storage` - Main storage server
- `sutra-user-storage` - User data storage
- `sutra-ml-base` - ML base service
- `embedding-single` - Embedding service (simple edition)
- `nlg-single` - NLG service (simple edition)
- `sutra-api` - API service
- `sutra-hybrid` - Hybrid reasoning service
- `sutra-bulk-ingester` - Bulk ingestion service
- `sutra-control` - Control panel
- `sutra-client` - Client UI

## Why the Prefix?

The `sutra-works-` prefix was added to this specific deployment to:

1. **Avoid conflicts** - Prevents image name collisions with other Sutra deployments (e.g., `sutra_md`)
2. **Clear identification** - Makes it easy to identify which images belong to this deployment
3. **Isolation** - Allows multiple Sutra deployments on the same Docker host

## Network Configuration

All services run on the `sutra-works_sutra-network` Docker network, completely isolated from other deployments.

## Verification

To verify all images are using the correct prefix:

```bash
docker images | grep "sutra-works-"
```

To check which images containers are using:

```bash
docker ps --filter "name=sutra-" --format "{{.Names}}\t{{.Image}}"
```

## Build Configuration

The image prefix is configured in:
- `.sutra/compose/production.yml` - Docker Compose image definitions
- `sutra-optimize.sh` - Build script image tagging

## Standard Documentation

Note: Most documentation in this repository uses generic image names like `sutra-api` without the prefix. When following documentation examples:

- Replace `sutra-api` with `sutra-works-api`
- Replace `sutra-storage-server` with `sutra-works-storage-server`
- And so on for all services

Or simply use the provided `./sutra` CLI commands, which handle the correct naming automatically.

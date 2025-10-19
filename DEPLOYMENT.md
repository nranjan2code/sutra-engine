# Sutra Grid Deployment Guide

## Single Source of Truth

All deployment operations are managed through **`./sutra-deploy.sh`** - this is the only script you need.

## Quick Start

### First-Time Installation
```bash
./sutra-deploy.sh install
```

This will:
1. Check prerequisites (Docker, Docker Compose)
2. Build all Docker images
3. Start all services
4. Display service URLs

### Common Operations

```bash
# Start all services
./sutra-deploy.sh up

# Stop all services
./sutra-deploy.sh down

# Restart all services
./sutra-deploy.sh restart

# View system status
./sutra-deploy.sh status

# View logs (all services)
./sutra-deploy.sh logs

# View logs (specific service)
./sutra-deploy.sh logs sutra-api

# Interactive maintenance menu
./sutra-deploy.sh maintenance
```

## Service URLs

Once deployed, access services at:

- **Sutra Control Center**: http://localhost:9000
  - Dashboard: http://localhost:9000/
  - Grid Management: http://localhost:9000/grid
  - Bulk Ingester UI: http://localhost:9000/bulk-ingester ✅
- **Sutra Client (UI)**: http://localhost:8080
- **Sutra API**: http://localhost:8000
- **Sutra Hybrid API**: http://localhost:8001
- **Grid Master (HTTP)**: http://localhost:7001
- **Grid Master (gRPC)**: localhost:7002

## Architecture

### Services
- **Storage Layer**: Main storage (50051), Grid event storage (50052)
- **Grid Infrastructure**: Grid Master (7001/7002), Grid Agents (8003/8004)
- **API Layer**: Sutra API (8000), Sutra Hybrid (8001)
- **Web Interfaces**: Sutra Control (9000), Sutra Client (8080)

### Data Persistence
All data is stored in Docker volumes:
- `storage-data`: Main knowledge graph storage
- `grid-event-data`: Grid observability events
- `agent1-data` & `agent2-data`: Grid agent storage nodes

## Maintenance

### Interactive Maintenance Menu
```bash
./sutra-deploy.sh maintenance
```

Options include:
1. View system status
2. Check service health
3. Restart unhealthy services
4. View logs
5. Clean up unused resources
6. Backup data volumes

### Backup Data
```bash
./sutra-deploy.sh maintenance
# Select option 6
```

Backups are saved to `./backups/` directory.

### Complete Cleanup
```bash
./sutra-deploy.sh clean
```

**Warning**: This removes all containers, volumes, and images. Use with caution!

## Troubleshooting

### Services Not Starting
```bash
# Check logs
./sutra-deploy.sh logs

# Check specific service
./sutra-deploy.sh logs grid-master

# Restart all services
./sutra-deploy.sh restart
```

### Port Conflicts
If ports are already in use, modify `docker-compose-grid.yml` to use different ports.

### Build Errors
```bash
# Clean and rebuild
./sutra-deploy.sh clean
./sutra-deploy.sh install
```

## Development

### Building Specific Services
```bash
docker-compose -f docker-compose-grid.yml build sutra-api
```

### Accessing Container Shells
```bash
docker exec -it sutra-api /bin/sh
docker exec -it sutra-grid-master /bin/sh
```

## Migration from Old Scripts

**Deprecated scripts** (do not use):
- `build-images.sh` → Use `./sutra-deploy.sh build`
- `deploy-optimized.sh` → Use `./sutra-deploy.sh up`
- `deploy-docker-grid.sh` → Use `./sutra-deploy.sh install`
- `docker-compose.yml` → Use `docker-compose-grid.yml`
- `docker-compose-v2.yml` → Use `docker-compose-grid.yml`

These files are kept for reference but should not be used.

## Configuration Files

- **`docker-compose-grid.yml`**: Service definitions (do not edit directly)
- **`sutra-deploy.sh`**: Deployment script (single source of truth)
- **`DEPLOYMENT.md`**: This documentation

## Support

For issues or questions:
1. Check logs: `./sutra-deploy.sh logs`
2. Check status: `./sutra-deploy.sh status`
3. Run maintenance: `./sutra-deploy.sh maintenance`
4. Review documentation: `WARP.md` in project root

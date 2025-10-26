# Sutra Grid Deployment Guide

## Single Command Center: `sutra-deploy.sh`

**This is the ONLY script you need for all deployment operations.**

All other build/deploy scripts are deprecated and will be removed.

## Quick Start

```bash
# Clean install (recommended)
./sutra-deploy.sh clean    # Remove everything
./sutra-deploy.sh install  # Build and start fresh

# Or one-step (will prompt to clean if needed)
./sutra-deploy.sh install
```

## Available Commands

### Core Operations
- `install` - Complete first-time setup (build + start)
- `up` - Start all services (auto-builds if needed)
- `down` - Stop all services gracefully
- `restart` - Restart all services
- `clean` - Complete system reset (removes all containers, volumes, images)

### Monitoring
- `status` - Show service status and URLs
- `validate` - Run comprehensive health checks
- `logs [service]` - View logs (all or specific service)

### Maintenance
- `build` - Rebuild Docker images only
- `maintenance` - Interactive maintenance menu

## Common Workflows

### First Time Setup
```bash
./sutra-deploy.sh install
```

### Daily Development
```bash
# Start system
./sutra-deploy.sh up

# Check status
./sutra-deploy.sh status

# View logs
./sutra-deploy.sh logs sutra-hybrid

# Stop when done
./sutra-deploy.sh down
```

### After Code Changes
```bash
# Rebuild and restart
./sutra-deploy.sh build
./sutra-deploy.sh restart
```

### Complete Reset
```bash
# Clean everything and reinstall
./sutra-deploy.sh clean
./sutra-deploy.sh install
```

### Troubleshooting
```bash
# Full health check
./sutra-deploy.sh validate

# Check specific service logs
./sutra-deploy.sh logs sutra-storage

# Interactive maintenance menu
./sutra-deploy.sh maintenance
```

## System Features

### Self-Healing
- Auto-fixes common HAProxy configuration issues
- Auto-builds images if missing during `up`
- Validates prerequisites before operations

### Idempotent Operations
- Safe to run commands multiple times
- State-aware (knows if system is CLEAN/BUILT/STOPPED/RUNNING)
- No side effects from repeated execution

### Production-Grade
- Fail-fast validation
- Graceful shutdown with timeouts
- Comprehensive error handling
- Clear observability

## Clean Single-Path Deployment

**`sutra-deploy.sh`** is the ONLY deployment script in this project.

All redundant build/deploy scripts have been removed to eliminate confusion.

## Architecture

The command center handles the critical HA embedding service properly:
1. Builds embedding service image **once**
2. Uses that single image for 3 HA replicas
3. HAProxy load balances across replicas
4. Auto-validates configuration

## Access Points (After Deployment)

- Control Center: http://localhost:9000
- Client UI: http://localhost:8080
- API: http://localhost:8000
- Hybrid API: http://localhost:8001
- Embedding Service: http://localhost:8888
- Grid Master: http://localhost:7001

## Debugging

Enable debug output:
```bash
DEBUG=1 ./sutra-deploy.sh status
```

View detailed logs:
```bash
./sutra-deploy.sh logs         # All services
./sutra-deploy.sh logs sutra-api  # Specific service
```

## Support

If something goes wrong:
1. Run `./sutra-deploy.sh validate` to see what's failing
2. Check logs with `./sutra-deploy.sh logs [service]`
3. Try `./sutra-deploy.sh restart`
4. Last resort: `./sutra-deploy.sh clean && ./sutra-deploy.sh install`

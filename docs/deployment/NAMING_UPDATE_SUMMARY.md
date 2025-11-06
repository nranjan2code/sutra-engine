# Docker Naming Update Summary

**Date:** 2025-11-06
**Change:** Added `sutra-works-` prefix to all Docker images

## Overview

Updated this deployment to use the `sutra-works-` prefix for all Docker images to avoid conflicts with other Sutra deployments (specifically `sutra_md`) on the same Docker host.

## Files Modified

### Build Configuration
1. **`.sutra/compose/production.yml`**
   - Changed all `image:` directives from `sutra-*` to `sutra-works-*`
   - Fixed double-prefix issue (`sutra-works-works-storage-server`)
   - Added `profiles: [explorer]` to optional explorer services

2. **`sutra-optimize.sh`**
   - Updated all build tag commands to use `sutra-works-` prefix
   - Modified image name mappings in display functions
   - Updated size check functions
   - Changed validation and test functions

### Documentation Updates
3. **`CLAUDE.md`**
   - Added prominent note about image naming at the top
   - Reference to detailed naming documentation

4. **`README.md`**
   - Added note about `sutra-works-` prefix
   - Reference to detailed naming documentation

5. **`docs/deployment/README.md`**
   - Added link to IMAGE_NAMING.md as first item in contents
   - Added important note at top

6. **`docs/deployment/IMAGE_NAMING.md`** ⭐ **New File**
   - Complete explanation of naming convention
   - Lists all image names with prefix
   - Lists all container names
   - Explains why the prefix is needed
   - Provides verification commands
   - Notes for users following generic documentation

## Image Names (Before → After)

| Before | After |
|--------|-------|
| `sutra-storage-server:latest` | `sutra-works-storage-server:latest` |
| `sutra-ml-base:latest` | `sutra-works-ml-base:latest` |
| `sutra-ml-base-service:latest` | `sutra-works-ml-base-service:latest` |
| `sutra-embedding-service:latest` | `sutra-works-embedding-service:latest` |
| `sutra-nlg-service:latest` | `sutra-works-nlg-service:latest` |
| `sutra-api:latest` | `sutra-works-api:latest` |
| `sutra-hybrid:latest` | `sutra-works-hybrid:latest` |
| `sutra-bulk-ingester:latest` | `sutra-works-bulk-ingester:latest` |
| `sutra-control:latest` | `sutra-works-control:latest` |
| `sutra-client:latest` | `sutra-works-client:latest` |

## Container Names (Unchanged)

Container names remain the same for consistency:
- `sutra-storage`
- `sutra-user-storage`
- `sutra-ml-base`
- `embedding-single`
- `nlg-single`
- `sutra-api`
- `sutra-hybrid`
- `sutra-bulk-ingester`
- `sutra-control`
- `sutra-client`

## Verification

All services deployed and healthy:

```bash
docker ps --filter "name=sutra-" --format "{{.Names}}\t{{.Image}}"
```

Expected output:
```
sutra-bulk-ingester     sutra-works-bulk-ingester:latest
sutra-control           sutra-works-control:latest
sutra-client            sutra-works-client:latest
sutra-hybrid            sutra-works-hybrid:latest
sutra-api               sutra-works-api:latest
sutra-storage           sutra-works-storage-server:latest
sutra-user-storage      sutra-works-storage-server:latest
embedding-single        sutra-works-embedding-service:latest
nlg-single              sutra-works-nlg-service:latest
sutra-ml-base           sutra-works-ml-base-service:latest
```

## Benefits

1. **No Conflicts** - Can run alongside `sutra_md` or other Sutra deployments
2. **Clear Identification** - Easy to see which images belong to this deployment
3. **Isolation** - Complete separation on the same Docker host
4. **Network Isolation** - Uses `sutra-works_sutra-network`

## Impact on Users

- Users should use the `./sutra` CLI commands which handle naming automatically
- When following generic documentation, users should mentally map:
  - `sutra-api` → `sutra-works-api`
  - `sutra-storage-server` → `sutra-works-storage-server`
  - etc.

## Build & Deploy Commands

All commands remain the same:

```bash
# Build with new naming
SUTRA_EDITION=simple ./sutra build

# Deploy with new naming
SUTRA_EDITION=simple ./sutra deploy

# Check status
docker ps --filter "name=sutra-"
```

The build and deploy scripts automatically handle the `sutra-works-` prefix.

## Errors Fixed

1. **Logger import error** in `sutra-api/main.py`
2. **Security middleware headers error** in `security_middleware.py`
3. **Explorer services profile configuration** in `production.yml`
4. **Double-prefix error** (`sutra-works-works-storage-server`)

## Current Status

✅ All 10 services healthy and running
✅ All using `sutra-works-*` images
✅ No conflicts with `sutra_md` deployment
✅ Documentation updated
✅ Build system updated

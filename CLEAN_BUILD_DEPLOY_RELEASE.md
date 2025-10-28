# Clean Build/Deploy/Release System - Quick Reference

**Updated: 2025-10-28**

## Overview

The Sutra AI build, deploy, and release system has been streamlined for clarity with:
- **Single-tag strategy** (`:latest` only, no intermediate tags)
- **Edition-based deployment** (simple, community, enterprise)
- **Professional release management** (semantic versioning)
- **Organized documentation** (clear user journeys)

## Quick Commands

### Build

```bash
# Build all services
SUTRA_EDITION=simple ./sutra-optimize.sh build-all      # 8 services (4.4GB)
SUTRA_EDITION=enterprise ./sutra-optimize.sh build-all  # 10 services (4.76GB)

# Check what was built
./sutra-optimize.sh sizes

# Build individual service
SUTRA_EDITION=simple ./sutra-optimize.sh build-service storage
```

### Deploy

```bash
# Deploy by edition
SUTRA_EDITION=simple ./sutra deploy        # Default, 8 services
SUTRA_EDITION=community ./sutra deploy     # HA configuration
SUTRA_EDITION=enterprise ./sutra deploy    # Grid-enabled

# Check status
./sutra status
docker compose ps
```

### Test

```bash
# Full integration tests
PYTHONPATH=packages/sutra-core python -m pytest tests/ -v

# Production smoke test (validates embeddings)
./scripts/smoke-test-embeddings.sh

# Rust storage tests
cd packages/sutra-storage && cargo test
```

### Release

```bash
# Check version
./sutra-deploy.sh version                  # Shows 2.0.0

# Create release
./sutra-deploy.sh release patch           # Bug fix (2.0.0 → 2.0.1)
./sutra-deploy.sh release minor           # Feature (2.0.0 → 2.1.0)
./sutra-deploy.sh release major           # Breaking (2.0.0 → 3.0.0)

# Push & deploy
git push origin main --tags
./sutra-deploy.sh deploy v2.0.1
```

## Image Tagging Strategy

**Current (Single-Tag):**
- All services: `sutra-<service>:${SUTRA_VERSION:-latest}`
- No intermediate or temporary tags
- Compose file uses: `${SUTRA_VERSION:-latest}`

**Legacy (Removed):**
- ~~`:latest-optimized`~~ - No longer used
- ~~Dual-build strategy~~ - Removed
- ~~Intermediate tags~~ - Eliminated

## Service Breakdown

### Simple Edition (8 services, 4.4GB)
1. sutra-storage (Rust 1.87-slim)
2. sutra-api (Python 3.11-slim)
3. sutra-hybrid (Python 3.11-slim)
4. sutra-embedding-service (Python 3.11-slim)
5. sutra-nlg (Python 3.11-slim)
6. sutra-bulk-ingester (Rust 1.87-slim)
7. sutra-client (nginx 1.26-alpine)
8. sutra-control (nginx 1.26-alpine)

### Enterprise Edition (+2 services, 4.76GB total)
9. sutra-grid-master (Rust 1.82-slim)
10. sutra-grid-agent (Rust 1.82-slim)

## Documentation Structure

```
docs/
├── README.md                    # Main hub with navigation
├── getting-started/             # User onboarding
│   ├── README.md               # Start here
│   ├── quickstart.md           # 5-minute setup
│   ├── editions.md             # Edition comparison
│   └── tutorial.md             # Complete walkthrough
├── build/                       # Building services
│   ├── README.md               # Build documentation hub
│   └── building-services.md    # Detailed build guide
├── deployment/                  # Deploying services
│   ├── README.md               # Complete deployment guide
│   ├── docker-compose.md       # Compose details
│   └── editions/               # Edition-specific configs
├── release/                     # Release management
│   ├── README.md               # Release guide
│   └── RELEASE_PROCESS.md      # Complete workflow
└── architecture/                # Technical deep dives
```

## User Journeys

1. **New Users**: `docs/getting-started/README.md` → quickstart.md → tutorial.md
2. **Developers**: `docs/build/README.md` → building-services.md → deployment/README.md
3. **DevOps**: `docs/deployment/README.md` → release/README.md
4. **Contributors**: `docs/guides/` → architecture/

## VS Code Tasks

Access via Command Palette (Cmd+Shift+P → "Tasks: Run Task"):

**Build Tasks:**
- Build: All Services (Simple)
- Build: All Services (Enterprise)
- Build: Storage Service
- Build: Check Sizes

**Deploy Tasks:**
- Deploy: Simple Edition
- Deploy: Community Edition
- Deploy: Enterprise Edition

**Status Tasks:**
- Status: Check Deployment
- Status: Docker Compose

**Test Tasks:**
- Test: Integration Tests
- Test: Smoke Test (Embeddings)
- Test: Storage (Rust)

**Release Tasks:**
- Release: Check Version
- Release: Patch (Bug Fix)
- Release: Minor (Feature)
- Release: Major (Breaking)

**Log Tasks:**
- Logs: View Storage
- Logs: View API
- Logs: View Hybrid

**Docker Tasks:**
- Docker: Stop All
- Docker: Clean Volumes

## Key Files Updated

1. **WARP.md**
   - Updated "Build, Deploy & Release System" section
   - Added "Build, Deploy & Test Workflows"
   - Single-tag strategy documented

2. **.github/copilot-instructions.md**
   - Updated "Essential Workflows"
   - Added "Documentation Structure"
   - Updated "Critical File Locations"

3. **.vscode/tasks.json** (NEW)
   - 21 tasks for all workflows
   - Organized by category
   - One-click build/deploy/test/release

## Next Steps

1. ✅ Build system clean (single :latest tag)
2. ✅ Documentation reorganized (user journey structure)
3. ✅ Instruction files updated (WARP.md, copilot, tasks)
4. ⏳ Validate community edition deployment
5. ⏳ Validate enterprise edition deployment

## References

- Build System: `./sutra-optimize.sh` (929 lines)
- Deployment: `./sutra-deploy.sh` (1100+ lines)
- Version: `VERSION` file (2.0.0)
- Compose: `.sutra/compose/production.yml`
- Docs Hub: `docs/README.md`

---

**Result: NO CONFUSION - Clear paths for build, deploy, and release!**

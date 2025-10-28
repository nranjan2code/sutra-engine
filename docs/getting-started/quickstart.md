# Sutra AI - Quick Start

**Domain-Specific Reasoning Engine for Your Knowledge**

## ⚠️ Choose Your Mode

**Sutra has TWO deployment modes:**

### 🔧 Development Mode (Default - NO Security)

```bash
./sutra-deploy.sh clean
./sutra-deploy.sh install
```

**Use for:** Local development, testing, learning  
**Security:** ❌ NO authentication, NO encryption  
**⚠️ WARNING:** Only use on localhost, never with real data

### 🔒 Production Mode (Secure)

```bash
# Generate secrets (one-time)
chmod +x scripts/generate-secrets.sh
./scripts/generate-secrets.sh

# Deploy securely
SUTRA_SECURE_MODE=true ./sutra-deploy.sh install
```

**Use for:** Production, real data, regulated industries  
**Security:** ✅ HMAC/JWT + TLS 1.3 + RBAC + Network isolation  
**See:** `docs/security/QUICK_START_SECURITY.md` for complete setup

---

## 🚀 Quick Deploy (Development)

This deploys **without security** for local development:

```bash
./sutra-deploy.sh clean
./sutra-deploy.sh install
```

The system will:
1. ✅ Build all Docker images (handles HA properly)
2. ✅ Start all 13 services (storage, reasoning, embeddings)
3. ✅ Validate critical components
4. ✅ Show access URLs

**Note:** Sutra starts empty. You provide the domain knowledge (protocols, cases, procedures), Sutra provides the explainable reasoning.

## 📊 Access Your System

- **Control Center**: http://localhost:9000
- **Client UI**: http://localhost:8080
- **API**: http://localhost:8000

## 🎯 Common Commands

```bash
./sutra-deploy.sh status      # Check what's running
./sutra-deploy.sh validate    # Full health check
./sutra-deploy.sh logs        # View all logs
./sutra-deploy.sh restart     # Restart services
./sutra-deploy.sh down        # Stop everything

# 🚀 NEW: Fast development workflow
./sutra-deploy.sh update sutra-api    # Update single service (30s!)
./scripts/detect-changes.sh           # See what changed
```

## � Development Mode (Hot Reload - NEW!)

**Want instant code changes without rebuilds?**

```bash
# Start dev mode with hot-reload
docker-compose -f docker-compose-grid.yml -f docker-compose.dev.yml up

# Now edit Python/React code → changes apply automatically!
# No docker rebuild needed!
```

**Benefits:**
- ✅ Python changes: Instant reload
- ✅ React changes: Browser auto-refresh
- ✅ 10x faster development cycle

## �📖 Full Documentation

- **[FAST_DEVELOPMENT.md](../FAST_DEVELOPMENT.md)** - **NEW: Quick development guide**
- **[QUICK_REFERENCE.txt](../QUICK_REFERENCE.txt)** - **NEW: Cheat sheet**
- **[DEPLOYMENT.md](DEPLOYMENT.md)** - Complete deployment guide
- **[WARP.md](WARP.md)** - Architecture & development guide

## ⚠️ Important

**Only use `./sutra-deploy.sh`** - it's the single command center for all deployment operations.

All redundant scripts have been removed.

## 🆘 Troubleshooting

System not working? Try this:

```bash
./sutra-deploy.sh clean     # Complete reset
./sutra-deploy.sh install   # Fresh install
./sutra-deploy.sh validate  # Check health
```

Still stuck? Check the logs:
```bash
./sutra-deploy.sh logs sutra-hybrid
```

# Sutra AI - Quick Start

**Domain-Specific Reasoning Engine for Your Knowledge**

## 🚀 Deploy in 2 Commands

```bash
./sutra-deploy.sh clean
./sutra-deploy.sh install
```

That's it! The system will:
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
```

## 📖 Full Documentation

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

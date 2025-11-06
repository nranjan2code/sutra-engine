# Quick Start: Secure Deployment

**Single-Path Security Deployment for Sutra Models**

---

## 🚀 Quick Start (Production)

### First-Time Installation with Security

```bash
# One command - does everything
SUTRA_SECURE_MODE=true sutra deploy
```

This will:
1. ✅ Auto-generate secrets (if not present)
2. ✅ Build all Docker images
3. ✅ Deploy with `docker-compose-secure.yml`
4. ✅ Enable authentication on all services
5. ✅ Configure network segregation
6. ✅ Start with TLS encryption

### Enable Security on Existing Deployment

```bash
# Switch to secure mode
SUTRA_SECURE_MODE=true sutra restart
```

---

## 🛠️ Common Commands

### Development Mode (Default - No Authentication)

```bash
# Start without security (local dev only)
sutra deploy
sutra start
sutra stop
sutra status
```

### Production Mode (With Authentication)

```bash
# Start with security enabled
SUTRA_SECURE_MODE=true sutra deploy

# Status check
SUTRA_SECURE_MODE=true sutra status

# Restart services
SUTRA_SECURE_MODE=true sutra restart

# Stop services
SUTRA_SECURE_MODE=true sutra stop

# Full validation
SUTRA_SECURE_MODE=true sutra validate
```

---

## 🔑 Using Authentication Tokens

### Service-to-Service Authentication

```bash
# Get service token
SERVICE_TOKEN=$(cat .secrets/tokens/service_token.txt)

# Use in API calls
curl -H "Authorization: Bearer $SERVICE_TOKEN" \
  http://localhost:8000/health
```

### User Authentication

```bash
# Get admin token
ADMIN_TOKEN=$(cat .secrets/tokens/admin_token.txt)

# Admin API call
curl -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"text": "Everest is the tallest mountain"}' \
  http://localhost:8000/learn
```

### Available Token Files

After running secure deployment, tokens are located in `.secrets/tokens/`:

```
.secrets/tokens/
├── admin_token.txt      # Admin role (full access)
├── writer_token.txt     # Writer role (read + write)
├── reader_token.txt     # Reader role (read only)
└── service_token.txt    # Service role (internal communication)
```

---

## 🔍 Verification

### Check Security Mode Status

```bash
# The header will show security status
sutra status

# Output shows:
# ╔═══════════════════════════════════════════════════════════════╗
# ║       Sutra Grid Command Center v2.0 (Production)            ║
# ║             🔒 SECURITY MODE ENABLED 🔒                      ║
# ╚═══════════════════════════════════════════════════════════════╝
```

### Test Authentication

```bash
# Should FAIL without token (403 Forbidden)
curl http://localhost:8000/stats

# Should SUCCEED with token (200 OK)
TOKEN=$(cat .secrets/tokens/service_token.txt)
curl -H "Authorization: Bearer $TOKEN" http://localhost:8000/stats
```

### Validate Network Segregation

```bash
# Internal services should NOT be accessible from host
curl http://localhost:50051/health  # Should FAIL (connection refused)

# Public services require authentication
curl http://localhost:8000/health   # Should FAIL (403 Forbidden)

# With token should work
TOKEN=$(cat .secrets/tokens/service_token.txt)
curl -H "Authorization: Bearer $TOKEN" http://localhost:8000/health  # Should SUCCEED
```

---

## 🎯 Architecture Overview

### Security Mode Enabled

```
┌─────────────────────────────────────────────────────────────┐
│                   Public Network (172.21.0.0/24)            │
│                  (Authenticated Access Only)                │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐                  │
│  │   sutra-api     │  │  sutra-hybrid   │                  │
│  │   Port: 8000    │  │   Port: 8001    │                  │
│  │   Auth: User    │  │   Auth: User    │                  │
│  └─────────────────┘  └─────────────────┘                  │
│  ┌─────────────────┐  ┌─────────────────┐                  │
│  │ sutra-control   │  │  sutra-client   │                  │
│  │   Port: 9000    │  │   Port: 8080    │                  │
│  │   Auth: Admin   │  │   Auth: Pass    │                  │
│  └─────────────────┘  └─────────────────┘                  │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                  Internal Network (172.20.0.0/24)           │
│                 (NO External Access - Isolated)             │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────────┐  ┌─────────────────────────────────┐ │
│  │ storage-server   │  │   embedding-ha (HAProxy)        │ │
│  │  Port: 50051     │  │      Port: 8888                 │ │
│  │  Auth + TLS      │  │   ┌─────────────────────────┐   │ │
│  └──────────────────┘  │   │ embedding-1, 2, 3       │   │ │
│                        │   │ (3 replicas)            │   │ │
│  ┌──────────────────┐  │   └─────────────────────────┘   │ │
│  │  grid-master     │  └─────────────────────────────────┘ │
│  │  Ports: 7001/2   │                                      │
│  │  Auth            │  ┌─────────────────────────────────┐ │
│  └──────────────────┘  │  grid-agents (port 8001)        │ │
│                        │  Auth                           │ │
│  ┌──────────────────┐  └─────────────────────────────────┘ │
│  │ bulk-ingester    │                                      │
│  │  Port: 8005      │                                      │
│  │  Auth            │                                      │
│  └──────────────────┘                                      │
└─────────────────────────────────────────────────────────────┘
```

### Development Mode (Default)

```
┌─────────────────────────────────────────────────────────────┐
│              Single Network (sutra-network)                 │
│                     (No Authentication)                     │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  All services accessible from host without authentication  │
│  ⚠️  For local development only - NOT for production ⚠️     │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 📚 Additional Documentation

- **Setup Guide**: `docs/security/PRODUCTION_SECURITY_SETUP.md`
- **Architecture**: `docs/security/SECURE_ARCHITECTURE.md`
- **Implementation**: `docs/security/SECURITY_IMPLEMENTATION_COMPLETE.md`
- **Audit Report**: `docs/security/SECURITY_AUDIT_REPORT.md`
- **Main Guide**: `docs/security/README.md`

---

## ⚠️ Important Notes

1. **Never commit secrets** - The `.secrets/` directory is in `.gitignore`
2. **Rotate secrets regularly** - Regenerate with `./scripts/generate-secrets.sh`
3. **Development vs Production**:
   - Development: `sutra` (no auth)
   - Production: `SUTRA_SECURE_MODE=true sutra` (full security)
4. **Single-Path Philosophy**: Always use `./sutra unified CLI` - never call `docker-compose` directly

---

**Last Updated**: 2025-10-25  
**Version**: 2.0 (Security Mode Support)

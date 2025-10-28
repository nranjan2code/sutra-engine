# Sutra AI Quick Start - Edition Guide

**Choose your edition in 5 minutes.**

## Decision Tree

```
How many concepts will you have?
  
  < 100K concepts
  ↓
  Are you just testing/developing?
    Yes → Simple Edition (FREE) ✅
    No → Community Edition ($99/mo)
  
  100K - 1M concepts
  ↓
  Community Edition ($99/mo) ✅
  
  > 1M concepts OR need 99.9% SLA
  ↓
  Enterprise Edition ($999/mo) ✅
```

## Quick Install

### Simple Edition (FREE)

```bash
# Clone repo
git clone https://github.com/yourusername/sutra-models
cd sutra-models

# Install
./sutra-deploy.sh install

# Access
# → Client UI: http://localhost:8080
# → Control Center: http://localhost:9000
```

**You're done! No license needed.**

### Community Edition ($99/mo)

```bash
# 1. Get license from https://sutra.ai/pricing
# 2. Set environment
export SUTRA_EDITION="community"
export SUTRA_LICENSE_KEY="your-license-key-here"

# 3. Install (same command as Simple)
./sutra-deploy.sh install

# Access (same URLs as Simple)
```

**Same 7 containers as Simple, 10× higher limits.**

### Enterprise Edition ($999/mo)

```bash
# 1. Get license from https://sutra.ai/pricing
# 2. Generate secrets (one-time)
export SUTRA_LICENSE_SECRET="$(openssl rand -hex 32)"
export SUTRA_EDITION="enterprise"
export SUTRA_LICENSE_KEY="your-enterprise-license-key"
export SUTRA_SECURE_MODE="true"

# 3. Install
./sutra-deploy.sh install

# Access
# → Control Center: http://localhost:9000
# → HAProxy Stats (Embedding): http://localhost:8404/stats
# → HAProxy Stats (NLG): http://localhost:8405/stats
# → Grid Master: http://localhost:7001
```

**16 containers with HA + Grid.**

## What You Get

| Feature | Simple | Community | Enterprise |
|---------|--------|-----------|------------|
| **Deploy Time** | 60 sec | 60 sec | 3 min |
| **Containers** | 7 | 7 | 16 |
| **Memory** | 4GB | 8GB | 20GB |
| **License** | None | Required | Required |
| **Learn API** | 10/min | 100/min | 1000/min |
| **Max Concepts** | 100K | 1M | 10M |
| **Support** | Community | Email (48h) | Dedicated (4h) |
| **SLA** | None | Best effort | 99.9% |

## Verify Installation

```bash
# 1. Check deployment status
./sutra-deploy.sh status

# 2. Check edition and limits (NEW!)
curl http://localhost:8000/edition | jq

# Expected output:
# {
#   "edition": "simple",  # or "community"/"enterprise"
#   "limits": {
#     "learn_per_min": 10,
#     "reason_per_min": 50,
#     "max_concepts": 100000,
#     ...
#   },
#   "features": { "ha_enabled": false, ... },
#   "license_valid": true
# }

# 3. Test learning
curl -X POST http://localhost:8000/learn \
  -H "Content-Type: application/json" \
  -d '{"content":"Paris is the capital of France"}'

# 4. Test stats
curl http://localhost:8000/stats | jq
```

**The `/edition` endpoint is your single source of truth for:**
- Current edition and license status
- Rate limits and quotas
- Available features (HA, Grid, etc.)
- License expiration dates

### Visual Verification in UI

**Both UIs display edition information automatically:**

**Control Center (http://localhost:9000):**
```bash
open http://localhost:9000
# Look for edition badge in top-right header
# Hover to see tooltip with:
#  • Rate limits
#  • Capacity
#  • Features
#  • License status
```

**Client UI (http://localhost:8080):**
```bash
open http://localhost:8080
# Look for edition badge next to "Sutra AI" logo
# Hover for detailed information
```

**Edition Colors:**
- Simple: Gray gradient
- Community: Blue gradient
- Enterprise: Gold gradient

## Upgrade Path

### Simple → Community

```bash
# 1. Stop services
./sutra-deploy.sh down

# 2. Get license from https://sutra.ai/pricing

# 3. Set edition
export SUTRA_EDITION="community"
export SUTRA_LICENSE_KEY="your-license-key"

# 4. Restart (data preserved!)
./sutra-deploy.sh install
```

**No data loss. Same 7 containers, 10× limits.**

### Community → Enterprise

```bash
# 1. Stop services
./sutra-deploy.sh down

# 2. Get enterprise license from https://sutra.ai/pricing

# 3. Set edition + security
export SUTRA_EDITION="enterprise"
export SUTRA_LICENSE_KEY="your-enterprise-license"
export SUTRA_SECURE_MODE="true"

# 4. Restart (data preserved!)
./sutra-deploy.sh install
```

**No data loss. Upgrades to 16 containers with HA + Grid.**

## Common Issues

### License Validation Failed

```bash
ERROR: COMMUNITY edition requires SUTRA_LICENSE_KEY
```

**Solution:** Set license key environment variable before install:
```bash
export SUTRA_LICENSE_KEY="your-license-key"
./sutra-deploy.sh install
```

### License Expired

```bash
ERROR: License expired on 2025-12-31
```

**Solution:** Renew license at https://sutra.ai/pricing

### Out of Memory

```bash
# Simple/Community: Need at least 8GB RAM
# Enterprise: Need at least 20GB RAM
```

**Solution:** Upgrade server or use lower edition.

### Port Already in Use

```bash
ERROR: Port 8080 already in use
```

**Solution:** Stop conflicting service or change ports in docker-compose-grid.yml

## Next Steps

1. **Read full docs:** `docs/EDITIONS.md`
2. **Try examples:** `python demo_simple.py`
3. **Bulk ingestion:** See `docs/BULK_INGESTER_ARCHITECTURE.md`
4. **Custom adapters:** See `docs/adapters/`
5. **Security setup:** See `docs/security/PRODUCTION_SECURITY_SETUP.md` (Enterprise only)

## Get Help

- **Simple Edition:** Community forums (https://community.sutra.ai)
- **Community Edition:** support@sutra.ai (48-hour response)
- **Enterprise Edition:** Dedicated support portal (4-hour response)

## Pricing

- **Simple:** FREE forever
- **Community:** $99/month (or $990/year - 2 months free)
- **Enterprise:** $999/month (or custom annual contract)

**Get license:** https://sutra.ai/pricing  
**Questions:** sales@sutra.ai

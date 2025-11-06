# Sutra AI Editions

**Philosophy: All features in all editions. Differentiation = scale + SLA, not functionality.**

## Quick Comparison

| Feature | Simple (FREE) | Community ($99/mo) | Enterprise ($999/mo) |
|---------|---------------|-------------------|----------------------|
| **Deployment** | | | |
| Containers | 7 | 7 | 16 |
| Memory | ~4GB | ~8GB | ~20GB |
| Topology | Single-node | Single-node | **HA + Grid** |
| | | | |
| **Core Features** | ✅ ALL | ✅ ALL | ✅ ALL |
| Graph reasoning | ✅ | ✅ | ✅ |
| Semantic embeddings | ✅ | ✅ | ✅ |
| Semantic reasoning | ✅ | ✅ | ✅ |
| NLG (template + LLM) | ✅ | ✅ | ✅ |
| Control Center | ✅ | ✅ | ✅ |
| Bulk ingestion | ✅ | ✅ | ✅ |
| Custom adapters | ✅ | ✅ | ✅ |
| | | | |
| **Scale Limits** | | | |
| Learn API | 10/min | 100/min | **1000/min** |
| Reason API | 50/min | 500/min | **5000/min** |
| Max concepts | 100K | 1M | **10M** |
| Max dataset size | 1GB | 10GB | **Unlimited** |
| Ingest workers | 2 | 4 | **16** |
| Audit retention | 7 days | 30 days | **365 days** |
| | | | |
| **High Availability** | ❌ | ❌ | **✅** |
| Embedding replicas | 1 | 1 | **3 + HAProxy** |
| NLG replicas | 1 | 1 | **3 + HAProxy** |
| Storage shards | 1 | 1 | **4** |
| Zero-downtime deploys | ❌ | ❌ | **✅** |
| | | | |
| **Grid Features** | ❌ | ❌ | **✅** |
| Grid orchestration | ❌ | ❌ | **✅** |
| Distributed processing | ❌ | ❌ | **✅** |
| Event observability | ❌ | ❌ | **✅** |
| Multi-node support | ❌ | ❌ | **✅** |
| | | | |
| **Security** | | | |
| TLS + Authentication | Optional | Optional | **Mandatory** |
| Network isolation | ❌ | ❌ | **✅** |
| Secret management | ❌ | ❌ | **✅** |
| | | | |
| **Support & SLA** | | | |
| Support channel | Community | Email | **Dedicated** |
| Response time | N/A | 48 hours | **4 hours** |
| SLA uptime | None | Best effort | **99.9%** |
| Professional services | ❌ | ❌ | **✅** |

## Simple Edition (FREE)

**Perfect for: Development, testing, demos, learning**

```bash
sutra deploy

# 7 containers deployed:
# - storage-server
# - sutra-api
# - sutra-hybrid  
# - sutra-control
# - sutra-client
# - embedding-single
# - nlg-single
```

**Limits:**
- 10 learn requests/min
- 50 reason requests/min
- 100,000 max concepts
- 1GB max dataset size
- 2 concurrent ingestion workers
- 7 days audit log retention

**No license key required.**

## Community Edition ($99/mo)

**Perfect for: Small teams, MVPs, startups, <1M concepts**

```bash
# Get license from https://sutra.ai/pricing
export SUTRA_EDITION="community"
export SUTRA_LICENSE_KEY="your-license-key-here"
sutra deploy

# Same 7 containers as Simple, but with higher limits
```

**Limits:**
- 100 learn requests/min (10× Simple)
- 500 reason requests/min (10× Simple)
- 1,000,000 max concepts (10× Simple)
- 10GB max dataset size (10× Simple)
- 4 concurrent ingestion workers (2× Simple)
- 30 days audit log retention

**Includes:**
- Email support (48-hour response)
- All features unlocked

**Same single-node topology as Simple - upgrade is just higher quotas.**

## Enterprise Edition ($999/mo)

**Perfect for: Production, regulated industries, mission-critical deployments, >1M concepts**

```bash
# Get license from https://sutra.ai/pricing
export SUTRA_EDITION="enterprise"
export SUTRA_LICENSE_KEY="your-license-key-here"
export SUTRA_SECURE_MODE="true"  # Required for enterprise
sutra deploy

# 16 containers deployed:
# - 7 core services (same as Simple/Community)
# - 3× embedding replicas + HAProxy
# - 3× NLG replicas + HAProxy  
# - Grid master + 2 agents + event storage
```

**Limits:**
- 1000 learn requests/min (100× Simple)
- 5000 reason requests/min (100× Simple)
- 10,000,000 max concepts (100× Simple)
- Unlimited dataset size
- 16 concurrent ingestion workers (8× Simple)
- 365 days audit log retention

**High Availability:**
- 3× embedding service replicas with load balancing
- 3× NLG service replicas with load balancing
- Automatic failover (<3s detection)
- Zero-downtime deployments
- 99.9% SLA uptime guarantee

**Grid Features:**
- Distributed processing across nodes
- Event-driven observability
- Multi-node orchestration
- Advanced monitoring & alerting

**Security:**
- TLS encryption (mandatory)
- Authentication & authorization
- Network isolation
- Secret management
- Audit logging (365 days)

**Support:**
- Dedicated support team
- 4-hour response SLA
- Professional services available
- Direct engineering escalation

## Upgrade Path

All editions use the same Docker volumes for data storage. **Upgrades are seamless:**

```bash
# Simple → Community (no data loss)
sutra stop
export SUTRA_EDITION="community"
export SUTRA_LICENSE_KEY="your-license-key"
sutra deploy

# Community → Enterprise (no data loss)
sutra stop
export SUTRA_EDITION="enterprise"
export SUTRA_LICENSE_KEY="your-enterprise-license-key"
export SUTRA_SECURE_MODE="true"
sutra deploy
```

**Your data persists across edition changes.**

## License Management

### Obtaining a License

1. **Simple Edition:** No license required
2. **Community Edition:** https://sutra.ai/pricing → $99/mo subscription
3. **Enterprise Edition:** https://sutra.ai/pricing → Custom quote

### License Format

Licenses are HMAC-signed tokens containing:
- Edition (community/enterprise)
- Customer ID
- Issued date
- Expiration date (if applicable)

Example:
```
eyJlZGl0aW9uIjogImNvbW11bml0eSIsICJjdXN0b21lcl9pZCI6ICJBY21lIENvcnAiLCAiaXNzdWVkIjogIjIwMjUtMDEtMDFUMTI6MDA6MDBaIiwgImV4cGlyZXMiOiAiMjAyNi0wMS0wMVQxMjowMDowMFoifQ==.a7f3e9d2c1b4f6e8d3a2c5b7f9e1d4c6a8b2f5e7d3c9a1b4f6e8d2c5b7f9e1d4
```

### Checking Edition via API

**All services expose an `/edition` endpoint to query current limits and features:**

```bash
# Check edition information
curl http://localhost:8000/edition
```

**Response format:**
```json
{
  "edition": "community",
  "limits": {
    "learn_per_min": 100,
    "reason_per_min": 500,
    "max_concepts": 1000000,
    "max_dataset_gb": 10,
    "ingest_workers": 4
  },
  "features": {
    "ha_enabled": false,
    "grid_enabled": false,
    "observability_enabled": false,
    "multi_node": false
  },
  "license_valid": true,
  "license_expires": "2026-01-01T12:00:00",
  "upgrade_url": "https://sutra.ai/pricing"
}
```

**Use this endpoint to:**
- Display current limits in your UI
- Implement client-side rate limiting
- Show upgrade prompts to users
- Validate license status

### Edition Display in UI

**Both Control Center and Client UI display edition information automatically:**

![Edition Badge Example]

**Control Center (http://localhost:9000):**
- Edition badge in top-right header
- Hover to see detailed tooltip with:
  - Rate limits (learn/reason per minute)
  - Capacity (max concepts, dataset size)
  - Enterprise features (HA, Grid, Multi-node)
  - License expiration date
  - Upgrade link

**Client UI (http://localhost:8080):**
- Compact badge next to "Sutra AI" logo
- Same detailed tooltip on hover
- Visual indicators:
  - Simple: Gray gradient
  - Community: Blue gradient
  - Enterprise: Gold gradient

**No configuration needed** - UI components automatically fetch from `/edition` endpoint and refresh every 5 minutes.

### Validating Your License

License validation happens automatically on startup:

```bash
# Check current edition
curl http://localhost:8000/edition

# Response
{
  "edition": "community",
  "limits": {
    "learn_per_min": 100,
    "reason_per_min": 500,
    "max_concepts": 1000000,
    ...
  },
  "support": {
    "type": "email",
    "sla_hours": 48
  },
  "upgrade_url": "https://sutra.ai/pricing"
}
```

## Quota Enforcement

### API Rate Limits

Rate limits are enforced per-minute:

```bash
# Simple edition (10/min learn)
for i in {1..15}; do
  curl -X POST http://localhost:8000/sutra/learn \
    -H "Content-Type: application/json" \
    -d "{\"text\":\"Concept $i\"}"
done

# First 10 requests: 200 OK
# Requests 11-15: 429 Too Many Requests
{
  "detail": "Rate limit exceeded: 10 requests/min (Simple edition). Upgrade to Community for 100/min."
}
```

### Storage Quotas

Concept limits are enforced:

```bash
# Simple edition (100K concepts)
curl http://localhost:8000/stats

{
  "total_concepts": 95000,
  "quota": {
    "limit": 100000,
    "usage_percent": 95.0,
    "warning": "Approaching quota limit. Upgrade to Community for 1M concepts."
  }
}
```

At 100% quota:
```json
{
  "error": "Quota exceeded: 100,000 concepts (Simple edition limit). Upgrade to Community for 1M concepts or Enterprise for 10M concepts.",
  "upgrade_url": "https://sutra.ai/pricing"
}
```

### Dataset Size Limits

Bulk ingestion enforces dataset size limits:

```bash
# Simple edition (1GB limit)
curl -X POST http://localhost:8005/ingest \
  -F "file=@large-dataset.json"

# If file > 1GB:
{
  "error": "Dataset size 2.3GB exceeds limit of 1GB (Simple edition). Upgrade to Community for 10GB or Enterprise for unlimited.",
  "upgrade_url": "https://sutra.ai/pricing"
}
```

## Feature Flag API

All services expose edition information:

```bash
# Get edition info
curl http://localhost:8000/edition

{
  "edition": "enterprise",
  "limits": {
    "learn_per_min": 1000,
    "reason_per_min": 5000,
    "max_concepts": 10000000,
    "max_dataset_gb": "unlimited",
    "ingest_workers": 16,
    "retention_days": 365
  },
  "features": {
    "ha_enabled": true,
    "grid_enabled": true,
    "sharded_storage": true,
    "secure_mode": true
  },
  "support": {
    "type": "dedicated",
    "sla_hours": 4
  },
  "upgrade_url": null
}
```

## Troubleshooting

### License Validation Errors

```bash
# Error: License required
ERROR: COMMUNITY edition requires SUTRA_LICENSE_KEY

Get your license at: https://sutra.ai/pricing
```

**Solution:** Set `SUTRA_LICENSE_KEY` environment variable

### License Expired

```bash
# Error: License expired
ERROR: Invalid license key for COMMUNITY edition

Please check:
  1. License key is correct
  2. License has not expired  # ← Check this
  3. License matches edition (community)
```

**Solution:** Renew license at https://sutra.ai/pricing

### Edition Mismatch

```bash
# Using Community license with SUTRA_EDITION=enterprise
ERROR: License edition mismatch: expected enterprise, got community
```

**Solution:** Either:
1. Set `SUTRA_EDITION=community` (use Community features)
2. Upgrade license to Enterprise

### Grid Without Enterprise License

```bash
# Trying to use Grid features without Enterprise
ERROR: Grid orchestration requires Enterprise edition
Current edition: community
Upgrade at https://sutra.ai/pricing
```

**Solution:** Upgrade to Enterprise edition

## FAQ

**Q: Can I try Enterprise features before buying?**  
A: Yes! Contact sales@sutra.ai for a 30-day Enterprise trial license.

**Q: What happens if my license expires?**  
A: Services continue running but won't restart. You have a 7-day grace period to renew.

**Q: Can I downgrade from Enterprise to Community?**  
A: Yes, but HA/Grid features will be disabled. Your data is preserved.

**Q: Do I need to rebuild Docker images when changing editions?**  
A: No! Same images work for all editions. Just change environment variables.

**Q: How do I get a license?**  
A: Visit https://sutra.ai/pricing or contact sales@sutra.ai

**Q: Is there a discount for annual payments?**  
A: Yes! 2 months free with annual payment. Contact sales@sutra.ai

**Q: Can I use Simple edition in production?**  
A: Technically yes, but not recommended. No SLA, low limits, single point of failure.

**Q: What's included in "Professional services"?**  
A: Custom adapter development, deployment assistance, performance tuning, training.

---

**Get Started:**
- Simple (Free): `sutra deploy`
- Community/Enterprise: https://sutra.ai/pricing
- Questions: support@sutra.ai

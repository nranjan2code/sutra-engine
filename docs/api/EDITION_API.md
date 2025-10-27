# Edition API Integration

**Status:** ✅ **COMPLETE (2025-10-25)**

## Overview

The Sutra API service (`sutra-api`) provides edition-aware rate limiting and quota enforcement. All services expose an `/edition` endpoint that clients can query to determine current limits, features, and license status.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                  Environment Variables                   │
│  SUTRA_EDITION, SUTRA_LICENSE_KEY, SUTRA_LICENSE_SECRET │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│              sutra-core/feature_flags.py                 │
│  - Edition detection & validation                        │
│  - EditionLimits dataclass                               │
│  - License validation (HMAC-SHA256)                      │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│              sutra-api/config.py                         │
│  - Import feature flags                                  │
│  - Apply edition limits at startup                       │
│  - Expose get_edition_limits() method                    │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│              sutra-api/main.py                           │
│  - /edition endpoint (GET)                               │
│  - Edition-aware rate limiting middleware                │
│  - License validation responses                          │
└─────────────────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│                  Client Applications                     │
│  - Control Center (React UI)                             │
│  - sutra-client (Streamlit)                              │
│  - Custom integrations                                   │
└─────────────────────────────────────────────────────────┘
```

## API Endpoints

## API Endpoints

### ML Foundation Services

The Sutra platform now includes standardized ML services built on the unified ML Foundation:

| Service | Port | Purpose | API Documentation |
|---------|------|---------|-------------------|
| **Embedding Service** | 8889 | Semantic embeddings with nomic-embed-text-v1.5 | [API Reference](./EMBEDDING_SERVICE_API.md) |
| **NLG Service** | 8890 | Grounded text generation with multi-mode support | [API Reference](./NLG_SERVICE_API.md) |

**Standard ML Foundation Endpoints** (all services):
- `GET /health` - Basic health check
- `GET /health/detailed` - Comprehensive health with dependencies
- `GET /metrics` - Prometheus metrics
- `GET /info` - Service info with edition limits

**Complete ML Foundation documentation:** [ML Foundation API](./ML_FOUNDATION_API.md)

### Core Sutra API Endpoints

#### `GET /edition`

Returns current edition information, limits, and features.

**No authentication required** - This is public information about the deployment.

**Request:**
```bash
curl http://localhost:8000/edition
```

**Response (200 OK):**
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

**Response Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `edition` | string | Current edition: "simple", "community", or "enterprise" |
| `limits.learn_per_min` | int | Learn API calls per minute |
| `limits.reason_per_min` | int | Reason API calls per minute |
| `limits.max_concepts` | int | Maximum concepts in knowledge base |
| `limits.max_dataset_gb` | int | Maximum dataset size in GB |
| `limits.ingest_workers` | int | Parallel ingestion workers |
| `features.ha_enabled` | bool | High availability enabled |
| `features.grid_enabled` | bool | Grid orchestration enabled |
| `features.observability_enabled` | bool | Event observability enabled |
| `features.multi_node` | bool | Multi-node support enabled |
| `license_valid` | bool | License validation status |
| `license_expires` | string? | License expiration (ISO 8601) or null |
| `upgrade_url` | string | URL to upgrade edition |

### Rate Limiting Behavior

Rate limits are **automatically enforced** based on edition:

**Simple Edition:**
- `/learn`: 10 requests/min
- `/learn/batch`: 5 requests/min
- `/search`: 100 requests/min

**Community Edition:**
- `/learn`: 100 requests/min
- `/learn/batch`: 50 requests/min
- `/search`: 500 requests/min

**Enterprise Edition:**
- `/learn`: 1000 requests/min
- `/learn/batch`: 500 requests/min
- `/search`: 5000 requests/min

**Rate Limit Response (429 Too Many Requests):**
```json
{
  "error": "RateLimitExceeded",
  "message": "Rate limit exceeded: 10 requests per 60 seconds",
  "retry_after": 60
}
```

**Response Headers:**
- `Retry-After`: Seconds until rate limit resets

## Client Integration

### JavaScript/TypeScript Example

```typescript
interface EditionInfo {
  edition: 'simple' | 'community' | 'enterprise';
  limits: {
    learn_per_min: number;
    reason_per_min: number;
    max_concepts: number;
    max_dataset_gb: number;
    ingest_workers: number;
  };
  features: {
    ha_enabled: boolean;
    grid_enabled: boolean;
    observability_enabled: boolean;
    multi_node: boolean;
  };
  license_valid: boolean;
  license_expires: string | null;
  upgrade_url: string;
}

async function fetchEditionInfo(): Promise<EditionInfo> {
  const response = await fetch('http://localhost:8000/edition');
  if (!response.ok) {
    throw new Error('Failed to fetch edition info');
  }
  return response.json();
}

// Usage in React component
function EditionBanner() {
  const [edition, setEdition] = useState<EditionInfo | null>(null);
  
  useEffect(() => {
    fetchEditionInfo().then(setEdition);
  }, []);
  
  if (!edition) return null;
  
  return (
    <div className="edition-banner">
      <span>Edition: {edition.edition.toUpperCase()}</span>
      <span>Learn: {edition.limits.learn_per_min}/min</span>
      {!edition.license_valid && (
        <a href={edition.upgrade_url}>License Invalid - Renew</a>
      )}
    </div>
  );
}
```

### Python Example

```python
import requests

def get_edition_info(api_url: str = "http://localhost:8000"):
    """Get edition information from API."""
    response = requests.get(f"{api_url}/edition")
    response.raise_for_status()
    return response.json()

# Usage
edition = get_edition_info()
print(f"Edition: {edition['edition']}")
print(f"Learn limit: {edition['limits']['learn_per_min']}/min")

# Implement client-side rate limiting
if edition['edition'] == 'simple':
    # Slower polling for simple edition
    poll_interval = 10
elif edition['edition'] == 'community':
    poll_interval = 5
else:
    poll_interval = 1
```

### Bash/cURL Example

```bash
#!/bin/bash

# Fetch edition info
EDITION_JSON=$(curl -s http://localhost:8000/edition)

# Parse with jq
EDITION=$(echo "$EDITION_JSON" | jq -r '.edition')
LEARN_LIMIT=$(echo "$EDITION_JSON" | jq -r '.limits.learn_per_min')
LICENSE_VALID=$(echo "$EDITION_JSON" | jq -r '.license_valid')

echo "Edition: $EDITION"
echo "Learn limit: $LEARN_LIMIT requests/min"

if [ "$LICENSE_VALID" != "true" ]; then
  echo "⚠️  License invalid or expired!"
  exit 1
fi
```

## UI Integration Guidelines

### Display Current Limits

**DO:**
- Query `/edition` on app startup
- Cache response for 5-10 minutes
- Display limits in settings/about page
- Show upgrade prompts for higher editions

**DON'T:**
- Query `/edition` on every user action
- Hardcode edition limits in UI
- Assume edition without checking API

### Example UI Components

**Edition Badge:**
```jsx
function EditionBadge({ edition }) {
  const colors = {
    simple: 'gray',
    community: 'blue',
    enterprise: 'gold'
  };
  
  return (
    <Badge color={colors[edition]}>
      {edition.toUpperCase()}
    </Badge>
  );
}
```

**Upgrade Prompt:**
```jsx
function UpgradePrompt({ edition, limits, upgradeUrl }) {
  if (edition === 'enterprise') return null;
  
  const nextEdition = edition === 'simple' ? 'Community' : 'Enterprise';
  const nextLimits = edition === 'simple' 
    ? { learn: 100, reason: 500 }
    : { learn: 1000, reason: 5000 };
  
  return (
    <Alert severity="info">
      <AlertTitle>Upgrade to {nextEdition}</AlertTitle>
      Get {nextLimits.learn}× more Learn requests and {nextLimits.reason}× more Reason requests.
      <Button href={upgradeUrl}>View Plans</Button>
    </Alert>
  );
}
```

**Rate Limit Progress:**
```jsx
function RateLimitProgress({ used, limit, window }) {
  const percentage = (used / limit) * 100;
  
  return (
    <div>
      <LinearProgress 
        variant="determinate" 
        value={percentage}
        color={percentage > 80 ? 'warning' : 'primary'}
      />
      <Typography variant="caption">
        {used}/{limit} requests used ({window}s window)
      </Typography>
    </div>
  );
}
```

## Configuration

### Environment Variables

**Required for Community/Enterprise:**
```bash
SUTRA_EDITION="community"              # or "enterprise"
SUTRA_LICENSE_KEY="base64.signature"   # Provided by Sutra AI
```

**Optional:**
```bash
SUTRA_LICENSE_SECRET="hex_secret"      # For license generation (admins only)
```

### Docker Compose Integration

```yaml
services:
  sutra-api:
    image: sutra-api:latest
    environment:
      - SUTRA_EDITION=${SUTRA_EDITION:-simple}
      - SUTRA_LICENSE_KEY=${SUTRA_LICENSE_KEY:-}
    ports:
      - "8000:8000"
```

## Testing

### Test Edition Detection

```bash
# Test Simple edition (no env vars)
docker-compose -f docker-compose-grid.yml up sutra-api

curl http://localhost:8000/edition | jq '.edition'
# Expected: "simple"

# Test Community edition
docker-compose down
export SUTRA_EDITION="community"
export SUTRA_LICENSE_KEY="$(python scripts/generate-license.py --edition community --customer Test --days 365)"
docker-compose up sutra-api

curl http://localhost:8000/edition | jq '.limits.learn_per_min'
# Expected: 100
```

### Test Rate Limiting

```bash
# Test rate limit enforcement
for i in {1..15}; do
  curl -X POST http://localhost:8000/learn \
    -H "Content-Type: application/json" \
    -d '{"content":"Test fact '$i'"}' \
    -w "Status: %{http_code}\n"
done

# Simple edition (10/min): First 10 succeed, next 5 get 429
# Community edition (100/min): All 15 succeed
```

## Implementation Details

### Config Changes

**`packages/sutra-api/sutra_api/config.py`:**
- Import `sutra_core.feature_flags`
- Add `edition` and `license_key` fields
- Add `get_edition_limits()` method
- Apply limits at module load

**`packages/sutra-api/sutra_api/main.py`:**
- Import edition models
- Add `/edition` endpoint
- Validate licenses for community/enterprise

**`packages/sutra-api/sutra_api/models.py`:**
- Add `EditionResponse` model
- Add `EditionLimitsResponse` model
- Add `EditionFeaturesResponse` model

### Feature Flag Integration

The API imports and uses:
- `Edition` enum (SIMPLE, COMMUNITY, ENTERPRISE)
- `EditionLimits` dataclass
- `detect_edition()` function
- `validate_license()` function

All from `sutra_core.feature_flags`.

## Troubleshooting

### License Validation Failed

**Symptom:** `/edition` returns `"license_valid": false`

**Causes:**
1. Invalid license format
2. License expired
3. Wrong edition in license
4. Tampered license signature

**Solution:**
```bash
# Check license format
echo $SUTRA_LICENSE_KEY | base64 -d | jq

# Regenerate license
python scripts/generate-license.py \
  --edition community \
  --customer "Your Company" \
  --days 365
```

### Rate Limit Too Low

**Symptom:** Getting 429 errors on low traffic

**Solution:**
```bash
# Check current edition
curl http://localhost:8000/edition | jq '.edition'

# If "simple", upgrade to Community
export SUTRA_EDITION="community"
export SUTRA_LICENSE_KEY="your-license"
./sutra-deploy.sh restart
```

### Edition Not Changing

**Symptom:** Edition still shows "simple" after setting env vars

**Causes:**
1. Environment variables not exported
2. Docker container not restarted
3. Wrong service targeted

**Solution:**
```bash
# Restart with environment variables
docker-compose down
export SUTRA_EDITION="community"
export SUTRA_LICENSE_KEY="your-key"
docker-compose up -d

# Verify environment inside container
docker exec sutra-api env | grep SUTRA_EDITION
```

## Security Considerations

### License Key Storage

**NEVER:**
- Commit license keys to git
- Log license keys
- Expose license keys in client-side code

**DO:**
- Store in environment variables
- Use secrets management (Kubernetes secrets, Docker secrets)
- Rotate keys periodically

### Rate Limit Bypass

The API uses IP-based rate limiting. In production:
- Deploy behind reverse proxy with `X-Forwarded-For` headers
- Use Redis-backed rate limiting for multi-server deployments
- Implement API keys for better tracking

### License Validation

- Licenses use HMAC-SHA256 signatures
- Validation happens offline (no phone-home)
- Tampered licenses are rejected
- Expired licenses are detected

## Future Enhancements

**Planned:**
- [ ] Per-user rate limiting (not just per-IP)
- [ ] Redis-backed rate limiting for HA deployments
- [ ] Usage analytics and reporting
- [ ] License auto-renewal webhooks
- [ ] Edition-based feature toggles in UI

**Not Planned:**
- ❌ Phone-home license validation
- ❌ Hardware-locked licenses
- ❌ Trial editions with time limits

## References

- **Feature Flags:** `packages/sutra-core/sutra_core/feature_flags.py`
- **API Config:** `packages/sutra-api/sutra_api/config.py`
- **API Models:** `packages/sutra-api/sutra_api/models.py`
- **License Generation:** `scripts/generate-license.py`
- **Edition Docs:** `docs/EDITIONS.md`
- **Quick Start:** `docs/QUICKSTART_EDITIONS.md`

---

**Status:** ✅ Production-ready  
**Last Updated:** 2025-10-25  
**Maintainer:** Sutra AI Team

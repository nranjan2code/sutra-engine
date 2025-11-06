# License Administration Guide

**For Sutra AI administrators managing licenses.**

## Overview

Sutra AI uses HMAC-SHA256 signed licenses for edition control. Licenses are:
- **Offline validated** (no phone-home)
- **Tamper-proof** (cryptographic signature)
- **Edition-specific** (community/enterprise)
- **Time-bound** (optional expiration)

## License Format

```
base64(json_data).hmac_signature

Example:
eyJlZGl0aW9uIjoiY29tbXVuaXR5IiwiY3VzdG9tZXJfaWQiOiJBY21lIENvcnAiLCJpc3N1ZWQiOiIyMDI1LTAxLTAxVDEyOjAwOjAwWiIsImV4cGlyZXMiOiIyMDI2LTAxLTAxVDEyOjAwOjAwWiJ9.a7f3e9d2c1b4f6e8d3a2c5b7f9e1d4c6a8b2f5e7d3c9a1b4f6e8d2c5b7f9e1d4
```

### JSON Data Structure

```json
{
  "edition": "community",
  "customer_id": "Acme Corp",
  "issued": "2025-01-01T12:00:00Z",
  "expires": "2026-01-01T12:00:00Z"
}
```

- **edition**: `community` or `enterprise` (simple has no license)
- **customer_id**: Customer identifier (company name, email, or ID)
- **issued**: ISO 8601 timestamp of license creation
- **expires**: ISO 8601 timestamp of expiration (null = permanent)

### HMAC Signature

- **Algorithm:** HMAC-SHA256
- **Secret Key:** `SUTRA_LICENSE_SECRET` environment variable
- **Input:** Base64-encoded JSON data
- **Output:** Hex-encoded signature

## Generating Licenses

### Prerequisites

```bash
# 1. Set secret key (CRITICAL - keep secure!)
export SUTRA_LICENSE_SECRET="$(openssl rand -hex 32)"

# Save secret key securely (e.g., AWS Secrets Manager, 1Password)
echo "SUTRA_LICENSE_SECRET=$SUTRA_LICENSE_SECRET" > .env.license
chmod 600 .env.license

# 2. Activate Python environment
source venv/bin/activate
```

### Community License (1 year)

```bash
python scripts/generate-license.py \
  --edition community \
  --customer "Acme Corp" \
  --days 365

# Output:
================================================================================
Sutra AI COMMUNITY License Generated
================================================================================

Customer:    Acme Corp
Edition:     community
Issued:      2025-01-26T12:00:00
Expires:     2026-01-26T12:00:00

License Key:
--------------------------------------------------------------------------------
eyJlZGl0aW9uIjoiY29tbXVuaXR5IiwiY3VzdG9tZXJfaWQiOiJBY21lIENvcnAiLCJpc3N1ZWQiOiIyMDI1LTAxLTI2VDEyOjAwOjAwIiwiZXhwaXJlcyI6IjIwMjYtMDEtMjZUMTI6MDA6MDAifQ==.e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
--------------------------------------------------------------------------------

Deployment:
  export SUTRA_EDITION="community"
  export SUTRA_LICENSE_KEY="eyJlZGl0aW9uIjoiY29tbXVuaXR5IiwiY3VzdG9tZXJfaWQiOiJBY21lIENvcnAiLCJpc3N1ZWQiOiIyMDI1LTAxLTI2VDEyOjAwOjAwIiwiZXhwaXJlcyI6IjIwMjYtMDEtMjZUMTI6MDA6MDAifQ==.e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
  sutra deploy

================================================================================

License saved to: license-acme-corp-community.txt
```

### Enterprise License (Permanent)

```bash
python scripts/generate-license.py \
  --edition enterprise \
  --customer "BigCo Inc" \
  --days 0  # 0 = permanent

# License never expires
```

### Enterprise License (Custom Duration)

```bash
# 30-day trial
python scripts/generate-license.py \
  --edition enterprise \
  --customer "Trial Customer" \
  --days 30

# 3-year contract
python scripts/generate-license.py \
  --edition enterprise \
  --customer "Enterprise Customer" \
  --days 1095  # 365 * 3
```

## License Delivery

### Option 1: Email (Recommended)

```bash
# Generate license
python scripts/generate-license.py \
  --edition community \
  --customer "customer@example.com" \
  --days 365

# Email the generated .txt file to customer
# File contains full deployment instructions
```

### Option 2: Customer Portal

```bash
# Generate license
LICENSE_KEY=$(python scripts/generate-license.py \
  --edition enterprise \
  --customer "customer-id-12345" \
  --days 365 | grep "^eyJ")

# Store in database
# Customer retrieves from self-service portal
```

### Option 3: API Integration

```python
# For SaaS deployments
import requests

response = requests.post("https://license-api.sutra.ai/generate", {
    "edition": "community",
    "customer_id": "customer@example.com",
    "duration_days": 365,
    "api_key": "your-api-key"
})

license_key = response.json()["license_key"]
```

## License Renewal

### Renewing Existing License

```bash
# Generate new license with same customer ID
python scripts/generate-license.py \
  --edition community \
  --customer "Acme Corp" \
  --days 365

# Send new license to customer
# Customer updates SUTRA_LICENSE_KEY and restarts
```

### Grace Period

**7-day grace period after expiration:**
- Services continue running
- Services will NOT restart after expiration
- Warning logs appear daily

```bash
# Log example
2025-02-02 12:00:00 WARNING: License expires in 5 days. Renew at https://sutra.ai/pricing
2025-02-07 12:00:00 ERROR: License expired. Services will not restart. Contact support@sutra.ai
```

## License Validation

### Manual Validation

```bash
# Check license on running system
curl http://localhost:8000/edition

{
  "edition": "community",
  "license_valid": true,
  "license_expires": "2026-01-26T12:00:00",
  "days_remaining": 365
}
```

### Validation Logs

```bash
# Check validation in logs
docker logs sutra-api 2>&1 | grep -i license

# Successful validation
INFO: License validated: community for Acme Corp
INFO: Sutra AI COMMUNITY Edition initialized. Limits: 100 learn/min, 1,000,000 max concepts

# Failed validation
ERROR: Invalid license signature
ERROR: License expired on 2025-12-31
ERROR: License edition mismatch: expected enterprise, got community
```

## License Revocation

### Immediate Revocation

```bash
# Method 1: Delete secret key (breaks all licenses)
unset SUTRA_LICENSE_SECRET

# Method 2: Rotate secret key (invalidates old licenses)
export SUTRA_LICENSE_SECRET="$(openssl rand -hex 32)"

# Method 3: Use short expiration (e.g., 1 day for trials)
python scripts/generate-license.py \
  --edition community \
  --customer "Trial User" \
  --days 1
```

### Grace Period Revocation

```bash
# Set expiration to past date
# Customer enters 7-day grace period
# Services won't restart after grace period
```

## Secret Key Management

### Best Practices

1. **Generate Strong Key**
   ```bash
   # 32-byte (256-bit) key
   openssl rand -hex 32
   ```

2. **Store Securely**
   - AWS Secrets Manager
   - HashiCorp Vault
   - 1Password / Bitwarden
   - Encrypted environment variable

3. **Never Commit to Git**
   ```bash
   # .gitignore
   .env.license
   *.license.txt
   ```

4. **Rotate Periodically**
   ```bash
   # Rotate secret key annually
   # Generate new licenses with new key
   # Allow overlap period for migration
   ```

5. **Backup Secret Key**
   ```bash
   # Multiple secure locations
   # Recovery procedure documented
   ```

### Secret Key Rotation

```bash
# 1. Generate new secret key
NEW_SECRET=$(openssl rand -hex 32)

# 2. Generate new licenses for all active customers
export SUTRA_LICENSE_SECRET="$NEW_SECRET"
python scripts/generate-license.py --edition community --customer "Customer 1" --days 365
python scripts/generate-license.py --edition community --customer "Customer 2" --days 365
# ... repeat for all customers

# 3. Email new licenses to customers

# 4. After 30-day migration period, retire old secret key
```

## Monitoring & Analytics

### License Usage Tracking

```bash
# Query license info from deployed systems
curl http://localhost:8000/edition

# Aggregate across all customers
# Track:
# - Active licenses
# - Expiring soon (< 30 days)
# - Usage patterns (API calls, concepts)
# - Upgrade candidates (approaching quotas)
```

### Quota Alerts

```bash
# Monitor quota usage
curl http://localhost:8000/stats

{
  "total_concepts": 850000,
  "quota": {
    "limit": 1000000,
    "usage_percent": 85.0
  }
}

# Alert customer when approaching limits
# Suggest upgrade to next tier
```

## Troubleshooting

### Invalid Signature

**Cause:** Secret key mismatch

**Solution:**
```bash
# Verify secret key matches
echo $SUTRA_LICENSE_SECRET

# Re-generate license with correct secret
python scripts/generate-license.py \
  --edition community \
  --customer "Customer" \
  --days 365 \
  --secret "$CORRECT_SECRET"
```

### License Expired

**Cause:** Expiration date passed

**Solution:**
```bash
# Generate new license with fresh expiration
python scripts/generate-license.py \
  --edition community \
  --customer "Customer" \
  --days 365

# Send to customer immediately
```

### Edition Mismatch

**Cause:** Using Community license with `SUTRA_EDITION=enterprise`

**Solution:**
```bash
# Option 1: Correct edition
export SUTRA_EDITION="community"

# Option 2: Upgrade license
python scripts/generate-license.py \
  --edition enterprise \
  --customer "Customer" \
  --days 365
```

## Customer Support

### Common Customer Questions

**Q: How do I check my license expiration?**
```bash
curl http://localhost:8000/edition | jq '.license_expires'
```

**Q: Can I move my license to a new server?**
Yes - just set the same `SUTRA_LICENSE_KEY` on the new server.

**Q: What happens when my license expires?**
7-day grace period. Services continue running but won't restart.

**Q: Can I upgrade from Community to Enterprise?**
Yes - get new Enterprise license, set `SUTRA_EDITION=enterprise`, restart.

### Support Escalation

**License Issues:**
1. Check validation logs: `docker logs sutra-api | grep -i license`
2. Verify secret key: `echo $SUTRA_LICENSE_SECRET`
3. Re-generate license if needed
4. Escalate to engineering if validation logic fails

**Quota Issues:**
1. Check current usage: `curl http://localhost:8000/stats`
2. Verify edition limits: `curl http://localhost:8000/edition`
3. Suggest upgrade if approaching limits

## Security Considerations

### Threat Model

**Threats Mitigated:**
- ✅ License tampering (HMAC signature)
- ✅ License forgery (secret key required)
- ✅ License replay (expiration dates)
- ✅ Edition escalation (edition in signed payload)

**Threats NOT Mitigated:**
- ❌ Key compromise (if attacker gets secret key, can generate licenses)
- ❌ Binary patching (attacker modifies validation code)
- ❌ License sharing (one license used on multiple servers)

### Mitigation Strategies

1. **Key Compromise**
   - Rotate secret key periodically
   - Monitor for unusual license generation patterns
   - Use hardware security module (HSM) for production

2. **Binary Patching**
   - Code signing (Docker content trust)
   - Integrity monitoring
   - Regular security audits

3. **License Sharing**
   - Add hardware fingerprinting (future)
   - Monitor concurrent usage patterns
   - Enforce unique customer IDs

## Appendix

### License Generation Script

Location: `scripts/generate-license.py`

### Feature Flags Implementation

Location: `packages/sutra-core/sutra_core/feature_flags.py`

### Edition Documentation

- Full comparison: `docs/EDITIONS.md`
- Quick start: `docs/QUICKSTART_EDITIONS.md`
- WARP guide: `WARP.md`

---

**For questions:** licensing@sutra.ai  
**For security issues:** security@sutra.ai

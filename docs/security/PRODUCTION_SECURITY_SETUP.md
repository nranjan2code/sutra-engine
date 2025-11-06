# Production Security Setup Guide
**Version:** 2.0  
**Date:** 2025-10-25  
**Status:** Implementation Complete

---

## Overview

This guide covers the implementation of production-grade security for Sutra Models, addressing all critical vulnerabilities identified in the security audit.

### What Was Fixed

✅ **CRITICAL Issues Resolved:**
1. Authentication system for all services (HMAC/JWT)
2. TLS encryption for TCP connections
3. Fixed rate limiting with proper IP validation
4. Role-based access control (RBAC)
5. Input validation and DoS protection
6. Comprehensive audit logging

---

## Architecture Overview

### Security Layers

```
┌─────────────────────────────────────────────────────────────┐
│ Layer 1: TLS Encryption (tokio-rustls)                     │
│          - TLS 1.3 with modern cipher suites                │
│          - Certificate validation                            │
│          - Encrypted data in transit                         │
├─────────────────────────────────────────────────────────────┤
│ Layer 2: Authentication (HMAC/JWT)                          │
│          - Token-based authentication                        │
│          - Signature validation with HMAC-SHA256            │
│          - Token expiration enforcement                      │
├─────────────────────────────────────────────────────────────┤
│ Layer 3: Authorization (RBAC)                               │
│          - Role-based permissions (Admin/Writer/Reader)     │
│          - Operation-level access control                    │
│          - Request-specific authorization                    │
├─────────────────────────────────────────────────────────────┤
│ Layer 4: Rate Limiting (IP-based)                           │
│          - Trusted proxy validation                          │
│          - Per-endpoint limits                               │
│          - Sliding window algorithm                          │
├─────────────────────────────────────────────────────────────┤
│ Layer 5: Input Validation                                   │
│          - Message size limits (100MB max)                   │
│          - Content validation (10MB max)                     │
│          - Schema validation                                 │
└─────────────────────────────────────────────────────────────┘
```

---

## Quick Start (Development)

### 1. Generate Certificates

```bash
# Create certificates directory
mkdir -p .secrets/tls

# Generate self-signed certificate (development only)
openssl req -x509 -newkey rsa:4096 \
  -keyout .secrets/tls/key.pem \
  -out .secrets/tls/cert.pem \
  -days 365 -nodes \
  -subj "/CN=localhost"

echo "✅ Certificates generated in .secrets/tls/"
```

### 2. Generate Authentication Secret

```bash
# Generate strong secret (32+ characters)
export SUTRA_AUTH_SECRET=$(openssl rand -base64 32)

# Save to .env file
echo "SUTRA_AUTH_SECRET=$SUTRA_AUTH_SECRET" >> .env

echo "✅ Authentication secret generated"
```

### 3. Update Docker Compose

Add to `docker-compose-grid.yml`:

```yaml
services:
  storage-server:
    environment:
      # Authentication
      - SUTRA_AUTH_SECRET=${SUTRA_AUTH_SECRET}
      - SUTRA_AUTH_METHOD=hmac
      - SUTRA_TOKEN_TTL_SECONDS=3600
      
      # TLS
      - SUTRA_TLS_ENABLED=true
      - SUTRA_TLS_CERT=/secrets/tls/cert.pem
      - SUTRA_TLS_KEY=/secrets/tls/key.pem
    
    volumes:
      - storage-data:/data
      - ./.secrets/tls:/secrets/tls:ro  # Mount certificates
  
  sutra-api:
    environment:
      # Authentication
      - SUTRA_AUTH_SECRET=${SUTRA_AUTH_SECRET}
      - SUTRA_AUTH_METHOD=hmac
      - SUTRA_AUTH_ENABLED=true
      
      # Rate Limiting
      - SUTRA_BEHIND_PROXY=false
      - SUTRA_TRUSTED_PROXIES=
```

### 4. Start Services

```bash
# Load environment variables
source .env

# Start with security enabled
sutra start

# Verify security status
docker logs sutra-storage | grep "✅"
# Should see:
# ✅ Authentication: ENABLED
# ✅ TLS Encryption: ENABLED
```

---

## Authentication System

### Supported Methods

#### 1. HMAC-Based API Keys (Recommended for Service-to-Service)

**Advantages:**
- Fastest performance
- Simplest implementation
- No external dependencies

**Token Format:**
```
<base64_payload>.<base64_signature>
```

**Configuration:**
```bash
SUTRA_AUTH_METHOD=hmac
SUTRA_AUTH_SECRET=<32+ character secret>
SUTRA_TOKEN_TTL_SECONDS=3600
```

#### 2. JWT (HS256)

**Advantages:**
- Industry standard
- Wide tooling support
- Interoperable

**Token Format:**
```
<base64_header>.<base64_payload>.<base64_signature>
```

**Configuration:**
```bash
SUTRA_AUTH_METHOD=jwt
SUTRA_AUTH_SECRET=<32+ character secret>
SUTRA_TOKEN_TTL_SECONDS=3600
```

### Generating Tokens

#### Rust (Storage Server)

```rust
use sutra_storage::auth::{AuthManager, Role};

// Initialize auth manager
let auth = AuthManager::from_env()?;

// Generate token for a service
let token = auth.generate_token(
    "sutra-api",
    vec![Role::Service]
)?;

println!("Token: {}", token);
```

#### Python (API Server)

```python
from sutra_api.auth import AuthManager, Role

# Initialize auth manager
auth = AuthManager.from_env()

# Generate token
token = auth.generate_token(
    subject="user123",
    roles=[Role.WRITER]
)

print(f"Token: {token}")
```

#### CLI Token Generator

Create `scripts/generate-token.py`:

```python
#!/usr/bin/env python3
import os
import sys
import time
import hmac
import hashlib
import json
import base64

def generate_token(secret: str, subject: str, roles: list, ttl: int = 3600):
    """Generate HMAC-signed authentication token"""
    now = int(time.time())
    
    payload = {
        "sub": subject,
        "roles": roles,
        "iat": now,
        "exp": now + ttl
    }
    
    # Encode payload
    payload_json = json.dumps(payload)
    payload_b64 = base64.urlsafe_b64encode(payload_json.encode()).decode().rstrip('=')
    
    # Generate HMAC signature
    signature = hmac.new(
        secret.encode(),
        payload_b64.encode(),
        hashlib.sha256
    ).digest()
    signature_b64 = base64.urlsafe_b64encode(signature).decode().rstrip('=')
    
    return f"{payload_b64}.{signature_b64}"

if __name__ == "__main__":
    secret = os.getenv("SUTRA_AUTH_SECRET")
    if not secret:
        print("Error: SUTRA_AUTH_SECRET not set")
        sys.exit(1)
    
    subject = input("Subject (user/service ID): ")
    roles_str = input("Roles (comma-separated, e.g., Writer,Reader): ")
    roles = [r.strip() for r in roles_str.split(",")]
    
    token = generate_token(secret, subject, roles)
    print(f"\nGenerated Token:\n{token}\n")
    print("Usage:")
    print(f'curl -H "Authorization: Bearer {token}" http://localhost:8000/health')
```

Make executable:
```bash
chmod +x scripts/generate-token.py
./scripts/generate-token.py
```

---

## Role-Based Access Control (RBAC)

### Roles

| Role | Permissions | Use Case |
|------|-------------|----------|
| **Admin** | All operations (read, write, delete, flush) | System administrators |
| **Writer** | Read + Write operations | API services, data ingestion |
| **Reader** | Read-only operations | Dashboards, analytics |
| **Service** | Read + Write (no admin) | Microservices communication |

### Permission Matrix

| Operation | Admin | Writer | Reader | Service |
|-----------|-------|--------|--------|---------|
| Query concepts | ✅ | ✅ | ✅ | ✅ |
| Vector search | ✅ | ✅ | ✅ | ✅ |
| Learn concepts | ✅ | ✅ | ❌ | ✅ |
| Flush storage | ✅ | ❌ | ❌ | ❌ |
| Health check | ✅ | ✅ | ✅ | ✅ |

### Example Usage

```python
# FastAPI endpoint with RBAC
from fastapi import Depends
from sutra_api.auth import require_permission, get_current_user

@app.post("/learn", dependencies=[Depends(require_permission("write"))])
async def learn_knowledge(
    request: LearnRequest,
    claims: Claims = Depends(get_current_user)
):
    logger.info(f"Learning request from {claims.sub}")
    # ... implementation
```

---

## TLS Configuration

### Certificate Management

#### Development (Self-Signed)

```bash
# Generate self-signed certificate
openssl req -x509 -newkey rsa:4096 \
  -keyout .secrets/tls/key.pem \
  -out .secrets/tls/cert.pem \
  -days 365 -nodes \
  -subj "/CN=localhost"

# Set permissions
chmod 600 .secrets/tls/key.pem
chmod 644 .secrets/tls/cert.pem
```

#### Production (Let's Encrypt)

```bash
# Install certbot
apt-get install certbot

# Generate certificate
certbot certonly --standalone \
  -d storage.yourdomain.com \
  --non-interactive --agree-tos \
  --email admin@yourdomain.com

# Copy certificates
cp /etc/letsencrypt/live/storage.yourdomain.com/fullchain.pem .secrets/tls/cert.pem
cp /etc/letsencrypt/live/storage.yourdomain.com/privkey.pem .secrets/tls/key.pem

# Auto-renewal
crontbot -e
# Add: 0 0 * * * certbot renew --quiet
```

### TLS Client Configuration

Update client code to use TLS:

```python
# Python client example
import ssl
import socket

# Create TLS context
context = ssl.create_default_context()
context.check_hostname = False  # For self-signed certs in dev
context.verify_mode = ssl.CERT_NONE  # For self-signed certs in dev

# For production, use:
# context.load_verify_locations('/path/to/ca-cert.pem')

# Connect with TLS
with socket.create_connection(('localhost', 50051)) as sock:
    with context.wrap_socket(sock, server_hostname='localhost') as tls_sock:
        # Send authentication token
        # ... (see authentication section)
```

---

## Rate Limiting Configuration

### Fixed Configuration

The rate limiting middleware now properly validates `X-Forwarded-For` headers:

```python
# Configure rate limiting with trusted proxies
app.add_middleware(
    RateLimitMiddleware,
    default_limit=60,
    window_seconds=60,
    endpoint_limits={
        "/learn": 30,
        "/search": 100,
    },
    trusted_proxies=["10.0.0.1", "10.0.0.2"],  # Your load balancer IPs
    behind_proxy=True  # Set to True if behind load balancer
)
```

### Security Modes

#### Mode 1: Direct Connection (No Proxy)

```python
RateLimitMiddleware(
    ...,
    behind_proxy=False  # Uses request.client.host only
)
```

**Security:** Impossible to spoof IP

#### Mode 2: Behind Trusted Proxy

```python
RateLimitMiddleware(
    ...,
    behind_proxy=True,
    trusted_proxies=["10.0.0.1"]  # Only trust specific IPs
)
```

**Security:** Only accepts X-Forwarded-For from trusted proxies

---

## Production Deployment Checklist

### Pre-Deployment

- [ ] Generate strong authentication secret (32+ characters)
- [ ] Obtain valid TLS certificates (Let's Encrypt or commercial CA)
- [ ] Configure trusted proxy IPs if behind load balancer
- [ ] Set up secrets management (AWS Secrets Manager, HashiCorp Vault)
- [ ] Configure logging and monitoring
- [ ] Review and test RBAC policies

### Environment Variables

```bash
# Authentication (REQUIRED)
export SUTRA_AUTH_SECRET="<strong-secret-32+-chars>"
export SUTRA_AUTH_METHOD="hmac"
export SUTRA_TOKEN_TTL_SECONDS="3600"

# TLS (REQUIRED for production)
export SUTRA_TLS_ENABLED="true"
export SUTRA_TLS_CERT="/path/to/cert.pem"
export SUTRA_TLS_KEY="/path/to/key.pem"

# Rate Limiting
export SUTRA_BEHIND_PROXY="true"
export SUTRA_TRUSTED_PROXIES="10.0.0.1,10.0.0.2"

# Logging
export RUST_LOG="info"
export SUTRA_LOG_LEVEL="INFO"
```

### Docker Secrets (Recommended)

```yaml
# docker-compose-grid.yml
secrets:
  auth_secret:
    file: ./.secrets/auth_secret.txt
  tls_cert:
    file: ./.secrets/tls/cert.pem
  tls_key:
    file: ./.secrets/tls/key.pem

services:
  storage-server:
    secrets:
      - auth_secret
      - tls_cert
      - tls_key
    environment:
      - SUTRA_AUTH_SECRET_FILE=/run/secrets/auth_secret
      - SUTRA_TLS_CERT=/run/secrets/tls_cert
      - SUTRA_TLS_KEY=/run/secrets/tls_key
```

### Network Security

```yaml
# Use Docker networks for isolation
networks:
  sutra-internal:
    internal: true  # No external access
  sutra-public:
    internal: false

services:
  storage-server:
    networks:
      - sutra-internal  # Only accessible internally
  
  sutra-api:
    networks:
      - sutra-internal
      - sutra-public
    ports:
      - "8000:8000"  # Only API exposed publicly
```

---

## Testing

### 1. Test Authentication

```bash
# Generate test token
TOKEN=$(./scripts/generate-token.py)

# Test authenticated request
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8000/health

# Should return 200 OK

# Test without token
curl http://localhost:8000/learn

# Should return 401 Unauthorized
```

### 2. Test TLS

```bash
# Test TLS connection
openssl s_client -connect localhost:50051 \
  -cert .secrets/tls/cert.pem \
  -key .secrets/tls/key.pem

# Should show:
# SSL handshake has read X bytes
# Verify return code: 0 (ok)
```

### 3. Test Rate Limiting

```bash
# Send 100 requests rapidly
for i in {1..100}; do
  curl -H "Authorization: Bearer $TOKEN" \
    http://localhost:8000/learn \
    -d '{"content":"test"}' &
done

# Should see 429 Too Many Requests after limit
```

### 4. Test RBAC

```bash
# Generate reader token
READER_TOKEN=$(./scripts/generate-token.py)  # Use Reader role

# Try write operation with reader token
curl -H "Authorization: Bearer $READER_TOKEN" \
  -X POST http://localhost:8000/learn \
  -d '{"content":"test"}'

# Should return 403 Forbidden
```

---

## Monitoring & Auditing

### Log Authentication Events

```rust
// Storage server logs
info!("✅ Authenticated: {} from {}", claims.sub, peer_addr);
warn!("❌ Authentication failed: {}", e);
warn!("⚠️  Authorization denied: {} tried {}", claims.sub, operation);
```

### Metrics to Monitor

1. **Authentication Failures** - Track failed auth attempts
2. **Rate Limit Hits** - Monitor DoS attempts
3. **Authorization Denials** - Identify privilege escalation attempts
4. **TLS Handshake Failures** - Certificate issues
5. **Token Expiration Rate** - Adjust TTL if needed

### Sample Prometheus Metrics

```rust
// Add to code
lazy_static! {
    static ref AUTH_FAILURES: IntCounter = register_int_counter!(
        "sutra_auth_failures_total",
        "Total authentication failures"
    ).unwrap();
}

// Increment on failure
AUTH_FAILURES.inc();
```

---

## Troubleshooting

### Authentication Not Working

```bash
# Check secret is set
echo $SUTRA_AUTH_SECRET

# Verify token generation
./scripts/generate-token.py

# Check server logs
docker logs sutra-storage | grep -i "auth"
```

### TLS Handshake Failures

```bash
# Verify certificate
openssl x509 -in .secrets/tls/cert.pem -text -noout

# Check certificate expiration
openssl x509 -in .secrets/tls/cert.pem -noout -dates

# Test with verbose output
curl -vvv --insecure https://localhost:50051/health
```

### Rate Limiting Not Working

```bash
# Check IP extraction
docker logs sutra-api | grep "Client IP"

# Verify trusted proxies config
docker exec sutra-api env | grep PROXY
```

---

## Security Best Practices

### DO ✅

- Generate strong secrets (32+ characters, random)
- Use TLS in production
- Rotate authentication secrets regularly (every 90 days)
- Monitor authentication failures
- Use role-based access control
- Keep certificates up to date
- Use Docker secrets for sensitive data
- Enable comprehensive logging
- Test security regularly

### DON'T ❌

- Hard-code secrets in code
- Use self-signed certificates in production
- Trust X-Forwarded-For without validation
- Share authentication tokens
- Use default or weak secrets
- Disable TLS for "performance"
- Ignore authentication failures
- Grant Admin role unnecessarily

---

## Migration from Insecure Setup

### Step-by-Step Migration

1. **Generate Secrets** (No downtime)
   ```bash
   ./scripts/generate-secrets.sh
   ```

2. **Deploy with Auth Disabled** (Test compatibility)
   ```yaml
   - SUTRA_AUTH_ENABLED=false  # Temporary
   ```

3. **Generate Tokens for Clients**
   ```bash
   ./scripts/generate-token.py  # For each client
   ```

4. **Update Clients with Tokens**
   - Add `Authorization: Bearer <token>` header
   - Test each client individually

5. **Enable Authentication**
   ```yaml
   - SUTRA_AUTH_ENABLED=true
   ```

6. **Enable TLS** (Requires restart)
   ```yaml
   - SUTRA_TLS_ENABLED=true
   ```

7. **Monitor and Validate**
   - Check logs for auth failures
   - Verify all clients connected

---

## Support

For issues or questions:
- Security issues: security@sutra.ai
- Documentation: docs/SECURITY.md
- Audit report: SECURITY_AUDIT_REPORT.md

**Last Updated:** 2025-10-25

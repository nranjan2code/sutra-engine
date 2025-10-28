# Security Integration Guide - Sutra Storage

**Document Purpose**: Enable existing security features in Sutra storage system  
**Target Audience**: DevOps, Security Engineers, System Administrators  
**Completion Time**: 30-60 minutes  

---

## Overview

The Sutra storage system has comprehensive security features implemented but not enabled by default. This guide walks through enabling authentication, authorization, and encryption for production deployment.

**Security Features Available**:
- ✅ HMAC-SHA256 + JWT authentication  
- ✅ Role-based access control (RBAC)
- ✅ TLS 1.3 encryption
- ✅ Input validation and rate limiting  

**Current Issue**: Main server binary doesn't use `SecureStorageServer` wrapper

---

## Quick Start (5 minutes)

### 1. Generate Authentication Secret

```bash
# Generate secure authentication secret (64 characters)
openssl rand -hex 32 > /etc/sutra/auth-secret

# Set environment variable  
export SUTRA_AUTH_SECRET=$(cat /etc/sutra/auth-secret)
export SUTRA_SECURE_MODE=true
```

### 2. Enable Security Mode

```bash
# For Docker deployment
docker run -d \
  -e SUTRA_SECURE_MODE=true \
  -e SUTRA_AUTH_SECRET="$(cat /etc/sutra/auth-secret)" \
  -p 50051:50051 \
  sutra-storage:latest

# For direct binary
SUTRA_SECURE_MODE=true \
SUTRA_AUTH_SECRET="$(cat /etc/sutra/auth-secret)" \
./target/release/storage-server
```

### 3. Verify Security is Enabled

```bash
# Check server logs for security confirmation
docker logs sutra-storage | grep -E "(🔒|SECURE|Authentication)"

# Expected output:
# 🔒 Initializing authentication...
# ✅ Authentication enabled: HMAC-SHA256
# 🚀 Starting SECURE SINGLE TCP server on 0.0.0.0:50051
```

---

## Complete Security Configuration

### Authentication Setup

#### Environment Variables

```bash
# Required for secure mode
export SUTRA_SECURE_MODE=true
export SUTRA_AUTH_SECRET="your-64-character-secret-key-here-make-it-random-and-secure"

# Optional authentication settings
export SUTRA_AUTH_METHOD=hmac          # or "jwt"  
export SUTRA_TOKEN_TTL_SECONDS=3600    # 1 hour token lifetime
```

#### Generate Production Secret

```bash
#!/bin/bash
# generate-auth-secret.sh

SECRET_FILE="/etc/sutra/auth-secret"
SECRET_LENGTH=64

# Create secure directory
sudo mkdir -p /etc/sutra
sudo chmod 700 /etc/sutra

# Generate cryptographically secure secret
openssl rand -hex $SECRET_LENGTH | sudo tee "$SECRET_FILE" > /dev/null

# Set restrictive permissions
sudo chmod 600 "$SECRET_FILE"
sudo chown sutra:sutra "$SECRET_FILE"

echo "Generated authentication secret: $SECRET_FILE"
echo "Configure environment: export SUTRA_AUTH_SECRET=\$(cat $SECRET_FILE)"
```

#### User Role Configuration

The system supports 4 built-in roles:

```rust
// Built-in roles in packages/sutra-storage/src/auth.rs
pub enum Role {
    Admin,    // Full access - read, write, delete, flush
    Writer,   // Read and write access
    Reader,   // Read-only access  
    Service,  // Service-to-service authentication
}
```

**Default Role Permissions**:

| Operation | Admin | Writer | Reader | Service |
|-----------|-------|---------|---------|---------|
| `read`, `query`, `search` | ✅ | ✅ | ✅ | ✅ |
| `write`, `learn`, `create` | ✅ | ✅ | ❌ | ✅ |
| `delete`, `flush` | ✅ | ❌ | ❌ | ❌ |

### TLS Configuration

#### Generate Development Certificates

```bash
#!/bin/bash  
# generate-tls-certs.sh

CERT_DIR="/etc/sutra/certs"
DAYS_VALID=365

# Create certificate directory
sudo mkdir -p "$CERT_DIR"
cd "$CERT_DIR"

# Generate server private key
sudo openssl genrsa -out server-key.pem 4096

# Generate server certificate (self-signed for development)
sudo openssl req -new -x509 -days $DAYS_VALID \
    -key server-key.pem -out server-cert.pem \
    -subj "/C=US/ST=CA/L=San Francisco/O=Sutra AI/CN=sutra-storage"

# Set secure permissions
sudo chmod 600 server-key.pem
sudo chmod 644 server-cert.pem
sudo chown -R sutra:sutra "$CERT_DIR"

echo "TLS certificates generated in $CERT_DIR"
```

#### Enable TLS

```bash
# TLS environment variables
export SUTRA_TLS_ENABLED=true
export SUTRA_TLS_CERT=/etc/sutra/certs/server-cert.pem
export SUTRA_TLS_KEY=/etc/sutra/certs/server-key.pem

# Optional: Require client certificates (mTLS)
export SUTRA_TLS_CLIENT_AUTH=false
```

### Docker Compose Configuration

```yaml
# docker-compose.yml - Complete security configuration
version: '3.8'

services:
  sutra-storage:
    image: sutra-storage:latest
    container_name: sutra-storage
    ports:
      - "50051:50051"
    environment:
      # Security configuration
      - SUTRA_SECURE_MODE=true
      - SUTRA_AUTH_SECRET=${SUTRA_AUTH_SECRET}
      - SUTRA_AUTH_METHOD=hmac
      - SUTRA_TOKEN_TTL_SECONDS=3600
      
      # TLS configuration
      - SUTRA_TLS_ENABLED=true
      - SUTRA_TLS_CERT=/certs/server-cert.pem
      - SUTRA_TLS_KEY=/certs/server-key.pem
      
      # Storage configuration
      - SUTRA_STORAGE_PATH=/data
      - RUST_LOG=info
    
    volumes:
      - ./data:/data
      - ./certs:/certs:ro
      - ./auth-secret:/run/secrets/auth-secret:ro
    
    secrets:
      - auth-secret
    
    networks:
      - sutra-internal
    
    healthcheck:
      test: ["CMD", "nc", "-z", "localhost", "50051"]
      interval: 30s
      timeout: 5s
      retries: 3
      start_period: 10s
    
    restart: unless-stopped

  # API Gateway with rate limiting
  sutra-api:
    image: sutra-api:latest
    depends_on:
      - sutra-storage
    ports:
      - "8000:8000"
    environment:
      - SUTRA_STORAGE_SERVER=sutra-storage:50051
      - BEHIND_PROXY=false
      - RATE_LIMIT_LEARN=60
      - RATE_LIMIT_QUERY=100
    networks:
      - sutra-internal
      - sutra-external

networks:
  sutra-internal:
    driver: bridge
    internal: true  # No external access
  sutra-external:
    driver: bridge

secrets:
  auth-secret:
    file: ./auth-secret

volumes:
  data:
    driver: local
```

### Environment File (.env)

```bash
# .env file for production deployment

# Security settings
SUTRA_SECURE_MODE=true
SUTRA_AUTH_SECRET=your-production-secret-key-64-characters-long-change-this
SUTRA_AUTH_METHOD=hmac
SUTRA_TOKEN_TTL_SECONDS=3600

# TLS settings  
SUTRA_TLS_ENABLED=true
SUTRA_TLS_CERT=/etc/sutra/certs/server-cert.pem
SUTRA_TLS_KEY=/etc/sutra/certs/server-key.pem
SUTRA_TLS_CLIENT_AUTH=false

# Storage settings
SUTRA_STORAGE_PATH=/var/lib/sutra/data
SUTRA_WAL_FSYNC=true

# API settings
BEHIND_PROXY=true
TRUSTED_PROXIES=10.0.0.0/8,172.16.0.0/12,192.168.0.0/16
RATE_LIMIT_LEARN=60
RATE_LIMIT_QUERY=100
RATE_LIMIT_SEARCH=200

# Logging
RUST_LOG=info
```

---

## Client Authentication

### Generate Authentication Token

```python
# Python client authentication example
import hmac
import hashlib  
import base64
import json
import time

def generate_auth_token(secret_key: str, user_id: str, roles: list) -> str:
    """Generate HMAC authentication token"""
    
    # Create claims
    now = int(time.time())
    claims = {
        "sub": user_id,
        "iat": now,
        "exp": now + 3600,  # 1 hour expiration
        "roles": roles
    }
    
    # Serialize claims  
    payload = json.dumps(claims, separators=(',', ':'))
    payload_b64 = base64.urlsafe_b64encode(payload.encode()).decode().rstrip('=')
    
    # Generate HMAC signature
    signature = hmac.new(
        secret_key.encode(),
        payload_b64.encode(), 
        hashlib.sha256
    ).hexdigest()
    
    # Return token
    return f"{payload_b64}.{signature}"

# Usage example
secret = "your-64-character-secret-key-here"
token = generate_auth_token(secret, "user123", ["Writer"])
print(f"Auth token: {token}")
```

### Client Connection Example

```python
# Authenticated TCP client example
import socket
import msgpack
import struct

class AuthenticatedStorageClient:
    def __init__(self, host: str, port: int, auth_token: str):
        self.host = host
        self.port = port  
        self.auth_token = auth_token
        self.sock = None
    
    def connect(self):
        """Connect and authenticate with storage server"""
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.sock.connect((self.host, self.port))
        
        # Perform authentication handshake
        self._authenticate()
    
    def _authenticate(self):
        """Send authentication token"""
        token_bytes = self.auth_token.encode('utf-8')
        
        # Send token length + token
        self.sock.send(struct.pack('>I', len(token_bytes)))
        self.sock.send(token_bytes)
        
        # Read authentication response
        response = self.sock.recv(1)
        if response != b'\x01':  # 1 = success
            raise Exception("Authentication failed")
    
    def learn_concept(self, content: str, embedding: list = None):
        """Send authenticated learn request"""
        request = {
            "LearnConceptV2": {
                "content": content,
                "embedding": embedding,
                "metadata": {}
            }
        }
        
        # Serialize and send
        payload = msgpack.packb(request)
        self.sock.send(struct.pack('>I', len(payload)))
        self.sock.send(payload)
        
        # Read response
        resp_len = struct.unpack('>I', self.sock.recv(4))[0]
        response = msgpack.unpackb(self.sock.recv(resp_len))
        
        return response

# Usage
secret = "your-secret-key"
token = generate_auth_token(secret, "client123", ["Writer"])

client = AuthenticatedStorageClient("localhost", 50051, token)
client.connect()

result = client.learn_concept("Authenticated knowledge", [0.1] * 768)
print(f"Result: {result}")
```

---

## Grid System Authentication

### Grid Master Configuration

```bash
# Grid authentication environment variables
export GRID_AUTH_SECRET="grid-cluster-secret-64-characters-long-change-this"
export GRID_CLUSTER_ID="production-cluster-001"
```

### Agent Token Generation

```python
# Generate grid agent authentication token
import jwt
import time

def generate_agent_token(secret: str, cluster_id: str, agent_id: str) -> str:
    """Generate JWT token for grid agent"""
    
    now = int(time.time())
    payload = {
        'agent_id': agent_id,
        'cluster_id': cluster_id,
        'roles': ['agent'],
        'exp': now + (24 * 3600),  # 24 hours
        'iat': now
    }
    
    token = jwt.encode(payload, secret, algorithm='HS256')
    return token

# Usage
secret = "grid-cluster-secret-64-characters-long-change-this"
cluster_id = "production-cluster-001"
agent_id = "agent-001"

token = generate_agent_token(secret, cluster_id, agent_id)
print(f"Agent token: {token}")

# Save token for agent configuration
with open('/etc/sutra/grid-agent-token', 'w') as f:
    f.write(token)
```

### Agent Configuration

```toml
# /etc/sutra/grid-agent.toml
[agent]
agent_id = "agent-001"
master_host = "grid-master.internal"
master_port = 7000
auth_token_file = "/etc/sutra/grid-agent-token"

[security]
require_tls = true
validate_master_cert = true
```

---

## Rate Limiting Configuration

### API Rate Limiting

```python
# packages/sutra-api/sutra_api/main.py - Production rate limiting
app.add_middleware(
    RateLimitMiddleware,
    default_limit=100,           # 100 requests per minute default
    window_seconds=60,
    endpoint_limits={
        "/learn": 60,            # Learning operations: 60/min
        "/learn/batch": 30,      # Batch learning: 30/min  
        "/query": 200,           # Queries: 200/min
        "/search": 300,          # Search: 300/min
        "/health": 1000,         # Health checks: unlimited
    },
    trusted_proxies=[
        "10.0.0.0/8",           # Private networks
        "172.16.0.0/12", 
        "192.168.0.0/16",
        "127.0.0.0/8",          # Localhost
    ],
    behind_proxy=True,           # Enable for load balancer deployments
    require_proxy_validation=True, # Enforce proxy validation
)
```

### Network-Level Rate Limiting

```bash
# Using iptables for additional protection
iptables -A INPUT -p tcp --dport 50051 -m connlimit --connlimit-above 10 -j DROP
iptables -A INPUT -p tcp --dport 50051 -m hashlimit --hashlimit-mode srcip --hashlimit 10/min --hashlimit-name storage-limit -j ACCEPT
iptables -A INPUT -p tcp --dport 50051 -j DROP
```

---

## Monitoring and Logging

### Security Event Logging

```toml
# logging configuration - log4rs.yml
appenders:
  security:
    kind: file
    path: "/var/log/sutra/security.log"
    encoder:
      kind: json
      
loggers:
  sutra_storage::auth:
    level: info
    appenders:
      - security
  sutra_storage::secure_tcp_server:
    level: info  
    appenders:
      - security

root:
  level: warn
  appenders:
    - security
```

### Security Metrics

```bash
# Monitor authentication events
tail -f /var/log/sutra/security.log | jq 'select(.level == "WARN" or .level == "ERROR")'

# Count authentication failures
grep "Authentication failed" /var/log/sutra/security.log | wc -l

# Monitor rate limiting
grep "Rate limit exceeded" /var/log/sutra/api.log | tail -10
```

---

## Health Checks and Validation

### Security Health Check Script

```bash
#!/bin/bash
# security-health-check.sh

set -euo pipefail

echo "🔒 Sutra Security Health Check"
echo "=============================="

# Check if secure mode is enabled
if [ "${SUTRA_SECURE_MODE:-false}" != "true" ]; then
    echo "❌ CRITICAL: SUTRA_SECURE_MODE not enabled"
    exit 1
else
    echo "✅ Secure mode: ENABLED"
fi

# Check authentication secret
if [ -z "${SUTRA_AUTH_SECRET:-}" ]; then
    echo "❌ CRITICAL: SUTRA_AUTH_SECRET not set"
    exit 1
elif [ ${#SUTRA_AUTH_SECRET} -lt 32 ]; then
    echo "❌ CRITICAL: SUTRA_AUTH_SECRET too short (${#SUTRA_AUTH_SECRET} < 32)"
    exit 1
else
    echo "✅ Authentication secret: CONFIGURED"
fi

# Check TLS configuration
if [ "${SUTRA_TLS_ENABLED:-false}" == "true" ]; then
    if [ -f "${SUTRA_TLS_CERT:-}" ] && [ -f "${SUTRA_TLS_KEY:-}" ]; then
        echo "✅ TLS encryption: ENABLED"
    else
        echo "⚠️  TLS enabled but certificates not found"
    fi
else
    echo "⚠️  TLS encryption: DISABLED"
fi

# Test storage server connectivity
echo -n "Testing storage server connection... "
if timeout 5 nc -z localhost 50051 2>/dev/null; then
    echo "✅ CONNECTED"
else
    echo "❌ FAILED"
    exit 1
fi

# Test unauthenticated access is blocked  
echo -n "Testing authentication enforcement... "
if timeout 2 nc localhost 50051 < /dev/null 2>/dev/null; then
    echo "❌ UNAUTHENTICATED ACCESS ALLOWED"
    exit 1
else
    echo "✅ AUTHENTICATION REQUIRED"
fi

echo ""
echo "🎉 Security health check passed!"
```

### Automated Security Validation

```python
# security-validation.py
import subprocess
import json
import sys

def check_security_config():
    """Validate security configuration"""
    
    checks = []
    
    # Check environment variables
    checks.append({
        'name': 'Secure Mode Enabled',
        'check': lambda: os.getenv('SUTRA_SECURE_MODE') == 'true',
        'critical': True
    })
    
    checks.append({
        'name': 'Auth Secret Length',  
        'check': lambda: len(os.getenv('SUTRA_AUTH_SECRET', '')) >= 32,
        'critical': True
    })
    
    checks.append({
        'name': 'TLS Enabled',
        'check': lambda: os.getenv('SUTRA_TLS_ENABLED') == 'true', 
        'critical': False
    })
    
    # Run checks
    results = []
    for check in checks:
        try:
            passed = check['check']()
            results.append({
                'name': check['name'],
                'passed': passed,
                'critical': check['critical']
            })
        except Exception as e:
            results.append({
                'name': check['name'],  
                'passed': False,
                'critical': check['critical'],
                'error': str(e)
            })
    
    # Report results
    critical_failures = [r for r in results if not r['passed'] and r['critical']]
    
    if critical_failures:
        print("❌ CRITICAL security issues found:")
        for failure in critical_failures:
            print(f"  - {failure['name']}")
        sys.exit(1)
    else:
        print("✅ Security configuration validated")

if __name__ == '__main__':
    check_security_config()
```

---

## Troubleshooting

### Common Issues

#### 1. Authentication Not Working

**Symptoms**: 
- Clients can connect without authentication
- No security logs generated

**Solutions**:
```bash
# Check environment variables
echo $SUTRA_SECURE_MODE
echo $SUTRA_AUTH_SECRET

# Verify binary is using secure server  
grep -i "SECURE\|Authentication" /var/log/sutra/storage.log

# Restart with explicit configuration
SUTRA_SECURE_MODE=true SUTRA_AUTH_SECRET="$(cat /etc/sutra/auth-secret)" \
./storage-server
```

#### 2. Rate Limiting Not Working

**Symptoms**:
- Unlimited requests getting through
- 429 responses never seen

**Solutions**:
```bash
# Check trusted proxy configuration
grep "trusted_proxies" /app/main.py

# Test direct connection (should hit rate limit)
for i in {1..100}; do curl http://localhost:8000/health; done

# Check logs for rate limiting events
grep -i "rate limit" /var/log/sutra/api.log
```

#### 3. TLS Certificate Issues

**Symptoms**:
- TLS handshake failures  
- Certificate verification errors

**Solutions**:
```bash
# Verify certificate validity
openssl x509 -in /etc/sutra/certs/server-cert.pem -text -noout

# Test TLS connection
openssl s_client -connect localhost:50051 -servername sutra-storage

# Generate new certificates
./scripts/generate-tls-certs.sh /etc/sutra/certs
```

#### 4. Grid Agent Authentication

**Symptoms**:
- Agents can't register with master
- "Authentication failed" in grid master logs

**Solutions**:
```bash
# Check grid authentication configuration
echo $GRID_AUTH_SECRET  
echo $GRID_CLUSTER_ID

# Verify agent token
python -c "
import jwt
token = open('/etc/sutra/grid-agent-token').read().strip()
secret = '$GRID_AUTH_SECRET'  
try:
    payload = jwt.decode(token, secret, algorithms=['HS256'])
    print('Token valid:', payload)
except Exception as e:
    print('Token invalid:', e)
"

# Regenerate agent token
python scripts/generate-agent-token.py agent-001 > /etc/sutra/grid-agent-token
```

### Debug Mode

```bash
# Enable debug logging for security components
export RUST_LOG="sutra_storage::auth=debug,sutra_storage::secure_tcp_server=debug"

# Start server with verbose security logging
./storage-server 2>&1 | grep -E "(🔒|✅|❌|Authentication|TLS)"
```

---

## Production Deployment Checklist

### Pre-Deployment

- [ ] Generate strong authentication secret (≥64 characters)
- [ ] Create TLS certificates (Let's Encrypt or CA-signed)
- [ ] Configure trusted proxy networks
- [ ] Set appropriate rate limits
- [ ] Test authentication workflow
- [ ] Validate certificate chain
- [ ] Review security logs configuration

### Deployment

- [ ] Deploy with `SUTRA_SECURE_MODE=true`
- [ ] Verify authentication is enforced
- [ ] Test TLS connectivity
- [ ] Confirm rate limiting works
- [ ] Check grid authentication (if using distributed mode)
- [ ] Validate monitoring and alerting

### Post-Deployment

- [ ] Run security health check
- [ ] Monitor authentication events
- [ ] Track rate limiting metrics  
- [ ] Review security logs
- [ ] Test incident response procedures
- [ ] Document configuration for operations team

---

## Security Best Practices

### Secret Management

```bash
# Use external secret management
export SUTRA_AUTH_SECRET="$(aws secretsmanager get-secret-value --secret-id sutra/auth --query SecretString --output text)"

# Or use Kubernetes secrets
kubectl create secret generic sutra-auth --from-literal=secret="$(openssl rand -hex 32)"
```

### Network Security

```bash
# Firewall configuration
ufw allow from 10.0.0.0/8 to any port 50051    # Internal networks only
ufw allow from 172.16.0.0/12 to any port 50051
ufw allow from 192.168.0.0/16 to any port 50051
ufw deny 50051                                  # Block external access
```

### Monitoring Integration

```yaml
# Prometheus monitoring
- job_name: 'sutra-security'
  static_configs:
    - targets: ['sutra-storage:9090']
  metrics_path: /security-metrics
```

### Backup and Recovery

```bash
# Backup authentication configuration
tar -czf sutra-security-backup.tar.gz \
    /etc/sutra/auth-secret \
    /etc/sutra/certs/ \
    /etc/sutra/*.toml

# Store backup securely
aws s3 cp sutra-security-backup.tar.gz s3://sutra-security-backups/
```

---

**Document Version**: 1.0  
**Last Updated**: October 28, 2025  
**Next Review**: November 28, 2025
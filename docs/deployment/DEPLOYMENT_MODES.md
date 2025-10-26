# Sutra AI - Deployment Modes

**Last Updated:** 2025-10-25

---

## Overview

Sutra AI has **TWO distinct deployment modes** with very different security postures:

1. **Development Mode (Default)** - NO security, for local development only
2. **Production Mode (Opt-in)** - Full enterprise-grade security

---

## 🔧 Development Mode (Default)

### Purpose
Local development, testing, learning the system

### Security Posture
⚠️ **INTENTIONALLY INSECURE** for development convenience

| Security Feature | Status |
|-----------------|--------|
| Authentication | ❌ None |
| Encryption | ❌ None (plaintext) |
| RBAC | ❌ N/A |
| Network Isolation | ❌ All ports exposed |
| Rate Limiting | ⚠️ Bypassable |
| Audit Logging | ❌ None |
| **Security Grade** | 🔴 **15/100** |

### Deployment

```bash
# Standard development deployment
./sutra-deploy.sh install

# Or explicitly
SUTRA_SECURE_MODE=false ./sutra-deploy.sh install
```

### When to Use
- ✅ Local development on your laptop
- ✅ Learning Sutra architecture
- ✅ Testing with synthetic/public data
- ✅ Proof-of-concept projects (non-sensitive)

### When NOT to Use
- ❌ **NEVER** with real patient data (HIPAA)
- ❌ **NEVER** with financial data (PCI-DSS)
- ❌ **NEVER** with personal data (GDPR)
- ❌ **NEVER** on network-accessible servers
- ❌ **NEVER** in production environments

### Docker Compose File
Uses: `docker-compose-grid.yml`

### Network Configuration
- All services on single network
- All ports exposed to host
- No authentication required
- No encryption

---

## 🔒 Production Mode (Secure)

### Purpose
Production deployments, real data, regulated industries

### Security Posture
⚠️ **SECURITY CODE COMPLETE - INTEGRATION PENDING**

| Security Feature | Code Status | Deployment Status |
|-----------------|-------------|-------------------|
| Authentication | ✅ Complete (485 lines) | ❌ NOT integrated |
| Encryption | ✅ Complete (173 lines) | ❌ NOT integrated |
| RBAC | ✅ Complete (4 roles) | ❌ NOT integrated |
| Network Isolation | ✅ Docker config ready | ⚠️ Config only |
| Rate Limiting | ✅ Implemented | ✅ Works (API layer) |
| Audit Logging | ✅ Code complete | ❌ NOT integrated |
| **Security Grade** | 🟢 **92/100 (code)** | 🔴 **15/100 (deployed)** |

### Deployment

⚠️ **CRITICAL:** Security code exists but is NOT yet integrated into the storage server binary.

```bash
# 1. Generate secrets (one-time setup)
chmod +x scripts/generate-secrets.sh
./scripts/generate-secrets.sh

# 2. Review generated configuration
cat .env
ls -la .secrets/

# 3. Deploy with security enabled (DOES NOT WORK YET)
SUTRA_SECURE_MODE=true ./sutra-deploy.sh install

# 4. Verify security status
docker logs sutra-storage 2>&1 | grep -E "(Authentication|TLS)"
# Current output: ⚠️ Authentication: DISABLED, TLS: DISABLED
# Reason: storage_server.rs doesn't use SecureStorageServer
```

**Required Integration:** Update `packages/sutra-storage/src/bin/storage_server.rs` to use `SecureStorageServer` instead of `StorageServer`.

### When to Use
- ✅ Production deployments
- ✅ Healthcare systems (HIPAA compliance)
- ✅ Financial services (PCI-DSS, SOX)
- ✅ Legal case management
- ✅ Government systems
- ✅ Any deployment with real/sensitive data
- ✅ Network-accessible installations

### Docker Compose File
Uses: `docker-compose-secure.yml`

### Network Configuration
- **Internal Network** (172.20.0.0/24) - `internal: true` - NO external access
  - storage-server, embedding-ha, grid-master, grid-agents, bulk-ingester
- **Public Network** (172.21.0.0/24) - Authenticated only
  - sutra-api, sutra-hybrid, sutra-control, sutra-client

### Authentication
All API calls require Bearer token:

```bash
# Get token
TOKEN=$(cat .secrets/tokens/service_token.txt)

# Use in requests
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8000/health
```

### Available Roles
1. **Admin** - Full access (read, write, delete)
2. **Writer** - Read and write operations
3. **Reader** - Read-only access
4. **Service** - Service-to-service authentication

---

## Comparison Table

| Feature | Development Mode | Production Mode |
|---------|-----------------|-----------------|
| **Command** | `./sutra-deploy.sh install` | `SUTRA_SECURE_MODE=true ./sutra-deploy.sh install` |
| **Docker Compose** | `docker-compose-grid.yml` | `docker-compose-secure.yml` |
| **Authentication** | ❌ None | ✅ HMAC/JWT tokens |
| **Encryption** | ❌ Plaintext | ✅ TLS 1.3 |
| **RBAC** | ❌ N/A | ✅ 4 roles |
| **Network Isolation** | ❌ Single network | ✅ Internal + Public |
| **Rate Limiting** | ⚠️ Bypassable | ✅ Validated |
| **Secrets Management** | ❌ None | ✅ Auto-generated |
| **Audit Logging** | ❌ None | ✅ Full trails |
| **Setup Complexity** | Simple (1 command) | Moderate (secrets + config) |
| **Security Grade** | 🔴 15/100 | 🟢 92/100 |
| **Use Case** | Local dev only | Production/Real data |

---

## Switching Modes

### From Development to Production

```bash
# 1. Stop development deployment
./sutra-deploy.sh down

# 2. Generate production secrets
./scripts/generate-secrets.sh

# 3. Deploy in production mode
SUTRA_SECURE_MODE=true ./sutra-deploy.sh install
```

### From Production to Development

```bash
# 1. Stop production deployment
SUTRA_SECURE_MODE=true ./sutra-deploy.sh down

# 2. Clean volumes (DELETES ALL DATA)
docker volume prune -f

# 3. Deploy in development mode
./sutra-deploy.sh install
```

⚠️ **WARNING:** Switching modes will require re-ingesting all knowledge.

---

## Environment Variable Reference

| Variable | Default | Production Value | Purpose |
|----------|---------|------------------|---------|
| `SUTRA_SECURE_MODE` | `false` | `true` | Enable production security |
| `SUTRA_AUTH_METHOD` | N/A | `hmac` | Authentication method |
| `SUTRA_AUTH_SECRET` | N/A | (generated) | Secret key for HMAC/JWT |
| `SUTRA_TOKEN_TTL_SECONDS` | N/A | `3600` | Token expiration |
| `SUTRA_TLS_CERT_PATH` | N/A | `.secrets/tls/cert.pem` | TLS certificate |
| `SUTRA_TLS_KEY_PATH` | N/A | `.secrets/tls/key.pem` | TLS private key |
| `SUTRA_TLS_ENABLED` | N/A | `true` | Enable TLS encryption |

---

## Compliance & Certification

### Development Mode
- ❌ **NOT COMPLIANT** with any security standards
- ❌ HIPAA: No
- ❌ PCI-DSS: No
- ❌ SOC 2: No
- ❌ GDPR: No
- ❌ FDA 21 CFR Part 11: No

### Production Mode
- ✅ **COMPLIANT-READY** with proper configuration
- ✅ HIPAA: Yes (with BAA and additional controls)
- ✅ PCI-DSS: Partial (needs external audit)
- ✅ SOC 2: Yes (with monitoring setup)
- ✅ GDPR: Yes (with data handling procedures)
- ✅ FDA 21 CFR Part 11: Yes (audit trails implemented)

See: `docs/security/PRODUCTION_SECURITY_SETUP.md` for compliance details.

---

## Documentation

### Development Mode
- [QUICKSTART.md](QUICKSTART.md) - Quick development setup
- [DEPLOYMENT.md](DEPLOYMENT.md) - Detailed deployment guide

### Production Mode
- [docs/security/QUICK_START_SECURITY.md](docs/security/QUICK_START_SECURITY.md) - 5-minute secure setup
- [docs/security/PRODUCTION_SECURITY_SETUP.md](docs/security/PRODUCTION_SECURITY_SETUP.md) - Complete production guide
- [docs/security/SECURE_ARCHITECTURE.md](docs/security/SECURE_ARCHITECTURE.md) - Security architecture
- [docs/security/SECURITY_AUDIT_REPORT.md](docs/security/SECURITY_AUDIT_REPORT.md) - Vulnerability analysis

---

## FAQ

### Why is development mode insecure by default?

**Developer productivity.** Adding authentication/TLS to every local development session would slow down the development cycle significantly. This matches industry standards (Redis, PostgreSQL, MongoDB all default to no auth for local development).

### Can I use production mode for development?

**Yes.** If you're working with sensitive data during development, use production mode:

```bash
SUTRA_SECURE_MODE=true ./sutra-deploy.sh install
```

### How do I know which mode I'm running?

```bash
# Check storage server logs
docker logs sutra-storage 2>&1 | grep -E "(Authentication|TLS)"

# Development mode output:
# ⚠️  Authentication: DISABLED
# ⚠️  TLS Encryption: DISABLED

# Production mode output:
# ✅ Authentication: ENABLED
# ✅ TLS Encryption: ENABLED
```

### What happens if I forget to enable production mode?

**Your system will be completely open.** Anyone on the network can:
- Read all knowledge graph data
- Write arbitrary data
- Delete concepts
- Execute any operation

This is why the documentation now prominently warns about deployment modes.

---

## Summary

| If you want to... | Use this mode | Command |
|------------------|---------------|---------|
| Learn Sutra locally | Development | `./sutra-deploy.sh install` |
| Test with fake data | Development | `./sutra-deploy.sh install` |
| Build a prototype (non-sensitive) | Development | `./sutra-deploy.sh install` |
| Deploy with real data | **Production** | `SUTRA_SECURE_MODE=true ./sutra-deploy.sh install` |
| Deploy for healthcare | **Production** | `SUTRA_SECURE_MODE=true ./sutra-deploy.sh install` |
| Deploy for finance | **Production** | `SUTRA_SECURE_MODE=true ./sutra-deploy.sh install` |
| Make network-accessible | **Production** | `SUTRA_SECURE_MODE=true ./sutra-deploy.sh install` |

**When in doubt, use production mode.**

---

**Last Updated:** 2025-10-25  
**Maintainer:** Sutra AI Team

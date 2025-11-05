# PRODUCTION DEPLOYMENT - November 5, 2025

## 🚀 Breaking Changes (No Backward Compatibility)

**Version**: 3.0.0 → 3.1.0 (Production-Grade Release)

### ⚠️ BREAKING CHANGES

1. **Authentication**: localStorage removed, httpOnly cookies only
2. **gRPC Removed**: Deprecated server.rs deleted, TCP Binary Protocol enforced
3. **Strict Dependencies**: All versions pinned (no ranges allowed)
4. **Quality Gates**: Pre-commit hooks enforce code quality
5. **Bundle Limits**: Build fails if bundle size exceeds limits

---

## 🔒 Security Improvements (MANDATORY)

### 1. httpOnly Cookie Authentication

**Before (INSECURE)**:
```typescript
// ❌ REMOVED: localStorage token storage
localStorage.setItem('sutra_token', response.access_token)
const token = localStorage.getItem('sutra_token')
```

**After (PRODUCTION-GRADE)**:
```typescript
// ✅ httpOnly cookies (immune to XSS)
// Tokens never accessible to JavaScript
// Set automatically by server on login
// Sent automatically with credentials: 'include'
```

**Migration**: NO ACTION REQUIRED
- Client code already updated
- Server sets httpOnly cookies on `/auth/login`
- Automatic token refresh via `/auth/refresh`
- Logout clears cookies server-side

### 2. Security Headers Middleware

**Enabled Automatically**:
```bash
# Development mode (relaxed)
export SUTRA_SECURE_MODE=false

# Production mode (strict)
export SUTRA_SECURE_MODE=true
```

**Headers Added**:
- `Strict-Transport-Security`: HTTPS enforcement (production only)
- `Content-Security-Policy`: XSS/injection prevention
- `X-Frame-Options`: Clickjacking protection
- `X-Content-Type-Options`: MIME sniffing protection
- `X-XSS-Protection`: Legacy XSS protection
- `Referrer-Policy`: Information leakage control
- `Permissions-Policy`: Disable unnecessary features
- Secure cookie attributes (HttpOnly, Secure, SameSite)

### 3. Production Environment Variables

**REQUIRED for production**:
```bash
# Security
export SUTRA_SECURE_MODE=true
export SUTRA_AUTH_SECRET="$(openssl rand -base64 32)"  # Min 32 chars

# CORS (update with your domains)
export ALLOW_ORIGINS="https://app.yourdomain.com,https://control.yourdomain.com"

# Services
export SUTRA_STORAGE_SERVER="localhost:7000"  # TCP Binary Protocol
export SUTRA_EMBEDDING_SERVICE_URL="http://localhost:8888"
```

---

## 🗑️ Removed Components (No Backward Compatibility)

### Deleted Files:
```
✅ packages/sutra-storage/src/server.rs (deprecated gRPC server)
✅ packages/sutra-control/sutra_storage_client/ (gRPC client)
✅ All localStorage usage from client code
```

### Migration Required:
- **sutra-control**: Update to use REST API instead of gRPC
  - Use HTTP endpoints on `:5000`
  - Remove gRPC imports
  - Update connection logic

---

## 📦 Strict Dependency Pinning

### Python (ENFORCED):
```toml
# ✅ CORRECT: Exact versions only
fastapi==0.115.0
uvicorn[standard]==0.30.6
pydantic==2.9.2

# ❌ FORBIDDEN: Version ranges
fastapi>=0.104.0  # Build will fail
uvicorn>=0.24.0   # Build will fail
```

### JavaScript (ENFORCED):
```json
{
  "dependencies": {
    "react": "18.2.0",      // ✅ Exact version
    "@mui/material": "6.1.1" // ✅ Exact version
  }
}
```

### Rust (AUTO-PINNED):
```toml
# Cargo.lock committed to git
# Ensures reproducible builds
```

---

## 🛡️ Quality Gates (ENFORCED)

### 1. Pre-Commit Hooks

**Installation**:
```bash
pip install pre-commit
pre-commit install
pre-commit install --hook-type commit-msg
```

**Automated Checks** (runs before every commit):
- Black (Python formatting)
- isort (Import sorting)
- Flake8 (Python linting)
- Prettier (JS/TS formatting)
- Cargo fmt (Rust formatting)
- Bandit (Python security)
- detect-secrets (Credential scanning)
- File hygiene (whitespace, line endings, large files)
- Conventional commits (commit message format)

**Bypass** (emergencies only):
```bash
git commit --no-verify -m "emergency fix"
```

### 2. CI Validation

**Run locally**:
```bash
./scripts/ci-validate.sh
```

**Checks**:
- Code formatting (Python, JS, Rust)
- Linting (all languages)
- Security scanning (Bandit, Safety, npm audit)
- Secret detection
- Unit tests (all packages)
- Bundle size limits
- Docker image validation

**Build Fails If**:
- Formatting errors
- Linting errors
- Security vulnerabilities (high/critical)
- Tests fail
- Bundle size exceeds limits

### 3. Bundle Size Limits (see `.bundlesizerc`)

```
sutra-client:
  - main: 150KB
  - vendor-react: 200KB
  - vendor-mui: 250KB
  - vendor-graph: 120KB
  - total: 800KB (HARD LIMIT)

sutra-control:
  - vendor: 180KB
  - ui: 220KB
  - charts: 150KB
  - utils: 50KB
  - total: 700KB (HARD LIMIT)
```

**Build fails if exceeded** - no exceptions!

---

## 🚀 Deployment Steps

### 1. Pre-Deployment Validation

```bash
# Run all quality gates
./scripts/ci-validate.sh

# Expected: "✓ All quality gates passed!"
# If failures: Fix before proceeding
```

### 2. Environment Setup

```bash
# Copy environment template
cp .env.example .env.production

# Edit .env.production
SUTRA_SECURE_MODE=true
SUTRA_AUTH_SECRET="<generate-with-openssl>"
ALLOW_ORIGINS="https://app.yourdomain.com"
SUTRA_STORAGE_SERVER="sutra-storage:7000"
```

### 3. Build All Services

```bash
# Build with production edition
export SUTRA_EDITION=enterprise  # or simple/community
./sutra build

# Verify images
docker images | grep sutra
```

### 4. Deploy

```bash
# Deploy with production config
export SUTRA_SECURE_MODE=true
./sutra deploy

# Check health
curl -I https://api.yourdomain.com/health

# Expected headers:
# - Strict-Transport-Security
# - Content-Security-Policy
# - X-Frame-Options: DENY
# - X-Content-Type-Options: nosniff
```

### 5. Verify Authentication

```bash
# Test login (httpOnly cookies)
curl -X POST https://api.yourdomain.com/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"test"}' \
  -c cookies.txt \
  -v

# Check cookies.txt contains:
# - access_token (HttpOnly, Secure, SameSite=Lax)
# - refresh_token (HttpOnly, Secure, SameSite=Lax)

# Test authenticated request
curl https://api.yourdomain.com/auth/me \
  -b cookies.txt

# Test logout
curl -X POST https://api.yourdomain.com/auth/logout \
  -b cookies.txt \
  -c cookies.txt

# Verify cookies cleared
```

### 6. Monitor

```bash
# Check logs
docker logs sutra-api --tail=100 -f
docker logs sutra-storage --tail=100 -f

# Check metrics (self-monitoring)
curl https://api.yourdomain.com/stats

# Watch for errors
docker stats
```

---

## 🔄 Rollback Procedure

### If Critical Issues Found:

```bash
# 1. Stop services
docker-compose down

# 2. Rollback to previous version
git checkout tags/v3.0.0
export SUTRA_VERSION=3.0.0

# 3. Rebuild
./sutra build

# 4. Redeploy
./sutra deploy

# 5. Report issue
# File GitHub issue with `production-grade-release` label
```

---

## ✅ Post-Deployment Checklist

- [ ] All services healthy (`./sutra status`)
- [ ] Security headers present (`curl -I`)
- [ ] httpOnly cookies set on login
- [ ] Token refresh works automatically
- [ ] Logout clears cookies
- [ ] No localStorage usage (DevTools check)
- [ ] HTTPS enforced (if SUTRA_SECURE_MODE=true)
- [ ] Bundle sizes within limits
- [ ] Pre-commit hooks installed
- [ ] CI validation passes
- [ ] Monitoring active (Grid events)
- [ ] Backups configured
- [ ] Team notified

---

## 📞 Support

**Issues**: `production-grade-release` label on GitHub  
**Urgent**: Contact platform team immediately  
**Documentation**: See `PRODUCTION_READINESS_CHECKLIST.md`

---

**Deployment Date**: _____________  
**Deployed By**: _____________  
**Version**: 3.1.0  
**Edition**: _____________  
**Environment**: _____________

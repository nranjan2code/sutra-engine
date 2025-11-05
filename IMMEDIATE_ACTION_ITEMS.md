# Immediate Action Items - Post Production-Grade Fixes

**Date**: November 5, 2025  
**Priority**: HIGH  
**Timeline**: Complete within 1 week

## 🚨 Critical (Do Today)

### 1. Test Authentication Flow
**Why**: We changed token storage from localStorage to httpOnly cookies  
**Risk**: Users may not be able to login/logout

**Test Steps**:
```bash
# 1. Start services
cd /Users/nisheethranjan/Projects/sutra-memory
./sutra deploy

# 2. Test login flow
# - Open http://localhost:3000/login
# - Login with test credentials
# - Verify httpOnly cookie is set (DevTools → Application → Cookies)
# - Verify redirect to home page
# - Verify user info displayed

# 3. Test token refresh
# - Wait for token expiration (or manually delete access token cookie)
# - Make an API request
# - Verify automatic refresh occurs
# - Verify request succeeds after refresh

# 4. Test logout
# - Click logout
# - Verify cookies are cleared
# - Verify redirect to login
# - Verify cannot access protected routes
```

**Expected Behavior**:
- ✅ Login sets httpOnly cookies (not localStorage)
- ✅ Cookies have Secure, HttpOnly, SameSite=Lax flags
- ✅ Refresh token flow works automatically
- ✅ Logout clears cookies server-side

**If Issues Found**: Revert changes in `AuthContext.tsx` and `api.ts`

### 2. Verify Security Headers
**Why**: New security middleware must be active  
**Risk**: Missing headers leave API vulnerable

**Test Steps**:
```bash
# 1. Start API in development mode
export SUTRA_SECURE_MODE=false
./sutra deploy

# 2. Check headers
curl -I http://localhost:8000/health

# Expected (dev mode):
# X-Frame-Options: SAMEORIGIN
# X-Content-Type-Options: nosniff
# Content-Security-Policy: ...
# X-XSS-Protection: 1; mode=block

# 3. Start API in production mode
export SUTRA_SECURE_MODE=true
# Note: Requires HTTPS in production, test with reverse proxy

# 4. Check production headers (if HTTPS available)
curl -I https://localhost:8443/health

# Expected (prod mode):
# Strict-Transport-Security: max-age=31536000; includeSubDomains; preload
# All other headers from dev mode
```

**If Missing**: Check middleware is added in `main.py`

### 3. Run Quick Security Scan
**Why**: Pinned dependencies may have known vulnerabilities  
**Risk**: Deploying with known CVEs

**Test Steps**:
```bash
# Python dependencies
cd packages/sutra-api
pip install safety
safety check --json

# Expected: 0 vulnerabilities (or only low severity)

# JavaScript dependencies (if npm available)
cd ../sutra-client
npm audit

# Expected: 0 high/critical vulnerabilities
```

**If Vulnerabilities Found**:
- High/Critical: Fix immediately by updating affected package
- Medium: Plan fix in next sprint
- Low: Document and review

## ⚠️ Important (This Week)

### 4. Generate Lock Files
**Why**: Ensure reproducible builds across environments  
**Timeline**: By end of week

**Steps**:
```bash
# JavaScript lock file (requires pnpm or npm)
cd /Users/nisheethranjan/Projects/sutra-memory

# Option 1: If pnpm is installed
pnpm install
git add pnpm-lock.yaml

# Option 2: If using npm
npm install
git add package-lock.json

# Python lock file
source venv/bin/activate
pip freeze > requirements-lock.txt
git add requirements-lock.txt

# Rust (already auto-generated)
git add Cargo.lock

# Commit all lock files
git commit -m "chore: add dependency lock files for reproducible builds"
```

### 5. Update Deployment Scripts
**Why**: Security mode should be enabled in production  
**Timeline**: Before next deployment

**Files to Update**:
```bash
# 1. docker-compose.yml or .env
# Add to production environment:
SUTRA_SECURE_MODE=true
SUTRA_AUTH_SECRET=<generate-strong-32-char-secret>

# 2. Generate strong secret
openssl rand -base64 32

# 3. Update CORS origins
# In packages/sutra-api/sutra_api/config.py or environment:
ALLOW_ORIGINS=https://app.yourdomain.com,https://control.yourdomain.com
```

### 6. Test Code Splitting
**Why**: Verify bundle optimization is working  
**Timeline**: Before next release

**Steps**:
```bash
# 1. Build sutra-client
cd packages/sutra-client
npm run build

# 2. Check bundle size
ls -lh dist/assets/js/

# Expected chunks:
# - vendor-react-[hash].js (~150KB)
# - vendor-mui-[hash].js (~200KB)
# - vendor-query-[hash].js (~50KB)
# - vendor-graph-[hash].js (~100KB)
# - vendor-utils-[hash].js (~30KB)
# - main-[hash].js (~50KB)

# Total should be <1MB compressed

# 3. Build sutra-control
cd ../sutra-control
npm run build

# 4. Check bundle size
ls -lh dist/js/

# Expected chunks:
# - vendor-[hash].js
# - ui-[hash].js
# - charts-[hash].js
# - utils-[hash].js

# Total should be <800KB compressed
```

### 7. Review gRPC Usage
**Why**: Know where gRPC is still used before v4.0.0 removal  
**Timeline**: Document by end of week

**Find All gRPC Usage**:
```bash
cd /Users/nisheethranjan/Projects/sutra-memory

# Python gRPC imports
grep -r "import grpc" packages/ --include="*.py"

# Rust gRPC (tonic)
grep -r "use tonic" packages/ --include="*.rs"

# Document each usage
# Create migration plan for sutra-control
```

## 📝 Checklist

Track completion:

- [ ] **Critical #1**: Test authentication flow (login, refresh, logout)
- [ ] **Critical #2**: Verify security headers in dev and prod modes
- [ ] **Critical #3**: Run security scan (Python, JavaScript)
- [ ] **Important #4**: Generate and commit lock files
- [ ] **Important #5**: Update deployment scripts with SUTRA_SECURE_MODE
- [ ] **Important #6**: Test code splitting and bundle sizes
- [ ] **Important #7**: Review and document gRPC usage

## 🆘 Rollback Procedure

If critical issues found:

### Rollback Authentication Changes
```bash
git checkout HEAD~1 -- packages/sutra-client/src/contexts/AuthContext.tsx
git checkout HEAD~1 -- packages/sutra-client/src/services/api.ts
```

### Rollback Security Middleware
```bash
git checkout HEAD~1 -- packages/sutra-api/sutra_api/main.py
rm packages/sutra-api/sutra_api/security_middleware.py
```

### Rollback All Changes
```bash
git reset --hard HEAD~1
# Warning: Loses all uncommitted work
```

## 📞 Support

**Issues**: File GitHub issue with `production-fixes` label  
**Urgent**: Contact platform team immediately  
**Documentation**: See `PRODUCTION_READINESS_CHECKLIST.md`

---

**Created**: November 5, 2025  
**Owner**: Platform Engineering Team  
**Review Date**: November 12, 2025

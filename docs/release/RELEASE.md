# Quick Reference: Release Management

## Check Version
```bash
./sutra-deploy.sh version
```

## Create Release

### Bug Fix (2.0.0 → 2.0.1)
```bash
./sutra-deploy.sh release patch
git push origin main --tags
```

### New Feature (2.0.0 → 2.1.0)
```bash
./sutra-deploy.sh release minor
git push origin main --tags
```

### Breaking Change (2.0.0 → 3.0.0)
```bash
./sutra-deploy.sh release major
git push origin main --tags
```

## Deploy Specific Version
```bash
./sutra-deploy.sh deploy v2.0.1
```

## Customer Deployment
```bash
# Clone repo
git clone https://github.com/YOUR_ORG/sutra-models.git
cd sutra-models

# Checkout version
git checkout v2.0.1

# Configure & deploy
export SUTRA_EDITION=community
export SUTRA_LICENSE_KEY=your-key
export SUTRA_VERSION=2.0.1
./sutra-deploy.sh install
```

## Image Names
```
ghcr.io/YOUR_ORG/sutra-api:2.0.1
ghcr.io/YOUR_ORG/sutra-storage-server:2.0.1
ghcr.io/YOUR_ORG/sutra-hybrid:2.0.1
```

## Rollback
```bash
# Deploy previous version
./sutra-deploy.sh deploy v2.0.0
```

---
**Full docs:** [docs/release/](docs/release/)
- [Complete Release Process](docs/release/RELEASE_PROCESS.md)
- [Release Management Overview](docs/release/README.md)
- [Versioning Strategy](docs/release/VERSIONING_STRATEGY.md)

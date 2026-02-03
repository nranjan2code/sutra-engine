# Release Management

Sutra uses a centralized versioning system with automated builds and deployment workflows.

## Contents

- **[Release Process](RELEASE_PROCESS.md)** - Step-by-step release workflow
- **[Versioning Strategy](VERSIONING_STRATEGY.md)** - Semantic versioning guidelines  
- **[Quick Reference](QUICK_REFERENCE.md)** - Command cheat sheet
- **[Changelog](changelog.md)** - Version history and changes
- **[Setup](SETUP_COMPLETE.md)** - Release infrastructure setup
- **[Standalone Releases](../STANDALONE_RELEASES.md)** - Single-binary engine releases

## Quick Start

### Check Current Version

```bash
./sutra-deploy.sh version
cat VERSION  # 2.0.0
```

### Create New Release

```bash
# Bug fix (2.0.0 → 2.0.1)
./sutra-deploy.sh release patch

# New features (2.0.0 → 2.1.0)
./sutra-deploy.sh release minor

# Breaking changes (2.0.0 → 3.0.0)
./sutra-deploy.sh release major
```

### Push Release

```bash
git push origin main --tags
```

This triggers automated builds on GitHub Actions.

### Deploy Specific Version

```bash
./sutra-deploy.sh deploy v2.0.1
```

## Version Control

### Single Source of Truth

The `VERSION` file at the project root controls all package versions:

```bash
$ cat VERSION
2.0.0
```

All services, Docker images, and packages sync to this version.

### Semantic Versioning

Format: `MAJOR.MINOR.PATCH`

- **MAJOR**: Breaking changes (API incompatible)
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

Examples:
- `2.0.0` → `2.0.1` - Bug fix
- `2.0.0` → `2.1.0` - New feature
- `2.0.0` → `3.0.0` - Breaking change

## Release Workflow

### 1. Development

Make changes in feature branches:

```bash
git checkout -b feature/new-capability
# ... make changes ...
git commit -m "feat: add new capability"
git push origin feature/new-capability
```

### 2. Version Bump

On `main` branch:

```bash
# Determine version type
./sutra-deploy.sh release patch|minor|major

# This updates:
# - VERSION file
# - README.md badge
# - Creates git commit
# - Creates git tag
```

### 3. Push & Build

```bash
git push origin main --tags
```

GitHub Actions automatically:
- Builds all Docker images
- Tags with version (e.g., `v2.0.1`)
- Pushes to container registry
- Creates GitHub release

### 4. Deploy

```bash
# Deploy specific version
./sutra-deploy.sh deploy v2.0.1

# Or use latest
./sutra-deploy.sh deploy latest
```

## Docker Image Tagging

All images follow the version in `VERSION` file:

```bash
# After version 2.0.1 release
sutra-api:v2.0.1
sutra-api:latest
sutra-embedding-service:v2.0.1
sutra-embedding-service:latest
sutra-storage-server:v2.0.1
sutra-storage-server:latest
```

Both version tag and `:latest` are applied.

## Version Files Updated

When you run `./sutra-deploy.sh release <type>`:

1. **VERSION** - Single source of truth
2. **README.md** - Version badge
3. **Git commit** - "chore: bump version to X.Y.Z"
4. **Git tag** - `vX.Y.Z`

## Automated Builds

`.github/workflows/release.yml` triggers on tag push:

```yaml
on:
  push:
    tags:
      - 'v*'
```

Builds all services and pushes to registry.

## Rollback

To rollback to previous version:

```bash
# Deploy older version
./sutra-deploy.sh deploy v2.0.0

# Or revert git tag
git tag -d v2.0.1
git push origin :refs/tags/v2.0.1
```

## Best Practices

### Before Release

- ✅ All tests passing
- ✅ Documentation updated
- ✅ CHANGELOG.md updated
- ✅ Breaking changes documented
- ✅ Migration guide provided (if needed)

### Version Selection

**Patch (X.Y.Z):**
- Bug fixes
- Security patches
- Documentation fixes
- Minor performance improvements

**Minor (X.Y.0):**
- New features
- New API endpoints
- New services
- Deprecations (with backward compatibility)

**Major (X.0.0):**
- Breaking API changes
- Removed deprecated features
- Major architecture changes
- Incompatible data formats

### Release Notes

Always include:
- What's new
- Bug fixes
- Breaking changes (if major)
- Migration guide (if needed)
- Known issues

## CI/CD Integration

### GitHub Actions Workflow

```yaml
name: Release
on:
  push:
    tags:
      - 'v*'
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Build and push images
        run: |
          VERSION=${GITHUB_REF#refs/tags/v}
          ./sutra-optimize.sh build-all
          # Push to registry
```

### Docker Registry

Images are pushed to:
- Docker Hub: `sutra/SERVICE:VERSION`
- GitHub Container Registry: `ghcr.io/nranjan2code/sutra-SERVICE:VERSION`

## Troubleshooting

### Version Mismatch

```bash
# Verify VERSION file
cat VERSION

# Check git tags
git tag -l

# Verify Docker images
docker images | grep sutra
```

### Failed Build

```bash
# Check GitHub Actions
# Visit: https://github.com/nranjan2code/sutra-memory/actions

# Retry locally
SUTRA_VERSION=v2.0.1 ./sutra-optimize.sh build-all
```

### Tag Already Exists

```bash
# Delete local tag
git tag -d v2.0.1

# Delete remote tag
git push origin :refs/tags/v2.0.1

# Create new release
./sutra-deploy.sh release patch
git push origin main --tags
```

## Standalone Binaries

In addition to the full Docker stack, we support releasing the storage engine as a standalone binary for embedded use cases.

See: **[Standalone Releases](../STANDALONE_RELEASES.md)**

## Related Documentation

- [Build Guide](../build/README.md)
- [Deployment Guide](../deployment/README.md)
- [Contributing Guide](../guides/contributing.md)
- [Changelog](changelog.md)

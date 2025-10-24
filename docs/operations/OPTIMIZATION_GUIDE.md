# Production Docker Image Optimization

## Problem Analysis

**Current Image Sizes (Bloated):**
- `sutra-base-rust`: **2.28 GB** ❌ (Full Rust toolchain + build deps)
- `sutra-api`: **601 MB** ❌ (Includes unnecessary build tools)
- `sutra-base-python`: **563 MB** ❌ (gcc, g++, dev headers)
- `sutra-storage-server`: 43.2 MB ✅ (Already optimized)

**Root Causes:**
1. **No multi-stage separation**: Builder tools leak into runtime images
2. **Development dependencies**: gcc, g++, python3-dev, rust toolchain in runtime
3. **No cleanup**: Test files, docs, .pyc files included
4. **Unstripped binaries**: Debug symbols in Rust binaries

## Solution: Production-Optimized Build System

### Multi-Stage Build Strategy

```
┌─────────────────────────────────────────────────┐
│  Stage 1: BUILDER (Large, with dev tools)      │
│  - Full toolchain (gcc, cargo, pip)             │
│  - Build all dependencies                       │
│  - Compile Python wheels                        │
│  - Compile Rust binaries                        │
│  - Strip binaries                               │
│  - Remove tests/docs                            │
└─────────────────┬───────────────────────────────┘
                  │ Copy ONLY artifacts
                  ↓
┌─────────────────────────────────────────────────┐
│  Stage 2: RUNTIME (Minimal, production-only)    │
│  - Alpine base (5 MB)                           │
│  - Runtime libraries ONLY                       │
│  - NO build tools                               │
│  - NO development headers                       │
│  - Stripped binaries only                       │
└─────────────────────────────────────────────────┘
```

### Target Image Sizes

| Service | Current | Target | Savings |
|---------|---------|--------|---------|
| Storage Server | 43.2 MB | 10-15 MB | 65-75% |
| API | 601 MB | 50-80 MB | 87-92% |
| Hybrid | (Not built) | 150-200 MB | N/A |
| Base Python | 563 MB | 50-80 MB | 86-91% |
| Base Rust | 2.28 GB | 8-15 MB | 99.4% |

**Total Savings:** ~3 GB reduced to ~300 MB = **90% reduction**

## Implementation

### New Production Dockerfiles

1. **`base-images/python-prod.Dockerfile`**
   - Alpine base + Python runtime only
   - NO gcc, g++, python3-dev
   - Target: 50-80 MB

2. **`base-images/rust-runtime-prod.Dockerfile`**
   - Alpine base + libgcc only
   - NO Rust toolchain
   - Target: 8-15 MB

3. **`packages/sutra-api/Dockerfile.prod`**
   - Multi-stage: builder → runtime
   - Aggressive cleanup (tests, .pyc, docs)
   - Strip .so files
   - Target: 50-80 MB

4. **`packages/sutra-hybrid/Dockerfile.prod`**
   - Multi-stage with numpy/sklearn
   - Remove dataset files
   - Strip binaries
   - Target: 150-200 MB

5. **`packages/sutra-storage/Dockerfile.prod`**
   - Fully static Rust binary
   - Ultra-minimal runtime
   - Target: 10-15 MB

### Optimization Techniques

#### 1. Multi-Stage Builds
```dockerfile
# Stage 1: Builder (large)
FROM sutra-base-python:latest AS builder
RUN pip install --prefix=/install ...

# Stage 2: Runtime (small)
FROM alpine:3.18
COPY --from=builder /install /usr/local
```

#### 2. Aggressive Cleanup
```dockerfile
RUN find /install -type d -name __pycache__ -exec rm -rf {} + && \
    find /install -type f -name "*.pyc" -delete && \
    find /install -type d -name tests -exec rm -rf {} + && \
    find /install -type f -name "*.so" -exec strip {} + && \
    rm -rf /install/lib/python*/site-packages/numpy/tests
```

#### 3. Minimal Runtime Dependencies
```dockerfile
# ❌ WRONG (Development)
RUN apk add python3 python3-dev gcc g++ musl-dev libffi-dev

# ✅ CORRECT (Production)
RUN apk add python3 curl ca-certificates libffi
```

#### 4. Binary Stripping
```dockerfile
RUN cargo build --release && \
    strip /build/target/release/storage_server_simple
```

## Usage

### Quick Start (Production Build)

```bash
# Option 1: Build production images
./build-prod.sh

# Option 2: Complete cleanup and rebuild
./clean-and-rebuild.sh
```

### Manual Build

```bash
# Build production base images
cd base-images
docker build -f python-prod.Dockerfile -t sutra-base-python:prod .
docker build -f rust-runtime-prod.Dockerfile -t sutra-base-rust-runtime:prod .

# Build production services
docker build -f packages/sutra-api/Dockerfile.prod -t sutra-api:prod .
docker build -f packages/sutra-hybrid/Dockerfile.prod -t sutra-hybrid:prod .
docker build -f packages/sutra-storage/Dockerfile.prod -t sutra-storage-server:prod .

# Tag as latest
docker tag sutra-api:prod sutra-api:latest
docker tag sutra-hybrid:prod sutra-hybrid:latest
docker tag sutra-storage-server:prod sutra-storage-server:latest

# Deploy
./sutra-deploy.sh up
```

### Verification

```bash
# Check image sizes
docker images | grep -E "(sutra-|REPOSITORY)" | sort -k3 -h -r

# Expected output:
# sutra-hybrid:prod          150-200 MB
# sutra-api:prod             50-80 MB
# sutra-storage-server:prod  10-15 MB
```

## Migration Guide

### For Development
```bash
# Use existing build.sh (includes dev tools)
./build.sh
```

### For Production
```bash
# Use new production build
./build-prod.sh

# Or complete cleanup
./clean-and-rebuild.sh
```

### For CI/CD
```yaml
# .github/workflows/build.yml
- name: Build Production Images
  run: ./build-prod.sh

- name: Verify Image Sizes
  run: |
    SIZE=$(docker images sutra-api:prod --format '{{.Size}}' | sed 's/MB//')
    if (( $(echo "$SIZE > 100" | bc -l) )); then
      echo "Image too large: ${SIZE}MB"
      exit 1
    fi
```

## Benefits

✅ **90% smaller images** - 3 GB → 300 MB total  
✅ **Faster deployments** - Less data to transfer  
✅ **Lower storage costs** - Disk and registry savings  
✅ **Improved security** - Smaller attack surface  
✅ **Faster container startup** - Less to load  
✅ **Production-grade** - No dev tools in production  

## What's Removed

### From Python Images
- ❌ gcc, g++, musl-dev
- ❌ python3-dev, libffi-dev, openssl-dev
- ❌ linux-headers, libc6-compat (build-time)
- ❌ __pycache__, *.pyc, *.pyo
- ❌ tests/, test/, *.dist-info/direct_url.json
- ❌ numpy/tests, sklearn/datasets/data

### From Rust Images
- ❌ Entire Rust toolchain (rustc, cargo)
- ❌ rustup, rust-src
- ❌ Build dependencies (gcc, musl-dev, make)
- ❌ Debug symbols (via strip)

### What's Kept
✅ Runtime libraries (libgcc, libffi, libssl3)  
✅ Python interpreter  
✅ Compiled binaries  
✅ Application code  
✅ Essential utilities (curl for health checks)  

## Troubleshooting

### "ImportError: No module named X"
**Cause:** Missing runtime dependency  
**Fix:** Add to runtime stage (not builder)

### "Error loading shared libraries"
**Cause:** Stripped too aggressively or missing runtime lib  
**Fix:** Add library to alpine runtime stage

### "Permission denied"
**Cause:** File ownership issue in multi-stage  
**Fix:** Use `COPY --chown=sutra:sutra` or `RUN chown`

## Maintenance

### Adding New Services
1. Create `Dockerfile.prod` with multi-stage build
2. Use production base images for runtime stage
3. Add aggressive cleanup in builder stage
4. Update `build-prod.sh` to include service

### Updating Base Images
```bash
# Rebuild production bases
cd base-images
docker build -f python-prod.Dockerfile -t sutra-base-python:prod .

# Rebuild all services
./build-prod.sh
```

## References

- **Multi-stage builds:** https://docs.docker.com/build/building/multi-stage/
- **Alpine packages:** https://pkgs.alpinelinux.org/packages
- **Docker best practices:** https://docs.docker.com/develop/dev-best-practices/

## See Also

- `SELF_CONTAINED_BUILD.md` - Self-contained build system
- `build-prod.sh` - Production build script
- `clean-and-rebuild.sh` - Complete cleanup and rebuild
- `DEPLOYMENT.md` - Deployment documentation

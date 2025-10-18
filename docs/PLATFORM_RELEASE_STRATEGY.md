JUST A RESEARCH NOT TO BE IMPLEMENTED RIGHT NOW


# Platform Release Strategy for sutra-storage
## Multi-Platform Build, Package, and Distribution Guide

**Document Version**: 1.0  
**Date**: October 18, 2025  
**Status**: Implementation Guide  
**Related**: [CLOUD_NATIVE_ALPINE_OPTIMIZATIONS.md](./CLOUD_NATIVE_ALPINE_OPTIMIZATIONS.md)

---

## 📋 Table of Contents

1. [Overview](#overview)
2. [Platform Support Matrix](#platform-support-matrix)
3. [Build Strategy](#build-strategy)
4. [Packaging Approaches](#packaging-approaches)
5. [Distribution Channels](#distribution-channels)
6. [Implementation Guide](#implementation-guide)
7. [CI/CD Pipeline](#cicd-pipeline)
8. [Testing & Validation](#testing--validation)
9. [Release Process](#release-process)
10. [Maintenance Guidelines](#maintenance-guidelines)

---

## Overview

### Problem Statement

The [CLOUD_NATIVE_ALPINE_OPTIMIZATIONS.md](./CLOUD_NATIVE_ALPINE_OPTIMIZATIONS.md) document identifies **4 distinct deployment scenarios** requiring different storage layout optimizations:

1. **Alpine x86_64** (Cloud/Kubernetes) - 3-5x performance gain
2. **ARM64 Graviton3** (Cloud) - 5-6x performance gain
3. **Apple Silicon** (Development) - 5-8x performance gain  
4. **Raspberry Pi 5** (Edge/IoT) - 50-100x performance gain

Each platform has unique optimization requirements:
- Different page sizes (4KB, 16KB, 64KB)
- Different SIMD instruction sets (AVX2, SVE, NEON, AMX)
- Different memory alignment requirements
- Platform-specific features (THP, unified memory, compression)

### Solution Approach

We implement a **hybrid packaging strategy** that provides:
- ✅ **Universal build** with runtime platform detection (development/testing)
- ✅ **Platform-optimized builds** for production deployments (maximum performance)
- ✅ **Single codebase** with conditional compilation
- ✅ **Multiple distribution channels** (Docker, PyPI, crates.io, direct binary)

---

## Platform Support Matrix

| Platform | Architecture | OS | Distribution Method | Optimization Level | Priority |
|----------|-------------|-----|---------------------|-------------------|----------|
| **Alpine Linux x86_64** | x86_64 | Alpine Linux 3.21+ | Docker, Binary | High | P0 |
| **Graviton3** | aarch64 | Alpine Linux 3.21+ | Docker, Binary | High | P0 |
| **Graviton4** | aarch64 | Alpine Linux 3.21+ | Docker, Binary | Medium (use G3) | P1 |
| **Apple Silicon** | aarch64 | macOS 13+ | Binary, PyPI | High | P0 |
| **Raspberry Pi 5** | aarch64 | Raspberry Pi OS | Binary, Docker | High | P1 |
| **Generic x86_64** | x86_64 | Linux (glibc) | Docker, Binary | Low | P2 |
| **Generic ARM64** | aarch64 | Linux (glibc) | Docker, Binary | Low | P2 |

**Priority Levels**:
- **P0**: Critical - Full optimization, tested in CI/CD
- **P1**: Important - Optimized, community-tested
- **P2**: Supported - Universal build, fallback optimizations

---

## Build Strategy

### Strategy 1: Single Binary with Runtime Detection (Recommended for Development)

**Pros**:
- ✅ One codebase, simpler maintenance
- ✅ Automatic optimization selection
- ✅ Easier CI/CD setup
- ✅ Flexible deployment

**Cons**:
- ❌ Larger binary size (includes all platform code)
- ❌ Slight runtime overhead for detection
- ❌ Cannot use compile-time optimizations fully

**Use Cases**:
- Development and testing
- Docker images with `linux/amd64,linux/arm64` multi-arch
- Quick deployments where binary size isn't critical

---

### Strategy 2: Multiple Build Targets (Recommended for Production)

**Pros**:
- ✅ Maximum performance (compile-time optimization)
- ✅ Smaller binaries (only relevant code)
- ✅ Platform-specific compiler flags
- ✅ Cleaner code separation

**Cons**:
- ❌ Multiple CI/CD pipelines
- ❌ More complex release process
- ❌ Need to maintain multiple build configurations

**Use Cases**:
- Production deployments
- Performance-critical workloads
- Edge devices with storage constraints

---

### Strategy 3: Hybrid Approach (Recommended Overall)

**Implementation**:
- **Universal build** (`sutra-storage:universal`) for development/testing
- **Platform-optimized builds** (`sutra-storage:alpine-x86`, `sutra-storage:graviton3`, etc.) for production
- **Automatic deployment** using Kubernetes node selectors

**Benefits**:
- ✅ Best of both worlds
- ✅ Developer-friendly (universal build)
- ✅ Performance-optimized (production builds)
- ✅ Flexible deployment options

---

## Packaging Approaches

### 1. Cargo Features for Conditional Compilation

Create platform-specific feature flags in `Cargo.toml`:

```toml
# packages/sutra-storage/Cargo.toml
[features]
default = ["universal"]

# Universal optimizations (work everywhere)
universal = []

# Platform-specific optimizations
alpine-x86 = ["universal", "mimalloc", "avx2"]
graviton3 = ["universal", "mimalloc", "sve", "lse"]
graviton4 = ["universal", "mimalloc", "neon", "lse"]
apple-silicon = ["universal", "mimalloc", "accelerate"]
raspberry-pi5 = ["universal", "mimalloc", "compression", "neon"]

# Component features
avx2 = []
sve = []
neon = []
lse = []
accelerate = []
compression = []
metal = []

[target.'cfg(target_os = "macos")'.dependencies]
accelerate-src = { version = "0.3", optional = true }
metal = { version = "0.27", optional = true }

[dependencies]
mimalloc = { version = "0.1", optional = true, default-features = false }
# ... existing dependencies
```

### 2. Platform Detection Module

Create `packages/sutra-storage/src/platform.rs`:

```rust
/// Platform detection and optimization configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    AlpineX86,      // Alpine Linux x86_64 (Cloud)
    Graviton3,      // AWS Graviton3 r7g (ARM64, 256-bit SVE)
    Graviton4,      // AWS Graviton4 r8g (ARM64, 128-bit SVE)
    AppleSilicon,   // macOS M1/M2/M3/M4
    RaspberryPi5,   // Raspberry Pi 5 (ARM64)
    GenericX86,     // Generic x86_64 Linux
    GenericARM64,   // Generic ARM64 Linux
}

#[derive(Debug, Clone)]
pub struct PlatformConfig {
    pub platform: Platform,
    pub page_size: usize,
    pub cache_line_size: usize,
    pub simd_alignment: usize,
    pub use_thp: bool,
    pub use_compression: bool,
    pub optimization_level: OptimizationLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationLevel {
    Maximum,    // Platform-specific build
    High,       // Universal build with runtime optimizations
    Baseline,   // Fallback, no special optimizations
}

impl Platform {
    /// Detect current platform at runtime
    pub fn detect() -> Self {
        #[cfg(target_os = "macos")]
        {
            if Self::is_apple_silicon() {
                return Platform::AppleSilicon;
            }
        }
        
        #[cfg(all(target_arch = "x86_64", target_env = "musl"))]
        {
            return Platform::AlpineX86;
        }
        
        #[cfg(all(target_arch = "x86_64", target_env = "gnu"))]
        {
            return Platform::GenericX86;
        }
        
        #[cfg(all(target_arch = "aarch64", target_os = "linux"))]
        {
            if Self::is_graviton3() {
                return Platform::Graviton3;
            }
            if Self::is_graviton4() {
                return Platform::Graviton4;
            }
            if Self::is_raspberry_pi5() {
                return Platform::RaspberryPi5;
            }
            return Platform::GenericARM64;
        }
        
        Platform::GenericX86
    }
    
    /// Get platform configuration
    pub fn config(self) -> PlatformConfig {
        match self {
            Platform::AlpineX86 => PlatformConfig {
                platform: self,
                page_size: 4096,
                cache_line_size: 64,
                simd_alignment: 32,  // AVX2
                use_thp: true,
                use_compression: false,
                optimization_level: OptimizationLevel::Maximum,
            },
            Platform::Graviton3 => PlatformConfig {
                platform: self,
                page_size: 65536,  // 64KB pages
                cache_line_size: 64,
                simd_alignment: 32,  // SVE 256-bit
                use_thp: true,
                use_compression: false,
                optimization_level: OptimizationLevel::Maximum,
            },
            Platform::Graviton4 => PlatformConfig {
                platform: self,
                page_size: 65536,
                cache_line_size: 64,
                simd_alignment: 16,  // SVE regressed to 128-bit
                use_thp: true,
                use_compression: false,
                optimization_level: OptimizationLevel::High,
            },
            Platform::AppleSilicon => PlatformConfig {
                platform: self,
                page_size: 16384,  // 16KB pages on macOS
                cache_line_size: 64,
                simd_alignment: 64,  // AMX tiles
                use_thp: true,
                use_compression: false,
                optimization_level: OptimizationLevel::Maximum,
            },
            Platform::RaspberryPi5 => PlatformConfig {
                platform: self,
                page_size: 4096,
                cache_line_size: 64,
                simd_alignment: 16,  // NEON
                use_thp: false,  // Can cause OOM
                use_compression: true,  // Critical for bandwidth
                optimization_level: OptimizationLevel::Maximum,
            },
            Platform::GenericX86 | Platform::GenericARM64 => PlatformConfig {
                platform: self,
                page_size: 4096,
                cache_line_size: 64,
                simd_alignment: 16,
                use_thp: false,
                use_compression: false,
                optimization_level: OptimizationLevel::Baseline,
            },
        }
    }
    
    // Platform detection helpers
    #[cfg(target_os = "macos")]
    fn is_apple_silicon() -> bool {
        use std::process::Command;
        Command::new("sysctl")
            .arg("-n")
            .arg("machdep.cpu.brand_string")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.contains("Apple"))
            .unwrap_or(false)
    }
    
    #[cfg(all(target_arch = "aarch64", target_os = "linux"))]
    fn is_graviton3() -> bool {
        std::fs::read_to_string("/proc/cpuinfo")
            .ok()
            .map(|s| s.contains("Neoverse-V1"))
            .unwrap_or(false)
    }
    
    #[cfg(all(target_arch = "aarch64", target_os = "linux"))]
    fn is_graviton4() -> bool {
        std::fs::read_to_string("/proc/cpuinfo")
            .ok()
            .map(|s| s.contains("Neoverse-V2"))
            .unwrap_or(false)
    }
    
    #[cfg(all(target_arch = "aarch64", target_os = "linux"))]
    fn is_raspberry_pi5() -> bool {
        std::fs::read_to_string("/proc/device-tree/model")
            .ok()
            .map(|s| s.contains("Raspberry Pi 5"))
            .unwrap_or(false)
    }
}
```

### 3. Conditional Vector Operations

Create `packages/sutra-storage/src/vector_ops.rs` with runtime dispatch:

```rust
/// High-level vector operations with platform dispatch
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    // Compile-time dispatch for platform-specific builds
    #[cfg(all(feature = "apple-silicon", target_os = "macos"))]
    {
        return vector_ops_apple::cosine_similarity_accelerate(a, b);
    }
    
    #[cfg(all(feature = "alpine-x86", target_arch = "x86_64", target_feature = "avx2"))]
    {
        return vector_ops_x86::cosine_similarity_avx2(a, b);
    }
    
    #[cfg(all(feature = "graviton3", target_arch = "aarch64", target_feature = "sve"))]
    {
        return vector_ops_arm::cosine_similarity_sve(a, b);
    }
    
    #[cfg(all(feature = "raspberry-pi5", target_arch = "aarch64"))]
    {
        return vector_ops_pi::cosine_similarity_compressed(a, b);
    }
    
    // Runtime dispatch for universal build
    #[cfg(feature = "universal")]
    {
        use crate::platform::Platform;
        match Platform::detect() {
            Platform::AppleSilicon => {
                #[cfg(target_os = "macos")]
                return vector_ops_apple::cosine_similarity_accelerate(a, b);
            }
            Platform::AlpineX86 | Platform::GenericX86 => {
                #[cfg(target_arch = "x86_64")]
                if is_x86_feature_detected!("avx2") {
                    return unsafe { vector_ops_x86::cosine_similarity_avx2(a, b) };
                }
            }
            Platform::Graviton3 => {
                #[cfg(target_arch = "aarch64")]
                return vector_ops_arm::cosine_similarity_neon(a, b);
            }
            _ => {}
        }
    }
    
    // Fallback to scalar
    cosine_similarity_scalar(a, b)
}

fn cosine_similarity_scalar(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    dot / (norm_a * norm_b)
}
```

---

## Distribution Channels

### 1. Docker Images (Primary)

**Universal Multi-Arch Image**:
```dockerfile
# packages/sutra-storage/Dockerfile.universal
FROM rust:1.83-alpine AS builder

RUN apk add --no-cache musl-dev protobuf-dev

WORKDIR /build
COPY . .

# Build with universal features
RUN cargo build --release --bin storage_server_simple \
    --features universal

FROM alpine:3.21
RUN apk add --no-cache ca-certificates libgcc
COPY --from=builder /build/target/release/storage_server_simple /app/storage_server
EXPOSE 50051
CMD ["/app/storage_server"]
```

**Platform-Optimized Images**:

**Alpine x86_64**:
```dockerfile
# packages/sutra-storage/Dockerfile.alpine-x86
FROM rust:1.83-alpine AS builder

RUN apk add --no-cache musl-dev protobuf-dev mimalloc-dev

WORKDIR /build
COPY . .

ENV RUSTFLAGS="-C target-cpu=x86-64-v3 -C target-feature=+avx2"
RUN cargo build --release --bin storage_server_simple \
    --features alpine-x86

FROM alpine:3.21
RUN apk add --no-cache ca-certificates libgcc mimalloc
COPY --from=builder /build/target/release/storage_server_simple /app/storage_server
EXPOSE 50051
CMD ["/app/storage_server"]
```

**Graviton3 ARM64**:
```dockerfile
# packages/sutra-storage/Dockerfile.graviton3
FROM rust:1.83-alpine AS builder

RUN apk add --no-cache musl-dev protobuf-dev mimalloc-dev

WORKDIR /build
COPY . .

ENV RUSTFLAGS="-C target-cpu=neoverse-v1 -C target-feature=+sve,+lse"
RUN cargo build --release --bin storage_server_simple \
    --target aarch64-unknown-linux-musl \
    --features graviton3

FROM alpine:3.21
RUN apk add --no-cache ca-certificates libgcc mimalloc
COPY --from=builder /build/target/aarch64-unknown-linux-musl/release/storage_server_simple /app/storage_server
EXPOSE 50051
CMD ["/app/storage_server"]
```

**Raspberry Pi 5**:
```dockerfile
# packages/sutra-storage/Dockerfile.pi5
FROM rust:1.83-slim AS builder

RUN apt-get update && apt-get install -y \
    build-essential \
    protobuf-compiler \
    libmimalloc-dev

WORKDIR /build
COPY . .

ENV RUSTFLAGS="-C target-cpu=cortex-a76 -C target-feature=+neon"
RUN cargo build --release --bin storage_server_simple \
    --target aarch64-unknown-linux-gnu \
    --features raspberry-pi5

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libmimalloc2 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/aarch64-unknown-linux-gnu/release/storage_server_simple /app/storage_server
EXPOSE 50051
CMD ["/app/storage_server"]
```

### 2. PyPI Package (Python Bindings)

Use `maturin` for platform-specific wheels:

```toml
# pyproject.toml (add to existing)
[tool.maturin]
features = ["pyo3/extension-module"]

# Platform-specific targets
[[tool.maturin.target]]
name = "x86_64-unknown-linux-musl"
features = ["alpine-x86"]

[[tool.maturin.target]]
name = "aarch64-unknown-linux-musl"
features = ["graviton3"]

[[tool.maturin.target]]
name = "aarch64-apple-darwin"
features = ["apple-silicon"]

[[tool.maturin.target]]
name = "aarch64-unknown-linux-gnu"
features = ["raspberry-pi5"]
```

Build script for wheels:
```bash
#!/bin/bash
# scripts/build-wheels.sh
set -e

echo "Building platform-specific Python wheels..."

# Alpine x86_64
maturin build --release \
  --target x86_64-unknown-linux-musl \
  --features alpine-x86 \
  -o dist/

# Graviton3 ARM64
maturin build --release \
  --target aarch64-unknown-linux-musl \
  --features graviton3 \
  -o dist/

# Apple Silicon
maturin build --release \
  --target aarch64-apple-darwin \
  --features apple-silicon \
  -o dist/

echo "✅ All wheels built in dist/"
ls -lh dist/*.whl
```

### 3. Direct Binary Distribution

For edge devices and direct installation:

```bash
#!/bin/bash
# scripts/build-binaries.sh
set -e

VERSION=${VERSION:-$(git describe --tags --always)}
OUTPUT_DIR="releases/${VERSION}"
mkdir -p "$OUTPUT_DIR"

echo "Building sutra-storage binaries v${VERSION}..."

# Alpine x86_64
cargo build --release \
  --target x86_64-unknown-linux-musl \
  --features alpine-x86
cp target/x86_64-unknown-linux-musl/release/storage_server_simple \
  "$OUTPUT_DIR/sutra-storage-${VERSION}-alpine-x86_64"

# Graviton3 ARM64
cargo build --release \
  --target aarch64-unknown-linux-musl \
  --features graviton3
cp target/aarch64-unknown-linux-musl/release/storage_server_simple \
  "$OUTPUT_DIR/sutra-storage-${VERSION}-graviton3-aarch64"

# Apple Silicon
cargo build --release \
  --target aarch64-apple-darwin \
  --features apple-silicon
cp target/aarch64-apple-darwin/release/storage_server_simple \
  "$OUTPUT_DIR/sutra-storage-${VERSION}-apple-aarch64"

# Raspberry Pi 5
cargo build --release \
  --target aarch64-unknown-linux-gnu \
  --features raspberry-pi5
cp target/aarch64-unknown-linux-gnu/release/storage_server_simple \
  "$OUTPUT_DIR/sutra-storage-${VERSION}-pi5-aarch64"

echo "✅ Binaries built in $OUTPUT_DIR"
ls -lh "$OUTPUT_DIR"
```

---

## Implementation Guide

### Step 1: Update Cargo.toml

Add platform features and dependencies:

```bash
cd packages/sutra-storage
```

Modify `Cargo.toml` as shown in [Packaging Approaches](#1-cargo-features-for-conditional-compilation).

### Step 2: Create Platform Detection Module

```bash
cat > src/platform.rs << 'EOF'
// Platform detection and configuration
// (Full code from section above)
EOF
```

Update `src/lib.rs`:
```rust
pub mod platform;
pub use platform::{Platform, PlatformConfig};
```

### Step 3: Refactor Vector Operations

```bash
# Create platform-specific modules
mkdir -p src/vector_ops
touch src/vector_ops/mod.rs
touch src/vector_ops/scalar.rs
touch src/vector_ops/x86.rs
touch src/vector_ops/arm.rs
touch src/vector_ops/apple.rs
touch src/vector_ops/pi.rs
```

Implement conditional dispatch in `src/vector_ops/mod.rs`.

### Step 4: Create Dockerfiles

```bash
cd packages/sutra-storage
touch Dockerfile.universal
touch Dockerfile.alpine-x86
touch Dockerfile.graviton3
touch Dockerfile.pi5
```

Add content from [Distribution Channels](#1-docker-images-primary) section.

### Step 5: Update Build Scripts

Modify `build-images.sh` at repo root:

```bash
#!/bin/bash
set -e

echo "🚀 Building Sutra Storage Docker images..."
cd "$(dirname "$0")"

# Enable Docker Buildx
docker buildx create --name sutra-builder --use || true
docker buildx inspect --bootstrap

VERSION="${VERSION:-latest}"

# Universal multi-arch build
echo "📦 Building universal multi-arch image..."
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -t sutra-storage:${VERSION} \
  -t sutra-storage:universal \
  --build-arg FEATURES="universal" \
  -f packages/sutra-storage/Dockerfile.universal \
  packages/sutra-storage

# Platform-optimized builds
echo "📦 Building platform-optimized images..."

# Alpine x86_64
docker build \
  -t sutra-storage:${VERSION}-alpine-x86 \
  -f packages/sutra-storage/Dockerfile.alpine-x86 \
  packages/sutra-storage

# Graviton3 ARM64
docker buildx build \
  --platform linux/arm64 \
  -t sutra-storage:${VERSION}-graviton3 \
  -f packages/sutra-storage/Dockerfile.graviton3 \
  packages/sutra-storage

# Raspberry Pi 5
docker buildx build \
  --platform linux/arm64 \
  -t sutra-storage:${VERSION}-pi5 \
  -f packages/sutra-storage/Dockerfile.pi5 \
  packages/sutra-storage

echo "✅ All images built successfully!"
docker images | grep sutra-storage
```

### Step 6: Create Build Matrix Scripts

```bash
mkdir -p scripts
cat > scripts/build-all-platforms.sh << 'EOF'
#!/bin/bash
# Build all platform variants
set -e

VERSION=${VERSION:-$(git describe --tags --always)}
echo "Building sutra-storage ${VERSION} for all platforms..."

# Build Docker images
./build-images.sh

# Build Python wheels (if maturin installed)
if command -v maturin &> /dev/null; then
    ./scripts/build-wheels.sh
fi

# Build direct binaries
./scripts/build-binaries.sh

echo "✅ All platform builds complete!"
EOF

chmod +x scripts/build-all-platforms.sh
```

---

## CI/CD Pipeline

### GitHub Actions Workflow

Create `.github/workflows/platform-builds.yml`:

```yaml
name: Multi-Platform Build

on:
  push:
    branches: [main, develop]
    tags: ['v*']
  pull_request:
    branches: [main]

jobs:
  # Job 1: Build and test on each platform
  build-matrix:
    name: Build ${{ matrix.platform }}
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        include:
          # Alpine x86_64
          - platform: alpine-x86
            os: ubuntu-latest
            target: x86_64-unknown-linux-musl
            features: alpine-x86
            
          # Graviton3 (cross-compile)
          - platform: graviton3
            os: ubuntu-latest
            target: aarch64-unknown-linux-musl
            features: graviton3
            
          # Apple Silicon
          - platform: apple-silicon
            os: macos-14  # M1 runner
            target: aarch64-apple-darwin
            features: apple-silicon
            
          # Raspberry Pi 5 (cross-compile)
          - platform: raspberry-pi5
            os: ubuntu-latest
            target: aarch64-unknown-linux-gnu
            features: raspberry-pi5

    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      
      - name: Install cross-compilation tools
        if: matrix.os == 'ubuntu-latest'
        run: |
          sudo apt-get update
          sudo apt-get install -y protobuf-compiler
          cargo install cross --git https://github.com/cross-rs/cross
      
      - name: Build
        working-directory: packages/sutra-storage
        run: |
          if [ "${{ matrix.os }}" == "ubuntu-latest" ]; then
            cross build --release --target ${{ matrix.target }} --features ${{ matrix.features }}
          else
            cargo build --release --target ${{ matrix.target }} --features ${{ matrix.features }}
          fi
      
      - name: Run tests
        working-directory: packages/sutra-storage
        run: |
          if [ "${{ matrix.os }}" == "macos-14" ]; then
            cargo test --release --target ${{ matrix.target }} --features ${{ matrix.features }}
          fi
      
      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: sutra-storage-${{ matrix.platform }}
          path: packages/sutra-storage/target/${{ matrix.target }}/release/storage_server_simple

  # Job 2: Build Docker images
  docker-images:
    name: Build Docker Images
    runs-on: ubuntu-latest
    needs: build-matrix
    if: github.event_name == 'push'
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3
      
      - name: Login to Docker Hub
        uses: docker/login-action@v3
        with:
          username: ${{ secrets.DOCKER_USERNAME }}
          password: ${{ secrets.DOCKER_PASSWORD }}
      
      - name: Build and push images
        run: |
          VERSION=${GITHUB_REF#refs/tags/v}
          if [[ ! "$VERSION" =~ ^[0-9] ]]; then
            VERSION=latest
          fi
          export VERSION
          ./build-images.sh
          
          # Push to registry
          for tag in universal alpine-x86 graviton3 pi5; do
            docker tag sutra-storage:${VERSION}-${tag} sutraai/sutra-storage:${VERSION}-${tag}
            docker push sutraai/sutra-storage:${VERSION}-${tag}
          done

  # Job 3: Build Python wheels
  python-wheels:
    name: Build Python Wheels
    runs-on: ${{ matrix.os }}
    needs: build-matrix
    strategy:
      matrix:
        os: [ubuntu-latest, macos-14]
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Install maturin
        run: pip install maturin
      
      - name: Build wheels
        working-directory: packages/sutra-storage
        run: |
          if [ "${{ matrix.os }}" == "ubuntu-latest" ]; then
            maturin build --release --target x86_64-unknown-linux-musl --features alpine-x86 -o ../../dist/
            maturin build --release --target aarch64-unknown-linux-musl --features graviton3 -o ../../dist/
          else
            maturin build --release --target aarch64-apple-darwin --features apple-silicon -o ../../dist/
          fi
      
      - name: Upload wheels
        uses: actions/upload-artifact@v4
        with:
          name: wheels-${{ matrix.os }}
          path: dist/*.whl

  # Job 4: Create release
  release:
    name: Create Release
    runs-on: ubuntu-latest
    needs: [docker-images, python-wheels]
    if: startsWith(github.ref, 'refs/tags/v')
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Download artifacts
        uses: actions/download-artifact@v4
        with:
          path: artifacts
      
      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: artifacts/**/*
          body: |
            ## Platform-Specific Builds
            
            - Alpine x86_64: Optimized for cloud/Kubernetes
            - Graviton3: Optimized for AWS ARM instances
            - Apple Silicon: Optimized for M1/M2/M3/M4
            - Raspberry Pi 5: Optimized for edge/IoT
            
            See [PLATFORM_RELEASE_STRATEGY.md](docs/PLATFORM_RELEASE_STRATEGY.md) for details.
```

---

## Testing & Validation

### Platform-Specific Tests

Create `packages/sutra-storage/tests/platform_tests.rs`:

```rust
#[cfg(test)]
mod platform_tests {
    use sutra_storage::platform::{Platform, OptimizationLevel};

    #[test]
    fn test_platform_detection() {
        let platform = Platform::detect();
        println!("Detected platform: {:?}", platform);
        
        let config = platform.config();
        assert!(config.page_size > 0);
        assert!(config.cache_line_size > 0);
    }
    
    #[test]
    fn test_page_alignment() {
        let config = Platform::detect().config();
        let size = 10_000_000;
        let aligned = (size + config.page_size - 1) & !(config.page_size - 1);
        assert_eq!(aligned % config.page_size, 0);
    }
    
    #[test]
    #[cfg(feature = "alpine-x86")]
    fn test_alpine_x86_optimization() {
        let config = Platform::AlpineX86.config();
        assert_eq!(config.optimization_level, OptimizationLevel::Maximum);
        assert_eq!(config.page_size, 4096);
        assert_eq!(config.simd_alignment, 32); // AVX2
        assert!(config.use_thp);
    }
    
    #[test]
    #[cfg(feature = "graviton3")]
    fn test_graviton3_optimization() {
        let config = Platform::Graviton3.config();
        assert_eq!(config.page_size, 65536); // 64KB pages
        assert_eq!(config.simd_alignment, 32); // SVE 256-bit
    }
}
```

### Performance Benchmarks

Create `packages/sutra-storage/benches/platform_bench.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sutra_storage::vector_ops::cosine_similarity;

fn bench_cosine_similarity(c: &mut Criterion) {
    let a: Vec<f32> = (0..1536).map(|i| i as f32).collect();
    let b: Vec<f32> = (0..1536).map(|i| (i * 2) as f32).collect();
    
    c.bench_function("cosine_similarity_1536d", |bencher| {
        bencher.iter(|| {
            cosine_similarity(black_box(&a), black_box(&b))
        });
    });
}

criterion_group!(benches, bench_cosine_similarity);
criterion_main!(benches);
```

---

## Release Process

### 1. Pre-Release Checklist

- [ ] All tests passing on all platforms
- [ ] Benchmarks show expected performance gains
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Version bumped in Cargo.toml

### 2. Version Tagging

```bash
# Update version
VERSION="0.2.0"
sed -i '' "s/^version = .*/version = \"${VERSION}\"/" packages/sutra-storage/Cargo.toml

# Commit and tag
git add packages/sutra-storage/Cargo.toml
git commit -m "chore: bump sutra-storage to v${VERSION}"
git tag -a "v${VERSION}" -m "Release v${VERSION}"
git push origin main --tags
```

### 3. Build and Publish

```bash
# Build all platforms
VERSION=v${VERSION} ./scripts/build-all-platforms.sh

# Publish to Docker Hub
docker push sutraai/sutra-storage:${VERSION}-alpine-x86
docker push sutraai/sutra-storage:${VERSION}-graviton3
docker push sutraai/sutra-storage:${VERSION}-apple-silicon
docker push sutraai/sutra-storage:${VERSION}-pi5

# Publish Python wheels to PyPI
maturin upload dist/*.whl
```

### 4. Deploy to Production

**Kubernetes with Node Selectors**:

```yaml
# k8s/sutra-storage-optimized.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: sutra-storage-x86
  namespace: sutra-ai
spec:
  replicas: 3
  selector:
    matchLabels:
      app: sutra-storage
      platform: x86
  template:
    metadata:
      labels:
        app: sutra-storage
        platform: x86
    spec:
      nodeSelector:
        kubernetes.io/arch: amd64
      containers:
      - name: sutra-storage
        image: sutraai/sutra-storage:v0.2.0-alpine-x86
        resources:
          requests:
            memory: "4Gi"
            cpu: "2"
          limits:
            memory: "8Gi"
            cpu: "4"
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: sutra-storage-graviton
  namespace: sutra-ai
spec:
  replicas: 3
  selector:
    matchLabels:
      app: sutra-storage
      platform: graviton
  template:
    metadata:
      labels:
        app: sutra-storage
        platform: graviton
    spec:
      nodeSelector:
        kubernetes.io/arch: arm64
        node.kubernetes.io/instance-type: r7g.2xlarge  # Graviton3
      containers:
      - name: sutra-storage
        image: sutraai/sutra-storage:v0.2.0-graviton3
        resources:
          requests:
            memory: "4Gi"
            cpu: "2"
          limits:
            memory: "8Gi"
            cpu: "4"
```

---

## Maintenance Guidelines

### 1. Adding New Platform Support

When adding a new platform (e.g., RISC-V, future ARM versions):

1. Add new platform variant to `Platform` enum
2. Add detection logic in `Platform::detect()`
3. Add configuration in `Platform::config()`
4. Create platform-specific Dockerfile
5. Add to CI/CD build matrix
6. Update documentation

### 2. Updating Optimizations

When research identifies new optimizations:

1. Implement in platform-specific module
2. Add feature flag if needed
3. Update benchmarks
4. Document expected gains
5. Test on actual hardware

### 3. Version Compatibility

Maintain backward compatibility:
- Universal build always available
- Platform-specific builds optional
- Graceful fallback to baseline

### 4. Documentation

Keep these documents synchronized:
- `CLOUD_NATIVE_ALPINE_OPTIMIZATIONS.md` - Research findings
- `PLATFORM_RELEASE_STRATEGY.md` - This document
- `README.md` - User-facing guide
- `CHANGELOG.md` - Version history

---

## Quick Reference

### Build Commands

```bash
# Universal build (development)
cargo build --release --features universal

# Alpine x86_64 (production)
cargo build --release --features alpine-x86 --target x86_64-unknown-linux-musl

# Graviton3 (production)
cargo build --release --features graviton3 --target aarch64-unknown-linux-musl

# Apple Silicon (development)
cargo build --release --features apple-silicon

# Raspberry Pi 5 (edge)
cargo build --release --features raspberry-pi5 --target aarch64-unknown-linux-gnu
```

### Docker Commands

```bash
# Build all images
./build-images.sh

# Run specific platform
docker run -p 50051:50051 sutra-storage:latest-alpine-x86
docker run -p 50051:50051 sutra-storage:latest-graviton3
docker run -p 50051:50051 sutra-storage:latest-pi5
```

### Kubernetes Deployment

```bash
# Deploy platform-specific versions
kubectl apply -f k8s/sutra-storage-optimized.yaml

# Check deployment status
kubectl get pods -n sutra-ai -l app=sutra-storage

# View logs by platform
kubectl logs -n sutra-ai -l platform=x86 -f
kubectl logs -n sutra-ai -l platform=graviton -f
```

---

## Support Matrix Summary

| Platform | Docker | PyPI | Binary | K8s | Status |
|----------|--------|------|--------|-----|--------|
| Alpine x86_64 | ✅ | ✅ | ✅ | ✅ | Production |
| Graviton3 | ✅ | ✅ | ✅ | ✅ | Production |
| Graviton4 | ✅ | ✅ | ✅ | ✅ | Supported |
| Apple Silicon | ❌ | ✅ | ✅ | ❌ | Development |
| Raspberry Pi 5 | ✅ | ❌ | ✅ | ✅ | Edge |
| Generic x86 | ✅ | ✅ | ✅ | ✅ | Fallback |
| Generic ARM64 | ✅ | ✅ | ✅ | ✅ | Fallback |

---

## Additional Resources

- [CLOUD_NATIVE_ALPINE_OPTIMIZATIONS.md](./CLOUD_NATIVE_ALPINE_OPTIMIZATIONS.md) - Detailed optimization research
- [Rust Cross-Compilation Guide](https://rust-lang.github.io/rustup/cross-compilation.html)
- [Docker Buildx Documentation](https://docs.docker.com/buildx/working-with-buildx/)
- [maturin User Guide](https://www.maturin.rs/)
- [GitHub Actions Workflow Syntax](https://docs.github.com/en/actions/using-workflows/workflow-syntax-for-github-actions)

---

**Document Status**: Ready for implementation  
**Next Steps**: Begin with Step 1 (Update Cargo.toml) and proceed through implementation guide  
**Questions**: Open an issue or contact the Sutra AI team

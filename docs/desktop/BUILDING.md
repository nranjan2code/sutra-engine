# Building Sutra Desktop

**Version:** 1.0.0  
**Updated:** November 26, 2025

Complete guide to building Sutra Desktop from source.

## Prerequisites

### Required

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | 1.70+ | Compiler and Cargo |
| Xcode CLT | Latest | macOS compilation |

### Installing Rust

```bash
# Install rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version
cargo --version
```

### Installing Xcode Command Line Tools

```bash
xcode-select --install
```

## Workspace Structure

Desktop is part of the Sutra monorepo:

```
sutra-memory/
├── Cargo.toml              # Workspace manifest
├── desktop/                # Desktop application
│   ├── Cargo.toml         # Package manifest
│   └── src/               # Source code
├── packages/
│   └── sutra-storage/     # Storage engine (dependency)
└── target/                # Build output
```

## Build Commands

### Development Build

Fast compilation with debug symbols:

```bash
# From workspace root
cd /path/to/sutra-memory

# Build desktop package
cargo build -p sutra-desktop

# Binary output
./target/debug/sutra-desktop
```

### Release Build

Optimized for performance:

```bash
# Optimized build
cargo build -p sutra-desktop --release

# Binary output (much smaller, faster)
./target/release/sutra-desktop
```

### Run Directly

Build and run in one command:

```bash
# Development
cargo run -p sutra-desktop

# Release
cargo run -p sutra-desktop --release
```

## Cargo.toml Configuration

```toml
[package]
name = "sutra-desktop"
version = "1.0.0"
edition = "2021"
description = "Sutra AI Desktop Edition - Self-contained semantic reasoning"
authors = ["Sutra Works"]
license = "MIT"

[dependencies]
# GUI framework
eframe = "0.29"
egui = "0.29"

# Storage engine (workspace crate)
sutra-storage = { path = "../packages/sutra-storage" }

# Utilities
md5 = "0.7"
chrono = "0.4"
directories = "5.0"
tracing = "0.1"
tracing-subscriber = "0.3"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

## macOS App Bundle

### Bundle Structure

```
Sutra Desktop.app/
└── Contents/
    ├── Info.plist          # App metadata
    ├── MacOS/
    │   └── sutra-desktop   # Executable
    └── Resources/
        └── AppIcon.icns    # App icon
```

### Build Script

```bash
#!/bin/bash
# desktop/scripts/build-macos.sh

set -e

APP_NAME="Sutra Desktop"
BUNDLE_ID="ai.sutra.desktop"
VERSION="1.0.0"

# Build release binary
cargo build -p sutra-desktop --release

# Create bundle structure
BUNDLE_DIR="target/release/bundle/${APP_NAME}.app"
mkdir -p "${BUNDLE_DIR}/Contents/MacOS"
mkdir -p "${BUNDLE_DIR}/Contents/Resources"

# Copy binary
cp target/release/sutra-desktop "${BUNDLE_DIR}/Contents/MacOS/"

# Create Info.plist
cat > "${BUNDLE_DIR}/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" 
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleDisplayName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>${BUNDLE_ID}</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundleExecutable</key>
    <string>sutra-desktop</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>LSMinimumSystemVersion</key>
    <string>12.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.productivity</string>
</dict>
</plist>
EOF

echo "✅ Bundle created: ${BUNDLE_DIR}"
```

### Running the Script

```bash
cd desktop
chmod +x scripts/build-macos.sh
./scripts/build-macos.sh

# Open the app
open "target/release/bundle/Sutra Desktop.app"
```

## Build Options

### Feature Flags

```toml
[features]
default = []
local-embeddings = ["ort"]  # Future: ONNX runtime
```

```bash
# Build with specific features
cargo build -p sutra-desktop --release --features local-embeddings
```

### Target Architectures

```bash
# Apple Silicon (M1/M2/M3)
cargo build -p sutra-desktop --release --target aarch64-apple-darwin

# Intel Mac
cargo build -p sutra-desktop --release --target x86_64-apple-darwin

# Universal Binary (both architectures)
cargo build -p sutra-desktop --release --target aarch64-apple-darwin
cargo build -p sutra-desktop --release --target x86_64-apple-darwin
lipo -create \
  target/aarch64-apple-darwin/release/sutra-desktop \
  target/x86_64-apple-darwin/release/sutra-desktop \
  -output target/release/sutra-desktop-universal
```

## Troubleshooting

### Compilation Errors

**Problem**: Missing system frameworks
```
error: linking with `cc` failed
```

**Solution**: Install Xcode CLT
```bash
xcode-select --install
```

---

**Problem**: sutra-storage not found
```
error: failed to load manifest for workspace member `desktop`
```

**Solution**: Build from workspace root
```bash
cd /path/to/sutra-memory  # Not desktop/
cargo build -p sutra-desktop
```

---

**Problem**: egui version mismatch
```
error: failed to select a version for `egui`
```

**Solution**: Check Cargo.lock and update
```bash
cargo update -p egui
cargo update -p eframe
```

### Runtime Errors

**Problem**: App crashes on startup
```
thread 'main' panicked at 'Failed to initialize storage'
```

**Solution**: Check data directory permissions
```bash
ls -la ~/Library/Application\ Support/
mkdir -p ~/Library/Application\ Support/ai.sutra.SutraDesktop
```

---

**Problem**: Window doesn't appear (macOS)
```
No visible window after launch
```

**Solution**: Check for accessibility permissions in System Preferences

### Performance Issues

**Problem**: Slow compilation
```bash
# Use faster linker (macOS)
RUSTFLAGS="-C link-arg=-fuse-ld=lld" cargo build -p sutra-desktop

# Or use mold (Linux)
RUSTFLAGS="-C linker=clang -C link-arg=-fuse-ld=mold" cargo build
```

**Problem**: Large binary size
```bash
# Enable LTO and stripping
cargo build -p sutra-desktop --release

# Check size
ls -lh target/release/sutra-desktop
# Typical: 15-25 MB
```

## Continuous Integration

### GitHub Actions Workflow

```yaml
# .github/workflows/desktop.yml
name: Desktop Build

on:
  push:
    paths:
      - 'desktop/**'
      - 'packages/sutra-storage/**'

jobs:
  build-macos:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-action@stable
      
      - name: Build Desktop
        run: cargo build -p sutra-desktop --release
      
      - name: Create App Bundle
        run: |
          cd desktop
          ./scripts/build-macos.sh
      
      - name: Upload Artifact
        uses: actions/upload-artifact@v4
        with:
          name: sutra-desktop-macos
          path: target/release/bundle/
```

## Next Steps

After building:

1. **Run the app**: `cargo run -p sutra-desktop`
2. **Learn concepts** in the Chat view
3. **Browse knowledge** in the Knowledge view
4. **Configure settings** in Settings

See [Desktop README](./README.md) for usage guide.

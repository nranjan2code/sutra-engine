# Building Sutra Desktop

**Version:** 3.3.0  
**Updated:** November 26, 2025

This guide covers building Sutra Desktop from source, including platform-specific configurations and optimization options.

---

## Prerequisites

### Required

| Tool | Version | Check Command |
|------|---------|---------------|
| Rust | 1.70+ | `rustc --version` |
| Cargo | 1.70+ | `cargo --version` |
| Git | 2.0+ | `git --version` |

### Platform-Specific

**macOS:**
```bash
# Xcode command line tools
xcode-select --install
```

**Linux (Debian/Ubuntu):**
```bash
# Build essentials and graphics libraries
sudo apt install build-essential libxcb-render0-dev libxcb-shape0-dev \
    libxcb-xfixes0-dev libxkbcommon-dev libssl-dev
```

**Linux (Fedora):**
```bash
sudo dnf install gcc gcc-c++ libxcb-devel libxkbcommon-devel openssl-devel
```

**Windows:**
```powershell
# Visual Studio Build Tools (or full Visual Studio with C++ workload)
winget install Microsoft.VisualStudio.2022.BuildTools
```

---

## Quick Build

### Development Build

```bash
# From workspace root (models auto-download on first launch)
cargo build -p sutra-desktop

# Run directly (requires internet for initial model download)
cargo run -p sutra-desktop
```

**First Launch Note:** The application downloads AI models (~500MB nomic-embed-text-v1.5) automatically on first run. This takes 1-3 minutes depending on your internet connection.

### Release Build

```bash
# Optimized build (recommended for daily use)
cargo build -p sutra-desktop --release

# Run release build
./target/release/sutra-desktop
```

---

## Build Options

### Features

The desktop crate supports these features in `desktop/Cargo.toml`:

```toml
[features]
default = ["native"]
native = ["eframe/default", "eframe/persistence"]
```

**Feature Details:**
- `native`: Full native window support with state persistence (recommended)
- Without `native`: Reduced functionality, mainly for testing

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info` | Log level (trace, debug, info, warn, error) |
| `SUTRA_DATA_DIR` | Platform-specific | Override data directory |

Example:
```bash
RUST_LOG=debug cargo run -p sutra-desktop
```

---

## Platform Builds

### macOS App Bundle

Create a proper `.app` bundle for distribution:

```bash
cd desktop
./scripts/build-macos.sh
```

This script:
1. Builds release binary
2. Creates `Sutra Desktop.app` bundle structure
3. Copies binary and resources
4. Sets up `Info.plist` metadata
5. Creates app icon (icns)

Output location: `target/release/bundle/Sutra Desktop.app`

**Manual Bundle Creation:**

```bash
# Build release
cargo build -p sutra-desktop --release

# Create bundle structure
mkdir -p "target/release/bundle/Sutra Desktop.app/Contents/MacOS"
mkdir -p "target/release/bundle/Sutra Desktop.app/Contents/Resources"

# Copy binary
cp target/release/sutra-desktop "target/release/bundle/Sutra Desktop.app/Contents/MacOS/"

# Create Info.plist
cat > "target/release/bundle/Sutra Desktop.app/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>Sutra Desktop</string>
    <key>CFBundleDisplayName</key>
    <string>Sutra Desktop</string>
    <key>CFBundleIdentifier</key>
    <string>ai.sutra.SutraDesktop</string>
    <key>CFBundleVersion</key>
    <string>3.3.0</string>
    <key>CFBundleShortVersionString</key>
    <string>3.3.0</string>
    <key>CFBundleExecutable</key>
    <string>sutra-desktop</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF
```

### Linux AppImage

```bash
# Install appimage tools
wget https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
chmod +x linuxdeploy-x86_64.AppImage

# Build release
cargo build -p sutra-desktop --release

# Create AppImage (requires .desktop file and icon)
./linuxdeploy-x86_64.AppImage \
    --appdir AppDir \
    --executable target/release/sutra-desktop \
    --desktop-file sutra-desktop.desktop \
    --icon-file sutra-desktop.png \
    --output appimage
```

Example `sutra-desktop.desktop`:
```ini
[Desktop Entry]
Name=Sutra Desktop
Comment=Semantic Knowledge Reasoning
Exec=sutra-desktop
Icon=sutra-desktop
Type=Application
Categories=Utility;Development;
```

### Windows Installer

Using `cargo-wix` for MSI installer:

```powershell
# Install cargo-wix
cargo install cargo-wix

# Initialize WiX configuration (once)
cargo wix init -p sutra-desktop

# Build installer
cargo wix -p sutra-desktop
```

Output: `target/wix/sutra-desktop-3.3.0-x86_64.msi`

---

## Optimization Options

### Release Profile

The default release profile in `desktop/Cargo.toml`:

```toml
[profile.release]
opt-level = 3      # Maximum optimization
lto = true         # Link-time optimization
codegen-units = 1  # Better optimization, slower compile
strip = true       # Strip debug symbols
panic = "abort"    # Smaller binary, no unwinding
```

**Build times vs binary size:**

| Profile | Build Time | Binary Size | Performance |
|---------|------------|-------------|-------------|
| Debug | ~30s | ~80MB | Baseline |
| Release (default) | ~2min | ~20MB | 10x faster |
| Release + thin LTO | ~1min | ~25MB | 8x faster |

### Thin LTO (Faster Builds)

For faster release builds with slightly larger binary:

```bash
# One-time: add to ~/.cargo/config.toml
[profile.release]
lto = "thin"

# Or via environment
CARGO_PROFILE_RELEASE_LTO=thin cargo build -p sutra-desktop --release
```

### Static Linking (Linux)

For maximum portability on Linux:

```bash
# Build with musl for static linking
rustup target add x86_64-unknown-linux-musl
cargo build -p sutra-desktop --release --target x86_64-unknown-linux-musl
```

---

## Dependencies

### Direct Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `sutra-storage` | workspace | Core storage engine |
| `eframe` | 0.29 | Native windowing framework |
| `egui` | 0.29 | Immediate mode GUI |
| `egui_extras` | 0.29 | Additional widgets |
| `tokio` | workspace | Async runtime |
| `serde` | workspace | Serialization |
| `tracing` | workspace | Logging |
| `directories` | 5.0 | Platform data paths |
| `chrono` | 0.4 | Date/time handling |
| `anyhow` | workspace | Error handling |
| `md5` | 0.7 | Concept ID generation |
| `rand` | 0.8 | Graph layout |
| `csv` | 1.3 | CSV export |
| `quick-xml` | 0.36 | GraphML export |
| `webbrowser` | 1.0 | Open URLs in browser |

### macOS-Specific

| Crate | Version | Purpose |
|-------|---------|---------|
| `cocoa` | 0.26 | macOS bindings |
| `objc` | 0.2 | Objective-C runtime |

---

## Troubleshooting

### Compilation Errors

**Missing OpenSSL (Linux):**
```bash
# Debian/Ubuntu
sudo apt install libssl-dev pkg-config

# Fedora
sudo dnf install openssl-devel
```

**Missing xcb (Linux):**
```bash
# Debian/Ubuntu
sudo apt install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

**Linker errors (macOS):**
```bash
# Ensure Xcode tools are installed
xcode-select --install

# Reset if corrupted
sudo xcode-select --reset
```

### Runtime Issues

**Icon not showing (macOS):**
- Ensure `Info.plist` has correct `CFBundleIconFile`
- Icon must be `.icns` format in `Resources/` folder

**Window blank on startup (Linux Wayland):**
```bash
# Force X11 backend
WINIT_UNIX_BACKEND=x11 ./sutra-desktop
```

**High DPI issues (Windows):**
- Set `WINIT_HIDPI_FACTOR=1.0` for fixed scaling
- Or let Windows handle via compatibility settings

---

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Build Desktop

on:
  push:
    tags: ['v*']

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: windows-latest
            target: x86_64-pc-windows-msvc

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          targets: ${{ matrix.target }}
      
      - name: Install Linux dependencies
        if: matrix.os == 'ubuntu-latest'
        run: |
          sudo apt-get update
          sudo apt-get install -y libxcb-render0-dev libxcb-shape0-dev \
            libxcb-xfixes0-dev libxkbcommon-dev
      
      - name: Build
        run: cargo build -p sutra-desktop --release --target ${{ matrix.target }}
      
      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: sutra-desktop-${{ matrix.target }}
          path: target/${{ matrix.target }}/release/sutra-desktop*
```

---

## Development Workflow

### Watch Mode

```bash
# Install cargo-watch
cargo install cargo-watch

# Auto-rebuild on changes
cargo watch -x "run -p sutra-desktop"
```

### Profiling

```bash
# Build with profiling symbols
RUSTFLAGS="-C force-frame-pointers=yes" cargo build -p sutra-desktop --release

# Profile with perf (Linux)
perf record -g ./target/release/sutra-desktop
perf report

# Profile with Instruments (macOS)
xcrun xctrace record --template 'Time Profiler' --launch ./target/release/sutra-desktop
```

### Binary Size Analysis

```bash
# Install bloat analyzer
cargo install cargo-bloat

# Analyze binary size
cargo bloat -p sutra-desktop --release -n 20
```

---

## Related Documentation

- [README.md](./README.md) - Overview and usage
- [ARCHITECTURE.md](./ARCHITECTURE.md) - Internal design
- [UI_COMPONENTS.md](./UI_COMPONENTS.md) - UI reference

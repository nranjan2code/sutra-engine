# Sutra Desktop Edition

**Pure Rust Native Application for macOS - No Docker, No Python**

## Overview

Sutra Desktop is a self-contained, native application that embeds the same storage engine as server deployments but runs entirely locally as a single binary. Perfect for:

- **Individual developers** wanting local AI reasoning
- **Offline usage** without cloud dependencies
- **Privacy-focused** deployments
- **Resource-constrained** machines (runs in ~100MB RAM)

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Sutra Desktop                         │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │   Sidebar   │  │    Chat     │  │  Knowledge  │     │
│  │   (egui)    │  │   Panel     │  │   Browser   │     │
│  └─────────────┘  └─────────────┘  └─────────────┘     │
├─────────────────────────────────────────────────────────┤
│                     App State (app.rs)                   │
├─────────────────────────────────────────────────────────┤
│              sutra-storage (ConcurrentMemory)           │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │  Write Log  │  │  Read View  │  │    HNSW     │     │
│  │  (atomic)   │  │ (snapshots) │  │  (vectors)  │     │
│  └─────────────┘  └─────────────┘  └─────────────┘     │
├─────────────────────────────────────────────────────────┤
│                   Persistent Storage                     │
│            ~/Library/Application Support/                │
│               ai.sutra.SutraDesktop/                    │
└─────────────────────────────────────────────────────────┘
```

## Key Features

- 🚀 **Zero External Dependencies** - Pure Rust, single binary
- 🧠 **Same Storage Engine** - Identical to Docker deployments (sutra-storage crate)
- 💾 **Local Data** - All knowledge stored securely on your machine
- 🎨 **Modern UI** - Native macOS app with dark theme (egui/eframe)
- ⚡ **Fast Startup** - Sub-second cold start
- 🔐 **Privacy First** - Everything runs 100% locally

## Quick Start

### From Release

1. Download `SutraDesktop-X.X.X-macOS.dmg` from releases
2. Open DMG and drag `Sutra Desktop.app` to Applications
3. Launch from Applications or Spotlight

### From Source

```bash
# Build release
./desktop/scripts/build-macos.sh

# Or build with cargo directly
cargo build -p sutra-desktop --release

# Run
./target/release/sutra-desktop
```

## Usage

### Learning Knowledge

Type in the chat interface to teach Sutra:

```
learn: The capital of France is Paris
learn: Machine learning is a subset of artificial intelligence
learn: Water boils at 100 degrees Celsius at sea level
```

### Querying Knowledge

Ask questions about what you've taught:

```
What is the capital of France?
Tell me about machine learning
At what temperature does water boil?
```

### Knowledge Browser

Click "Knowledge" in the sidebar to browse all stored concepts, view connections, and see confidence scores.

## Data Location

All data is stored locally in platform-specific directories:

| OS     | Location |
|--------|----------|
| macOS  | `~/Library/Application Support/ai.sutra.SutraDesktop/` |
| Linux  | `~/.local/share/SutraDesktop/` |
| Windows | `C:\Users\<user>\AppData\Roaming\sutra\SutraDesktop\` |

## Configuration

Default settings (can be changed in Settings panel):

| Setting | Default | Description |
|---------|---------|-------------|
| Vector Dimension | 256 | Fast mode for desktop (Matryoshka) |
| Memory Threshold | 10,000 | Concepts before disk flush |
| Reconciler Interval | 100ms | Background sync frequency |

## Comparison: Desktop vs Server Editions

| Feature | Desktop | Simple | Community | Enterprise |
|---------|---------|--------|-----------|------------|
| **Deployment** | Native app | Docker | Docker | Docker |
| **Infrastructure** | None | Docker | Docker | K8s/Docker |
| **Language** | Pure Rust | Rust + Python | Rust + Python | Rust + Python |
| **Grid Support** | ❌ | ❌ | ❌ | ✅ |
| **HA/Clustering** | ❌ | ❌ | ❌ | ✅ |
| **Vector Dim** | 256 | 768 | 768 | 768 |
| **Max Concepts** | 100K | 100K | 1M | 10M |
| **Memory** | ~100MB | ~4GB | ~8GB | ~20GB |
| **Ideal For** | Individual | Dev/Test | Small Team | Production |

## Building

### Prerequisites

- Rust 1.75+ (with cargo)
- Xcode Command Line Tools (for macOS)

### Development Build

```bash
cargo build -p sutra-desktop
cargo run -p sutra-desktop
```

### Release Build

```bash
cargo build -p sutra-desktop --release
# Binary: target/release/sutra-desktop
```

### macOS App Bundle + DMG

```bash
./desktop/scripts/build-macos.sh
# Creates: desktop/build/Sutra Desktop.app
# Creates: desktop/build/SutraDesktop-X.X.X-macOS.dmg
```

## Technical Details

- **GUI Framework**: egui/eframe 0.29 (pure Rust, hardware-accelerated)
- **Storage Engine**: sutra-storage (same crate as server deployments)
- **Vector Index**: HNSW with USearch (persistent mmap)
- **Persistence**: Write-Ahead Log (WAL) for crash recovery

## License

Desktop Edition follows the same license as Sutra Simple Edition (Free).

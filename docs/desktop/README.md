# Sutra Desktop

**Version:** 3.3.1  
**Release Date:** November 28, 2025  
**Status:** Production Ready ✅ (Local AI + UX Enhancements)

Pure Rust native application for personal knowledge management with semantic reasoning, temporal analysis, and causal understanding - no servers, no Docker, no external dependencies required.

---

## Overview

Sutra Desktop is a self-contained knowledge management application that brings enterprise-grade semantic reasoning capabilities to your local machine. It directly uses the `sutra-storage` crate from the Sutra ecosystem and integrates **local AI models** via ONNX, ensuring feature parity with the server edition while maintaining complete data privacy.

### Key Features

| Feature | Description |
|---------|-------------|
| 🚀 **Native Performance** | Pure Rust from storage to UI, ~300ms startup |
| 🧠 **Local AI** | Real ONNX neural networks with auto-download (~500MB) - no API keys needed |
| ⚡ **Async Architecture** | Non-blocking UI with background processing |
| 🔒 **Complete Privacy** | All data stays on your machine |
| 📦 **Self-Contained** | Single binary + auto-downloaded AI models (~500MB) |
| 🎨 **Modern UI** | Premium themes (Dark/Light/High Contrast) with enhanced menu bar |
| 🏠 **Home Dashboard** | Stats, quick actions, activity feed, and learning tips |
| 🎓 **Onboarding Tour** | Interactive 7-step first-launch guide |
| ↩️ **Undo/Redo** | Global command stack with ⌘Z/⌘⇧Z shortcuts |
| 💾 **Full Storage Engine** | Reuses `sutra-storage` crate (no code duplication) |
| 💬 **Enhanced Chat** | Multiline input, improved autocomplete, better visual design |
| 🔍 **Multi-View Analysis** | Graph, temporal, causal, and path visualization |
| 📊 **Real-time Analytics** | Performance metrics and usage statistics |
| 🗑️ **Knowledge Management** | Safe deletion with confirmation across multiple screens |
| 🍎 **Native Integration** | Full menu bar with File/View/Help menus |
| 📥 **Drag-and-Drop** | Drop .json/.csv files directly to import data |
| 💡 **Smart Guidance** | Context-aware empty states guide you to the right actions |

### What's Included

- **Home Dashboard**: Default landing view with stats, quick actions, recent activity, and tips
- **Onboarding Tour**: Interactive 7-step overlay guide for first-time users
- **Enhanced Menu Bar**: File/View/Help menus with keyboard shortcuts
- **Chat Interface**: Natural language interaction with multiline input and autocomplete
- **Knowledge Browser**: Responsive grid view to browse, search, and safely delete concepts
- **Graph Visualization**: Force-directed interactive knowledge graph
- **Reasoning Paths**: MPPA-style multi-path consensus analysis
- **Temporal View**: Timeline and matrix visualization of temporal relationships
- **Causal Explorer**: Root cause analysis and causal chain discovery
- **Analytics Dashboard**: Real-time performance metrics and usage statistics
- **Query Builder**: Visual advanced search with filters
- **Export/Import**: JSON, CSV, GraphML, and Cypher formats
- **Concept Deletion**: Permanent removal across Knowledge and Quick Learn panels
- **Theme System**: Dark, Light, and High Contrast modes
- **Undo/Redo**: Global command stack (⌘Z/⌘⇧Z) for destructive actions

---

## Quick Start

### Prerequisites

- **Rust 1.70+** with cargo
- **macOS 11+**, **Linux (glibc 2.31+)**, or **Windows 10+**

### Build and Run

```bash
# Development build (models auto-download on first run)
cargo run -p sutra-desktop

# Release build
cargo build -p sutra-desktop --release

# Create macOS app bundle
cd desktop && ./scripts/build-macos.sh
```

**First Launch:** The app automatically downloads AI models (~420MB all-mpnet-base-v2) on first run for real neural network embeddings (768D vectors, server-compatible).

**Vendored Dependencies:** The desktop app uses `sutra-embedder` and `sutraworks-model` which are embedded directly in the workspace at `packages/sutra-embedder` and `packages/sutraworks-model`. No external git repositories or submodules required.

### macOS App Bundle

```bash
cd desktop
./scripts/build-macos.sh

# The app bundle will be at:
# target/release/bundle/Sutra Desktop.app
```

---

## Usage

### Slash Commands

Type `/` in the chat to see available commands with autocomplete:

| Command | Shortcut | Description |
|---------|----------|-------------|
| `/learn <text>` | `/l` | Teach new knowledge |
| `/search <query>` | `/s`, `/find` | Search knowledge base |
| `/help` | `/h`, `/?` | Show all commands |
| `/clear` | `/c` | Clear chat history |
| `/stats` | `/status` | Show knowledge statistics |

### Keyboard Shortcuts

**Navigation:**
- **↑/↓ arrows**: Navigate autocomplete suggestions
- **Enter**: Send message
- **Shift+Enter**: Insert new line
- **Tab**: Accept autocomplete suggestion
- **Esc**: Close autocomplete

**Undo/Redo:**
- **⌘Z** (Mac) / **Ctrl+Z** (Win/Linux): Undo last action
- **⌘⇧Z** (Mac) / **Ctrl+Shift+Z** (Win/Linux): Redo action
- Supports up to 50 undoable commands
- Available for: Delete concept, Delete batch, Clear all data

### Navigation

**Menu Bar (NEW):**
- **File**: Import/Export data, Settings, Quit (⌘Q)
- **View**: Quick navigation to all views with keyboard shortcuts
- **Help**: Documentation links, About dialog

**Sidebar:**

**MAIN**
- 🏠 **Home** - Dashboard with stats, actions, and tips (DEFAULT)
- 💬 **Chat** - Conversational interface
- 📚 **Knowledge** - Browse concepts (responsive grid)
- 🔍 **Search** - Quick search with deletion options

**ANALYSIS** (collapsible)
- 🕸️ **Graph View** - Visual knowledge graph
- 🛤️ **Reasoning** - Multi-path exploration
- ⏱️ **Timeline** - Temporal analysis
- 🔗 **Causality** - Root cause analysis

**TOOLS** (collapsible)
- 📊 **Analytics** - Performance metrics
- 🔎 **Query Builder** - Advanced search
- 📤 **Export/Import** - Data portability

### Delete Functionality

Concepts can be permanently deleted from multiple screens:

**Knowledge Panel:**
- Click the **🗑️ button** on any concept card
- **Safety:** A confirmation dialog appears to prevent accidental deletion
- Concept is immediately removed from storage with full cleanup upon confirmation

**Quick Learn Panel:**  
- Click the **🗑️ button** next to any successful "Recent learns" entry
- Concept is deleted from knowledge base and removed from history

**What Gets Deleted:**
- ✅ Concept content and metadata
- ✅ All associated edges and relationships
- ✅ Vector embeddings and search indices
- ✅ WAL entries (logged for durability)

**Recovery:** Deletions are permanent. Back up your data directory for recovery options.

---

## Data Storage

### Location

Data is stored in platform-specific directories:

| Platform | Location |
|----------|----------|
| macOS | `~/Library/Application Support/ai.sutra.SutraDesktop/` |
| Linux | `~/.local/share/SutraDesktop/` |
| Windows | `%APPDATA%\sutra\SutraDesktop\` |

### Contents

```
SutraDesktop/
├── storage.dat       # Binary concept and edge store (SUTRADAT v2)
├── hnsw.index       # HNSW vector index (persistent)
└── wal.log          # Write-Ahead Log for durability
```

### Backup

Simply copy the entire data directory to back up your knowledge base. The WAL ensures crash recovery and data consistency. All deletions are logged for proper recovery.

---

## Architecture

Sutra Desktop follows a thin-wrapper architecture with asynchronous processing:

```
┌─────────────────────────────────────────────────────────────┐
│                    Sutra Desktop App                         │
├─────────────────────────────────────────────────────────────┤
│  UI Layer (egui)  ↔  Async Controller  ↔  sutra-storage      │
└─────────────────────────────────────────────────────────────┘
```

The application directly uses the same `sutra-storage` crate that powers the Docker-based server deployments, ensuring:

- **Zero code duplication** between desktop and server
- **Feature parity** with enterprise capabilities
- **Consistent behavior** across deployment modes
- **Non-blocking UI** via background thread offloading

See [ARCHITECTURE.md](./ARCHITECTURE.md) for detailed technical documentation.

---

## Performance

| Metric | Target | Actual |
|--------|--------|--------|
| Startup Time | <500ms | ~300ms ✅ |
| Frame Rate | 60 FPS | 60 FPS ✅ |
| Query Latency | <50ms | <10ms ✅ |
| Memory Usage | <200 MB | ~100 MB ✅ |
| Concept Capacity | 100K+ | Tested to 50K ✅ |
| UI Responsiveness | 60 FPS | 60 FPS ✅ |

---

## Documentation

- **[README.md](./README.md)** - This overview
- **[ARCHITECTURE.md](./ARCHITECTURE.md)** - Technical design and internals
- **[BUILDING.md](./BUILDING.md)** - Build instructions and configuration
- **[UI_COMPONENTS.md](./UI_COMPONENTS.md)** - UI panel reference
- **[ENHANCED_UI_DESIGN.md](./ENHANCED_UI_DESIGN.md)** - Original design specifications

---

## Data Storage

**Application Data:**
- Location: `~/Library/Application Support/ai.sutra.SutraDesktop/` (macOS)
- Contents: Knowledge graphs, user settings, query logs

**AI Models:**
- Location: Hugging Face cache (auto-managed by `sutra-embedder`)
- Size: ~420MB (all-mpnet-base-v2 ONNX model + tokenizer)
- Dimensions: 768D vectors (server-compatible)
- Download: Automatic on first launch
- Optimization: Apple Silicon CoreML with FP16 precision

**Vendored AI Packages:**
- `packages/sutra-embedder` - Rust ONNX embedding library
- `packages/sutraworks-model` - Text generation and NLG support

---

## Troubleshooting

### Application Won't Start

1. Ensure Rust 1.70+ is installed: `rustc --version`
2. Check for compilation errors: `cargo build -p sutra-desktop 2>&1`
3. Verify data directory permissions

### Slow Performance

1. Check available system memory (needs ~100MB free)
2. Consider clearing old query logs in Analytics
3. Run `cargo build --release` for optimized builds

### Data Not Persisting

1. Verify write permissions to data directory
2. Check disk space availability
3. Ensure clean shutdown (don't force-quit)

---

## Contributing

See the main project [contribution guidelines](../../CONTRIBUTING.md).

Desktop-specific considerations:
- UI changes should maintain 60 FPS performance
- New panels should follow the existing action/handler pattern
- Test on macOS, Linux, and Windows before submitting PRs

---

## License

Sutra Desktop is part of the Sutra AI project. See [LICENSE](../../LICENSE) for details.

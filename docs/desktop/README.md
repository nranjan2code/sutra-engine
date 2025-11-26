# Sutra Desktop Edition

**Version:** 1.0.0  
**Updated:** November 26, 2025

A self-contained, native macOS application for semantic reasoning with temporal, causal, and explainable AI. No Docker, no servers, no external dependencies—just a single app that runs entirely on your machine.

## Overview

Sutra Desktop is a **pure Rust** application that packages the complete Sutra semantic reasoning engine into a native desktop experience. Unlike the server edition which requires Docker and multiple services, the Desktop Edition runs everything locally with zero configuration.

### Key Features

- 🚀 **Native Performance**: Pure Rust from storage to UI
- 🔒 **Complete Privacy**: All data stays on your machine
- 📦 **Self-Contained**: Single app bundle, no dependencies
- 🎨 **Modern UI**: Premium dark theme with smooth animations
- 🧠 **Full Reasoning Engine**: Same MPPA algorithm as server edition
- 💾 **Persistent Storage**: WAL-backed graph database with HNSW indexing

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Sutra Desktop App                         │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────┐   │
│  │                    UI Layer (egui)                    │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌─────────┐ │   │
│  │  │ Sidebar  │ │   Chat   │ │Knowledge │ │Settings │ │   │
│  │  └──────────┘ └──────────┘ └──────────┘ └─────────┘ │   │
│  └─────────────────────────────────────────────────────┘   │
│                            │                                 │
│  ┌─────────────────────────▼─────────────────────────────┐ │
│  │              Application Controller                    │ │
│  │         (app.rs - State Management)                   │ │
│  └─────────────────────────────────────────────────────────┘ │
│                            │                                 │
│  ┌─────────────────────────▼─────────────────────────────┐ │
│  │           sutra-storage (Rust Crate)                  │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌──────────────┐  │ │
│  │  │ Concurrent  │  │    HNSW     │  │     WAL      │  │ │
│  │  │   Memory    │  │   Indexing  │  │  Durability  │  │ │
│  │  └─────────────┘  └─────────────┘  └──────────────┘  │ │
│  └─────────────────────────────────────────────────────────┘ │
│                            │                                 │
│  ┌─────────────────────────▼─────────────────────────────┐ │
│  │              Local File System                         │ │
│  │     ~/Library/Application Support/ai.sutra.Desktop    │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Components

### 1. UI Layer (`desktop/src/ui/`)

Built with **egui/eframe** - a pure Rust immediate-mode GUI framework.

| Component | File | Purpose |
|-----------|------|---------|
| Sidebar | `sidebar.rs` | Navigation with animated logo |
| Chat | `chat.rs` | Conversational learning interface |
| Knowledge | `knowledge.rs` | Browse and search concepts |
| Settings | `settings.rs` | Configuration and stats |
| Status Bar | `status_bar.rs` | System status display |
| Theme | `theme.rs` | Premium dark color scheme |

### 2. Application Controller (`desktop/src/app.rs`)

Manages application state and bridges UI with storage:

```rust
pub struct SutraApp {
    storage: ConcurrentMemory,      // Direct storage access
    sidebar: Sidebar,               // Navigation state
    chat: ChatPanel,                // Chat UI state
    knowledge: KnowledgePanel,      // Knowledge browser state
    settings: SettingsPanel,        // Settings state
    status_bar: StatusBar,          // Status display
}
```

### 3. Storage Engine (`packages/sutra-storage/`)

The **same high-performance Rust storage** used in the server edition:

- **ConcurrentMemory**: Thread-safe in-memory graph with persistence
- **HNSW Indexing**: Fast vector similarity search via USearch
- **WAL**: Write-ahead logging for crash recovery
- **ConceptNode**: Core data structure for semantic concepts

## Data Model

### ConceptNode

```rust
pub struct ConceptNode {
    pub id: ConceptId,           // 16-byte unique identifier
    pub content: Arc<[u8]>,      // Concept content (UTF-8 text)
    pub vector: Option<Vec<f32>>, // Optional embedding vector
    pub strength: f32,           // Learning strength (0.0-1.0)
    pub confidence: f32,         // Confidence score (0.0-1.0)
    pub neighbors: Vec<ConceptId>, // Connected concepts
    pub created_at: i64,         // Unix timestamp
    pub updated_at: i64,         // Last modification
}
```

### ConceptId

A 16-byte identifier generated from content hash:

```rust
let id = ConceptId::from_bytes(md5::compute(content).0);
```

## Building

### Prerequisites

- Rust 1.70+ with Cargo
- macOS 12+ (for app bundle)
- Xcode Command Line Tools

### Development Build

```bash
# From workspace root
cargo build -p sutra-desktop

# Run in development mode
cargo run -p sutra-desktop
```

### Release Build

```bash
# Optimized release build
cargo build -p sutra-desktop --release

# Binary location
./target/release/sutra-desktop
```

### macOS App Bundle

```bash
# Create .app bundle
cd desktop
./scripts/build-macos.sh

# Output: target/release/bundle/Sutra Desktop.app
```

## Configuration

### Data Directory

All data is stored in:
```
~/Library/Application Support/ai.sutra.SutraDesktop/
├── concepts/          # Concept data files
├── indexes/           # HNSW vector indexes
├── wal/              # Write-ahead log
└── config.json       # User preferences
```

### Settings

| Setting | Default | Description |
|---------|---------|-------------|
| Vector Dimensions | 768 | Embedding vector size |
| Theme | Dark | UI color scheme |
| Font Size | 14px | Base font size |

## Usage

### Learning Concepts

In the Chat view, prefix messages with `learn:` to teach Sutra:

```
learn: The capital of France is Paris
learn: Python is a programming language created by Guido van Rossum
learn: Machine learning is a subset of artificial intelligence
```

### Querying Knowledge

Ask questions naturally:

```
What is the capital of France?
Tell me about Python
What do you know about machine learning?
```

### Browsing Knowledge

The Knowledge view shows all learned concepts:
- Search by content
- View concept details (ID, strength, confidence)
- See connected concepts

## Theme Design

Premium dark theme inspired by modern design systems:

### Colors

```rust
// Primary palette
PRIMARY: #a78bfa      // Vibrant Purple
SECONDARY: #60a5fa    // Sky Blue  
ACCENT: #fbbf24       // Amber/Gold
SUCCESS: #34d399      // Emerald
WARNING: #fb923c      // Orange

// Backgrounds
BG_DARK: #0f0f19      // Darkest
BG_PANEL: #16162e     // Panels
BG_SIDEBAR: #12121e   // Sidebar
BG_WIDGET: #23233a    // Inputs/cards
```

### Typography

- **Headings**: 20-28px, bold
- **Body**: 14px, regular
- **Captions**: 10-12px, muted color
- **Monospace**: 13px for IDs and code

## Comparison: Desktop vs Server Edition

| Feature | Desktop Edition | Server Edition |
|---------|----------------|----------------|
| Deployment | Single app | Docker containers |
| Storage | Local files | Distributed shards |
| Embeddings | Built-in/optional | HA cluster |
| Grid/HA | ❌ Not included | ✅ Full support |
| Multi-user | ❌ Single user | ✅ RBAC |
| API | ❌ None | ✅ REST/TCP |
| Use Case | Personal/Dev | Production |

## File Structure

```
desktop/
├── Cargo.toml              # Package manifest
├── src/
│   ├── main.rs            # Entry point
│   ├── app.rs             # Application controller
│   ├── theme.rs           # Color scheme & styling
│   └── ui/
│       ├── mod.rs         # UI module exports
│       ├── sidebar.rs     # Navigation sidebar
│       ├── chat.rs        # Chat interface
│       ├── knowledge.rs   # Knowledge browser
│       ├── settings.rs    # Settings panel
│       └── status_bar.rs  # Status display
├── scripts/
│   └── build-macos.sh     # macOS bundle script
└── assets/
    └── icon.png           # App icon (planned)
```

## Roadmap

### v1.1 (Planned)
- [ ] Local embedding generation (ONNX)
- [ ] Import/Export functionality
- [ ] Keyboard shortcuts
- [ ] Search improvements

### v1.2 (Planned)
- [ ] Windows support
- [ ] Linux support
- [ ] Plugin system
- [ ] Custom themes

## Troubleshooting

### App Won't Start

1. Check macOS version (requires 12+)
2. Allow app in Security & Privacy settings
3. Check Console.app for errors

### Data Not Persisting

1. Verify write permissions to Application Support
2. Check disk space
3. Look for WAL corruption in logs

### Performance Issues

1. Check concept count (Settings → Status)
2. Reduce vector dimensions if not using embeddings
3. Restart app to rebuild indexes

## License

Same license as main Sutra project. See [LICENSE](../../LICENSE).

## Related Documentation

- [System Architecture](../architecture/SYSTEM_ARCHITECTURE.md)
- [Storage Engine](../storage/README.md)
- [Building Services](../build/README.md)

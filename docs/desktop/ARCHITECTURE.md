# Sutra Desktop Architecture

**Version:** 3.3.1  
**Updated:** November 28, 2025

This document describes the internal architecture of Sutra Desktop, including module structure, data flow, and design decisions.

---

## Design Principles

### 1. Zero Code Duplication

The desktop application is a **thin UI wrapper** around `sutra-storage`. All storage logic, including:
- Concept learning and retrieval
- Graph traversal and pathfinding
- HNSW vector indexing
- WAL-based persistence

...comes directly from the shared crate. This ensures:
- Feature parity with server deployments
- Single source of truth for storage logic
- Automatic updates when storage crate improves

### 2. Local AI Pipeline (PRODUCTION)

The desktop edition integrates **real neural networks** directly via vendored crates:
- **Embedding Engine**: `sutra-embedder` crate (vendored in `packages/sutra-embedder`)
- **Embedding Model**: `all-mpnet-base-v2` (768D, 420MB) - matches server dimensions
- **NLG Engine**: `sutraworks-model` crate (vendored in `packages/sutraworks-model`)
- **Auto-Download**: Models downloaded automatically on first launch (~500MB total)
- **Hardware Optimized**: CoreML (Apple Silicon) + FP16 mixed precision
- **Privacy**: Complete offline operation after initial model download
- **Zero External Deps**: Both AI packages are workspace members, no git submodules

### 3. Immediate Mode GUI

Using **egui** (immediate mode GUI), the UI is:
- Rebuilt every frame (~60 FPS) with smooth 16ms budget
- Stateless rendering with enhanced theme system
- Highly responsive with improved visual feedback
- Memory efficient with optimized allocations
- Native menu bar integration for better platform integration

### 4. Asynchronous Architecture

- **UI Thread**: Handles all rendering and user interaction (60 FPS).
- **Background Threads**: Heavy operations (learning, graph layout, pathfinding) run in dedicated threads.
- **Message Passing**: Communication via `std::sync::mpsc` channels.
- **Non-blocking**: The UI never freezes, even during complex reasoning tasks.

---

## Module Structure

```
desktop/
├── Cargo.toml              # Dependencies and features
├── scripts/
│   └── build-macos.sh      # macOS app bundle script
└── src/
    ├── main.rs             # Entry point, window setup
    ├── app.rs              # Main application controller
    ├── local_embedding.rs  # Local embedding (sutra-embedder - vendored)
    ├── local_nlg.rs        # Local NLG (sutraworks-model - vendored)
    ├── theme.rs            # Color palette, themes, styling
    ├── types.rs            # Shared data types & AppMessage
    └── ui/
        ├── mod.rs          # Module exports
        ├── sidebar.rs      # Navigation sidebar
        ├── home.rs         # Home dashboard (NEW)
        ├── chat.rs         # Chat interface
        ├── knowledge.rs    # Concept browser
        ├── settings.rs     # Settings panel
        ├── status_bar.rs   # Bottom status bar
        ├── onboarding.rs   # First-launch tour (NEW)
        ├── undo_redo.rs    # Command stack (NEW)
        ├── graph_view.rs   # Interactive graph visualization
        ├── reasoning_paths.rs   # MPPA path explorer
        ├── temporal_view.rs     # Timeline analysis
        ├── causal_view.rs       # Root cause analysis
        ├── analytics.rs         # Performance dashboard
        ├── query_builder.rs     # Advanced search
        └── export_import.rs     # Data portability
```

---

## Core Components

### main.rs - Application Entry Point

```rust
fn main() -> Result<()> {
    // 1. Initialize logging
    // 2. Configure native window options
    // 3. Apply custom theme
    // 4. Create and run SutraApp
}
```

Responsibilities:
- Tracing/logging setup
- Window configuration (size, icon, title)
- Theme application to egui context
- eframe application lifecycle

### app.rs - Application Controller

```rust
pub struct SutraApp {
    // Storage engine (from sutra-storage crate)
    storage: Arc<ConcurrentMemory>,
    
    // Async communication
    sender: Sender<AppMessage>,
    receiver: Receiver<AppMessage>,
    
    // Core UI panels
    sidebar: Sidebar,
    chat: ChatPanel,
    // ... other panels
}
```

Key methods:
- `new()` - Initialize storage, channels, and panels
- `update()` - egui frame update (called 60x/sec)
- `handle_async_messages()` - Process results from background threads
- `spawn_task()` - Helper to run closures in background threads

### theme.rs - Visual Design System

Color palette following modern dark theme principles and **WCAG AA compliance**.
Now supports **three theme modes**: Dark, Light, and High Contrast.

```rust
// Theme mode selection
pub enum ThemeMode {
    Dark,         // Default premium dark theme
    Light,        // Clean light theme for bright environments
    HighContrast, // WCAG AAA for accessibility
}

// Enhanced primary colors (Dark theme)
pub const PRIMARY: Color32 = Color32::from_rgb(167, 139, 250);       // Vibrant Purple
pub const PRIMARY_LIGHT: Color32 = Color32::from_rgb(196, 181, 253); // Light Purple
pub const INFO: Color32 = Color32::from_rgb(96, 165, 250);           // Blue

// Theme-aware getters
pub fn primary() -> Color32;     // Returns theme-appropriate primary color
pub fn bg_panel() -> Color32;    // Returns theme-appropriate background
pub fn text_primary() -> Color32; // Returns theme-appropriate text color
```

Theme can be changed in Settings → Appearance. The High Contrast theme uses
pure black backgrounds with bright cyan/yellow accents for maximum visibility.

### types.rs - Shared Data Structures

Types used across multiple panels, including async messages:

```rust
// Async Messages
pub enum AppMessage {
    LearnResult { content, concept_id, ... },
    GraphLoaded { nodes, edges },
    ReasoningPathsFound { paths, consensus },
    // ...
}

// Graph visualization
pub struct GraphNode { id, label, content, confidence, x, y, vx, vy }
// ...
```

---

## Data Flow

### Async Command-Event Pattern

The application uses a non-blocking command-event pattern:

1.  **UI Action**: User clicks a button (e.g., "Learn").
2.  **State Update**: UI updates to "Processing" state immediately.
3.  **Task Spawn**: `SutraApp` spawns a background thread with a clone of `storage` and `sender`.
4.  **Background Work**: Thread performs heavy storage operation.
5.  **Message Send**: Thread sends `AppMessage::LearnResult` via channel.
6.  **Message Handle**: Main thread's `update()` loop receives message via `handle_async_messages()`.
7.  **UI Refresh**: App state is updated with results, and UI re-renders.

```rust
// 1. UI triggers action
if ui.button("Learn").clicked() {
    self.chat.is_processing = true;
    let storage = self.storage.clone();
    let sender = self.sender.clone();
    
    // 3. Spawn background task
    std::thread::spawn(move || {
        // 4. Heavy work
        let result = storage.learn_concept(...);
        
        // 5. Send result
        sender.send(AppMessage::LearnResult { ... }).unwrap();
    });
}

// 6. Main loop handles message
fn handle_async_messages(&mut self) {
    while let Ok(msg) = self.receiver.try_recv() {
        match msg {
            AppMessage::LearnResult { ... } => {
                // 7. Update UI state
                self.chat.add_response(...);
                self.chat.is_processing = false;
            }
        }
    }
}
```

Benefits:
- **Responsiveness**: UI remains interactive (60 FPS) during heavy loads.
- **Simplicity**: No complex async runtime (tokio) needed for UI logic.
- **Safety**: `Arc<ConcurrentMemory>` handles thread-safe storage access.

---

## View Architecture

### Sidebar Navigation

```rust
pub enum SidebarView {
    // MAIN section
    Home,      // NEW: Default dashboard landing page
    Chat, Knowledge, Search,
    // ANALYSIS section (collapsible)
    Graph, Paths, Timeline, Causal,
    // TOOLS section (collapsible)
    Analytics, Query, Export,
    // Always visible
    Settings,
}
```

The sidebar renders differently based on current selection and provides visual feedback.
**Home** is now the default landing view.

### Panel Update Cycle

```rust
impl eframe::App for SutraApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. Initial data load (once)
        if !self.initialized {
            self.handle_knowledge_action(KnowledgeAction::Refresh);
            self.graph_view.load_from_storage(&self.storage);
            self.initialized = true;
        }
        
        // 2. Request repaint for animations
        ctx.request_repaint_after(Duration::from_millis(100));
        
        // 3. Render sidebar
        egui::SidePanel::left("sidebar").show(ctx, |ui| {
            self.sidebar.ui(ui);
        });
        
        // 4. Render status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            self.status_bar.ui(ui);
        });
        
        // 5. Render main content based on current view
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.sidebar.current_view {
                SidebarView::Chat => {
                    if let Some(action) = self.chat.ui(ui) {
                        self.handle_chat_action(action);
                    }
                }
                // ... other views
            }
        });
    }
}
```

---

## Graph Visualization

### Force-Directed Layout

The graph view implements a simple force-directed algorithm:

```rust
impl GraphView {
    fn simulate_step(&mut self) {
        // 1. Repulsion: all nodes push each other away
        for i in 0..nodes.len() {
            for j in (i+1)..nodes.len() {
                let force = self.repulsion / distance_squared;
                // Apply to velocities
            }
        }
        
        // 2. Attraction: connected nodes pull together
        for edge in &self.edges {
            let force = distance * self.attraction * edge.strength;
            // Apply to velocities
        }
        
        // 3. Apply velocities with damping
        for node in &mut self.nodes {
            node.vx *= self.damping;
            node.vy *= self.damping;
            node.x += node.vx;
            node.y += node.vy;
        }
        
        // 4. Stop when movement is minimal
        if total_movement < 0.5 {
            self.simulation_running = false;
        }
    }
}
```

### Camera System

```rust
pub struct Camera {
    pub offset: Vec2,  // Pan offset
    pub zoom: f32,     // Zoom level (0.1 - 5.0)
}

impl Camera {
    pub fn world_to_screen(&self, world: Pos2, rect: Rect) -> Pos2 {
        let center = rect.center();
        Pos2::new(
            center.x + (world.x + self.offset.x) * self.zoom,
            center.y + (world.y + self.offset.y) * self.zoom,
        )
    }
}
```

---

## MPPA Consensus Analysis

The reasoning paths panel implements MPPA-style clustering:

```rust
fn analyze_consensus(&self) -> ConsensusResult {
    // 1. Cluster paths by destination concept
    let mut clusters: HashMap<ConceptId, Vec<ReasoningPath>> = HashMap::new();
    for path in &self.paths {
        let destination = path.path.last().unwrap().concept_id;
        clusters.entry(destination).or_default().push(path.clone());
    }
    
    // 2. Score each cluster
    for cluster in &mut clusters {
        let support_ratio = cluster.paths.len() as f32 / total_paths as f32;
        let avg_confidence = cluster.paths.iter().map(|p| p.confidence).sum() / cluster.len();
        
        // MPPA scoring formula
        let consensus_bonus = if support_ratio >= 0.5 { 1.0 + (support_ratio - 0.5) } else { 1.0 };
        let outlier_penalty = if support_ratio < 0.2 { 0.7 } else { 1.0 };
        cluster.consensus_weight = avg_confidence * support_ratio * consensus_bonus * outlier_penalty;
    }
    
    // 3. Rank by consensus weight
    clusters.sort_by_consensus_weight();
    
    ConsensusResult {
        primary_cluster: clusters[0],
        alternatives: clusters[1..],
        total_paths,
    }
}
```

---

## Export Formats

### JSON (Full Fidelity)

```json
{
  "version": "2.0.0",
  "exported_at": "2025-11-26T14:32:00Z",
  "format": "sutra-desktop",
  "concepts": [
    {
      "id": "a3f2e8c1...",
      "content": "Memory leak in connection pool",
      "confidence": 0.92,
      "strength": 0.88,
      "neighbors": ["b8e4...", "c9f1..."]
    }
  ],
  "edges": [
    {"from": "a3f2...", "to": "b8e4..."}
  ]
}
```

### CSV (Simplified)

```csv
id,content,confidence,strength,neighbor_count
a3f2e8c1,Memory leak in connection pool,0.92,0.88,8
```

### GraphML (Neo4j Compatible)

```xml
<?xml version="1.0" encoding="UTF-8"?>
<graphml xmlns="http://graphml.graphdrawing.org/xmlns">
  <key id="content" for="node" attr.name="content" attr.type="string"/>
  <graph id="sutra" edgedefault="directed">
    <node id="a3f2e8c1">
      <data key="content">Memory leak</data>
    </node>
  </graph>
</graphml>
```

### Cypher (Neo4j Import Script)

```cypher
CREATE (c1:Concept {id: 'a3f2e8c1', content: 'Memory leak', confidence: 0.92});
MATCH (a:Concept {id: 'a3f2e8c1'}), (b:Concept {id: 'b8e4f1a2'})
CREATE (a)-[:RELATES_TO]->(b);
```

---

## Performance Optimizations

### Frame Budget

At 60 FPS, each frame has ~16ms budget:
- UI layout: ~2ms
- Storage queries: ~1-5ms (cached)
- Graph rendering: ~3ms (up to 1000 nodes)
- Reserve: ~6ms for spikes

### Memory Management

- **Lazy loading**: Graph nodes loaded on-demand
- **Ring buffers**: Analytics history capped at 1440 entries
- **LOD rendering**: Distant graph nodes simplified
- **Culling**: Only visible elements rendered

### Storage Efficiency

- **WAL batching**: Multiple writes combined
- **HNSW caching**: Vector index memory-mapped
- **Snapshot sharing**: Read-only snapshots avoid locks

---

## Testing

### Unit Tests

```bash
cd desktop
cargo test
```

### Integration Tests

```bash
# Full build and run cycle
cargo build -p sutra-desktop --release
./target/release/sutra-desktop --test-mode
```

### UI Testing

Manual testing checklist:
- [ ] All slash commands work
- [ ] Autocomplete navigation (keyboard and mouse)
- [ ] Graph zoom/pan/select
- [ ] Path finding between concepts
- [ ] Export/import round-trip
- [ ] Graceful shutdown (data persisted)

---

## Future Enhancements

### Planned (v3.4+)

1. ✅ **Local Embeddings**: ONNX Runtime for offline vector generation (DONE)
2. **Multi-Window**: Detachable panels using egui viewports
3. ✅ **Themes**: Light mode and High Contrast (DONE - Nov 28, 2025)
4. ✅ **Undo/Redo**: Transaction-based history (DONE - Nov 28, 2025)
5. ✅ **Home Dashboard**: Default landing page with stats (DONE - Nov 28, 2025)
6. ✅ **Onboarding Tour**: Interactive first-launch guide (DONE - Nov 28, 2025)

### Considered (v4.0+)

1. **Plugin System**: Rust-based extensions
2. **Collaborative**: Multi-user sync with conflict resolution
3. **Mobile**: Touch-optimized UI for tablets
4. **AI Assistant**: Local LLM integration (llama.cpp)
5. **Context Menus**: Right-click actions for nodes and sidebar items
6. **LOD Rendering**: Level-of-detail for massive graphs (10k+ nodes)

---

## Related Documentation

- [README.md](./README.md) - Overview and quick start
- [BUILDING.md](./BUILDING.md) - Build configuration
- [UI_COMPONENTS.md](./UI_COMPONENTS.md) - Panel reference
- [sutra-storage docs](../storage/) - Storage engine internals

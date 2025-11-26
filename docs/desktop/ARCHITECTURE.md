# Desktop Edition Architecture

**Version:** 1.0.0  
**Updated:** November 26, 2025

Deep dive into the architectural decisions and internal design of Sutra Desktop.

## Design Philosophy

### 1. Zero External Dependencies

Unlike the server edition which requires:
- Docker runtime
- Multiple service containers
- Network configuration
- External embedding service

Desktop Edition runs **entirely self-contained**:
- Single binary executable
- No network required
- No containerization
- Local file storage only

### 2. Crate Reuse

Desktop Edition **reuses existing workspace crates** rather than duplicating code:

```
┌─────────────────────────────────────────────┐
│              sutra-desktop                   │
│           (thin GUI wrapper)                │
└─────────────────┬───────────────────────────┘
                  │ depends on
                  ▼
┌─────────────────────────────────────────────┐
│             sutra-storage                    │
│    (same crate used by server edition)      │
│                                             │
│  • ConcurrentMemory                         │
│  • ConcurrentConfig                         │
│  • ConceptNode                              │
│  • HNSW indexing                            │
│  • WAL persistence                          │
└─────────────────────────────────────────────┘
```

This ensures:
- ✅ Single source of truth for storage logic
- ✅ Bug fixes apply to both editions
- ✅ Consistent behavior across editions
- ✅ Reduced maintenance burden

### 3. Pure Rust Stack

Every layer is written in Rust:

| Layer | Technology | Why Rust? |
|-------|------------|-----------|
| UI | egui/eframe | Cross-platform, immediate mode |
| App Logic | Native Rust | Type safety, performance |
| Storage | sutra-storage | Zero-copy, thread-safe |
| Persistence | WAL + mmap | Crash recovery |

No Python, no JavaScript, no Swift—just Rust.

## Component Architecture

### Main Entry Point

```rust
// desktop/src/main.rs
fn main() -> eframe::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Configure native window
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Sutra AI - Desktop Edition"),
        ..Default::default()
    };
    
    // Run application
    eframe::run_native(
        "Sutra Desktop",
        options,
        Box::new(|cc| Ok(Box::new(SutraApp::new(cc))))
    )
}
```

### Application State

```rust
// desktop/src/app.rs
pub struct SutraApp {
    // Core storage - directly uses sutra-storage crate
    storage: ConcurrentMemory,
    
    // UI component states
    sidebar: Sidebar,
    chat: ChatPanel,
    knowledge: KnowledgePanel,
    settings: SettingsPanel,
    status_bar: StatusBar,
    
    // Runtime state
    data_dir: PathBuf,
}
```

### Storage Integration

Desktop directly instantiates `ConcurrentMemory` from sutra-storage:

```rust
impl SutraApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Apply custom theme
        theme::setup_custom_theme(&cc.egui_ctx);
        
        // Determine data directory
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ai.sutra.SutraDesktop");
        
        // Initialize storage with configuration
        let config = ConcurrentConfig {
            data_path: data_dir.clone(),
            vector_dimension: 768,
            enable_wal: true,
            ..Default::default()
        };
        
        let storage = ConcurrentMemory::new(config)
            .expect("Failed to initialize storage");
        
        Self {
            storage,
            sidebar: Sidebar::default(),
            chat: ChatPanel::default(),
            knowledge: KnowledgePanel::default(),
            settings: SettingsPanel::default(),
            status_bar: StatusBar::default(),
            data_dir,
        }
    }
}
```

## UI Architecture

### Immediate Mode GUI

egui uses **immediate mode** rendering—UI is rebuilt every frame:

```rust
impl eframe::App for SutraApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Sidebar panel (left)
        egui::SidePanel::left("sidebar")
            .exact_width(260.0)
            .show(ctx, |ui| {
                self.sidebar.ui(ui);
            });
        
        // Status bar (bottom)
        egui::TopBottomPanel::bottom("status")
            .exact_height(32.0)
            .show(ctx, |ui| {
                self.status_bar.ui(ui);
            });
        
        // Main content (center)
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.sidebar.current_view {
                SidebarView::Chat => self.render_chat(ui),
                SidebarView::Knowledge => self.render_knowledge(ui),
                SidebarView::Search => self.render_search(ui),
                SidebarView::Settings => self.render_settings(ui),
            }
        });
    }
}
```

### Component Communication

Components return **actions** rather than mutating state directly:

```rust
// In chat.rs
pub enum ChatAction {
    Query(String),      // Search knowledge
    Learn(String),      // Teach new knowledge  
    Help,               // Show help
    Clear,              // Clear chat
    Stats,              // Show statistics
}

impl ChatPanel {
    pub fn ui(&mut self, ui: &mut egui::Ui) -> Option<ChatAction> {
        // ... render UI ...
        
        if send_button_clicked {
            self.parse_command(&self.input)
        } else {
            None
        }
    }
}

// In app.rs
fn handle_chat_action(&mut self, action: ChatAction) {
    match action {
        ChatAction::Learn(content) => {
            self.storage.learn_concept(...);
        }
        ChatAction::Query(query) => {
            // Uses shared text_search from sutra-storage
            let results = self.storage.text_search(&query, 5);
        }
        ChatAction::Help => {}, // Handled in ChatPanel
        ChatAction::Clear => { /* clear messages */ }
        ChatAction::Stats => { /* show stats */ }
    }
}
```

## Storage Operations

### Learning a Concept

```rust
fn learn_concept(&mut self, content: &str) {
    // Generate deterministic ID from content
    let id = ConceptId::from_bytes(md5::compute(content).0);
    
    // Store in sutra-storage
    self.storage.learn_concept(
        id,
        content.as_bytes().to_vec(),
        None,  // No embedding vector (local mode)
        1.0,   // Initial strength
        1.0,   // Initial confidence
    ).expect("Failed to store concept");
    
    // Update UI state
    self.status_bar.set_concept_count(
        self.storage.get_snapshot().all_concepts().len()
    );
    self.status_bar.set_activity("Learned new concept");
}
```

### Querying Concepts

Desktop uses the **shared `text_search()` method** from `sutra-storage` - the same method available to the enterprise edition. This ensures no code duplication.

```rust
fn handle_query(&mut self, query: &str) {
    // Uses ConcurrentMemory::text_search() - shared with enterprise
    // Filters stop words, ranks by keyword overlap, returns relevance scores
    let results = self.storage.text_search(&query, 5);
    
    // Results: Vec<(ConceptId, String content, f32 relevance)>
    for (id, content, relevance) in results {
        println!("{}: {} ({}%)", id.to_hex(), content, relevance * 100.0);
    }
}
```

**How text_search works:**
1. Extracts keywords from query (filters stop words like "what", "is", "the")
2. Searches all concepts for keyword matches
3. Scores by number of matching keywords
4. Returns results sorted by relevance (0.0 - 1.0)

Example: Query "What is the capital of India?" → Keywords: ["capital", "india"] → Matches "Delhi is capital of India" with 100% relevance.

### Browsing All Concepts

```rust
fn get_all_concepts(&self) -> Vec<ConceptInfo> {
    let snapshot = self.storage.get_snapshot();
    
    snapshot.all_concepts()
        .iter()
        .map(|node| ConceptInfo {
            id: format!("{:032x}", node.id),
            content: String::from_utf8_lossy(&node.content).to_string(),
            strength: node.strength,
            confidence: node.confidence,
            neighbors: node.neighbors.iter()
                .map(|n| format!("{:032x}", n))
                .collect(),
        })
        .collect()
}
```

## Theme System

### Color Palette Definition

```rust
// desktop/src/theme.rs

// Primary palette - Vibrant and modern
pub const PRIMARY: Color32 = Color32::from_rgb(167, 139, 250);    // Purple
pub const PRIMARY_DIM: Color32 = Color32::from_rgb(139, 92, 246); // Deep Purple
pub const SECONDARY: Color32 = Color32::from_rgb(96, 165, 250);   // Sky Blue
pub const ACCENT: Color32 = Color32::from_rgb(251, 191, 36);      // Amber
pub const SUCCESS: Color32 = Color32::from_rgb(52, 211, 153);     // Emerald
pub const WARNING: Color32 = Color32::from_rgb(251, 146, 60);     // Orange

// Background hierarchy
pub const BG_DARK: Color32 = Color32::from_rgb(15, 15, 25);       // Deepest
pub const BG_PANEL: Color32 = Color32::from_rgb(22, 22, 35);      // Panels
pub const BG_SIDEBAR: Color32 = Color32::from_rgb(18, 18, 30);    // Sidebar
pub const BG_WIDGET: Color32 = Color32::from_rgb(35, 35, 55);     // Cards
pub const BG_HOVER: Color32 = Color32::from_rgb(45, 45, 70);      // Hover
pub const BG_ELEVATED: Color32 = Color32::from_rgb(40, 40, 62);   // Raised

// Text hierarchy
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(248, 250, 252);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(148, 163, 184);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(100, 116, 139);
```

### Applying Theme to egui

```rust
pub fn setup_custom_theme(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();
    
    // Window styling
    visuals.window_fill = BG_PANEL;
    visuals.window_stroke = Stroke::new(1.0, Color32::from_rgb(50, 50, 75));
    visuals.window_rounding = Rounding::same(16.0);
    
    // Widget styling
    visuals.widgets.noninteractive.bg_fill = BG_WIDGET;
    visuals.widgets.noninteractive.rounding = Rounding::same(10.0);
    
    visuals.widgets.hovered.bg_fill = BG_HOVER;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, PRIMARY.gamma_multiply(0.4));
    
    visuals.widgets.active.bg_fill = PRIMARY.gamma_multiply(0.25);
    visuals.widgets.active.fg_stroke = Stroke::new(1.5, PRIMARY);
    
    // Selection
    visuals.selection.bg_fill = PRIMARY.gamma_multiply(0.25);
    visuals.selection.stroke = Stroke::new(1.0, PRIMARY);
    
    ctx.set_visuals(visuals);
}
```

## Data Flow

### Learning Flow

```
User Input → ChatPanel → ChatAction::LearnConcept
                              ↓
                        SutraApp::learn_concept()
                              ↓
                        ConceptId::from_bytes(md5)
                              ↓
                        storage.learn_concept()
                              ↓
                   ┌──────────┴──────────┐
                   ↓                     ↓
              ConcurrentMemory      WAL Write
                   ↓                     ↓
              In-Memory Graph      Disk Persistence
```

### Query Flow

```
User Query → ChatPanel → ChatAction::QueryConcept
                              ↓
                        SutraApp::query_concept()
                              ↓
                        storage.get_snapshot()
                              ↓
                        Text Matching / Vector Search
                              ↓
                        Best Match Found
                              ↓
                        ChatPanel::add_message(Response)
```

## Performance Characteristics

### Memory Usage

| Component | Typical Usage |
|-----------|---------------|
| Base App | ~50 MB |
| Per 1K Concepts | ~2-5 MB |
| HNSW Index | ~4 bytes × vectors × dimensions |
| UI Overhead | ~20 MB |

### Startup Time

| Phase | Duration |
|-------|----------|
| Window Creation | ~100ms |
| Theme Setup | ~10ms |
| Storage Init | ~50ms |
| Index Load | ~100-500ms (depends on size) |
| **Total** | **~300-700ms** |

### Operations

| Operation | Latency |
|-----------|---------|
| Learn Concept | <1ms |
| Query (text match) | <10ms for 10K concepts |
| Query (vector) | <50ms for 100K concepts |
| UI Frame | ~16ms (60 FPS) |

## Future Considerations

### Local Embeddings

Planned integration with ONNX Runtime for local embedding generation:

```rust
// Future: Local embedding service
pub struct LocalEmbedder {
    session: ort::Session,
    tokenizer: Tokenizer,
}

impl LocalEmbedder {
    pub fn embed(&self, text: &str) -> Vec<f32> {
        let tokens = self.tokenizer.encode(text);
        let output = self.session.run(tokens);
        output.into_vec()
    }
}
```

### Multi-Window Support

egui supports multiple viewports:

```rust
// Future: Detachable panels
ctx.show_viewport_immediate(
    ViewportId::from_hash_of("knowledge_popup"),
    ViewportBuilder::default().with_title("Knowledge Browser"),
    |ctx, _| {
        self.knowledge.ui(ctx);
    }
);
```

### Plugin System

Planned extension points:

```rust
// Future: Plugin trait
pub trait SutraPlugin {
    fn name(&self) -> &str;
    fn on_concept_learned(&mut self, concept: &ConceptNode);
    fn on_query(&mut self, query: &str) -> Option<String>;
    fn render_panel(&mut self, ui: &mut egui::Ui);
}
```

## Related Documentation

- [Desktop README](./README.md)
- [Building Desktop](./BUILDING.md)
- [UI Components](./UI_COMPONENTS.md)

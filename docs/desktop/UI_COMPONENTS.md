# Sutra Desktop UI Components

**Version:** 3.3.1  
**Updated:** November 28, 2025

This document provides a detailed reference for all UI panels and components in Sutra Desktop.

---

## Architecture Overview

**Vendored AI Dependencies:**
- `packages/sutra-embedder` - ONNX embedding library (all-mpnet-base-v2, 768D)
- `packages/sutraworks-model` - NLG/text generation library with RWKV support
- Both packages embedded in workspace, no external git dependencies

**Key Files:**
- `desktop/src/local_embedding.rs` - Integration with vendored `sutra-embedder`
- `desktop/src/local_nlg.rs` - Integration with vendored `sutraworks-model`
- First launch downloads AI models (~420MB) automatically

---

## Panel Overview

| Panel | File | Purpose | Action Type |
|-------|------|---------|-------------|
| Menu Bar | `app.rs` | Native menu integration | Direct navigation |
| Sidebar | `sidebar.rs` | Navigation | Direct state update |
| Home | `home.rs` | Dashboard landing page | `HomeAction` |
| Chat | `chat.rs` | Conversational learning | `ChatAction` |
| Knowledge | `knowledge.rs` | Concept browser | `KnowledgeAction` |
| Settings | `settings.rs` | Configuration | `SettingsAction` |
| Status Bar | `status_bar.rs` | Status display | None (display only) |
| Onboarding | `onboarding.rs` | First-launch tour | None (overlay) |
| Graph View | `graph_view.rs` | Visual graph | `GraphAction` |
| Reasoning Paths | `reasoning_paths.rs` | MPPA analysis | `ReasoningPathsAction` |
| Temporal View | `temporal_view.rs` | Timeline analysis | `TemporalViewAction` |
| Causal View | `causal_view.rs` | Root cause analysis | `CausalViewAction` |
| Analytics | `analytics.rs` | Performance metrics | `AnalyticsAction` |
| Query Builder | `query_builder.rs` | Advanced search | `QueryBuilderAction` |
| Export/Import | `export_import.rs` | Data portability | `ExportImportAction` |
| Undo/Redo | `undo_redo.rs` | Command stack | `UndoCommand` trait |

---

## Menu Bar (`app.rs`)

### Purpose
Native menu bar integration providing quick access to core functionality.

### Structure
The menu bar is rendered as part of the main application update loop.

### Features

**File Menu:**
- Import Data... (⌘I) - Open import dialog
- Export Data... (⌘E) - Open export dialog  
- Settings (⌘,) - Open settings panel
- Quit (⌘Q) - Close application (macOS/Linux)

**View Menu:**
- Chat (⌘1) - Switch to chat panel
- Knowledge (⌘2) - Switch to knowledge browser
- Search (⌘3) - Switch to search
- Analysis submenu:
  - Graph View (⌘G) - Visual knowledge graph
  - Reasoning Paths (⌘R) - MPPA analysis
  - Timeline (⌘T) - Temporal analysis
  - Causality (⌘C) - Root cause analysis
- Tools submenu:
  - Analytics (⌘A) - Performance metrics
  - Query Builder (⌘Q) - Advanced search

**Help Menu:**
- Documentation - Open GitHub docs
- Quick Start Guide - Open getting started
- About Sutra - Show version and info

### Layout
```
┌─────────────────────────────────────────────────┐
│ File  View  Help         💬 Chat (breadcrumb)  │
└─────────────────────────────────────────────────┘
```

---

## Sidebar (`sidebar.rs`)

### Purpose
Primary navigation component with collapsible sections.

### Structure
```rust
pub struct Sidebar {
    pub current_view: SidebarView,
    pub analysis_collapsed: bool,
    pub tools_collapsed: bool,
}

pub enum SidebarView {
    // MAIN
    Home,      // NEW: Default landing page
    Chat, Knowledge, Search,
    // ANALYSIS (collapsible)
    Graph, Paths, Timeline, Causal,
    // TOOLS (collapsible)
    Analytics, Query, Export,
    // Always visible
    Settings,
}
```

### Visual Design
- Logo with enhanced neural/brain icon and glow effect
- Version badge in pill style with PRIMARY_LIGHT color
- Section headers in uppercase with smooth collapsible arrows
- Navigation items with enhanced icon backgrounds and hover states
- Selected state shows thicker left accent bar (4px) with glow effect
- Improved hover states with better color transitions
- Tooltips on hover for additional context
- Subtle right border separating from main content

### Layout
```
┌──────────────────────┐
│  [Logo] Sutra AI     │
│  Desktop    v3.3     │
├──────────────────────┤
│  MAIN                │
│  🏠 Home (DEFAULT)    │
│  💬 Chat             │
│  📚 Knowledge        │
│  🔍 Search           │
├──────────────────────┤
│  ▼ ANALYSIS          │
│  🕸️ Graph View       │
│  🛤️ Reasoning        │
│  ⏱️ Timeline         │
│  🔗 Causality        │
├──────────────────────┤
│  ▼ TOOLS             │
│  📊 Analytics        │
│  🔎 Query Builder    │
│  📤 Export/Import    │
├──────────────────────┤
│  ⚙️ Settings         │
└──────────────────────┘
```

---

## Home Dashboard (`home.rs`) - NEW

### Purpose
Default landing page with at-a-glance stats, quick actions, and personalized guidance.

### Structure
```rust
pub struct HomePanel {
    pub stats: DashboardStats,
    pub recent_activity: Vec<ActivityItem>,
    pub tips: Vec<LearningTip>,
    pub current_tip_index: usize,
}

pub enum HomeAction {
    NavigateToChat,
    NavigateToKnowledge,
    NavigateToGraph,
    NavigateToImport,
    LearnExample(String),
    RefreshStats,
}
```

### Features

**Personalized Greeting:**
- Time-of-day aware greeting (Good morning/afternoon/evening/night)
- Username display (if configured)

**Stats Cards:**
- Total concepts with trend indicator
- Total connections count
- Knowledge health score (0-100%)
- Last activity timestamp

**Quick Actions:**
- "Start Learning" → Chat panel
- "Browse Knowledge" → Knowledge panel
- "View Graph" → Graph visualization
- "Import Data" → Export/Import panel

**Recent Activity:**
- Last 5 learning/search activities
- Timestamp with relative time ("2 minutes ago")
- Activity type icons

**Learning Tips:**
- Rotating tips carousel
- Examples of what to teach Sutra
- "Try This" button to auto-populate chat

### Layout
```
┌────────────────────────────────────────────────────────────┐
│ 👋 Good afternoon!                         [↻ Refresh]    │
├────────────────────────────────────────────────────────────┤
│                                                            │
│ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐           │
│ │ 📚 Concepts │ │ 🔗 Connect  │ │ 💚 Health   │           │
│ │    1,247    │ │    4,892    │ │     92%     │           │
│ │   +47 today │ │  3.9 ratio  │ │  Excellent  │           │
│ └─────────────┘ └─────────────┘ └─────────────┘           │
│                                                            │
├────────────────────────────────────────────────────────────┤
│ ⚡ Quick Actions                                           │
│ [💬 Start Learning] [📚 Browse] [🕸️ Graph] [📥 Import]   │
├────────────────────────────────────────────────────────────┤
│ 📋 Recent Activity                                         │
│ • 2m ago: Learned "Memory management in Rust"             │
│ • 15m ago: Searched for "async patterns"                  │
│ • 1h ago: Learned "Error handling best practices"         │
├────────────────────────────────────────────────────────────┤
│ 💡 Learning Tip                              [← 1/5 →]    │
│ "Try teaching causal relationships:                       │
│  'Memory leaks cause performance degradation'"            │
│                                        [✨ Try This]      │
└────────────────────────────────────────────────────────────┘
```

---

## Onboarding Tour (`onboarding.rs`) - NEW

### Purpose
Interactive first-launch overlay that guides new users through key features.

### Structure
```rust
pub struct OnboardingTour {
    pub is_active: bool,
    pub current_step: usize,
    pub steps: Vec<TourStep>,
}

pub struct TourStep {
    pub title: String,
    pub description: String,
    pub highlight_area: Option<Rect>,
    pub icon: String,
}
```

### Tour Steps (7 total)

1. **Welcome** - Introduction to Sutra Desktop
2. **Concepts** - How knowledge is stored
3. **Search** - Finding information with `/search`
4. **Commands** - Slash command overview
5. **Keyboard** - Keyboard shortcuts
6. **Tips** - Pro tips for power users
7. **Complete** - Ready to start!

### Features

**Overlay Rendering:**
- Semi-transparent dark backdrop
- Spotlight effect on highlighted areas
- Smooth fade transitions

**Navigation:**
- "Next" / "Previous" buttons
- Step indicator dots (○ ● ○ ○)
- "Skip Tour" option
- Auto-dismisses on final step

**Persistence:**
- Config file: `~/.sutra/desktop/onboarding.conf`
- Set `completed=true` after tour
- "Reset Onboarding" in Settings to replay

### Layout
```
┌────────────────────────────────────────────────────────────┐
│░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░│
│░░░░┌──────────────────────────────────────────┐░░░░░░░░░░░│
│░░░░│  🎉 Welcome to Sutra Desktop!            │░░░░░░░░░░░│
│░░░░│                                          │░░░░░░░░░░░│
│░░░░│  Your personal knowledge companion.      │░░░░░░░░░░░│
│░░░░│  Let's take a quick tour of the          │░░░░░░░░░░░│
│░░░░│  main features.                          │░░░░░░░░░░░│
│░░░░│                                          │░░░░░░░░░░░│
│░░░░│  ○ ● ○ ○ ○ ○ ○                          │░░░░░░░░░░░│
│░░░░│                                          │░░░░░░░░░░░│
│░░░░│  [Skip Tour]              [Next →]       │░░░░░░░░░░░│
│░░░░└──────────────────────────────────────────┘░░░░░░░░░░░│
│░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░│
└────────────────────────────────────────────────────────────┘
```

---

## Undo/Redo System (`undo_redo.rs`) - NEW

### Purpose
Global command stack for reversing destructive actions.

### Structure
```rust
pub trait UndoCommand: Send + Sync {
    fn execute(&self, storage: &ConcurrentMemory) -> Result<()>;
    fn undo(&self, storage: &ConcurrentMemory) -> Result<()>;
    fn description(&self) -> String;
}

pub struct UndoStack {
    pub commands: VecDeque<Box<dyn UndoCommand>>,
    pub redo_stack: VecDeque<Box<dyn UndoCommand>>,
    pub max_size: usize,
    pub enabled: bool,
}
```

### Supported Commands

| Command | Description | Undo Behavior |
|---------|-------------|---------------|
| `DeleteConceptCommand` | Delete single concept | Restores concept + edges |
| `DeleteBatchCommand` | Delete multiple concepts | Restores all concepts + edges |
| `ClearAllDataCommand` | Clear entire knowledge base | Restores full backup |

### Features

**Keyboard Shortcuts:**
- **⌘Z** (Mac) / **Ctrl+Z** (Win/Linux): Undo
- **⌘⇧Z** (Mac) / **Ctrl+Shift+Z** (Win/Linux): Redo

**Settings Integration:**
- Enable/disable toggle in Settings → Appearance
- Default: Enabled
- When disabled, destructive actions are permanent

**Capacity:**
- Maximum 50 commands in history
- FIFO eviction when full
- Redo stack clears on new command

### Usage Flow
```
1. User deletes concept
   ↓
2. DeleteConceptCommand created with backup
   ↓
3. Command pushed to undo stack
   ↓
4. User presses ⌘Z
   ↓
5. Command.undo() restores concept
   ↓
6. Command moved to redo stack
```

---

## Chat Panel (`chat.rs`)

### Purpose
Primary conversational interface for learning and querying knowledge with enhanced visual design.

### Structure
```rust
pub struct ChatPanel {
    pub messages: Vec<Message>,
    pub input: String,
    pub is_processing: bool,
    autocomplete_index: i32,
    show_autocomplete: bool,
}

pub enum ChatAction {
    Query(String),
    Learn(String),
    Help,
    Clear,
    Stats,
}
```

### Enhanced Features

**Modern Header Design:**
- Card-style elevated header with subtle borders
- Icon in colored background pill
- Vertical layout for title and message count
- Enhanced Clear button with tooltip
- Better visual hierarchy

**Slash Commands:**
| Command | Shortcut | Description |
|---------|----------|-------------|
| `/learn` | `/l` | Teach new knowledge |
| `/search` | `/s`, `/find` | Search knowledge |
| `/help` | `/h`, `/?` | Show help |
| `/clear` | `/c` | Clear chat |
| `/stats` | `/status` | Show statistics |

**Input Features:**
- **Multiline Support**: Auto-expanding text area for long content
- **Smart Keys**: `Enter` to send, `Shift+Enter` for new line
- **Async Processing**: Non-blocking message handling with typing indicator

**Autocomplete:**
- Triggered when input starts with `/`
- Keyboard navigation: ↑/↓ arrows
- Accept with Enter or Tab
- Close with Escape
- Mouse hover updates selection

**Message Bubbles:**
- User messages: Right-aligned, purple theme
- Assistant messages: Left-aligned, gray theme
- System messages: Centered, amber theme
- All include timestamp and avatar

### Layout
```
┌────────────────────────────────┐
│ 💬 Chat           [5] 🗑 Clear │
├────────────────────────────────┤
│                                │
│  [System] Welcome! ...         │
│                                │
│              [User] /learn ... │
│                                │
│  [Sutra] ✅ Learned! ...       │
│                                │
│                                │
├────────────────────────────────┤
│ ⌨️ Commands   ↑↓ Enter Esc    │
│ /learn   /l   Teach knowledge  │
│ /search  /s   Search knowledge │
├────────────────────────────────┤
│ [Type a question, or /...]     │
│                      [Send →]  │
└────────────────────────────────┘
```

---

## Knowledge Panel (`knowledge.rs`)

### Purpose
Browse, search, and manage learned concepts.

### Structure
```rust
pub struct KnowledgePanel {
    pub concepts: Vec<ConceptInfo>,
    pub selected_concept: Option<String>,
    pub search_query: String,
    pub is_loading: bool,
    pub delete_confirmation: Option<String>,
}

pub enum KnowledgeAction {
    Search(String),
    Refresh,
    SelectConcept(String),
    DeleteConcept(String),
    SwitchToChat,
}
```

### Features
- **Responsive Grid**: Masonry-style layout adapting to window width
- **Real-time Search**: Instant filtering of concepts
- **Concept Cards**: Compact view with preview, confidence, and stats
- **Safe Deletion**: Confirmation modal to prevent accidental data loss
- **Interactive Empty State**: "Start Learning" button guides users to chat

### Layout
```
┌──────────────────┬─────────────────────────┐
│ 🧠 Knowledge Base│  [Refresh] [Count]      │
├──────────────────┴─────────────────────────┤
│ 🔍 [Search concepts...]                    │
├────────────────────────────────────────────┤
│                                            │
│ ┌──────────────┐  ┌──────────────┐         │
│ │ Concept 1    │  │ Concept 2    │         │
│ │ Content...   │  │ Content...   │         │
│ │ ⚡92% 🔗 3   │  │ ⚡88% 🔗 5   │         │
│ └──────────────┘  └──────────────┘         │
│                                            │
│ ┌──────────────┐  ┌──────────────┐         │
│ │ Concept 3    │  │ Concept 4    │         │
│ │ ...          │  │ ...          │         │
│ └──────────────┘  └──────────────┘         │
└────────────────────────────────────────────┘
```

---

## Graph View (`graph_view.rs`)

### Purpose
Interactive force-directed visualization of the knowledge graph.

### Structure
```rust
pub struct GraphView {
    pub nodes: HashMap<ConceptId, GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub camera: Camera,
    pub selected: Option<ConceptId>,
    pub hovered: Option<ConceptId>,
    pub filters: GraphFilters,
    pub layout_type: LayoutType,
    pub simulation_running: bool,
}

pub enum GraphAction {
    SelectNode(ConceptId),
    Refresh,
    ExportImage,
}
```

### Features

**Visualization:**
- Force-directed layout with physics simulation
- Node sizing based on confidence (larger = more confident)
- Edge coloring by type (semantic, causal, temporal, etc.)
- Zoom (scroll) and pan (drag) controls
- Node selection and hover states

**Filters:**
- Confidence threshold slider
- Toggle edge types visibility
- Layout type selector (Force, Radial, Grid)

**Interaction:**
- Click node to select
- Drag node to reposition
- Drag background to pan
- Scroll to zoom
- Pause/resume simulation
- **Smart Empty State:** Shows "Knowledge Graph is Empty" with quick actions to `/learn` or `/import` when no data exists.

### Edge Type Colors
| Type | Color | Pattern |
|------|-------|---------|
| Semantic | Purple | Solid |
| Causal | Red | Solid |
| Temporal | Blue | Solid |
| Hierarchical | Green | Solid |
| Similar | Amber | Solid |

### Layout
```
┌──────────────────────────────────────────┐
│ 🧠 Knowledge Graph  [24 nodes] [56 edges]│
│                    [Filters ▼] [↻ Refresh]│
├──────────────────────────────────────────┤
│                                          │
│         ● Concept A                      │
│         │                                │
│    ┌────┼────┐                          │
│    ●    ●    ●                          │
│    B    C    D                          │
│                                          │
│  [Legend]                                │
│  ─── Semantic  ─── Causal  ─── Temporal │
├──────────────────────────────────────────┤
│ [⌖ Center] [−] 100% [+] Layout: Force   │
│ [⏸ Pause] Selected: Concept A | 3 neighbors│
└──────────────────────────────────────────┘
```

---

## Reasoning Paths Panel (`reasoning_paths.rs`)

### Purpose
MPPA-style multi-path reasoning with consensus analysis.

### Structure
```rust
pub struct ReasoningPathsPanel {
    pub query_from: String,
    pub query_to: String,
    pub paths: Vec<ReasoningPath>,
    pub consensus: Option<ConsensusResult>,
    pub expanded_path: Option<usize>,
    pub max_depth: usize,
    pub max_paths: usize,
}

pub enum ReasoningPathsAction {
    FindPaths(String, String),
    ExportReasoning,
}
```

### Features

**Path Finding:**
- Enter source and target concepts
- Parallel pathfinding using storage engine
- Configurable max depth and max paths

**Consensus Analysis:**
- Clusters paths by destination
- Calculates confidence and support ratio
- MPPA scoring formula with consensus bonus
- Identifies primary answer and alternatives

**Visualization:**
- Consensus summary with confidence badge
- Path cards with expand/collapse
- Step-by-step path detail view
- **Smart Empty State:** Shows "Discover Connections" guide explaining how to use the path finder when no paths are active.
- Color-coded support indicators

### Layout
```
┌──────────────────────────────────────────┐
│ 🕸️ Reasoning Paths            [5 found] │
├──────────────────────────────────────────┤
│ Find paths between concepts:             │
│ From: [memory leak___] To: [crash___]    │
│                           [🔍 Find Paths] │
├──────────────────────────────────────────┤
│ ╔════════════════════════════════════╗   │
│ ║ ✓ CONSENSUS: System failure        ║   │
│ ║ Confidence: 89% | 4/5 paths agree  ║   │
│ ╚════════════════════════════════════╝   │
├──────────────────────────────────────────┤
│ ✓ Path 1 (92% conf) - 3 hops       [▶]  │
│   memory → resource → failure → crash    │
│                                          │
│ ✓ Path 2 (87% conf) - 2 hops       [▶]  │
│   memory → OOM → crash                   │
├──────────────────────────────────────────┤
│ Max depth: [═══●═══] 6                   │
│ Max paths: [═══●═══] 10  [📤 Export]    │
└──────────────────────────────────────────┘
```

---

## Temporal View (`temporal_view.rs`)

### Purpose
Timeline-based visualization of temporal relationships.

### Structure
```rust
pub struct TemporalView {
    pub time_range: TimeRange,
    pub zoom_level: f32,
    pub events: Vec<TimelineEvent>,
    pub selected_event: Option<usize>,
    pub temporal_relations: Vec<(usize, usize, TemporalRelation)>,
    pub view_mode: TimelineViewMode,
}

pub enum TemporalViewAction {
    ViewInGraph(String),
    ExploreRelations(String, TemporalRelation),
    RefreshData,
}
```

### View Modes

**Timeline View:**
- Horizontal timeline with event markers
- Events positioned above/below for clarity
- Curved relation lines between events
- Color-coded by relation type

**List View:**
- Scrollable list of events
- Timestamp badges
- Relation count indicators

**Matrix View:**
- N×N grid showing all relations
- Symbols: → Before, ← After, ⊂ During, = Concurrent
- Color-coded cells
**Smart Empty State:**
- Displays "No Temporal Events Found" when the timeline is empty.
- Provides examples of temporal statements (e.g., "Event A happened before Event B").


### Layout
```
┌──────────────────────────────────────────┐
│ ⏱️ Temporal Analysis  [📅] [📋] [📊]    │
├──────────────────────────────────────────┤
│ Range: [All Time ▼] Zoom: [═══●═══]     │
│ 🔍 [Filter events...] ☑ Show Relations   │
├──────────────────────────────────────────┤
│                                          │
│         ●──────●──────●──────●           │
│    T-2     T-1     T0     T+1            │
│  Deploy   Load   Crash  Restart          │
│                                          │
│     ╭─────── CAUSES ───────╮            │
│                                          │
├──────────────────────────────────────────┤
│ 📌 Selected: System crash (T0)           │
│    Related: 2 temporal relations         │
│    [View in Graph]                       │
└──────────────────────────────────────────┘
```

---

## Causal View (`causal_view.rs`)

### Purpose
Root cause analysis and causal chain exploration.

### Structure
```rust
pub struct CausalView {
    pub effect_query: String,
    pub max_hops: usize,
    pub causal_chains: Vec<CausalChain>,
    pub root_causes: Vec<CausalNode>,
    pub view_mode: CausalViewMode,
}

pub enum CausalViewAction {
    AnalyzeCause { effect: String, max_hops: usize },
    ExploreNode(String),
    ExportChains,
}
```

### View Modes

**Chain List:**
- Numbered chains with arrow notation
- Confidence and hop count badges
- Click to select chain

**Tree View:**
- Hierarchical node display
- Expand/collapse nodes
- Root cause highlighting
- Click to explore node

**Graph View:**
**Smart Empty State:**
- Displays "Root Cause Analysis" guide when no causal chains are found.
- Explains how to teach causal relationships (e.g., "X causes Y").

- Circular layout of all nodes
- Directed edges between causes
- Root causes in red
- Interactive selection

### Layout
```
┌─────────────────────────────────────────────────┐
│ 🔗 Causal Analysis         [📋] [🌳] [🕸️]     │
├─────────────────────────────────────────────────┤
│ Effect: [system crash____________] Max: [5]    │
│ ☑ Include indirect          [🔍 Analyze]       │
├─────────────────────────┬───────────────────────┤
│ Found 3 causal chains   │ 🎯 Root Causes        │
│                         │ • Unclosed connections│
│ #1 (92% conf)          │ • Memory leak         │
│ ↑ Memory leak           │                       │
│ ↑ Unclosed connections  │ 📊 Impact Analysis    │
│ 🎯 ROOT CAUSE           │ [████████░░] High     │
│                         │ [█████░░░░░] Medium   │
└─────────────────────────┴───────────────────────┘
```

---

## Analytics Dashboard (`analytics.rs`)

### Purpose
Real-time performance and usage metrics.

### Structure
```rust
pub struct AnalyticsDashboard {
    pub metrics: AnalyticsMetrics,
    pub history: VecDeque<MetricsSnapshot>,
    pub activity_log: VecDeque<ActivityEntry>,
    pub query_log: VecDeque<QueryLogEntry>,
    pub top_queries: Vec<(String, usize)>,
}

pub enum AnalyticsAction {
    ExportReport,
    ClearHistory,
}
```

### Metrics Displayed

**Knowledge Growth:**
- Total concept count
- Daily additions
- Sparkline chart

**Query Performance:**
- Average latency
- P95 latency
- P99 latency
- Latency histogram

**Storage Statistics:**
- Concept count
- Edge count
- Vector count

**HNSW Health:**
- Coverage percentage
- Indexed vs total
- Health status indicator

### Layout
```
┌─────────────────────────────────────────────────┐
│ 📊 Analytics Dashboard    [Last 24h ▼] [📤]    │
├────────────────────────┬────────────────────────┤
│ Knowledge Growth       │ Learning Activity      │
│ 1,247 ↗ +47           │ +47 concepts today     │
│ concepts               │ Peak: 14:00            │
│ [▃▅▆▇▅▄▃▂▃▄▅▆]        │                        │
├────────────────────────┼────────────────────────┤
│ Query Performance      │ Storage Statistics     │
│ Avg: 12ms  P95: 34ms  │ Concepts: 1,247        │
│ P99: 89ms             │ Edges: 4,892           │
│ [▃▅▃▄▃▂▃▅▃▄]         │ Vectors: 1,147         │
├────────────────────────┴────────────────────────┤
│ Top Queries    │ HNSW Index Health              │
│ 1. "crash" 24  │ [████████████░] 92%           │
│ 2. "memory" 18 │ Indexed: 1147/1247            │
│ 3. "db" 12     │ ✓ Healthy                      │
├─────────────────────────────────────────────────┤
│ Recent Activity                                 │
│ 14:32 📚 Learned: Memory leak in...            │
│ 14:30 🔍 Query: "system crash" (12ms)          │
│ 14:28 📚 Learned: OOM killer...                │
└─────────────────────────────────────────────────┘
```

---

## Query Builder (`query_builder.rs`)

### Purpose
Visual advanced search with filters and saved queries.

### Structure
```rust
pub struct QueryBuilder {
    pub query_type: QueryType,
    pub query_text: String,
    pub filters: QueryFilters,
    pub results: Vec<ConceptInfo>,
    pub saved_queries: Vec<SavedQueryEntry>,
}

pub enum QueryBuilderAction {
    RunQuery { query_type: QueryType, query: String, filters: QueryFilters },
    ExportResults,
    VisualizeResults,
}
```

### Query Types
- **Text Search**: Keyword-based content search
- **Semantic Search**: Vector similarity search
- **Path Finding**: Find paths between concepts

### Filters
- Confidence threshold slider
- Max results limit
- EF search parameter (for semantic)
- Relationship filters (must have causal/temporal)
- Minimum neighbor count

### Layout
```
┌──────────────────────────────────────────────────┐
│ 🔍 Advanced Query Builder    [📁 Saved Queries] │
├──────────────────────────────────────────────────┤
│ Query Type: [📝 Text] [🧠 Semantic] [🔗 Path]   │
│ Keyword-based search through concept content     │
├──────────────────────────────────────────────────┤
│ Query Text:                                      │
│ ┌──────────────────────────────────────────────┐│
│ │ memory management                             ││
│ └──────────────────────────────────────────────┘│
├──────────────────────────────────────────────────┤
│ 🎛️ Filters ▼                                    │
│ ☑ Confidence: [════●════] 70%                   │
│   Max results: [════●════] 10                   │
│ ☑ Must have CAUSAL  ☐ Must have TEMPORAL        │
├──────────────────────────────────────────────────┤
│ [Clear] [Save: _________ 💾]      [▶ Run Query] │
├──────────────────────────────────────────────────┤
│ Results ✓ 47 found (12ms)    [📤 Export] [📊]  │
│ ┌────────────────────────────────────────────┐  │
│ │ #1 Memory leak detection     92% conf     │  │
│ │     a3f2e8c1... | 8 connections           │  │
│ └────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────┘
```

---

## Export/Import Panel (`export_import.rs`)

### Purpose
Data portability with multiple format support.

### Structure
```rust
pub struct ExportImportPanel {
    pub export_format: ExportFormat,
    pub export_options: ExportOptions,
    pub import_mode: ImportMode,
    pub batch_progress: BatchProgress,
    pub active_tab: ExportImportTab,
}

pub enum ExportImportAction {
    Export(String),
    Import(String),
    BatchImport(String),
    CancelBatch,
}
```

### Export Formats
| Format | Extension | Use Case |
|--------|-----------|----------|
| JSON | `.json` | Full fidelity backup |
| CSV | `.csv` | Spreadsheet analysis |
| GraphML | `.graphml` | Neo4j import |
| Cypher | `.cypher` | Neo4j scripts |

### Import Modes
- **Merge**: Skip duplicates
- **Overwrite**: Replace all
- **New Workspace**: Fresh start

### Layout
```
┌──────────────────────────────────────────────────┐
│ 📤 Export / Import                               │
├──────────────────────────────────────────────────┤
│ [Export] [Import] [Batch]                        │
├──────────────────────────────────────────────────┤
│ Export Format:                                   │
│ [JSON✓] [CSV] [GraphML] [Cypher]                │
│                                                  │
│ Options:                                         │
│ ☑ Include vectors                                │
│ ☑ Include metadata                               │
│   Min confidence: [════●════] 0.0               │
│   Filter: [___________________]                  │
│                                                  │
│ [📁 Choose Location...] ~/Documents/export.json │
│                                                  │
│ Estimated size: ~45 MB                           │
│                         [⬇ Export]              │
└──────────────────────────────────────────────────┘
```

---

## Settings Panel (`settings.rs`)

### Purpose
Application configuration and data management.

### Structure
```rust
pub struct SettingsPanel {
    pub data_path: String,
    pub vector_dimensions: String,
    pub theme: Theme,
    pub font_size: f32,
    pub stats: StorageStatsUI,
}

pub enum SettingsAction {
    Save,
    ExportData,
    ImportData,
    ClearData,
}
```

### Sections

**Status:**
- Connection status indicator
- Concept and dimension counts

**Storage:**
- Data path configuration
- Vector dimensions setting
- Warning about restart requirements

**Appearance:**
- Theme selector (Dark/Light/High Contrast) - NEW
- Font size slider
- Undo/Redo toggle - NEW

**Onboarding:**
- "Reset Onboarding Tour" button - NEW

**Actions:**
- Export/Import buttons
- Clear all data (with warning)

**About:**
- Version information
- Documentation links

---

## Status Bar (`status_bar.rs`)

### Purpose
Persistent footer showing system status.

### Structure
```rust
pub struct StatusBar {
    pub status: ConnectionStatus,
    pub concept_count: usize,
    pub last_activity: String,
    pub version: String,
}

pub enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
    Error(String),
}
```

### Layout
```
┌──────────────────────────────────────────────────────────────┐
│ [●] Active │ 🧠 1,247 concepts │ ⚡ Searched: memory (12ms) │
│                             💾 Local Storage      v3.3.0    │
└──────────────────────────────────────────────────────────────┘
```

### Enhanced Features
- **Connection indicator**: Always shows "Connected" for local storage
- **Concept count**: Live count from storage engine
- **Activity tracker**: Shows recent operations (learn, search, etc.)
- **AI Model status**: Indicates if real ONNX models are loaded vs fallback
- **Performance metrics**: Query latency, memory usage
- **Model download progress**: Shows download status during first launch

**Visual Improvements:**
- Increased height (32px) for better spacing
- Concept count in highlighted PRIMARY badge
- Activity indicator with lightning bolt icon
- Storage type in SUCCESS color badge
- Version badge with PRIMARY_LIGHT color
- Consistent 16px separators between sections

---

## Theming Reference

### Theme Modes (NEW)

Sutra Desktop supports three theme modes, selectable in Settings:

| Mode | Description | Use Case |
|------|-------------|----------|
| **Dark** | Default premium dark theme | General use, reduces eye strain |
| **Light** | Clean light theme | Bright environments, printing |
| **High Contrast** | WCAG AAA compliant | Accessibility, visual impairments |

### Color Palette (Dark Theme - Default)

```rust
// Primary colors
PRIMARY:      #A78BFA  // Vibrant Purple
PRIMARY_DIM:  #8B5CF6  // Deep Purple  
PRIMARY_LIGHT:#C4B5FD  // Light Purple
SECONDARY:    #60A5FA  // Sky Blue
ACCENT:       #FBBF24  // Amber
SUCCESS:      #34D399  // Emerald
WARNING:      #FB923C  // Orange
ERROR:        #F87171  // Red
INFO:         #60A5FA  // Blue

// Backgrounds
BG_DARK:      #0F0F19  // Darkest
BG_PANEL:     #161623  // Panels
BG_SIDEBAR:   #12121E  // Sidebar
BG_WIDGET:    #232337  // Inputs/cards
BG_HOVER:     #2D2D46  // Hover state
BG_ELEVATED:  #28283E  // Elevated cards

// Text
TEXT_PRIMARY:   #F8FAFC  // Almost white
TEXT_SECONDARY: #A0AEC0  // Improved contrast
TEXT_MUTED:     #7D8CAD  // Better visibility
```

### Color Palette (Light Theme)

```rust
// Backgrounds
BG_DARK:      #FFFFFF  // Pure white
BG_PANEL:     #F8FAFC  // Off-white panels
BG_SIDEBAR:   #F1F5F9  // Light gray sidebar
BG_WIDGET:    #E2E8F0  // Light inputs
BG_HOVER:     #CBD5E1  // Hover state
BG_ELEVATED:  #FFFFFF  // Elevated cards

// Text
TEXT_PRIMARY:   #0F172A  // Near black
TEXT_SECONDARY: #475569  // Medium gray
TEXT_MUTED:     #94A3B8  // Light gray
```

### Color Palette (High Contrast Theme)

```rust
// Backgrounds - Pure black for maximum contrast
BG_DARK:      #000000  // Pure black
BG_PANEL:     #000000  // Pure black
BG_SIDEBAR:   #000000  // Pure black
BG_WIDGET:    #1A1A1A  // Near black
BG_HOVER:     #333333  // Dark gray
BG_ELEVATED:  #0A0A0A  // Very dark

// Primary - Bright cyan for visibility
PRIMARY:      #00FFFF  // Cyan (high visibility)
PRIMARY_DIM:  #00CCCC  // Darker cyan
PRIMARY_LIGHT:#66FFFF  // Bright cyan

// Text - Pure white for maximum contrast
TEXT_PRIMARY:   #FFFFFF  // Pure white
TEXT_SECONDARY: #E0E0E0  // Near white
TEXT_MUTED:     #B0B0B0  // Light gray

// Accents - High visibility colors
ACCENT:       #FFFF00  // Yellow (high visibility)
SUCCESS:      #00FF00  // Bright green
WARNING:      #FFA500  // Orange
ERROR:        #FF0000  // Pure red
```

### Common Styles

**Cards:**
- Background: `BG_ELEVATED`
- Border: `1px` subtle
- Rounding: `12px`
- Padding: `16px`

**Buttons:**
- Primary: `PRIMARY` fill, white text
- Secondary: `BG_WIDGET` fill
- Rounding: `8px`

**Inputs:**
- Background: `BG_DARK`
- Border: `1px BG_WIDGET`
- Rounding: `10px`

---

## Related Documentation

- [README.md](./README.md) - Overview and quick start
- [ARCHITECTURE.md](./ARCHITECTURE.md) - Technical design
- [BUILDING.md](./BUILDING.md) - Build instructions

# Desktop UI Components

**Version:** 1.0.0  
**Updated:** November 26, 2025

Detailed documentation of each UI component in Sutra Desktop.

## Component Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         Window                                   │
├──────────────┬──────────────────────────────────────────────────┤
│              │                                                   │
│   Sidebar    │              Main Content Area                   │
│              │                                                   │
│  ┌────────┐  │  ┌─────────────────────────────────────────────┐│
│  │  Logo  │  │  │                                             ││
│  └────────┘  │  │            Chat / Knowledge /               ││
│              │  │            Search / Settings                 ││
│  ┌────────┐  │  │                                             ││
│  │  Nav   │  │  │                                             ││
│  │ Items  │  │  │                                             ││
│  └────────┘  │  │                                             ││
│              │  └─────────────────────────────────────────────┘│
│  ┌────────┐  │                                                   │
│  │Settings│  │                                                   │
│  └────────┘  │                                                   │
├──────────────┴──────────────────────────────────────────────────┤
│                        Status Bar                                │
└─────────────────────────────────────────────────────────────────┘
```

## Sidebar (`sidebar.rs`)

The navigation sidebar provides access to all main views.

### Structure

```rust
pub struct Sidebar {
    pub current_view: SidebarView,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SidebarView {
    #[default]
    Chat,
    Knowledge,
    Search,
    Settings,
}
```

### Visual Elements

#### Logo Section
- Neural network icon with hexagon pattern
- Animated glow effect
- "Sutra" brand text (24px)
- "AI" badge with background
- Version pill (v3.3)

#### Navigation Items
- Icon in colored circle (28×28px)
- Label text (14px)
- Description hint (11px, always visible)
- Selected state: purple highlight + left accent bar
- Hover state: elevated background

### Rendering

```rust
impl Sidebar {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        // Background with subtle gradient
        let rect = ui.available_rect_before_wrap();
        ui.painter().rect_filled(rect, 0.0, BG_SIDEBAR);
        
        // Right border
        ui.painter().line_segment(
            [rect.right_top(), rect.right_bottom()],
            Stroke::new(1.0, Color32::from_rgb(45, 45, 70))
        );
        
        ui.vertical(|ui| {
            self.draw_logo(ui);
            self.nav_item(ui, "💬", "Chat", "Have a conversation", SidebarView::Chat);
            self.nav_item(ui, "📚", "Knowledge", "Browse concepts", SidebarView::Knowledge);
            self.nav_item(ui, "🔍", "Search", "Find information", SidebarView::Search);
            
            // Bottom-anchored settings
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                self.nav_item(ui, "⚙️", "Settings", "Configure app", SidebarView::Settings);
            });
        });
    }
}
```

---

## Chat Panel (`chat.rs`)

Conversational interface for learning and querying.

### Structure

```rust
pub struct ChatPanel {
    pub messages: Vec<Message>,
    pub input: String,
    pub is_typing: bool,
}

pub struct Message {
    pub role: Role,
    pub content: String,
    pub timestamp: String,
}

pub enum Role {
    User,
    Assistant,
}

pub enum ChatAction {
    SendMessage(String),
    LearnConcept(String),
    QueryConcept(String),
}
```

### Visual Elements

#### Message Bubbles
- **User messages**: Right-aligned, purple background
- **Assistant messages**: Left-aligned, widget background
- Sender badge (You/Sutra) with colored indicator
- Timestamp in muted text
- Glass effect with subtle transparency

#### Input Area
- Multi-line text input with placeholder
- Send button with gradient fill
- Typing indicator (3 animated dots)

### Usage Patterns

```
# Learning (prefix with "learn:")
learn: Python was created by Guido van Rossum in 1991

# Querying (any other message)
Who created Python?
What programming languages do you know about?
```

### Rendering

```rust
impl ChatPanel {
    pub fn ui(&mut self, ui: &mut egui::Ui) -> Option<ChatAction> {
        let mut action = None;
        
        // Message list (scrollable)
        ScrollArea::vertical().show(ui, |ui| {
            for message in &self.messages {
                self.render_message(ui, message);
            }
        });
        
        // Input area
        ui.horizontal(|ui| {
            let resp = ui.add(
                TextEdit::multiline(&mut self.input)
                    .hint_text("Type a message or 'learn: <concept>'...")
            );
            
            if ui.button("Send").clicked() || 
               (resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                action = self.process_input();
            }
        });
        
        action
    }
}
```

---

## Knowledge Panel (`knowledge.rs`)

Browse and explore learned concepts.

### Structure

```rust
pub struct KnowledgePanel {
    pub concepts: Vec<ConceptInfo>,
    pub selected_concept: Option<String>,
    pub search_query: String,
    pub is_loading: bool,
}

pub struct ConceptInfo {
    pub id: String,
    pub content: String,
    pub strength: f32,
    pub confidence: f32,
    pub neighbors: Vec<String>,
}

pub enum KnowledgeAction {
    Search(String),
    Refresh,
    SelectConcept(String),
}
```

### Layout

```
┌──────────────────┬─────────────────────────────────┐
│   Concept List   │       Detail View               │
├──────────────────┼─────────────────────────────────┤
│ 🧠 Knowledge Base│  📋 Concept Details             │
│ Explore concepts │                                 │
│                  │  Identifier                     │
│ [🔍 Search...]   │  a1b2c3d4e5f6...               │
│                  │                                 │
│ ┌──────────────┐ │  Content                        │
│ │ Concept 1    │ │  ┌─────────────────────────┐   │
│ │ id... ⚡98%  │ │  │ The actual concept     │   │
│ └──────────────┘ │  │ content text here...   │   │
│                  │  └─────────────────────────┘   │
│ ┌──────────────┐ │                                 │
│ │ Concept 2    │ │  ┌──────────┐ ┌──────────┐    │
│ │ id... ⚡95%  │ │  │Strength  │ │Confidence│    │
│ └──────────────┘ │  │  98.5%   │ │  95.0%   │    │
│                  │  └──────────┘ └──────────┘    │
│ ...              │                                 │
└──────────────────┴─────────────────────────────────┘
```

### Visual Elements

#### Left Panel (List)
- Header with brain icon
- Search box with magnifier
- Concept count badge
- Refresh button
- Scrollable concept cards

#### Concept Cards
- Content preview (truncated to 60 chars)
- ID badge (first 8 chars)
- Strength indicator with ⚡
- Connection count with 🔗
- Selected state border

#### Right Panel (Detail)
- Full concept content
- Colored stat cards (strength, confidence)
- Connected concepts list

### Empty State

When no concepts exist:
```
       🧠
  No concepts yet
Start a conversation to teach me!
```

---

## Settings Panel (`settings.rs`)

Configuration and system status.

### Structure

```rust
pub struct SettingsPanel {
    pub data_path: String,
    pub vector_dimensions: String,
    pub theme: Theme,
    pub font_size: f32,
    pub stats: StorageStatsUI,
    dirty: bool,
}

pub struct StorageStatsUI {
    pub total_concepts: usize,
    pub vector_dimensions: usize,
    pub data_path: String,
    pub status: StorageStatus,
}

pub enum SettingsAction {
    Save,
    ExportData,
    ImportData,
    ClearData,
}
```

### Sections

#### Status Section
```
📊 Status
┌─────────────────────────────────────────┐
│  ● Running                              │
│                                         │
│  ┌───────────┐  ┌───────────────┐      │
│  │ Concepts  │  │ Dimensions    │      │
│  │    256    │  │    768        │      │
│  └───────────┘  └───────────────┘      │
└─────────────────────────────────────────┘
```

#### Storage Section
```
💾 Storage
┌─────────────────────────────────────────┐
│  Data Path        [.../Application Sup] │
│  Vector Dimensions              [768  ] │
│                                         │
│  ⚠️ Changing dimensions requires restart │
└─────────────────────────────────────────┘
```

#### Appearance Section
```
🎨 Appearance
┌─────────────────────────────────────────┐
│  Theme                      [Dark    ▼] │
│  Font Size               ○───●─── 14.0px│
└─────────────────────────────────────────┘
```

#### Actions Section
```
⚙️ Actions
┌─────────────────────────────────────────┐
│  [Export Data] [Import Data] [Clear All]│
└─────────────────────────────────────────┘
```

#### About Section
```
ℹ️ About
┌─────────────────────────────────────────┐
│  Sutra Desktop                          │
│  Version 3.3.0                          │
│                                         │
│  A self-contained semantic reasoning    │
│  engine with temporal, causal, and      │
│  explainable AI.                        │
│                                         │
│  Documentation • License                │
└─────────────────────────────────────────┘
```

---

## Status Bar (`status_bar.rs`)

System status footer.

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
┌───────────────────────────────────────────────────────────────┐
│ ● Active │ 🧠 256 │ Knowledge refreshed      💾 Local │ v3.3.0│
└───────────────────────────────────────────────────────────────┘
```

### Elements

- **Status badge**: Green dot + "Active" in pill
- **Concept count**: Brain emoji + count
- **Last activity**: Recent action description
- **Storage type**: Floppy + "Local Storage"
- **Version**: Monospace in badge

---

## Theme (`theme.rs`)

Centralized color and style definitions.

### Color Constants

```rust
// Primary palette
pub const PRIMARY: Color32 = Color32::from_rgb(167, 139, 250);    // Purple
pub const PRIMARY_DIM: Color32 = Color32::from_rgb(139, 92, 246);
pub const SECONDARY: Color32 = Color32::from_rgb(96, 165, 250);   // Blue
pub const ACCENT: Color32 = Color32::from_rgb(251, 191, 36);      // Gold
pub const SUCCESS: Color32 = Color32::from_rgb(52, 211, 153);     // Green
pub const WARNING: Color32 = Color32::from_rgb(251, 146, 60);     // Orange
pub const ERROR: Color32 = Color32::from_rgb(248, 113, 113);      // Red

// Backgrounds (darkest to lightest)
pub const BG_DARK: Color32 = Color32::from_rgb(15, 15, 25);
pub const BG_SIDEBAR: Color32 = Color32::from_rgb(18, 18, 30);
pub const BG_PANEL: Color32 = Color32::from_rgb(22, 22, 35);
pub const BG_WIDGET: Color32 = Color32::from_rgb(35, 35, 55);
pub const BG_ELEVATED: Color32 = Color32::from_rgb(40, 40, 62);
pub const BG_HOVER: Color32 = Color32::from_rgb(45, 45, 70);

// Text
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(248, 250, 252);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(148, 163, 184);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(100, 116, 139);
```

### Frame Helpers

```rust
/// Elevated card with border
pub fn elevated_card() -> Frame {
    Frame::none()
        .fill(BG_WIDGET)
        .inner_margin(Margin::same(20.0))
        .rounding(Rounding::same(14.0))
        .stroke(Stroke::new(1.0, Color32::from_rgb(60, 60, 90)))
}

/// Standard card
pub fn card_frame() -> Frame {
    Frame::none()
        .fill(BG_ELEVATED)
        .inner_margin(Margin::same(16.0))
        .rounding(Rounding::same(12.0))
        .stroke(Stroke::new(1.0, Color32::from_rgb(55, 55, 80)))
}
```

### Visual Design Principles

1. **Depth through color**: Darker = further back
2. **Purple as primary action color**
3. **Rounded corners everywhere** (10-16px)
4. **Subtle borders for definition**
5. **Consistent spacing** (4, 8, 12, 16, 20, 24px)
6. **High contrast text for readability**

---

## Component Communication

Components communicate via **action enums**:

```rust
// Component returns optional action
fn ui(&mut self, ui: &mut egui::Ui) -> Option<Action> {
    if button_clicked {
        Some(Action::DoSomething)
    } else {
        None
    }
}

// App handles action
fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    if let Some(action) = self.chat.ui(ui) {
        match action {
            ChatAction::LearnConcept(c) => self.learn(c),
            ChatAction::QueryConcept(q) => self.query(q),
        }
    }
}
```

This pattern ensures:
- Clear data flow
- No circular dependencies
- Easy testing
- Type-safe communication

//! Chat panel - Modern conversational interface

use eframe::egui::{self, Color32, RichText, Rounding, ScrollArea, Stroke, TextEdit, Vec2, Key};
use chrono::{DateTime, Local};
use crate::theme::{PRIMARY, SECONDARY, ACCENT, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_MUTED, BG_WIDGET, BG_DARK, BG_ELEVATED, SUCCESS};

/// Command definition for autocomplete
#[derive(Debug, Clone)]
struct CommandHint {
    command: &'static str,
    shortcut: &'static str,
    description: &'static str,
    needs_arg: bool,
}

const COMMANDS: &[CommandHint] = &[
    CommandHint { command: "/learn", shortcut: "/l", description: "Teach new knowledge", needs_arg: true },
    CommandHint { command: "/search", shortcut: "/s", description: "Search knowledge", needs_arg: true },
    CommandHint { command: "/help", shortcut: "/h", description: "Show all commands", needs_arg: false },
    CommandHint { command: "/clear", shortcut: "/c", description: "Clear chat history", needs_arg: false },
    CommandHint { command: "/stats", shortcut: "/status", description: "Show statistics", needs_arg: false },
];

#[derive(Debug, Clone)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Local>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

pub struct ChatPanel {
    pub messages: Vec<Message>,
    pub input: String,
    pub is_processing: bool,
    /// Selected index in autocomplete suggestions (-1 = none)
    autocomplete_index: i32,
    /// Whether to show autocomplete popup
    show_autocomplete: bool,
}

impl Default for ChatPanel {
    fn default() -> Self {
        Self {
            messages: vec![Message {
                role: MessageRole::System,
                content: "👋 Welcome! I'm Sutra, your personal knowledge assistant.\n\n\
                    • Type `/learn <text>` to teach me something\n\
                    • Ask questions to retrieve knowledge\n\
                    • Type `/help` for all commands\n\
                    • Browse the Knowledge tab to see what I know".into(),
                timestamp: Local::now(),
            }],
            input: String::new(),
            is_processing: false,
            autocomplete_index: -1,
            show_autocomplete: false,
        }
    }
}

impl ChatPanel {
    pub fn ui(&mut self, ui: &mut egui::Ui) -> Option<ChatAction> {
        let mut action = None;
        
        // Get suggestions to calculate space needed
        let suggestions = self.get_command_suggestions();
        let show_autocomplete = self.show_autocomplete && !suggestions.is_empty();
        
        // Calculate space needed for input area + autocomplete
        // Each row ~28px + header ~24px + padding ~12px + input ~56px
        let autocomplete_height = if show_autocomplete {
            (suggestions.len() as f32 * 28.0) + 40.0
        } else {
            0.0
        };
        let input_area_height = 60.0 + autocomplete_height;
        
        ui.vertical(|ui| {
            // Header with gradient accent
            self.render_header(ui);
            
            ui.add_space(12.0);
            
            // Messages area - takes remaining space minus input area
            let available_height = ui.available_height() - input_area_height;
            
            egui::Frame::none()
                .fill(Color32::TRANSPARENT)
                .show(ui, |ui| {
                    ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .stick_to_bottom(true)
                        .max_height(available_height.max(80.0))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            
                            for msg in &self.messages {
                                self.render_message(ui, msg);
                                ui.add_space(16.0);
                            }
                            
                            if self.is_processing {
                                self.render_typing_indicator(ui);
                            }
                        });
                });
            
            // Autocomplete dropdown (above input)
            if show_autocomplete {
                self.render_autocomplete(ui, &suggestions);
            }
            
            // Input area with modern styling
            action = self.render_input_box(ui, &suggestions);
        });
        
        action
    }
    
    fn render_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Title with icon
            ui.label(RichText::new("💬").size(20.0));
            ui.add_space(4.0);
            ui.label(RichText::new("Chat").size(22.0).color(TEXT_PRIMARY).strong());
            
            // Message count badge
            let msg_count = self.messages.len();
            if msg_count > 0 {
                ui.add_space(8.0);
                egui::Frame::none()
                    .fill(BG_WIDGET)
                    .rounding(Rounding::same(10.0))
                    .inner_margin(egui::Margin::symmetric(8.0, 2.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new(format!("{}", msg_count)).size(11.0).color(TEXT_SECONDARY));
                    });
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Clear button with icon
                let clear_btn = egui::Button::new(RichText::new("🗑 Clear").size(12.0).color(TEXT_SECONDARY))
                    .fill(Color32::TRANSPARENT)
                    .stroke(Stroke::new(1.0, BG_WIDGET));
                
                if ui.add(clear_btn).clicked() {
                    self.messages.clear();
                    self.messages.push(Message {
                        role: MessageRole::System,
                        content: "🧹 Chat cleared. Ready for new conversations!".into(),
                        timestamp: Local::now(),
                    });
                }
            });
        });
    }
    
    fn render_typing_indicator(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            egui::Frame::none()
                .fill(BG_WIDGET)
                .rounding(Rounding::same(16.0))
                .inner_margin(egui::Margin::symmetric(16.0, 10.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.add_space(8.0);
                        ui.label(RichText::new("Sutra is thinking...").size(13.0).color(TEXT_SECONDARY).italics());
                    });
                });
        });
    }
    
    fn render_input_box(&mut self, ui: &mut egui::Ui, suggestions: &[&CommandHint]) -> Option<ChatAction> {
        let mut action = None;
        let has_suggestions = !suggestions.is_empty();
        
        egui::Frame::none()
            .fill(BG_DARK)
            .rounding(Rounding::same(12.0))
            .inner_margin(egui::Margin::same(8.0))
            .stroke(Stroke::new(1.0, BG_WIDGET))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Text input with custom styling
                    let input_width = ui.available_width() - 90.0;
                    
                    let text_edit = TextEdit::singleline(&mut self.input)
                        .hint_text("Type a question, or / for commands...")
                        .frame(false)
                        .font(egui::FontId::proportional(14.0));
                    
                    let resp = ui.add_sized(Vec2::new(input_width, 36.0), text_edit);
                    
                    // Handle keyboard navigation for autocomplete
                    let mut submit = false;
                    let mut accept_selection = false;
                    
                    if resp.has_focus() {
                        ui.input(|i| {
                            // Down arrow to navigate down
                            if i.key_pressed(Key::ArrowDown) {
                                if has_suggestions {
                                    if self.autocomplete_index < 0 {
                                        self.autocomplete_index = 0;
                                    } else {
                                        self.autocomplete_index = ((self.autocomplete_index + 1) as usize % suggestions.len()) as i32;
                                    }
                                    self.show_autocomplete = true;
                                }
                            }
                            // Up arrow to navigate up
                            if i.key_pressed(Key::ArrowUp) {
                                if has_suggestions {
                                    if self.autocomplete_index <= 0 {
                                        self.autocomplete_index = suggestions.len() as i32 - 1;
                                    } else {
                                        self.autocomplete_index -= 1;
                                    }
                                    self.show_autocomplete = true;
                                }
                            }
                            // Tab to accept current selection (or first if none selected)
                            if i.key_pressed(Key::Tab) {
                                if has_suggestions && self.show_autocomplete {
                                    if self.autocomplete_index < 0 {
                                        self.autocomplete_index = 0;
                                    }
                                    accept_selection = true;
                                }
                            }
                            // Escape to close autocomplete
                            if i.key_pressed(Key::Escape) {
                                self.show_autocomplete = false;
                                self.autocomplete_index = -1;
                            }
                            // Enter to accept selection or submit
                            if i.key_pressed(Key::Enter) {
                                if self.show_autocomplete && self.autocomplete_index >= 0 && has_suggestions {
                                    accept_selection = true;
                                } else if !self.input.trim().is_empty() && !self.is_processing {
                                    submit = true;
                                }
                            }
                        });
                    }
                    
                    // Accept autocomplete selection
                    if accept_selection && has_suggestions && self.autocomplete_index >= 0 {
                        let selected = &suggestions[self.autocomplete_index as usize];
                        self.input = if selected.needs_arg {
                            format!("{} ", selected.command)
                        } else {
                            selected.command.to_string()
                        };
                        self.show_autocomplete = false;
                        self.autocomplete_index = -1;
                    }
                    
                    // Update autocomplete visibility based on input
                    if self.input.starts_with('/') && !self.input.contains(' ') {
                        if !self.show_autocomplete {
                            // Auto-select first when showing
                            self.autocomplete_index = 0;
                        }
                        self.show_autocomplete = true;
                    } else if !self.input.starts_with('/') {
                        self.show_autocomplete = false;
                        self.autocomplete_index = -1;
                    }
                    
                    ui.add_space(8.0);
                    
                    let can_send = !self.input.trim().is_empty() && !self.is_processing;
                    
                    // Send button with gradient effect when enabled
                    let btn_fill = if can_send { PRIMARY } else { BG_WIDGET };
                    let btn_text_color = if can_send { Color32::WHITE } else { TEXT_MUTED };
                    
                    let send_btn = egui::Button::new(
                        RichText::new(if self.is_processing { "..." } else { "Send →" })
                            .size(13.0)
                            .color(btn_text_color)
                    )
                    .fill(btn_fill)
                    .rounding(Rounding::same(8.0))
                    .min_size(Vec2::new(70.0, 36.0));
                    
                    let btn_resp = ui.add_enabled(can_send, send_btn);
                    
                    // Handle send
                    if btn_resp.clicked() || submit {
                        let content = self.input.trim().to_string();
                        self.input.clear();
                        self.show_autocomplete = false;
                        self.autocomplete_index = -1;
                        
                        self.messages.push(Message {
                            role: MessageRole::User,
                            content: content.clone(),
                            timestamp: Local::now(),
                        });
                        
                        action = self.parse_command(&content);
                    }
                });
            });
        
        action
    }
    
    /// Get command suggestions based on current input
    fn get_command_suggestions(&self) -> Vec<&'static CommandHint> {
        if !self.input.starts_with('/') || self.input.contains(' ') {
            return Vec::new();
        }
        
        let input_lower = self.input.to_lowercase();
        COMMANDS
            .iter()
            .filter(|cmd| {
                cmd.command.starts_with(&input_lower) || 
                cmd.shortcut.starts_with(&input_lower)
            })
            .collect()
    }
    
    /// Render autocomplete dropdown with proper interaction
    fn render_autocomplete(&mut self, ui: &mut egui::Ui, suggestions: &[&CommandHint]) {
        egui::Frame::none()
            .fill(BG_ELEVATED)
            .rounding(Rounding::same(10.0))
            .stroke(Stroke::new(1.0, PRIMARY.gamma_multiply(0.3)))
            .inner_margin(egui::Margin::same(6.0))
            .show(ui, |ui| {
                // Compact header
                ui.horizontal(|ui| {
                    ui.label(RichText::new("⌨️ Commands").size(10.0).color(TEXT_MUTED));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new("↑↓  Enter  Esc").size(9.0).color(TEXT_MUTED));
                    });
                });
                
                ui.add_space(4.0);
                
                let mut clicked_index: Option<usize> = None;
                
                for (i, cmd) in suggestions.iter().enumerate() {
                    let is_selected = i as i32 == self.autocomplete_index;
                    
                    // Create a clickable row - compact height
                    let response = ui.allocate_response(
                        egui::vec2(ui.available_width(), 28.0),
                        egui::Sense::click()
                    );
                    
                    // Handle hover - update selection
                    if response.hovered() && self.autocomplete_index != i as i32 {
                        self.autocomplete_index = i as i32;
                    }
                    
                    // Handle click
                    if response.clicked() {
                        clicked_index = Some(i);
                    }
                    
                    // Draw the row background
                    let bg_color = if is_selected || response.hovered() {
                        PRIMARY.gamma_multiply(0.15)
                    } else {
                        Color32::TRANSPARENT
                    };
                    
                    let rect = response.rect;
                    ui.painter().rect_filled(rect, Rounding::same(6.0), bg_color);
                    
                    // Left indicator for selected item
                    if is_selected {
                        let indicator_rect = egui::Rect::from_min_size(
                            rect.min,
                            egui::vec2(3.0, rect.height())
                        );
                        ui.painter().rect_filled(indicator_rect, Rounding::same(2.0), PRIMARY);
                    }
                    
                    // Draw content
                    let text_rect = rect.shrink2(egui::vec2(10.0, 0.0));
                    
                    // Command name (left)
                    let cmd_color = if is_selected { PRIMARY } else { TEXT_PRIMARY };
                    ui.painter().text(
                        egui::pos2(text_rect.left(), text_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        cmd.command,
                        egui::FontId::proportional(13.0),
                        cmd_color,
                    );
                    
                    // Shortcut (after command)
                    let shortcut_x = text_rect.left() + 70.0;
                    ui.painter().text(
                        egui::pos2(shortcut_x, text_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        &cmd.shortcut,
                        egui::FontId::proportional(11.0),
                        TEXT_MUTED,
                    );
                    
                    // Description (right aligned)
                    ui.painter().text(
                        egui::pos2(text_rect.right(), text_rect.center().y),
                        egui::Align2::RIGHT_CENTER,
                        cmd.description,
                        egui::FontId::proportional(12.0),
                        TEXT_SECONDARY,
                    );
                }
                
                // Handle click action after loop
                if let Some(idx) = clicked_index {
                    let cmd = suggestions[idx];
                    self.input = if cmd.needs_arg {
                        format!("{} ", cmd.command)
                    } else {
                        cmd.command.to_string()
                    };
                    self.show_autocomplete = false;
                    self.autocomplete_index = -1;
                }
            });
    }
    
    fn render_message(&self, ui: &mut egui::Ui, msg: &Message) {
        let is_user = msg.role == MessageRole::User;
        let is_system = msg.role == MessageRole::System;
        let max_w = ui.available_width() * 0.8;
        
        ui.horizontal(|ui| {
            // Alignment spacer for user messages (right-aligned)
            if is_user {
                ui.add_space(ui.available_width() - max_w);
            }
            
            // Message container
            let (bg, border_color, avatar_bg, avatar_icon) = match msg.role {
                MessageRole::User => (
                    PRIMARY.gamma_multiply(0.12),
                    PRIMARY.gamma_multiply(0.3),
                    PRIMARY,
                    "👤"
                ),
                MessageRole::Assistant => (
                    BG_WIDGET,
                    BG_WIDGET.gamma_multiply(1.2),
                    SECONDARY,
                    "🧠"
                ),
                MessageRole::System => (
                    ACCENT.gamma_multiply(0.08),
                    ACCENT.gamma_multiply(0.2),
                    ACCENT,
                    "ℹ️"
                ),
            };
            
            // Avatar (not for user - shown on left)
            if !is_user {
                egui::Frame::none()
                    .fill(avatar_bg.gamma_multiply(0.2))
                    .rounding(Rounding::same(16.0))
                    .inner_margin(egui::Margin::same(6.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new(avatar_icon).size(14.0));
                    });
                ui.add_space(8.0);
            }
            
            // Message bubble
            egui::Frame::none()
                .fill(bg)
                .stroke(Stroke::new(1.0, border_color))
                .rounding(Rounding {
                    nw: if is_user { 16.0 } else { 4.0 },
                    ne: 16.0,
                    sw: 16.0,
                    se: if is_user { 4.0 } else { 16.0 },
                })
                .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                .show(ui, |ui| {
                    ui.set_max_width(max_w - 60.0);
                    
                    // Header with name and time
                    if !is_system {
                        ui.horizontal(|ui| {
                            let (role_name, role_color) = match msg.role {
                                MessageRole::User => ("You", PRIMARY),
                                MessageRole::Assistant => ("Sutra", SECONDARY),
                                MessageRole::System => ("System", ACCENT),
                            };
                            ui.label(RichText::new(role_name).size(11.0).color(role_color).strong());
                            ui.add_space(8.0);
                            ui.label(RichText::new(msg.timestamp.format("%H:%M").to_string()).size(10.0).color(TEXT_MUTED));
                        });
                        ui.add_space(4.0);
                    }
                    
                    // Message content
                    ui.label(RichText::new(&msg.content).size(13.5).color(TEXT_PRIMARY));
                });
            
            // Avatar for user (shown on right)
            if is_user {
                ui.add_space(8.0);
                egui::Frame::none()
                    .fill(avatar_bg.gamma_multiply(0.2))
                    .rounding(Rounding::same(16.0))
                    .inner_margin(egui::Margin::same(6.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new(avatar_icon).size(14.0));
                    });
            }
        });
    }
    
    pub fn add_response(&mut self, content: String) {
        self.messages.push(Message {
            role: MessageRole::Assistant,
            content,
            timestamp: Local::now(),
        });
        self.is_processing = false;
    }
    
    /// Parse slash commands and legacy formats
    fn parse_command(&mut self, input: &str) -> Option<ChatAction> {
        let input_lower = input.to_lowercase();
        let input_trimmed = input.trim();
        
        // Slash commands (modern)
        if input_trimmed.starts_with('/') {
            let parts: Vec<&str> = input_trimmed.splitn(2, char::is_whitespace).collect();
            let cmd = parts[0].to_lowercase();
            let arg = parts.get(1).map(|s| s.trim()).unwrap_or("");
            
            match cmd.as_str() {
                "/learn" | "/l" => {
                    if arg.is_empty() {
                        self.add_response("❌ Usage: `/learn <knowledge>`\n\nExample: `/learn Paris is the capital of France`".into());
                        return None;
                    }
                    return Some(ChatAction::Learn(arg.to_string()));
                }
                "/search" | "/s" | "/find" => {
                    if arg.is_empty() {
                        self.add_response("❌ Usage: `/search <query>`\n\nExample: `/search capital of France`".into());
                        return None;
                    }
                    return Some(ChatAction::Query(arg.to_string()));
                }
                "/help" | "/h" | "/?" => {
                    self.show_help();
                    return None;
                }
                "/clear" | "/c" => {
                    return Some(ChatAction::Clear);
                }
                "/stats" | "/status" => {
                    return Some(ChatAction::Stats);
                }
                _ => {
                    self.add_response(format!(
                        "❓ Unknown command: `{}`\n\nType `/help` to see available commands.",
                        cmd
                    ));
                    return None;
                }
            }
        }
        
        // Legacy format: "learn: <content>"
        if input_lower.starts_with("learn:") {
            let content = input_trimmed.splitn(2, ':').nth(1).unwrap_or("").trim();
            if content.is_empty() {
                self.add_response("❌ Please provide something to learn.\n\nExample: `learn: Paris is the capital of France`".into());
                return None;
            }
            return Some(ChatAction::Learn(content.to_string()));
        }
        
        // Default: treat as query
        Some(ChatAction::Query(input_trimmed.to_string()))
    }
    
    /// Show help message with all commands
    fn show_help(&mut self) {
        self.messages.push(Message {
            role: MessageRole::Assistant,
            content: "📚 **Available Commands**\n\n\
                **Learning:**\n\
                • `/learn <text>` - Teach me new knowledge\n\
                • `/l <text>` - Shortcut for /learn\n\n\
                **Searching:**\n\
                • `/search <query>` - Search my knowledge\n\
                • `/s <query>` - Shortcut for /search\n\
                • Or just type a question!\n\n\
                **Utility:**\n\
                • `/help` - Show this help\n\
                • `/clear` - Clear chat history\n\
                • `/stats` - Show knowledge stats\n\n\
                **Legacy:**\n\
                • `learn: <text>` - Also works for learning".into(),
            timestamp: Local::now(),
        });
    }
}

#[derive(Debug, Clone)]
pub enum ChatAction {
    Query(String),
    Learn(String),
    Help,
    Clear,
    Stats,
}

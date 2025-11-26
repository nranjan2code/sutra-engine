//! Chat panel - Modern conversational interface

use eframe::egui::{self, Color32, RichText, Rounding, ScrollArea, Stroke, TextEdit, Vec2};
use chrono::{DateTime, Local};
use crate::theme::{PRIMARY, SECONDARY, ACCENT, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_MUTED, BG_WIDGET, BG_DARK, SUCCESS};

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
}

impl Default for ChatPanel {
    fn default() -> Self {
        Self {
            messages: vec![Message {
                role: MessageRole::System,
                content: "👋 Welcome! I'm Sutra, your personal knowledge assistant.\n\n• Type `learn: <text>` to teach me something\n• Ask questions to retrieve knowledge\n• Browse the Knowledge tab to see what I know".into(),
                timestamp: Local::now(),
            }],
            input: String::new(),
            is_processing: false,
        }
    }
}

impl ChatPanel {
    pub fn ui(&mut self, ui: &mut egui::Ui) -> Option<ChatAction> {
        let mut action = None;
        
        ui.vertical(|ui| {
            // Header with gradient accent
            self.render_header(ui);
            
            ui.add_space(12.0);
            
            // Messages area with custom styling
            let available_height = ui.available_height() - 80.0; // Reserve space for input
            
            egui::Frame::none()
                .fill(Color32::TRANSPARENT)
                .show(ui, |ui| {
                    ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .stick_to_bottom(true)
                        .max_height(available_height)
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
            
            ui.add_space(8.0);
            
            // Input area with modern styling
            action = self.render_input(ui);
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
    
    fn render_input(&mut self, ui: &mut egui::Ui) -> Option<ChatAction> {
        let mut action = None;
        
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
                        .hint_text("Ask a question or type 'learn: your knowledge'...")
                        .frame(false)
                        .font(egui::FontId::proportional(14.0));
                    
                    let resp = ui.add_sized(Vec2::new(input_width, 36.0), text_edit);
                    
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
                    if btn_resp.clicked() || (resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && can_send) {
                        let content = self.input.trim().to_string();
                        self.input.clear();
                        
                        self.messages.push(Message {
                            role: MessageRole::User,
                            content: content.clone(),
                            timestamp: Local::now(),
                        });
                        
                        action = Some(if content.to_lowercase().starts_with("learn:") {
                            ChatAction::Learn(content.splitn(2, ':').nth(1).unwrap_or("").trim().to_string())
                        } else {
                            ChatAction::Query(content)
                        });
                    }
                });
            });
        
        action
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
}

#[derive(Debug, Clone)]
pub enum ChatAction {
    Query(String),
    Learn(String),
}

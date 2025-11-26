//! Quick Learn panel - Fast knowledge entry interface

use eframe::egui::{self, Color32, RichText, Rounding, ScrollArea, Stroke, Vec2, TextEdit, Key};
use chrono::Local;
use crate::theme::{PRIMARY, SECONDARY, ACCENT, SUCCESS, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_MUTED, BG_PANEL, BG_WIDGET, BG_ELEVATED};

pub struct QuickLearnPanel {
    pub input: String,
    pub batch_input: String,
    pub recent_learns: Vec<LearnEntry>,
    pub is_processing: bool,
    pub mode: LearnMode,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LearnMode {
    Single,
    Batch,
}

#[derive(Debug, Clone)]
pub struct LearnEntry {
    pub content: String,
    pub timestamp: String,
    pub status: LearnStatus,
}

#[derive(Debug, Clone)]
pub enum LearnStatus {
    Success,
    Processing,
    Failed(String),
}

impl Default for QuickLearnPanel {
    fn default() -> Self {
        Self {
            input: String::new(),
            batch_input: String::new(),
            recent_learns: Vec::new(),
            is_processing: false,
            mode: LearnMode::Single,
            suggestions: vec![
                "JavaScript is a programming language".to_string(),
                "The human brain has about 86 billion neurons".to_string(),
                "Photosynthesis converts sunlight into energy".to_string(),
                "TCP stands for Transmission Control Protocol".to_string(),
                "The speed of light is 299,792,458 meters per second".to_string(),
                "DNA stands for Deoxyribonucleic acid".to_string(),
            ],
        }
    }
}

impl QuickLearnPanel {
    pub fn ui(&mut self, ui: &mut egui::Ui) -> Option<QuickLearnAction> {
        let mut action = None;
        
        ui.vertical(|ui| {
            // Header
            self.render_header(ui);
            ui.add_space(16.0);
            
            // Mode switcher
            self.render_mode_switcher(ui);
            ui.add_space(16.0);
            
            // Main input area
            match self.mode {
                LearnMode::Single => {
                    action = self.render_single_mode(ui);
                }
                LearnMode::Batch => {
                    action = self.render_batch_mode(ui);
                }
            }
            
            ui.add_space(20.0);
            
            // Recent learns section
            self.render_recent_learns(ui);
        });
        
        action
    }
    
    fn render_header(&self, ui: &mut egui::Ui) {
        egui::Frame::none()
            .fill(BG_ELEVATED)
            .rounding(Rounding::same(12.0))
            .inner_margin(egui::Margin::symmetric(16.0, 10.0))
            .stroke(Stroke::new(1.0, Color32::from_rgb(60, 60, 90)))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Icon with background
                    egui::Frame::none()
                        .fill(ACCENT.gamma_multiply(0.15))
                        .rounding(Rounding::same(8.0))
                        .inner_margin(egui::Margin::same(6.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new("⚡").size(24.0));
                        });
                    
                    ui.add_space(8.0);
                    
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Quick Learn").size(28.0).color(TEXT_PRIMARY).strong());
                        ui.label(RichText::new("Fast knowledge entry").size(15.0).color(TEXT_MUTED));
                    });
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if self.is_processing {
                            ui.spinner();
                        } else {
                            ui.label(RichText::new("Ready").size(12.0).color(SUCCESS));
                        }
                    });
                });
            });
    }
    
    fn render_mode_switcher(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Mode:").size(16.0).color(TEXT_SECONDARY));
            ui.add_space(12.0);
            
            // Single mode button
            let single_selected = self.mode == LearnMode::Single;
            let single_color = if single_selected { PRIMARY } else { BG_WIDGET };
            let single_text_color = if single_selected { Color32::WHITE } else { TEXT_SECONDARY };
            
            let single_btn = egui::Button::new(
                RichText::new("📝 Single").size(15.0).color(single_text_color)
            )
            .fill(single_color)
            .rounding(Rounding::same(8.0))
            .min_size(Vec2::new(100.0, 40.0));
            
            if ui.add(single_btn).clicked() {
                self.mode = LearnMode::Single;
            }
            
            ui.add_space(4.0);
            
            // Batch mode button
            let batch_selected = self.mode == LearnMode::Batch;
            let batch_color = if batch_selected { PRIMARY } else { BG_WIDGET };
            let batch_text_color = if batch_selected { Color32::WHITE } else { TEXT_SECONDARY };
            
            let batch_btn = egui::Button::new(
                RichText::new("📄 Batch").size(15.0).color(batch_text_color)
            )
            .fill(batch_color)
            .rounding(Rounding::same(8.0))
            .min_size(Vec2::new(100.0, 40.0));
            
            if ui.add(batch_btn).clicked() {
                self.mode = LearnMode::Batch;
            }
        });
    }
    
    fn render_single_mode(&mut self, ui: &mut egui::Ui) -> Option<QuickLearnAction> {
        let mut action = None;
        
        // Input area
        egui::Frame::none()
            .fill(BG_PANEL)
            .rounding(Rounding::same(10.0))
            .inner_margin(16.0)
            .stroke(Stroke::new(1.0, BG_WIDGET))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Teach me something:").size(18.0).color(TEXT_SECONDARY));
                    ui.add_space(12.0);
                    
                    // Large text input
                    let text_edit = TextEdit::multiline(&mut self.input)
                        .hint_text("Enter any fact, rule, or piece of knowledge...")
                        .desired_rows(4)
                        .frame(false)
                        .font(egui::FontId::proportional(16.0));
                    
                    let resp = ui.add_sized(
                        Vec2::new(ui.available_width(), 120.0),
                        text_edit
                    );
                    
                    // Handle keyboard shortcuts
                    if resp.has_focus() {
                        ui.input(|i| {
                            // Ctrl+Enter to learn
                            if i.key_pressed(Key::Enter) && i.modifiers.ctrl {
                                if !self.input.trim().is_empty() && !self.is_processing {
                                    action = Some(QuickLearnAction::Learn(self.input.trim().to_string()));
                                }
                            }
                        });
                    }
                    
                    ui.add_space(12.0);
                    
                    // Action buttons
                    ui.horizontal(|ui| {
                        let can_learn = !self.input.trim().is_empty() && !self.is_processing;
                        
                        let learn_btn = egui::Button::new(
                            RichText::new(if self.is_processing { "Learning..." } else { "🧠 Learn" })
                                .size(18.0)
                                .color(if can_learn { Color32::WHITE } else { TEXT_MUTED })
                        )
                        .fill(if can_learn { SUCCESS } else { BG_WIDGET })
                        .rounding(Rounding::same(10.0))
                        .min_size(Vec2::new(140.0, 50.0));
                        
                        if ui.add_enabled(can_learn, learn_btn).clicked() {
                            action = Some(QuickLearnAction::Learn(self.input.trim().to_string()));
                        }
                        
                        ui.add_space(8.0);
                        
                        // Clear button
                        let clear_btn = egui::Button::new(
                            RichText::new("Clear").size(16.0).color(TEXT_SECONDARY)
                        )
                        .fill(Color32::TRANSPARENT)
                        .stroke(Stroke::new(1.0, BG_WIDGET))
                        .rounding(Rounding::same(10.0))
                        .min_size(Vec2::new(80.0, 50.0));
                        
                        if ui.add(clear_btn).clicked() {
                            self.input.clear();
                        }
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new("Tip: Ctrl+Enter to learn quickly").size(14.0).color(TEXT_MUTED));
                        });
                    });
                });
            });
        
        ui.add_space(16.0);
        
        // Quick suggestions
        self.render_suggestions(ui, &mut action);
        
        action
    }
    
    fn render_batch_mode(&mut self, ui: &mut egui::Ui) -> Option<QuickLearnAction> {
        let mut action = None;
        
        egui::Frame::none()
            .fill(BG_PANEL)
            .rounding(Rounding::same(10.0))
            .inner_margin(16.0)
            .stroke(Stroke::new(1.0, BG_WIDGET))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Batch learn (one fact per line):").size(18.0).color(TEXT_SECONDARY));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new("💡 Tip: Paste lists or facts").size(14.0).color(TEXT_MUTED));
                        });
                    });
                    ui.add_space(12.0);
                    
                    // Large text area
                    let text_edit = TextEdit::multiline(&mut self.batch_input)
                        .hint_text("Enter multiple facts, one per line:\n\nPython is a programming language\nThe Earth orbits the Sun\nWater boils at 100 degrees Celsius")
                        .desired_rows(10)
                        .frame(false)
                        .font(egui::FontId::proportional(16.0));
                    
                    ui.add_sized(
                        Vec2::new(ui.available_width(), 240.0),
                        text_edit
                    );
                    
                    ui.add_space(12.0);
                    
                    // Stats and action
                    ui.horizontal(|ui| {
                        let line_count = self.batch_input.lines().filter(|l| !l.trim().is_empty()).count();
                        
                        ui.label(RichText::new(format!("{} facts ready", line_count)).size(16.0).color(TEXT_MUTED));
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let can_learn = line_count > 0 && !self.is_processing;
                            
                            let batch_btn = egui::Button::new(
                                RichText::new(if self.is_processing { "Processing..." } else { "🚀 Learn All" })
                                    .size(18.0)
                                    .color(if can_learn { Color32::WHITE } else { TEXT_MUTED })
                            )
                            .fill(if can_learn { PRIMARY } else { BG_WIDGET })
                            .rounding(Rounding::same(10.0))
                            .min_size(Vec2::new(160.0, 50.0));
                            
                            if ui.add_enabled(can_learn, batch_btn).clicked() {
                                action = Some(QuickLearnAction::BatchLearn(self.batch_input.clone()));
                            }
                            
                            ui.add_space(8.0);
                            
                            let clear_btn = egui::Button::new(
                                RichText::new("Clear").size(16.0).color(TEXT_SECONDARY)
                            )
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::new(1.0, BG_WIDGET))
                            .rounding(Rounding::same(10.0))
                            .min_size(Vec2::new(80.0, 50.0));
                            
                            if ui.add(clear_btn).clicked() {
                                self.batch_input.clear();
                            }
                        });
                    });
                });
            });
        
        action
    }
    
    fn render_suggestions(&self, ui: &mut egui::Ui, action: &mut Option<QuickLearnAction>) {
        ui.label(RichText::new("💡 Quick suggestions:").size(16.0).color(TEXT_SECONDARY));
        ui.add_space(10.0);
        
        egui::Frame::none()
            .fill(BG_WIDGET.gamma_multiply(0.5))
            .rounding(Rounding::same(10.0))
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.columns(2, |columns| {
                    for (i, suggestion) in self.suggestions.iter().enumerate() {
                        let col = i % 2;
                        
                        let btn = egui::Button::new(
                            RichText::new(suggestion).size(14.0).color(TEXT_SECONDARY)
                        )
                        .fill(Color32::TRANSPARENT)
                        .stroke(Stroke::new(1.0, BG_WIDGET))
                        .rounding(Rounding::same(8.0))
                        .min_size(Vec2::new(0.0, 40.0))
                        .wrap();
                        
                        if columns[col].add(btn).clicked() {
                            *action = Some(QuickLearnAction::Learn(suggestion.clone()));
                        }
                        
                        if i < self.suggestions.len() - 1 {
                            columns[col].add_space(4.0);
                        }
                    }
                });
            });
    }
    
    fn render_recent_learns(&self, ui: &mut egui::Ui) {
        if self.recent_learns.is_empty() {
            return;
        }
        
        ui.label(RichText::new("📝 Recent learns:").size(18.0).color(TEXT_SECONDARY));
        ui.add_space(12.0);
        
        ScrollArea::vertical()
            .max_height(200.0)
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                for entry in &self.recent_learns {
                    self.render_learn_entry(ui, entry);
                    ui.add_space(6.0);
                }
            });
    }
    
    fn render_learn_entry(&self, ui: &mut egui::Ui, entry: &LearnEntry) {
        let (icon, color) = match &entry.status {
            LearnStatus::Success => ("✅", SUCCESS),
            LearnStatus::Processing => ("⏳", ACCENT),
            LearnStatus::Failed(_) => ("❌", Color32::from_rgb(248, 113, 113)),
        };
        
        egui::Frame::none()
            .fill(color.gamma_multiply(0.08))
            .stroke(Stroke::new(1.0, color.gamma_multiply(0.2)))
            .rounding(Rounding::same(10.0))
            .inner_margin(14.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(icon).size(16.0));
                    ui.add_space(10.0);
                    
                    ui.vertical(|ui| {
                        let preview = if entry.content.len() > 80 {
                            format!("{}...", &entry.content[..80])
                        } else {
                            entry.content.clone()
                        };
                        
                        ui.label(RichText::new(preview).size(15.0).color(TEXT_PRIMARY));
                        
                        if let LearnStatus::Failed(error) = &entry.status {
                            ui.label(RichText::new(error).size(13.0).color(color));
                        }
                        
                        ui.label(RichText::new(&entry.timestamp).size(12.0).color(TEXT_MUTED));
                    });
                });
            });
    }
    
    pub fn add_learn_entry(&mut self, content: String, status: LearnStatus) {
        let timestamp = Local::now().format("%H:%M:%S").to_string();
        self.recent_learns.insert(0, LearnEntry {
            content,
            timestamp,
            status,
        });
        
        // Keep only last 10 entries
        self.recent_learns.truncate(10);
    }
    
    pub fn clear_input(&mut self) {
        match self.mode {
            LearnMode::Single => self.input.clear(),
            LearnMode::Batch => self.batch_input.clear(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum QuickLearnAction {
    Learn(String),
    BatchLearn(String),    Delete(String),}
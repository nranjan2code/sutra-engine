//! Settings panel - Premium design

use eframe::egui::{self, RichText, Rounding, ScrollArea, Stroke, TextEdit, Vec2};
use crate::theme::{PRIMARY, SECONDARY, ACCENT, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_MUTED, SUCCESS, WARNING, BG_WIDGET, BG_ELEVATED, elevated_card, ThemeMode, apply_theme};

#[derive(Clone)]
pub struct StorageStatsUI {
    pub total_concepts: usize,
    pub vector_dimensions: usize,
    pub data_path: String,
    pub status: StorageStatus,
}

#[derive(Clone, PartialEq)]
pub enum StorageStatus {
    Running,
    Stopped,
    Error(String),
}

impl Default for StorageStatsUI {
    fn default() -> Self {
        Self {
            total_concepts: 0,
            vector_dimensions: 768,
            data_path: "~/.sutra/desktop".into(),
            status: StorageStatus::Running,
        }
    }
}

pub struct SettingsPanel {
    // General
    pub data_path: String,
    pub vector_dimensions: String,
    
    // Appearance
    pub theme_mode: ThemeMode,
    pub font_size: f32,
    
    // Storage stats
    pub stats: StorageStatsUI,
    
    // State
    dirty: bool,
    
    // Undo/Redo
    pub show_undo_history: bool,
}

impl Default for SettingsPanel {
    fn default() -> Self {
        Self {
            data_path: "~/.sutra/desktop".into(),
            vector_dimensions: "768".into(),
            theme_mode: ThemeMode::Dark,
            font_size: 14.0,
            stats: StorageStatsUI::default(),
            dirty: false,
            show_undo_history: false,
        }
    }
}

impl SettingsPanel {
    pub fn ui(&mut self, ui: &mut egui::Ui) -> Option<SettingsAction> {
        let mut action = None;
        
        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.set_width(ui.available_width().min(700.0));
                
                // Header
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Settings").size(28.0).color(TEXT_PRIMARY).strong());
                });
                ui.add_space(24.0);
                
                // Status section
                self.status_section(ui);
                ui.add_space(20.0);
                
                // Storage section
                self.storage_section(ui, &mut action);
                ui.add_space(20.0);
                
                // Appearance section
                if let Some(a) = self.appearance_section(ui) {
                    action = Some(a);
                }
                ui.add_space(20.0);
                
                // Actions section
                self.actions_section(ui, &mut action);
                ui.add_space(20.0);
                
                // About section
                self.about_section(ui, &mut action);
                ui.add_space(40.0);
            });
        
        action
    }
    
    fn status_section(&mut self, ui: &mut egui::Ui) {
        elevated_card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            
            // Section header with icon
            ui.horizontal(|ui| {
                ui.label(RichText::new("📊").size(16.0));
                ui.add_space(4.0);
                ui.label(RichText::new("Status").size(15.0).color(TEXT_PRIMARY).strong());
            });
            ui.add_space(16.0);
            
            // Status indicator - prominent pill
            let (color, label) = match &self.stats.status {
                StorageStatus::Running => (SUCCESS, "Running"),
                StorageStatus::Stopped => (WARNING, "Stopped"),
                StorageStatus::Error(e) => (WARNING, e.as_str()),
            };
            
            ui.horizontal(|ui| {
                egui::Frame::none()
                    .fill(color.gamma_multiply(0.15))
                    .rounding(Rounding::same(12.0))
                    .inner_margin(egui::Margin::symmetric(12.0, 6.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Dot indicator
                            let (dot_rect, _) = ui.allocate_exact_size(Vec2::splat(8.0), egui::Sense::hover());
                            ui.painter().circle_filled(dot_rect.center(), 4.0, color);
                            ui.add_space(6.0);
                            ui.label(RichText::new(label).size(13.0).color(color));
                        });
                    });
            });
            
            ui.add_space(16.0);
            
            // Stats in nice cards
            ui.horizontal(|ui| {
                self.stat_box(ui, "Concepts", &self.stats.total_concepts.to_string(), ACCENT);
                ui.add_space(12.0);
                self.stat_box(ui, "Dimensions", &self.stats.vector_dimensions.to_string(), SECONDARY);
            });
        });
    }
    
    fn storage_section(&mut self, ui: &mut egui::Ui, action: &mut Option<SettingsAction>) {
        elevated_card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            
            ui.horizontal(|ui| {
                ui.label(RichText::new("💾").size(16.0));
                ui.add_space(4.0);
                ui.label(RichText::new("Storage").size(15.0).color(TEXT_PRIMARY).strong());
            });
            ui.add_space(16.0);
            
            // Data path row
            ui.horizontal(|ui| {
                ui.label(RichText::new("Data Path").size(13.0).color(TEXT_SECONDARY));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let resp = ui.add(
                        TextEdit::singleline(&mut self.data_path)
                            .desired_width(280.0)
                            .hint_text("Storage location")
                    );
                    if resp.changed() { self.dirty = true; }
                });
            });
            
            ui.add_space(12.0);
            
            // Vector dimensions row
            ui.horizontal(|ui| {
                ui.label(RichText::new("Vector Dimensions").size(13.0).color(TEXT_SECONDARY));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let resp = ui.add(
                        TextEdit::singleline(&mut self.vector_dimensions)
                            .desired_width(100.0)
                    );
                    if resp.changed() { self.dirty = true; }
                });
            });
            
            ui.add_space(12.0);
            
            // Warning note
            egui::Frame::none()
                .fill(WARNING.gamma_multiply(0.1))
                .rounding(Rounding::same(8.0))
                .inner_margin(egui::Margin::symmetric(10.0, 6.0))
                .show(ui, |ui| {
                    ui.label(RichText::new("⚠️ Changing dimensions requires restart").size(11.0).color(WARNING.gamma_multiply(0.9)));
                });
            
            if self.dirty {
                ui.add_space(16.0);
                let btn = egui::Button::new(RichText::new("Save Changes").color(TEXT_PRIMARY))
                    .fill(PRIMARY.gamma_multiply(0.3))
                    .stroke(Stroke::new(1.0, PRIMARY.gamma_multiply(0.5)))
                    .rounding(Rounding::same(8.0));
                if ui.add(btn).clicked() {
                    *action = Some(SettingsAction::Save);
                    self.dirty = false;
                }
            }
        });
    }
    
    fn appearance_section(&mut self, ui: &mut egui::Ui) -> Option<SettingsAction> {
        let mut action = None;
        
        elevated_card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            
            ui.horizontal(|ui| {
                ui.label(RichText::new("🎨").size(16.0));
                ui.add_space(4.0);
                ui.label(RichText::new("Appearance").size(15.0).color(TEXT_PRIMARY).strong());
            });
            ui.add_space(16.0);
            
            // Theme selector row with visual preview
            ui.horizontal(|ui| {
                ui.label(RichText::new("Theme").size(13.0).color(TEXT_SECONDARY));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let old_theme = self.theme_mode;
                    egui::ComboBox::from_id_salt("theme_mode")
                        .width(150.0)
                        .selected_text(self.theme_mode.name())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.theme_mode, ThemeMode::Dark, "🌙 Dark");
                            ui.selectable_value(&mut self.theme_mode, ThemeMode::Light, "☀️ Light");
                            ui.selectable_value(&mut self.theme_mode, ThemeMode::HighContrast, "🔲 High Contrast");
                        });
                    
                    if self.theme_mode != old_theme {
                        action = Some(SettingsAction::ChangeTheme(self.theme_mode));
                    }
                });
            });
            
            ui.add_space(8.0);
            
            // Theme description
            egui::Frame::none()
                .fill(BG_WIDGET.gamma_multiply(0.5))
                .rounding(Rounding::same(8.0))
                .inner_margin(egui::Margin::symmetric(10.0, 6.0))
                .show(ui, |ui| {
                    ui.label(RichText::new(self.theme_mode.description()).size(11.0).color(TEXT_MUTED));
                });
            
            ui.add_space(12.0);
            
            // Font size slider row
            ui.horizontal(|ui| {
                ui.label(RichText::new("Font Size").size(13.0).color(TEXT_SECONDARY));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add(egui::Slider::new(&mut self.font_size, 10.0..=24.0)
                        .suffix("px")
                        .fixed_decimals(1));
                });
            });
        });
        
        action
    }
    
    fn actions_section(&mut self, ui: &mut egui::Ui, action: &mut Option<SettingsAction>) {
        elevated_card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            
            ui.horizontal(|ui| {
                ui.label(RichText::new("⚙️").size(16.0));
                ui.add_space(4.0);
                ui.label(RichText::new("Actions").size(15.0).color(TEXT_PRIMARY).strong());
            });
            ui.add_space(16.0);
            
            ui.horizontal(|ui| {
                // Export button
                let export_btn = egui::Button::new(RichText::new("Export Data").size(13.0))
                    .rounding(Rounding::same(8.0));
                if ui.add(export_btn).clicked() {
                    *action = Some(SettingsAction::ExportData);
                }
                
                ui.add_space(8.0);
                
                // Import button
                let import_btn = egui::Button::new(RichText::new("Import Data").size(13.0))
                    .rounding(Rounding::same(8.0));
                if ui.add(import_btn).clicked() {
                    *action = Some(SettingsAction::ImportData);
                }
                
                ui.add_space(8.0);
                
                // Clear button - danger style
                let clear_btn = egui::Button::new(RichText::new("Clear All Data").size(13.0).color(WARNING))
                    .fill(WARNING.gamma_multiply(0.1))
                    .stroke(Stroke::new(1.0, WARNING.gamma_multiply(0.3)))
                    .rounding(Rounding::same(8.0));
                if ui.add(clear_btn).clicked() {
                    *action = Some(SettingsAction::ClearData);
                }
            });
        });
    }
    
    fn about_section(&self, ui: &mut egui::Ui, action: &mut Option<SettingsAction>) {
        elevated_card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            
            ui.horizontal(|ui| {
                ui.label(RichText::new("ℹ️").size(16.0));
                ui.add_space(4.0);
                ui.label(RichText::new("About").size(15.0).color(TEXT_PRIMARY).strong());
            });
            ui.add_space(16.0);
            
            // App name with version
            ui.label(RichText::new("Sutra Desktop").size(18.0).color(TEXT_PRIMARY).strong());
            ui.add_space(4.0);
            
            egui::Frame::none()
                .fill(PRIMARY.gamma_multiply(0.15))
                .rounding(Rounding::same(6.0))
                .inner_margin(egui::Margin::symmetric(8.0, 3.0))
                .show(ui, |ui| {
                    ui.label(RichText::new("Version 3.3.0").size(11.0).color(PRIMARY));
                });
            
            ui.add_space(12.0);
            
            ui.label(RichText::new("A self-contained semantic reasoning engine with temporal, causal, and explainable AI.").size(13.0).color(TEXT_SECONDARY));
            
            ui.add_space(16.0);
            
            // Links
            ui.horizontal(|ui| {
                ui.hyperlink_to(RichText::new("Documentation").size(13.0), "https://github.com/sutraworks/sutra");
                ui.label(RichText::new("•").color(TEXT_MUTED));
                ui.hyperlink_to(RichText::new("License").size(13.0), "https://github.com/sutraworks/sutra/blob/main/LICENSE");
            });
            
            ui.add_space(16.0);
            
            // Tour button
            let tour_btn = egui::Button::new(RichText::new("🎓 Start Interactive Tour").size(13.0))
                .rounding(Rounding::same(8.0));
            if ui.add(tour_btn).clicked() {
                *action = Some(SettingsAction::StartTour);
            }
        });
    }
    
    fn stat_box(&self, ui: &mut egui::Ui, label: &str, value: &str, color: egui::Color32) {
        egui::Frame::none()
            .fill(color.gamma_multiply(0.1))
            .stroke(Stroke::new(1.0, color.gamma_multiply(0.25)))
            .inner_margin(egui::Margin::symmetric(16.0, 12.0))
            .rounding(Rounding::same(10.0))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new(label).size(11.0).color(TEXT_MUTED));
                    ui.add_space(2.0);
                    ui.label(RichText::new(value).size(24.0).color(color).strong());
                });
            });
    }
    
    pub fn update_stats(&mut self, stats: StorageStatsUI) {
        self.stats = stats;
    }
}

#[derive(Debug, Clone)]
pub enum SettingsAction {
    Save,
    ExportData,
    ImportData,
    ClearData,
    ChangeTheme(ThemeMode),
    StartTour,
}

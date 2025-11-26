//! Status bar component - Modern, informative footer

use eframe::egui::{self, Color32, RichText, Rounding, Vec2};
use crate::theme::{TEXT_MUTED, TEXT_SECONDARY, SUCCESS, WARNING, BG_DARK, BG_WIDGET, PRIMARY};

#[derive(Clone, PartialEq)]
pub enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
    Error(String),
}

pub struct StatusBar {
    pub status: ConnectionStatus,
    pub concept_count: usize,
    pub last_activity: String,
    pub version: String,
}

impl Default for StatusBar {
    fn default() -> Self {
        Self {
            status: ConnectionStatus::Connected,
            concept_count: 0,
            last_activity: "Ready".into(),
            version: "v3.3.0".into(),
        }
    }
}

impl StatusBar {
    pub fn ui(&self, ui: &mut egui::Ui) {
        egui::Frame::none()
            .fill(BG_DARK)
            .inner_margin(egui::Margin::symmetric(16.0, 6.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Left: status indicator with animated dot
                    self.status_badge(ui);
                    
                    ui.add_space(12.0);
                    self.separator(ui);
                    ui.add_space(12.0);
                    
                    // Concept count with icon
                    self.stat_pill(ui, "🧠", &format!("{}", self.concept_count), "concepts");
                    
                    ui.add_space(12.0);
                    self.separator(ui);
                    ui.add_space(12.0);
                    
                    // Last activity
                    ui.label(RichText::new(&self.last_activity).size(11.0).color(TEXT_MUTED));
                    
                    // Right side
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Version badge
                        egui::Frame::none()
                            .fill(BG_WIDGET)
                            .rounding(Rounding::same(4.0))
                            .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                            .show(ui, |ui| {
                                ui.label(RichText::new(&self.version).size(10.0).color(TEXT_MUTED).monospace());
                            });
                        
                        ui.add_space(8.0);
                        
                        // Storage type indicator
                        ui.label(RichText::new("Local Storage").size(10.0).color(TEXT_MUTED));
                        ui.label(RichText::new("💾").size(10.0));
                    });
                });
            });
    }
    
    fn status_badge(&self, ui: &mut egui::Ui) {
        let (icon_color, text, pulse) = match &self.status {
            ConnectionStatus::Connected => (SUCCESS, "Active", false),
            ConnectionStatus::Connecting => (WARNING, "Connecting...", true),
            ConnectionStatus::Disconnected => (TEXT_MUTED, "Offline", false),
            ConnectionStatus::Error(e) => (WARNING, e.as_str(), false),
        };
        
        egui::Frame::none()
            .fill(icon_color.gamma_multiply(0.15))
            .rounding(Rounding::same(10.0))
            .inner_margin(egui::Margin::symmetric(8.0, 3.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Animated status dot
                    let (rect, _) = ui.allocate_exact_size(Vec2::new(8.0, 8.0), egui::Sense::hover());
                    let center = rect.center();
                    
                    // Glow effect
                    if pulse {
                        ui.painter().circle_filled(center, 5.0, icon_color.gamma_multiply(0.3));
                    }
                    ui.painter().circle_filled(center, 3.0, icon_color);
                    
                    ui.add_space(4.0);
                    ui.label(RichText::new(text).size(10.0).color(icon_color));
                });
            });
    }
    
    fn stat_pill(&self, ui: &mut egui::Ui, icon: &str, value: &str, _label: &str) {
        ui.horizontal(|ui| {
            ui.label(RichText::new(icon).size(11.0));
            ui.label(RichText::new(value).size(11.0).color(TEXT_SECONDARY).strong());
        });
    }
    
    fn separator(&self, ui: &mut egui::Ui) {
        let (rect, _) = ui.allocate_exact_size(Vec2::new(1.0, 14.0), egui::Sense::hover());
        ui.painter().rect_filled(rect, 0.0, BG_WIDGET);
    }
    
    pub fn set_status(&mut self, status: ConnectionStatus) {
        self.status = status;
    }
    
    pub fn set_concept_count(&mut self, count: usize) {
        self.concept_count = count;
    }
    
    pub fn set_activity(&mut self, activity: impl Into<String>) {
        self.last_activity = activity.into();
    }
}

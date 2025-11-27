//! Temporal View Panel
//!
//! Timeline-based visualization of temporal events and relationships.
//! Shows before/after/during relationships across knowledge base.

use eframe::egui::{self, Color32, RichText, Rounding, ScrollArea, Stroke, Vec2, Pos2, Rect};
use crate::theme::*;
use crate::types::{TimeRange, TimelineEvent, TemporalRelation};
use crate::types::ConceptInfo;
use std::collections::HashMap;

/// Temporal View Panel
pub struct TemporalView {
    // Timeline settings
    pub time_range: TimeRange,
    pub zoom_level: f32,
    pub pan_offset: f32,
    
    // Events data
    pub events: Vec<TimelineEvent>,
    pub selected_event: Option<usize>,
    pub hovered_event: Option<usize>,
    
    // Relations
    pub temporal_relations: Vec<(usize, usize, TemporalRelation)>,
    pub show_relations: bool,
    
    // Filter
    pub filter_text: String,
    pub show_only_with_relations: bool,
    
    // View mode
    pub view_mode: TimelineViewMode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimelineViewMode {
    Timeline,
    List,
    Matrix,
}

impl Default for TemporalView {
    fn default() -> Self {
        Self {
            time_range: TimeRange::AllTime,
            zoom_level: 1.0,
            pan_offset: 0.0,
            events: Vec::new(),
            selected_event: None,
            hovered_event: None,
            temporal_relations: Vec::new(),
            show_relations: true,
            filter_text: String::new(),
            show_only_with_relations: false,
            view_mode: TimelineViewMode::Timeline,
        }
    }
}

impl TemporalView {
    /// Render the temporal view
    pub fn ui(&mut self, ui: &mut egui::Ui) -> Option<TemporalViewAction> {
        let mut action = None;
        
        ui.vertical(|ui| {
            // Header
            self.render_header(ui);
            
            ui.add_space(16.0);
            
            // Controls
            self.render_controls(ui);
            
            ui.add_space(16.0);
            
            // Main view
            match self.view_mode {
                TimelineViewMode::Timeline => self.render_timeline(ui, &mut action),
                TimelineViewMode::List => self.render_list_view(ui, &mut action),
                TimelineViewMode::Matrix => self.render_matrix_view(ui, &mut action),
            }
            
            ui.add_space(16.0);
            
            // Selected event details
            if let Some(idx) = self.selected_event {
                self.render_event_details(ui, idx, &mut action);
            }
        });
        
        action
    }
    
    fn render_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("⏱️").size(20.0));
            ui.add_space(4.0);
            ui.label(RichText::new("Temporal Analysis").size(22.0).color(TEXT_PRIMARY).strong());
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // View mode buttons
                self.view_mode_button(ui, "📊 Matrix", TimelineViewMode::Matrix);
                ui.add_space(8.0);
                self.view_mode_button(ui, "📋 List", TimelineViewMode::List);
                ui.add_space(8.0);
                self.view_mode_button(ui, "📅 Timeline", TimelineViewMode::Timeline);
            });
        });
    }
    
    fn view_mode_button(&mut self, ui: &mut egui::Ui, label: &str, mode: TimelineViewMode) {
        let is_selected = self.view_mode == mode;
        let bg = if is_selected { PRIMARY.gamma_multiply(0.25) } else { BG_WIDGET };
        let text_color = if is_selected { PRIMARY } else { TEXT_SECONDARY };
        
        egui::Frame::none()
            .fill(bg)
            .rounding(Rounding::same(6.0))
            .inner_margin(egui::Margin::symmetric(10.0, 4.0))
            .show(ui, |ui| {
                if ui.add(egui::Label::new(RichText::new(label).size(11.0).color(text_color))
                    .sense(egui::Sense::click())).clicked() {
                    self.view_mode = mode;
                }
            });
    }
    
    fn render_controls(&mut self, ui: &mut egui::Ui) {
        egui::Frame::none()
            .fill(BG_WIDGET.gamma_multiply(0.6))
            .rounding(Rounding::same(10.0))
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Time range selector
                    ui.label(RichText::new("Range:").size(12.0).color(TEXT_MUTED));
                    egui::ComboBox::from_id_salt("time_range")
                        .selected_text(format!("{:?}", self.time_range))
                        .width(100.0)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.time_range, TimeRange::AllTime, "All Time");
                            ui.selectable_value(&mut self.time_range, TimeRange::Today, "Today");
                            ui.selectable_value(&mut self.time_range, TimeRange::Week, "This Week");
                            ui.selectable_value(&mut self.time_range, TimeRange::Month, "This Month");
                        });
                    
                    ui.add_space(16.0);
                    
                    // Zoom control
                    ui.label(RichText::new("Zoom:").size(12.0).color(TEXT_MUTED));
                    ui.add(egui::Slider::new(&mut self.zoom_level, 0.5..=3.0).show_value(false));
                    
                    ui.add_space(16.0);
                    
                    // Filter
                    ui.label(RichText::new("🔍").size(12.0));
                    ui.add_sized(
                        Vec2::new(150.0, 20.0),
                        egui::TextEdit::singleline(&mut self.filter_text)
                            .hint_text("Filter events...")
                    );
                    
                    ui.add_space(16.0);
                    
                    // Show relations toggle
                    ui.checkbox(&mut self.show_relations, 
                        RichText::new("Show Relations").size(12.0).color(TEXT_SECONDARY));
                });
            });
    }
    
    fn render_timeline(&mut self, ui: &mut egui::Ui, action: &mut Option<TemporalViewAction>) {
        let available_size = ui.available_size();
        let timeline_height = (available_size.y - 200.0).max(200.0);
        
        egui::Frame::none()
            .fill(BG_WIDGET.gamma_multiply(0.4))
            .stroke(Stroke::new(1.0, BG_WIDGET))
            .rounding(Rounding::same(12.0))
            .inner_margin(16.0)
            .show(ui, |ui| {
                // Clone painter for drawing (avoids borrow conflicts with ui)
                let painter = ui.painter().clone();
                let rect = ui.available_rect_before_wrap();
                let timeline_y = rect.center().y;
                
                // Main timeline line
                painter.line_segment(
                    [Pos2::new(rect.left() + 20.0, timeline_y), 
                     Pos2::new(rect.right() - 20.0, timeline_y)],
                    Stroke::new(2.0, TEXT_MUTED.gamma_multiply(0.5))
                );
                
                // Draw time markers
                let num_markers = 5;
                for i in 0..=num_markers {
                    let x = rect.left() + 20.0 + (rect.width() - 40.0) * (i as f32 / num_markers as f32);
                    painter.line_segment(
                        [Pos2::new(x, timeline_y - 5.0), Pos2::new(x, timeline_y + 5.0)],
                        Stroke::new(1.0, TEXT_MUTED.gamma_multiply(0.5))
                    );
                }
                
                // Draw events
                let filtered_events = self.filtered_events();
                
                if filtered_events.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(50.0);
                        ui.label(RichText::new("📅").size(40.0));
                        ui.add_space(16.0);
                        ui.label(RichText::new("No Temporal Events Found").size(18.0).color(TEXT_PRIMARY).strong());
                        ui.add_space(8.0);
                        ui.label(RichText::new("Sutra automatically detects time relationships.").size(14.0).color(TEXT_MUTED));
                        ui.add_space(24.0);
                        
                        egui::Frame::none()
                            .fill(BG_WIDGET)
                            .rounding(Rounding::same(8.0))
                            .inner_margin(12.0)
                            .show(ui, |ui| {
                                ui.label(RichText::new("Try teaching:").size(12.0).color(TEXT_SECONDARY));
                                ui.add_space(4.0);
                                ui.monospace("/learn The server crashed after the update");
                                ui.add_space(4.0);
                                ui.monospace("/learn Deployment happened before testing");
                            });
                    });
                } else {
                    // First pass: collect event data for interaction
                    let event_data: Vec<_> = filtered_events.iter().enumerate().map(|(i, event)| {
                        let progress = (i as f32) / (filtered_events.len().max(1) as f32);
                        let x = rect.left() + 40.0 + (rect.width() - 80.0) * progress;
                        let y_offset = if i % 2 == 0 { -40.0 } else { 40.0 };
                        let y = timeline_y + y_offset;
                        let is_hovered = self.hovered_event == Some(i);
                        let is_selected = self.selected_event == Some(i);
                        let radius = if is_hovered || is_selected { 12.0 } else { 8.0 };
                        let color = if is_selected { PRIMARY } else if is_hovered { SECONDARY } else { ACCENT };
                        let label = if event.label.len() > 20 {
                            format!("{}...", &event.label[..20])
                        } else {
                            event.label.clone()
                        };
                        (i, x, y, y_offset, radius, color, label)
                    }).collect();
                    
                    // Second pass: draw all events
                    for (_, x, y, y_offset, radius, color, label) in &event_data {
                        painter.circle_filled(Pos2::new(*x, *y), *radius, *color);
                        
                        // Connection to timeline
                        painter.line_segment(
                            [Pos2::new(*x, *y + if *y_offset < 0.0 { *radius } else { -*radius }),
                             Pos2::new(*x, timeline_y)],
                            Stroke::new(1.0, color.gamma_multiply(0.5))
                        );
                        
                        // Label
                        let label_y = if *y_offset < 0.0 { *y - *radius - 20.0 } else { *y + *radius + 10.0 };
                        painter.text(
                            Pos2::new(*x, label_y),
                            egui::Align2::CENTER_CENTER,
                            label,
                            egui::FontId::proportional(10.0),
                            TEXT_SECONDARY
                        );
                    }
                    
                    // Third pass: handle interactions (no painter borrow conflict)
                    for (i, x, y, _, radius, _, _) in &event_data {
                        let event_rect = Rect::from_center_size(
                            Pos2::new(*x, *y),
                            Vec2::splat(*radius * 2.0)
                        );
                        
                        let response = ui.allocate_rect(event_rect, egui::Sense::click());
                        if response.hovered() {
                            self.hovered_event = Some(*i);
                        }
                        if response.clicked() {
                            self.selected_event = Some(*i);
                        }
                    }
                    
                    // Draw relations if enabled
                    if self.show_relations {
                        let num_events = filtered_events.len();
                        for (from_idx, to_idx, relation) in &self.temporal_relations {
                            if *from_idx < num_events && *to_idx < num_events {
                                let from_progress = (*from_idx as f32) / (num_events.max(1) as f32);
                                let to_progress = (*to_idx as f32) / (num_events.max(1) as f32);
                                
                                let from_x = rect.left() + 40.0 + (rect.width() - 80.0) * from_progress;
                                let to_x = rect.left() + 40.0 + (rect.width() - 80.0) * to_progress;
                                
                                let from_y_offset = if from_idx % 2 == 0 { -40.0 } else { 40.0 };
                                let to_y_offset = if to_idx % 2 == 0 { -40.0 } else { 40.0 };
                                
                                let from_y = timeline_y + from_y_offset;
                                let to_y = timeline_y + to_y_offset;
                                
                                let color = match relation {
                                    TemporalRelation::Before => SUCCESS.gamma_multiply(0.6),
                                    TemporalRelation::After => WARNING.gamma_multiply(0.6),
                                    TemporalRelation::During => SECONDARY.gamma_multiply(0.6),
                                    TemporalRelation::Concurrent => ACCENT.gamma_multiply(0.6),
                                };
                                
                                // Draw curved connection
                                let control_y = (from_y + to_y) / 2.0 + 30.0;
                                let points: Vec<Pos2> = (0..=20).map(|i| {
                                    let t = i as f32 / 20.0;
                                    let x = (1.0 - t).powi(2) * from_x + 2.0 * (1.0 - t) * t * ((from_x + to_x) / 2.0) + t.powi(2) * to_x;
                                    let y = (1.0 - t).powi(2) * from_y + 2.0 * (1.0 - t) * t * control_y + t.powi(2) * to_y;
                                    Pos2::new(x, y)
                                }).collect();
                                
                                for pair in points.windows(2) {
                                    painter.line_segment([pair[0], pair[1]], Stroke::new(1.5, color));
                                }
                            }
                        }
                    }
                }
                
                ui.allocate_space(Vec2::new(ui.available_width(), timeline_height));
            });
    }
    
    fn render_list_view(&mut self, ui: &mut egui::Ui, action: &mut Option<TemporalViewAction>) {
        let filtered = self.filtered_events();
        
        if filtered.is_empty() {
            egui::Frame::none()
                .fill(BG_WIDGET.gamma_multiply(0.5))
                .rounding(Rounding::same(8.0))
                .inner_margin(20.0)
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("📋").size(32.0));
                        ui.label(RichText::new("No temporal events found").size(13.0).color(TEXT_MUTED));
                    });
                });
            return;
        }
        
        ScrollArea::vertical()
            .max_height(350.0)
            .show(ui, |ui| {
                for (i, event) in filtered.iter().enumerate() {
                    let is_selected = self.selected_event == Some(i);
                    let bg = if is_selected { PRIMARY.gamma_multiply(0.2) } else { BG_WIDGET.gamma_multiply(0.5) };
                    let border = if is_selected { PRIMARY.gamma_multiply(0.5) } else { Color32::TRANSPARENT };
                    
                    egui::Frame::none()
                        .fill(bg)
                        .stroke(Stroke::new(1.0, border))
                        .rounding(Rounding::same(8.0))
                        .inner_margin(12.0)
                        .show(ui, |ui| {
                            let response = ui.horizontal(|ui| {
                                // Timestamp badge
                                egui::Frame::none()
                                    .fill(SECONDARY.gamma_multiply(0.2))
                                    .rounding(Rounding::same(6.0))
                                    .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                                    .show(ui, |ui| {
                                        ui.label(RichText::new(&event.timestamp).size(10.0).color(SECONDARY).monospace());
                                    });
                                
                                ui.add_space(8.0);
                                
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(&event.label).size(13.0).color(TEXT_PRIMARY));
                                    if !event.description.is_empty() {
                                        ui.label(RichText::new(&event.description).size(11.0).color(TEXT_MUTED));
                                    }
                                });
                                
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    // Relations count
                                    let relations_count = self.temporal_relations.iter()
                                        .filter(|(from, to, _)| *from == i || *to == i)
                                        .count();
                                    
                                    if relations_count > 0 {
                                        ui.label(RichText::new(format!("🔗 {}", relations_count))
                                            .size(11.0).color(ACCENT));
                                    }
                                });
                            });
                            
                            if response.response.interact(egui::Sense::click()).clicked() {
                                self.selected_event = Some(i);
                            }
                        });
                    
                    ui.add_space(6.0);
                }
            });
    }
    
    fn render_matrix_view(&mut self, ui: &mut egui::Ui, _action: &mut Option<TemporalViewAction>) {
        let filtered = self.filtered_events();
        
        if filtered.is_empty() || filtered.len() > 20 {
            egui::Frame::none()
                .fill(BG_WIDGET.gamma_multiply(0.5))
                .rounding(Rounding::same(8.0))
                .inner_margin(20.0)
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        if filtered.is_empty() {
                            ui.label(RichText::new("📊").size(32.0));
                            ui.label(RichText::new("No events for matrix view").size(13.0).color(TEXT_MUTED));
                        } else {
                            ui.label(RichText::new("⚠️").size(32.0));
                            ui.label(RichText::new("Too many events (max 20)").size(13.0).color(WARNING));
                            ui.label(RichText::new("Apply filters to reduce").size(11.0).color(TEXT_MUTED));
                        }
                    });
                });
            return;
        }
        
        // Build relation map
        let mut relation_map: HashMap<(usize, usize), TemporalRelation> = HashMap::new();
        for (from, to, rel) in &self.temporal_relations {
            relation_map.insert((*from, *to), *rel);
        }
        
        ScrollArea::both().show(ui, |ui| {
            egui::Frame::none()
                .fill(BG_WIDGET.gamma_multiply(0.4))
                .rounding(Rounding::same(8.0))
                .inner_margin(12.0)
                .show(ui, |ui| {
                    let cell_size = 40.0;
                    let label_width = 100.0;
                    
                    // Header row
                    ui.horizontal(|ui| {
                        ui.add_space(label_width);
                        for event in filtered.iter() {
                            let short_label = if event.label.len() > 5 {
                                format!("{}...", &event.label[..5])
                            } else {
                                event.label.clone()
                            };
                            ui.add_sized(
                                Vec2::new(cell_size, 20.0),
                                egui::Label::new(RichText::new(&short_label).size(9.0).color(TEXT_MUTED))
                            );
                        }
                    });
                    
                    // Matrix rows
                    for (i, event) in filtered.iter().enumerate() {
                        ui.horizontal(|ui| {
                            // Row label
                            let short_label = if event.label.len() > 12 {
                                format!("{}...", &event.label[..12])
                            } else {
                                event.label.clone()
                            };
                            ui.add_sized(
                                Vec2::new(label_width, cell_size),
                                egui::Label::new(RichText::new(&short_label).size(10.0).color(TEXT_SECONDARY))
                            );
                            
                            // Cells
                            for j in 0..filtered.len() {
                                let (symbol, color) = if i == j {
                                    ("●", TEXT_MUTED.gamma_multiply(0.3))
                                } else if let Some(rel) = relation_map.get(&(i, j)) {
                                    match rel {
                                        TemporalRelation::Before => ("→", SUCCESS),
                                        TemporalRelation::After => ("←", WARNING),
                                        TemporalRelation::During => ("⊂", SECONDARY),
                                        TemporalRelation::Concurrent => ("=", ACCENT),
                                    }
                                } else {
                                    ("·", TEXT_MUTED.gamma_multiply(0.3))
                                };
                                
                                egui::Frame::none()
                                    .fill(color.gamma_multiply(0.1))
                                    .rounding(Rounding::same(4.0))
                                    .show(ui, |ui| {
                                        ui.add_sized(
                                            Vec2::new(cell_size, cell_size),
                                            egui::Label::new(RichText::new(symbol).size(16.0).color(color))
                                        );
                                    });
                            }
                        });
                    }
                    
                    // Legend
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Legend:").size(10.0).color(TEXT_MUTED));
                        ui.label(RichText::new("→ Before").size(10.0).color(SUCCESS));
                        ui.label(RichText::new("← After").size(10.0).color(WARNING));
                        ui.label(RichText::new("⊂ During").size(10.0).color(SECONDARY));
                        ui.label(RichText::new("= Concurrent").size(10.0).color(ACCENT));
                    });
                });
        });
    }
    
    fn render_event_details(&self, ui: &mut egui::Ui, idx: usize, action: &mut Option<TemporalViewAction>) {
        let events = self.filtered_events();
        if idx >= events.len() {
            return;
        }
        
        let event = &events[idx];
        
        egui::Frame::none()
            .fill(PRIMARY.gamma_multiply(0.1))
            .stroke(Stroke::new(1.0, PRIMARY.gamma_multiply(0.3)))
            .rounding(Rounding::same(10.0))
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("📌 Selected Event").size(13.0).color(PRIMARY).strong());
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(RichText::new("View in Graph").size(11.0)).clicked() {
                            *action = Some(TemporalViewAction::ViewInGraph(event.concept_id.clone()));
                        }
                    });
                });
                
                ui.add_space(8.0);
                
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new(&event.label).size(14.0).color(TEXT_PRIMARY));
                        ui.label(RichText::new(format!("🕐 {}", event.timestamp)).size(11.0).color(TEXT_MUTED));
                        if !event.description.is_empty() {
                            ui.add_space(4.0);
                            ui.label(RichText::new(&event.description).size(12.0).color(TEXT_SECONDARY));
                        }
                    });
                    
                    ui.add_space(16.0);
                    
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Relations:").size(11.0).color(TEXT_MUTED));
                        
                        let related: Vec<_> = self.temporal_relations.iter()
                            .filter(|(from, to, _)| *from == idx || *to == idx)
                            .collect();
                        
                        if related.is_empty() {
                            ui.label(RichText::new("No temporal relations").size(10.0).color(TEXT_MUTED));
                        } else {
                            for (from, to, rel) in related.iter().take(5) {
                                let other_idx = if *from == idx { *to } else { *from };
                                let direction = if *from == idx { "→" } else { "←" };
                                let other_label = events.get(other_idx)
                                    .map(|e| e.label.clone())
                                    .unwrap_or_else(|| "?".to_string());
                                
                                ui.label(RichText::new(format!("{} {:?} {}", direction, rel, other_label))
                                    .size(10.0).color(SECONDARY));
                            }
                        }
                    });
                });
            });
    }
    
    fn filtered_events(&self) -> Vec<TimelineEvent> {
        self.events.iter()
            .filter(|e| {
                if self.filter_text.is_empty() {
                    true
                } else {
                    e.label.to_lowercase().contains(&self.filter_text.to_lowercase()) ||
                    e.description.to_lowercase().contains(&self.filter_text.to_lowercase())
                }
            })
            .cloned()
            .collect()
    }
    
    /// Add events from concepts
    pub fn load_events(&mut self, events: Vec<TimelineEvent>) {
        self.events = events;
        self.selected_event = None;
        self.hovered_event = None;
    }
    
    /// Add temporal relations
    pub fn load_relations(&mut self, relations: Vec<(usize, usize, TemporalRelation)>) {
        self.temporal_relations = relations;
    }
}

/// Actions from temporal view
#[derive(Debug, Clone)]
pub enum TemporalViewAction {
    ViewInGraph(String),
    ExploreRelations(String, TemporalRelation),
    RefreshData,
}

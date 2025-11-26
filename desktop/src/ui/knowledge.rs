//! Knowledge browser panel - Premium visual design

use eframe::egui::{self, Color32, RichText, Rounding, ScrollArea, Stroke, Vec2};
use crate::app::ConceptInfo;
use crate::theme::{PRIMARY, SECONDARY, ACCENT, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_MUTED, SUCCESS, BG_PANEL, BG_WIDGET, card_frame};

pub struct KnowledgePanel {
    pub concepts: Vec<ConceptInfo>,
    pub selected_concept: Option<String>,
    pub search_query: String,
    pub is_loading: bool,
}

impl Default for KnowledgePanel {
    fn default() -> Self {
        Self {
            concepts: Vec::new(),
            selected_concept: None,
            search_query: String::new(),
            is_loading: false,
        }
    }
}

impl KnowledgePanel {
    pub fn ui(&mut self, ui: &mut egui::Ui) -> Option<KnowledgeAction> {
        let mut action = None;
        
        ui.horizontal(|ui| {
            // Left: concept list with border
            egui::Frame::none()
                .fill(BG_PANEL)
                .inner_margin(16.0)
                .show(ui, |ui| {
                    ui.set_width(320.0);
                    ui.set_min_height(ui.available_height());
                    
                    ui.vertical(|ui| {
                        // Header with gradient text effect
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("🧠").size(24.0));
                            ui.add_space(8.0);
                            ui.vertical(|ui| {
                                ui.label(RichText::new("Knowledge Base").size(18.0).color(TEXT_PRIMARY).strong());
                                ui.label(RichText::new("Explore learned concepts").size(11.0).color(TEXT_MUTED));
                            });
                        });
                        
                        ui.add_space(16.0);
                        
                        // Premium search box
                        self.search_box(ui, &mut action);
                        
                        ui.add_space(12.0);
                        
                        // Stats bar
                        self.stats_bar(ui, &mut action);
                        
                        ui.add_space(12.0);
                        
                        // Concept list
                        ScrollArea::vertical()
                            .auto_shrink([false; 2])
                            .show(ui, |ui| {
                                if self.is_loading {
                                    self.loading_state(ui);
                                } else if self.concepts.is_empty() {
                                    self.empty_state(ui);
                                } else {
                                    for concept in &self.concepts {
                                        let is_sel = self.selected_concept.as_ref() == Some(&concept.id);
                                        if self.concept_card(ui, concept, is_sel).clicked() {
                                            self.selected_concept = Some(concept.id.clone());
                                        }
                                        ui.add_space(6.0);
                                    }
                                }
                            });
                    });
                });
            
            // Separator line
            let sep_rect = ui.available_rect_before_wrap();
            ui.painter().line_segment(
                [sep_rect.left_top(), sep_rect.left_bottom()],
                Stroke::new(1.0, BG_WIDGET)
            );
            
            // Right: detail panel
            egui::Frame::none()
                .inner_margin(24.0)
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    
                    if let Some(id) = &self.selected_concept {
                        if let Some(c) = self.concepts.iter().find(|c| &c.id == id) {
                            self.concept_detail(ui, c);
                        } else {
                            self.no_selection(ui);
                        }
                    } else {
                        self.no_selection(ui);
                    }
                });
        });
        
        action
    }
    
    fn search_box(&mut self, ui: &mut egui::Ui, action: &mut Option<KnowledgeAction>) {
        egui::Frame::none()
            .fill(BG_WIDGET)
            .rounding(Rounding::same(10.0))
            .inner_margin(egui::Margin::symmetric(12.0, 8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("🔍").size(14.0).color(TEXT_MUTED));
                    ui.add_space(8.0);
                    let resp = ui.add_sized(
                        Vec2::new(ui.available_width(), 20.0),
                        egui::TextEdit::singleline(&mut self.search_query)
                            .hint_text("Search concepts...")
                            .frame(false)
                    );
                    if resp.changed() {
                        *action = Some(KnowledgeAction::Search(self.search_query.clone()));
                    }
                });
            });
    }
    
    fn stats_bar(&self, ui: &mut egui::Ui, action: &mut Option<KnowledgeAction>) {
        ui.horizontal(|ui| {
            // Count badge
            egui::Frame::none()
                .fill(PRIMARY.gamma_multiply(0.15))
                .rounding(Rounding::same(12.0))
                .inner_margin(egui::Margin::symmetric(8.0, 3.0))
                .show(ui, |ui| {
                    ui.label(RichText::new(format!("{} concepts", self.concepts.len()))
                        .size(11.0).color(PRIMARY));
                });
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Refresh button
                let btn = egui::Button::new(RichText::new("↻").size(14.0))
                    .fill(Color32::TRANSPARENT)
                    .rounding(Rounding::same(6.0));
                if ui.add(btn).on_hover_text("Refresh").clicked() {
                    *action = Some(KnowledgeAction::Refresh);
                }
            });
        });
    }
    
    fn loading_state(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(60.0);
            ui.spinner();
            ui.add_space(12.0);
            ui.label(RichText::new("Loading concepts...").size(13.0).color(TEXT_MUTED));
        });
    }
    
    fn empty_state(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(60.0);
            
            // Decorative empty state
            egui::Frame::none()
                .fill(BG_WIDGET)
                .rounding(Rounding::same(16.0))
                .inner_margin(24.0)
                .show(ui, |ui| {
                    ui.label(RichText::new("🧠").size(48.0));
                });
            
            ui.add_space(16.0);
            ui.label(RichText::new("No concepts yet").size(15.0).color(TEXT_SECONDARY));
            ui.add_space(4.0);
            ui.label(RichText::new("Start a conversation to teach me!").size(12.0).color(TEXT_MUTED));
        });
    }
    
    fn concept_card(&self, ui: &mut egui::Ui, c: &ConceptInfo, is_sel: bool) -> egui::Response {
        let bg = if is_sel { 
            PRIMARY.gamma_multiply(0.2) 
        } else { 
            BG_WIDGET.gamma_multiply(0.5) 
        };
        let border = if is_sel { PRIMARY.gamma_multiply(0.5) } else { Color32::TRANSPARENT };
        
        egui::Frame::none()
            .fill(bg)
            .stroke(Stroke::new(1.0, border))
            .rounding(Rounding::same(10.0))
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                
                // Content preview
                let title = if c.content.len() > 60 { 
                    format!("{}...", &c.content[..60]) 
                } else { 
                    c.content.clone() 
                };
                ui.label(RichText::new(title).size(13.0).color(TEXT_PRIMARY));
                
                ui.add_space(8.0);
                
                // Metadata row
                ui.horizontal(|ui| {
                    // ID badge
                    egui::Frame::none()
                        .fill(BG_WIDGET)
                        .rounding(Rounding::same(4.0))
                        .inner_margin(egui::Margin::symmetric(4.0, 2.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new(format!("{}...", &c.id[..8]))
                                .size(9.0).color(TEXT_MUTED).monospace());
                        });
                    
                    ui.add_space(6.0);
                    
                    // Strength indicator
                    self.mini_stat(ui, "⚡", format!("{:.0}%", c.strength * 100.0), ACCENT);
                    
                    // Connections
                    if !c.neighbors.is_empty() {
                        self.mini_stat(ui, "🔗", format!("{}", c.neighbors.len()), SECONDARY);
                    }
                });
            }).response
    }
    
    fn mini_stat(&self, ui: &mut egui::Ui, icon: &str, value: String, color: Color32) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            ui.label(RichText::new(icon).size(10.0));
            ui.label(RichText::new(value).size(10.0).color(color));
        });
    }
    
    fn concept_detail(&self, ui: &mut egui::Ui, c: &ConceptInfo) {
        // Header
        ui.horizontal(|ui| {
            ui.label(RichText::new("📋").size(24.0));
            ui.add_space(8.0);
            ui.label(RichText::new("Concept Details").size(20.0).color(TEXT_PRIMARY).strong());
        });
        
        ui.add_space(20.0);
        
        // ID card
        self.detail_section(ui, "Identifier", |ui| {
            ui.label(RichText::new(&c.id).size(12.0).color(TEXT_SECONDARY).monospace());
        });
        
        ui.add_space(12.0);
        
        // Content card
        self.detail_section(ui, "Content", |ui| {
            egui::Frame::none()
                .fill(BG_WIDGET)
                .rounding(Rounding::same(8.0))
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.label(RichText::new(&c.content).size(14.0).color(TEXT_PRIMARY));
                });
        });
        
        ui.add_space(12.0);
        
        // Stats row
        ui.horizontal(|ui| {
            self.stat_card(ui, "Strength", format!("{:.1}%", c.strength * 100.0), SUCCESS);
            ui.add_space(12.0);
            self.stat_card(ui, "Confidence", format!("{:.1}%", c.confidence * 100.0), ACCENT);
        });
        
        // Connections
        if !c.neighbors.is_empty() {
            ui.add_space(16.0);
            self.detail_section(ui, &format!("Connections ({})", c.neighbors.len()), |ui| {
                for n in &c.neighbors {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("→").size(12.0).color(SECONDARY));
                        ui.label(RichText::new(format!("{}...", &n[..8.min(n.len())]))
                            .size(12.0).color(TEXT_PRIMARY).monospace());
                    });
                }
            });
        }
    }
    
    fn detail_section(&self, ui: &mut egui::Ui, title: &str, content: impl FnOnce(&mut egui::Ui)) {
        ui.label(RichText::new(title).size(11.0).color(TEXT_MUTED).strong());
        ui.add_space(6.0);
        content(ui);
    }
    
    fn stat_card(&self, ui: &mut egui::Ui, label: &str, value: String, color: Color32) {
        egui::Frame::none()
            .fill(color.gamma_multiply(0.1))
            .stroke(Stroke::new(1.0, color.gamma_multiply(0.3)))
            .rounding(Rounding::same(10.0))
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new(label).size(11.0).color(TEXT_MUTED));
                    ui.label(RichText::new(value).size(28.0).color(color).strong());
                });
            });
    }
    
    fn no_selection(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() / 3.0);
            
            egui::Frame::none()
                .fill(BG_WIDGET)
                .rounding(Rounding::same(20.0))
                .inner_margin(32.0)
                .show(ui, |ui| {
                    ui.label(RichText::new("📚").size(56.0));
                });
            
            ui.add_space(20.0);
            ui.label(RichText::new("Select a concept").size(16.0).color(TEXT_SECONDARY));
            ui.add_space(6.0);
            ui.label(RichText::new("Click on any concept to view details").size(12.0).color(TEXT_MUTED));
        });
    }
    
    pub fn set_concepts(&mut self, concepts: Vec<ConceptInfo>) {
        self.concepts = concepts;
        self.is_loading = false;
    }
}

#[derive(Debug, Clone)]
pub enum KnowledgeAction {
    Search(String),
    Refresh,
    SelectConcept(String),
}

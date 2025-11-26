//! Knowledge browser panel - Premium visual design

use eframe::egui::{self, Color32, RichText, Rounding, ScrollArea, Stroke, Vec2};
use crate::types::ConceptInfo;
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
        
        ui.vertical(|ui| {
            // Clean header
            ui.horizontal(|ui| {
                ui.label(RichText::new("📚 Knowledge").size(20.0).color(TEXT_PRIMARY).strong());
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Refresh button
                    if ui.button("🔄").on_hover_text("Refresh").clicked() {
                        action = Some(KnowledgeAction::Refresh);
                    }
                    
                    ui.add_space(8.0);
                    
                    // Count
                    ui.label(RichText::new(format!("{} concepts", self.concepts.len()))
                        .size(12.0).color(TEXT_MUTED));
                });
            });
            
            ui.add_space(12.0);
            
            // Search bar - full width
            let search_resp = ui.add(
                egui::TextEdit::singleline(&mut self.search_query)
                    .hint_text("🔍 Search concepts...")
                    .desired_width(ui.available_width())
            );
            if search_resp.changed() {
                action = Some(KnowledgeAction::Search(self.search_query.clone()));
            }
            
            ui.add_space(16.0);
            
            // Content area
            if self.is_loading {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.spinner();
                    ui.add_space(8.0);
                    ui.label(RichText::new("Loading...").color(TEXT_MUTED));
                });
            } else if self.concepts.is_empty() {
                self.empty_state(ui);
            } else {
                // Concepts grid
                ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        self.concepts_grid(ui, &mut action);
                    });
            }
        });
        
        action
    }
    
    fn concepts_grid(&mut self, ui: &mut egui::Ui, action: &mut Option<KnowledgeAction>) {
        // Simple vertical list with delete functionality
        let concepts_clone = self.concepts.clone();
        for concept in &concepts_clone {
            ui.horizontal(|ui| {
                // Main concept card area
                let card_width = ui.available_width() - 40.0;
                ui.allocate_ui_with_layout(
                    Vec2::new(card_width, 60.0),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        if self.render_concept_card(ui, concept) {
                            *action = Some(KnowledgeAction::SelectConcept(concept.id.clone()));
                        }
                    }
                );
                
                // Delete button
                let delete_btn = egui::Button::new(
                    RichText::new("🗑").size(16.0).color(Color32::from_rgb(220, 80, 80))
                )
                .fill(Color32::TRANSPARENT)
                .frame(false)
                .rounding(Rounding::same(6.0));
                
                if ui.add(delete_btn).on_hover_text("Delete concept").clicked() {
                    *action = Some(KnowledgeAction::DeleteConcept(concept.id.clone()));
                }
            });
            ui.add_space(8.0);
        }
    }
    
    fn render_concept_card(&self, ui: &mut egui::Ui, concept: &ConceptInfo) -> bool {
        egui::Frame::none()
            .fill(BG_WIDGET)
            .stroke(Stroke::new(1.0, BG_WIDGET.gamma_multiply(1.5)))
            .rounding(Rounding::same(8.0))
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Icon
                    ui.label(RichText::new("📎").size(16.0));
                    
                    ui.add_space(8.0);
                    
                    // Content
                    ui.vertical(|ui| {
                        let preview = if concept.content.len() > 80 {
                            format!("{}...", &concept.content[..80])
                        } else {
                            concept.content.clone()
                        };
                        
                        ui.label(RichText::new(preview).size(14.0).color(TEXT_PRIMARY));
                        
                        ui.add_space(4.0);
                        
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("ID: {}", &concept.id[..8])).size(11.0).color(TEXT_MUTED));
                            ui.add_space(12.0);
                            ui.label(RichText::new(format!("Confidence: {:.1}%", concept.confidence * 100.0)).size(11.0).color(TEXT_MUTED));
                        });
                    });
                });
            })
            .response
            .clicked()
    }
    
    fn compact_concept_card(&self, ui: &mut egui::Ui, concept: &ConceptInfo) -> egui::Response {
        egui::Frame::none()
            .fill(BG_WIDGET)
            .stroke(Stroke::new(1.0, BG_WIDGET.gamma_multiply(1.5)))
            .rounding(Rounding::same(8.0))
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                
                ui.vertical(|ui| {
                    // Content preview
                    let content = if concept.content.len() > 120 {
                        format!("{}...", &concept.content[..117])
                    } else {
                        concept.content.clone()
                    };
                    
                    ui.label(RichText::new(content).size(14.0).color(TEXT_PRIMARY));
                    ui.add_space(8.0);
                    
                    // Bottom row with stats
                    ui.horizontal(|ui| {
                        // Confidence badge
                        let conf_color = if concept.confidence > 0.8 { 
                            SUCCESS 
                        } else if concept.confidence > 0.6 { 
                            ACCENT 
                        } else { 
                            TEXT_MUTED 
                        };
                        
                        ui.label(RichText::new(format!("{}%", (concept.confidence * 100.0) as u8))
                            .size(11.0).color(conf_color))
                            .on_hover_text("Confidence");
                        
                        ui.add_space(12.0);
                        
                        // Connections
                        if !concept.neighbors.is_empty() {
                            ui.label(RichText::new(format!("🔗 {}", concept.neighbors.len()))
                                .size(11.0).color(TEXT_MUTED))
                                .on_hover_text("Connections");
                        }
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Short ID
                            ui.label(RichText::new(&concept.id[..8])
                                .size(10.0).color(TEXT_MUTED).monospace())
                                .on_hover_text(&concept.id);
                        });
                    });
                });
            }).response
    }
    
    fn empty_state(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            
            // Simple empty state
            ui.label(RichText::new("📚").size(48.0));
            ui.add_space(16.0);
            
            ui.label(RichText::new("No concepts yet").size(16.0).color(TEXT_PRIMARY));
            ui.add_space(8.0);
            
            ui.label(RichText::new("Start learning by going to Chat and typing:").size(13.0).color(TEXT_MUTED));
            ui.add_space(12.0);
            
            // Example command
            egui::Frame::none()
                .fill(BG_WIDGET)
                .rounding(Rounding::same(6.0))
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.label(RichText::new("/learn The sky is blue").size(12.0).color(TEXT_PRIMARY).monospace());
                });
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
    DeleteConcept(String),
}

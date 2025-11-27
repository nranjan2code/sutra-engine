//! Reasoning Paths Panel - MPPA-style multi-path visualization
//!
//! Visualizes parallel reasoning paths with consensus analysis.
//! Features: path comparison, confidence scoring, consensus display.

use std::collections::HashMap;
use eframe::egui::{self, Color32, RichText, Rounding, ScrollArea, Stroke, Vec2};
use sutra_storage::{ConcurrentMemory, ConceptId, ParallelPathFinder, PathResult};
use crate::theme::*;
use crate::types::{ReasoningPath, PathStep, ConsensusResult, PathCluster};

/// Reasoning Paths Panel
pub struct ReasoningPathsPanel {
    pub query_from: String,
    pub query_to: String,
    pub paths: Vec<ReasoningPath>,
    pub consensus: Option<ConsensusResult>,
    pub expanded_path: Option<usize>,
    pub is_loading: bool,
    pub error_message: Option<String>,
    
    // Settings
    pub max_depth: usize,
    pub max_paths: usize,
    pub confidence_decay: f32,
    pub consensus_threshold: f32,
}

impl Default for ReasoningPathsPanel {
    fn default() -> Self {
        Self {
            query_from: String::new(),
            query_to: String::new(),
            paths: Vec::new(),
            consensus: None,
            expanded_path: None,
            is_loading: false,
            error_message: None,
            max_depth: 6,
            max_paths: 10,
            confidence_decay: 0.85,
            consensus_threshold: 0.5,
        }
    }
}

impl ReasoningPathsPanel {
    /// Find and analyze paths between concepts
    pub fn find_paths(&mut self, storage: &ConcurrentMemory, from_id: ConceptId, to_id: ConceptId) {
        self.is_loading = true;
        self.error_message = None;
        self.paths.clear();
        self.consensus = None;
        
        // Use storage's parallel pathfinder
        let snapshot = storage.get_snapshot();
        let pathfinder = ParallelPathFinder::new(self.confidence_decay);
        
        let raw_paths = pathfinder.find_paths_parallel(
            snapshot.clone(),
            from_id,
            to_id,
            self.max_depth,
            self.max_paths,
        );
        
        if raw_paths.is_empty() {
            self.error_message = Some("No paths found between these concepts.".to_string());
            self.is_loading = false;
            return;
        }
        
        // Convert to our path format
        self.paths = raw_paths
            .into_iter()
            .map(|p| self.convert_path(&p, &snapshot))
            .collect();
        
        // Run MPPA analysis
        self.consensus = Some(self.analyze_consensus());
        self.is_loading = false;
    }
    
    /// Convert PathResult to ReasoningPath with content
    fn convert_path(&self, path: &PathResult, snapshot: &std::sync::Arc<sutra_storage::GraphSnapshot>) -> ReasoningPath {
        let steps: Vec<PathStep> = path.path
            .iter()
            .enumerate()
            .map(|(i, id)| {
                let content = snapshot.get_concept(id)
                    .map(|c| String::from_utf8_lossy(&c.content).to_string())
                    .unwrap_or_else(|| format!("Concept {}", id.to_hex()[..8].to_string()));
                
                let confidence = self.confidence_decay.powi(i as i32);
                let relation = if i == 0 { "Start".to_string() } else { "→".to_string() };
                
                PathStep {
                    concept_id: *id,
                    content,
                    confidence,
                    relation,
                }
            })
            .collect();
        
        ReasoningPath {
            confidence: path.confidence,
            depth: path.depth,
            path: steps,
        }
    }
    
    /// Convert PathResult to ReasoningPath with content (Static version)
    pub fn convert_path_static(path: &PathResult, snapshot: &std::sync::Arc<sutra_storage::GraphSnapshot>, decay: f32) -> ReasoningPath {
        let steps: Vec<PathStep> = path.path
            .iter()
            .enumerate()
            .map(|(i, id)| {
                let content = snapshot.get_concept(id)
                    .map(|c| String::from_utf8_lossy(&c.content).to_string())
                    .unwrap_or_else(|| format!("Concept {}", id.to_hex()[..8].to_string()));
                
                let confidence = decay.powi(i as i32);
                let relation = if i == 0 { "Start".to_string() } else { "→".to_string() };
                
                PathStep {
                    concept_id: *id,
                    content,
                    confidence,
                    relation,
                }
            })
            .collect();
        
        ReasoningPath {
            confidence: path.confidence,
            depth: path.depth,
            path: steps,
        }
    }

    /// Run MPPA-style consensus analysis
    fn analyze_consensus(&self) -> ConsensusResult {
        if self.paths.is_empty() {
            return ConsensusResult {
                primary_cluster: PathCluster {
                    destination: ConceptId::from_bytes([0; 16]),
                    destination_content: "No paths".to_string(),
                    paths: vec![],
                    avg_confidence: 0.0,
                    consensus_weight: 0.0,
                    support_ratio: 0.0,
                },
                alternatives: vec![],
                total_paths: 0,
                explanation: "No paths found.".to_string(),
            };
        }
        
        // Cluster paths by destination
        let mut clusters: HashMap<String, Vec<ReasoningPath>> = HashMap::new();
        
        for path in &self.paths {
            if let Some(last_step) = path.path.last() {
                let key = last_step.concept_id.to_hex();
                clusters.entry(key).or_default().push(path.clone());
            }
        }
        
        // Convert to PathCluster and score
        let total_paths = self.paths.len();
        let mut scored_clusters: Vec<PathCluster> = clusters
            .into_iter()
            .map(|(_, paths)| {
                let last_step = paths[0].path.last().unwrap();
                let support_ratio = paths.len() as f32 / total_paths as f32;
                let avg_confidence = paths.iter().map(|p| p.confidence).sum::<f32>() / paths.len() as f32;
                
                // Calculate consensus weight (MPPA formula)
                let consensus_bonus = if support_ratio >= self.consensus_threshold {
                    1.0 + (support_ratio - self.consensus_threshold)
                } else {
                    1.0
                };
                let outlier_penalty = if support_ratio < 0.2 { 0.7 } else { 1.0 };
                let consensus_weight = avg_confidence * support_ratio * consensus_bonus * outlier_penalty;
                
                PathCluster {
                    destination: last_step.concept_id,
                    destination_content: last_step.content.clone(),
                    paths,
                    avg_confidence,
                    consensus_weight,
                    support_ratio,
                }
            })
            .collect();
        
        // Sort by consensus weight
        scored_clusters.sort_by(|a, b| b.consensus_weight.partial_cmp(&a.consensus_weight).unwrap());
        
        let primary = scored_clusters.remove(0);
        let primary_support = (primary.support_ratio * 100.0) as usize;
        
        let explanation = format!(
            "Consensus reached with {}% agreement ({}/{} paths). Confidence: {:.0}%.",
            primary_support,
            primary.paths.len(),
            total_paths,
            primary.avg_confidence * 100.0
        );
        
        ConsensusResult {
            primary_cluster: primary,
            alternatives: scored_clusters,
            total_paths,
            explanation,
        }
    }
    
    /// Run MPPA-style consensus analysis (Static version)
    pub fn analyze_consensus_static(paths: &[ReasoningPath], threshold: f32) -> ConsensusResult {
        if paths.is_empty() {
            return ConsensusResult {
                primary_cluster: PathCluster {
                    destination: ConceptId::from_bytes([0; 16]),
                    destination_content: "No paths".to_string(),
                    paths: vec![],
                    avg_confidence: 0.0,
                    consensus_weight: 0.0,
                    support_ratio: 0.0,
                },
                alternatives: vec![],
                total_paths: 0,
                explanation: "No paths found.".to_string(),
            };
        }
        
        // Cluster paths by destination
        let mut clusters: HashMap<String, Vec<ReasoningPath>> = HashMap::new();
        
        for path in paths {
            if let Some(last_step) = path.path.last() {
                let key = last_step.concept_id.to_hex();
                clusters.entry(key).or_default().push(path.clone());
            }
        }
        
        // Convert to PathCluster and score
        let total_paths = paths.len();
        let mut scored_clusters: Vec<PathCluster> = clusters
            .into_iter()
            .map(|(_, paths)| {
                let last_step = paths[0].path.last().unwrap();
                let support_ratio = paths.len() as f32 / total_paths as f32;
                let avg_confidence = paths.iter().map(|p| p.confidence).sum::<f32>() / paths.len() as f32;
                
                // Calculate consensus weight (MPPA formula)
                let consensus_bonus = if support_ratio >= threshold {
                    1.0 + (support_ratio - threshold)
                } else {
                    1.0
                };
                let outlier_penalty = if support_ratio < 0.2 { 0.7 } else { 1.0 };
                let consensus_weight = avg_confidence * support_ratio * consensus_bonus * outlier_penalty;
                
                PathCluster {
                    destination: last_step.concept_id,
                    destination_content: last_step.content.clone(),
                    paths,
                    avg_confidence,
                    consensus_weight,
                    support_ratio,
                }
            })
            .collect();
        
        // Sort by consensus weight
        scored_clusters.sort_by(|a, b| b.consensus_weight.partial_cmp(&a.consensus_weight).unwrap());
        
        let primary = scored_clusters.remove(0);
        let primary_support = (primary.support_ratio * 100.0) as usize;
        
        let explanation = format!(
            "Consensus reached with {}% agreement ({}/{} paths). Confidence: {:.0}%.",
            primary_support,
            primary.paths.len(),
            total_paths,
            primary.avg_confidence * 100.0
        );
        
        ConsensusResult {
            primary_cluster: primary,
            alternatives: scored_clusters,
            total_paths,
            explanation,
        }
    }
    
    /// Render the panel
    pub fn ui(&mut self, ui: &mut egui::Ui) -> Option<ReasoningPathsAction> {
        let mut action = None;
        
        ui.vertical(|ui| {
            // Header
            self.render_header(ui);
            
            ui.add_space(12.0);
            
            // Query input
            self.render_query_input(ui, &mut action);
            
            ui.add_space(16.0);
            
            // Consensus summary (if available)
            if let Some(consensus) = &self.consensus {
                self.render_consensus_summary(ui, consensus);
                ui.add_space(16.0);
            }
            
            // Error message
            if let Some(err) = &self.error_message {
                egui::Frame::none()
                    .fill(ERROR.gamma_multiply(0.15))
                    .rounding(Rounding::same(8.0))
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        ui.label(RichText::new(format!("⚠️ {}", err)).size(13.0).color(ERROR));
                    });
                ui.add_space(12.0);
            }
            
            // Loading state
            if self.is_loading {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.spinner();
                    ui.add_space(8.0);
                    ui.label(RichText::new("Finding reasoning paths...").color(TEXT_MUTED));
                });
                return;
            }
            
            // Smart Empty State
            if self.paths.is_empty() && self.error_message.is_none() {
                ui.vertical_centered(|ui| {
                    ui.add_space(60.0);
                    ui.label(RichText::new("🛣️").size(40.0));
                    ui.add_space(16.0);
                    ui.label(RichText::new("Discover Connections").size(18.0).color(TEXT_PRIMARY).strong());
                    ui.add_space(8.0);
                    ui.label(RichText::new("Find multi-hop reasoning paths between any two concepts.").size(14.0).color(TEXT_MUTED));
                    ui.add_space(24.0);
                    
                    egui::Frame::none()
                        .fill(BG_WIDGET)
                        .rounding(Rounding::same(8.0))
                        .inner_margin(12.0)
                        .show(ui, |ui| {
                            ui.label(RichText::new("Try searching for:").size(12.0).color(TEXT_SECONDARY));
                            ui.add_space(4.0);
                            ui.monospace("From: \"Rust\"  To: \"Performance\"");
                        });
                });
                
                // Still show controls at bottom
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.add_space(20.0);
                    self.render_controls(ui, &mut action);
                });
                
                return;
            }
            
            // Path list
            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    // Collect path data first to avoid borrow issues
                    let paths_data: Vec<_> = self.paths.iter().enumerate()
                        .map(|(i, path)| {
                            let is_expanded = self.expanded_path == Some(i);
                            let supports_consensus = self.consensus.as_ref()
                                .map(|c| path.path.last().map(|s| s.concept_id) == Some(c.primary_cluster.destination))
                                .unwrap_or(false);
                            (i, path.clone(), is_expanded, supports_consensus)
                        })
                        .collect();
                    
                    for (i, path, is_expanded, supports_consensus) in paths_data {
                        self.render_path_card(ui, i, &path, is_expanded, supports_consensus);
                        ui.add_space(8.0);
                    }
                    
                    // Alternatives section
                    if let Some(consensus) = &self.consensus {
                        if !consensus.alternatives.is_empty() {
                            ui.add_space(16.0);
                            ui.label(RichText::new("Alternative Conclusions:").size(13.0).color(TEXT_MUTED).strong());
                            ui.add_space(8.0);
                            
                            let alts: Vec<_> = consensus.alternatives.clone();
                            for alt in &alts {
                                self.render_alternative(ui, alt);
                                ui.add_space(4.0);
                            }
                        }
                    }
                });
            
            // Controls
            ui.add_space(12.0);
            self.render_controls(ui, &mut action);
        });
        
        action
    }
    
    fn render_header(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("🕸️").size(20.0));
            ui.add_space(4.0);
            ui.label(RichText::new("Reasoning Paths").size(22.0).color(TEXT_PRIMARY).strong());
            
            if !self.paths.is_empty() {
                ui.add_space(12.0);
                egui::Frame::none()
                    .fill(BG_WIDGET)
                    .rounding(Rounding::same(10.0))
                    .inner_margin(egui::Margin::symmetric(8.0, 2.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new(format!("{} paths found", self.paths.len())).size(11.0).color(TEXT_SECONDARY));
                    });
            }
        });
    }
    
    fn render_query_input(&mut self, ui: &mut egui::Ui, action: &mut Option<ReasoningPathsAction>) {
        egui::Frame::none()
            .fill(BG_WIDGET)
            .rounding(Rounding::same(12.0))
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.label(RichText::new("Find paths between concepts:").size(12.0).color(TEXT_MUTED));
                ui.add_space(8.0);
                
                ui.horizontal(|ui| {
                    ui.label(RichText::new("From:").size(12.0).color(TEXT_SECONDARY));
                    ui.add_sized(
                        Vec2::new(200.0, 24.0),
                        egui::TextEdit::singleline(&mut self.query_from)
                            .hint_text("Starting concept...")
                    );
                    
                    ui.add_space(16.0);
                    
                    ui.label(RichText::new("To:").size(12.0).color(TEXT_SECONDARY));
                    ui.add_sized(
                        Vec2::new(200.0, 24.0),
                        egui::TextEdit::singleline(&mut self.query_to)
                            .hint_text("Target concept...")
                    );
                    
                    ui.add_space(16.0);
                    
                    let can_search = !self.query_from.is_empty() && !self.query_to.is_empty();
                    if ui.add_enabled(can_search, egui::Button::new(RichText::new("🔍 Find Paths").size(13.0))).clicked() {
                        *action = Some(ReasoningPathsAction::FindPaths(
                            self.query_from.clone(),
                            self.query_to.clone(),
                        ));
                    }
                });
            });
    }
    
    fn render_consensus_summary(&self, ui: &mut egui::Ui, consensus: &ConsensusResult) {
        egui::Frame::none()
            .fill(SUCCESS.gamma_multiply(0.12))
            .stroke(Stroke::new(2.0, SUCCESS.gamma_multiply(0.4)))
            .rounding(Rounding::same(12.0))
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("✓ CONSENSUS:").size(14.0).color(SUCCESS).strong());
                    ui.add_space(8.0);
                    ui.label(RichText::new(&consensus.primary_cluster.destination_content)
                        .size(14.0).color(TEXT_PRIMARY).strong());
                });
                
                ui.add_space(8.0);
                
                ui.horizontal(|ui| {
                    // Confidence badge
                    egui::Frame::none()
                        .fill(BG_WIDGET)
                        .rounding(Rounding::same(6.0))
                        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new(format!("Confidence: {:.0}%", consensus.primary_cluster.avg_confidence * 100.0))
                                .size(11.0).color(ACCENT));
                        });
                    
                    // Paths agreement
                    egui::Frame::none()
                        .fill(BG_WIDGET)
                        .rounding(Rounding::same(6.0))
                        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new(format!("Paths: {}/{} agree ({:.0}%)",
                                consensus.primary_cluster.paths.len(),
                                consensus.total_paths,
                                consensus.primary_cluster.support_ratio * 100.0
                            )).size(11.0).color(SECONDARY));
                        });
                    
                    // Alternatives count
                    if !consensus.alternatives.is_empty() {
                        egui::Frame::none()
                            .fill(BG_WIDGET)
                            .rounding(Rounding::same(6.0))
                            .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                            .show(ui, |ui| {
                                ui.label(RichText::new(format!("Alternatives: {}", consensus.alternatives.len()))
                                    .size(11.0).color(WARNING));
                            });
                    }
                });
                
                ui.add_space(4.0);
                ui.label(RichText::new(&consensus.explanation).size(12.0).color(TEXT_SECONDARY));
            });
    }
    
    fn render_path_card(&mut self, ui: &mut egui::Ui, index: usize, path: &ReasoningPath, is_expanded: bool, supports_consensus: bool) {
        let border_color = if supports_consensus {
            SUCCESS.gamma_multiply(0.5)
        } else {
            WARNING.gamma_multiply(0.3)
        };
        
        egui::Frame::none()
            .fill(BG_WIDGET.gamma_multiply(0.7))
            .stroke(Stroke::new(1.0, border_color))
            .rounding(Rounding::same(10.0))
            .inner_margin(12.0)
            .show(ui, |ui| {
                // Header
                ui.horizontal(|ui| {
                    // Consensus indicator
                    if supports_consensus {
                        ui.label(RichText::new("✓").size(14.0).color(SUCCESS));
                    } else {
                        ui.label(RichText::new("○").size(14.0).color(WARNING));
                    }
                    
                    ui.label(RichText::new(format!("Path {}", index + 1)).size(13.0).color(TEXT_PRIMARY).strong());
                    
                    // Confidence
                    ui.add_space(8.0);
                    egui::Frame::none()
                        .fill(self.confidence_color(path.confidence).gamma_multiply(0.2))
                        .rounding(Rounding::same(6.0))
                        .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new(format!("Confidence: {:.0}%", path.confidence * 100.0))
                                .size(11.0).color(self.confidence_color(path.confidence)));
                        });
                    
                    // Depth
                    ui.label(RichText::new(format!("({} hops)", path.depth)).size(11.0).color(TEXT_MUTED));
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let btn_text = if is_expanded { "▼" } else { "▶" };
                        if ui.button(RichText::new(btn_text).size(12.0)).clicked() {
                            self.expanded_path = if is_expanded { None } else { Some(index) };
                        }
                    });
                });
                
                // Path preview (always shown)
                ui.add_space(8.0);
                self.render_path_preview(ui, path);
                
                // Expanded details
                if is_expanded {
                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(8.0);
                    
                    self.render_path_detail(ui, path);
                }
            });
    }
    
    fn render_path_preview(&self, ui: &mut egui::Ui, path: &ReasoningPath) {
        ui.horizontal_wrapped(|ui| {
            for (i, step) in path.path.iter().enumerate() {
                if i > 0 {
                    ui.label(RichText::new("→").size(12.0).color(SECONDARY));
                }
                
                let label = if step.content.len() > 25 {
                    format!("{}...", &step.content[..25])
                } else {
                    step.content.clone()
                };
                
                ui.label(RichText::new(&label).size(12.0).color(TEXT_PRIMARY));
            }
        });
    }
    
    fn render_path_detail(&self, ui: &mut egui::Ui, path: &ReasoningPath) {
        for (i, step) in path.path.iter().enumerate() {
            ui.horizontal(|ui| {
                // Step number
                egui::Frame::none()
                    .fill(PRIMARY.gamma_multiply(0.2))
                    .rounding(Rounding::same(10.0))
                    .inner_margin(egui::Margin::symmetric(8.0, 2.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new(format!("{}", i + 1)).size(11.0).color(PRIMARY));
                    });
                
                ui.add_space(8.0);
                
                // Content
                ui.vertical(|ui| {
                    ui.label(RichText::new(&step.content).size(13.0).color(TEXT_PRIMARY));
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("Confidence: {:.0}%", step.confidence * 100.0))
                            .size(10.0).color(TEXT_MUTED));
                        ui.add_space(8.0);
                        ui.label(RichText::new(format!("ID: {}...", &step.concept_id.to_hex()[..8]))
                            .size(10.0).color(TEXT_MUTED).monospace());
                    });
                });
            });
            
            if i < path.path.len() - 1 {
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    ui.label(RichText::new("│").size(14.0).color(BG_WIDGET));
                });
            }
        }
    }
    
    fn render_alternative(&self, ui: &mut egui::Ui, alt: &PathCluster) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("○").size(12.0).color(WARNING));
            ui.label(RichText::new(&alt.destination_content).size(12.0).color(TEXT_SECONDARY));
            ui.label(RichText::new(format!("({}/{} paths, {:.0}% confidence)",
                alt.paths.len(),
                alt.paths.len(),
                alt.avg_confidence * 100.0
            )).size(11.0).color(TEXT_MUTED));
        });
    }
    
    fn render_controls(&mut self, ui: &mut egui::Ui, action: &mut Option<ReasoningPathsAction>) {
        ui.horizontal(|ui| {
            // Settings
            ui.label(RichText::new("Max depth:").size(11.0).color(TEXT_MUTED));
            ui.add(egui::Slider::new(&mut self.max_depth, 2..=10).show_value(true));
            
            ui.add_space(16.0);
            
            ui.label(RichText::new("Max paths:").size(11.0).color(TEXT_MUTED));
            ui.add(egui::Slider::new(&mut self.max_paths, 3..=20).show_value(true));
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(RichText::new("📤 Export Reasoning").size(12.0)).clicked() {
                    *action = Some(ReasoningPathsAction::ExportReasoning);
                }
            });
        });
    }
    
    fn confidence_color(&self, confidence: f32) -> Color32 {
        if confidence > 0.8 {
            SUCCESS
        } else if confidence > 0.5 {
            ACCENT
        } else {
            WARNING
        }
    }
}

/// Actions from reasoning paths panel
#[derive(Debug, Clone)]
pub enum ReasoningPathsAction {
    FindPaths(String, String),
    ExportReasoning,
}

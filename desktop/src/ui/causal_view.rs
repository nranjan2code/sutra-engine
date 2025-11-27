//! Causal View Panel
//!
//! Root cause analysis and causal chain visualization.
//! Enables backward chain walking, multi-hop reasoning, and impact analysis.

use eframe::egui::{self, Color32, RichText, Rounding, ScrollArea, Stroke, Vec2, Pos2};
use crate::theme::*;
use crate::types::{CausalChain, CausalNode, CausalRelationType};
use std::collections::HashSet;

/// Causal View Panel
pub struct CausalView {
    // Query
    pub effect_query: String,
    pub max_hops: usize,
    pub include_indirect: bool,
    
    // Results
    pub causal_chains: Vec<CausalChain>,
    pub selected_chain: Option<usize>,
    pub expanded_nodes: HashSet<String>,
    
    // View settings
    pub view_mode: CausalViewMode,
    pub show_confidence: bool,
    pub highlight_root_causes: bool,
    
    // Analysis
    pub root_causes: Vec<CausalNode>,
    pub impact_analysis: Vec<(CausalNode, f32)>, // node and impact score
    
    // State
    pub is_analyzing: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CausalViewMode {
    ChainList,
    Tree,
    Graph,
}

impl Default for CausalView {
    fn default() -> Self {
        Self {
            effect_query: String::new(),
            max_hops: 5,
            include_indirect: true,
            causal_chains: Vec::new(),
            selected_chain: None,
            expanded_nodes: HashSet::new(),
            view_mode: CausalViewMode::Tree,
            show_confidence: true,
            highlight_root_causes: true,
            root_causes: Vec::new(),
            impact_analysis: Vec::new(),
            is_analyzing: false,
            error_message: None,
        }
    }
}

impl CausalView {
    /// Render the causal view
    pub fn ui(&mut self, ui: &mut egui::Ui) -> Option<CausalViewAction> {
        let mut action = None;
        
        ui.vertical(|ui| {
            // Header
            self.render_header(ui);
            
            ui.add_space(16.0);
            
            // Query input
            self.render_query_input(ui, &mut action);
            
            ui.add_space(16.0);
            
            // Main content area
            ui.horizontal(|ui| {
                // Left: Causal chains/tree
                egui::Frame::none()
                    .fill(BG_WIDGET.gamma_multiply(0.4))
                    .rounding(Rounding::same(12.0))
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width() * 0.6);
                        self.render_main_view(ui, &mut action);
                    });
                
                ui.add_space(12.0);
                
                // Right: Analysis panel
                ui.vertical(|ui| {
                    self.render_root_causes(ui, &mut action);
                    ui.add_space(12.0);
                    self.render_impact_analysis(ui, &mut action);
                });
            });
        });
        
        action
    }
    
    fn render_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("🔗").size(20.0));
            ui.add_space(4.0);
            ui.label(RichText::new("Causal Analysis").size(22.0).color(TEXT_PRIMARY).strong());
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // View mode buttons
                self.view_mode_button(ui, "🕸️ Graph", CausalViewMode::Graph);
                ui.add_space(8.0);
                self.view_mode_button(ui, "🌳 Tree", CausalViewMode::Tree);
                ui.add_space(8.0);
                self.view_mode_button(ui, "📋 List", CausalViewMode::ChainList);
            });
        });
    }
    
    fn view_mode_button(&mut self, ui: &mut egui::Ui, label: &str, mode: CausalViewMode) {
        let is_selected = self.view_mode == mode;
        let bg = if is_selected { ACCENT.gamma_multiply(0.25) } else { BG_WIDGET };
        let text_color = if is_selected { ACCENT } else { TEXT_SECONDARY };
        
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
    
    fn render_query_input(&mut self, ui: &mut egui::Ui, action: &mut Option<CausalViewAction>) {
        egui::Frame::none()
            .fill(BG_WIDGET.gamma_multiply(0.6))
            .rounding(Rounding::same(10.0))
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Effect / Question:").size(12.0).color(TEXT_MUTED));
                        ui.add_space(4.0);
                        
                        ui.add_sized(
                            Vec2::new(300.0, 28.0),
                            egui::TextEdit::singleline(&mut self.effect_query)
                                .hint_text("What caused this? e.g., 'system crash'")
                        );
                    });
                    
                    ui.add_space(16.0);
                    
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Max Hops:").size(12.0).color(TEXT_MUTED));
                        ui.add(egui::Slider::new(&mut self.max_hops, 1..=10).show_value(true));
                    });
                    
                    ui.add_space(16.0);
                    
                    ui.vertical(|ui| {
                        ui.add_space(18.0);
                        ui.checkbox(&mut self.include_indirect, 
                            RichText::new("Include indirect").size(11.0).color(TEXT_SECONDARY));
                    });
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let can_analyze = !self.effect_query.is_empty() && !self.is_analyzing;
                        
                        if ui.add_enabled(
                            can_analyze,
                            egui::Button::new(RichText::new("🔍 Analyze").size(13.0).color(Color32::WHITE))
                                .fill(ACCENT)
                                .min_size(Vec2::new(100.0, 32.0))
                        ).clicked() {
                            *action = Some(CausalViewAction::AnalyzeCause {
                                effect: self.effect_query.clone(),
                                max_hops: self.max_hops,
                            });
                            self.is_analyzing = true;
                        }
                    });
                });
            });
    }
    
    fn render_main_view(&mut self, ui: &mut egui::Ui, action: &mut Option<CausalViewAction>) {
        // Error message
        if let Some(err) = &self.error_message {
            egui::Frame::none()
                .fill(ERROR.gamma_multiply(0.15))
                .rounding(Rounding::same(8.0))
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.label(RichText::new(format!("⚠️ {}", err)).size(12.0).color(ERROR));
                });
            return;
        }
        
        // Loading state
        if self.is_analyzing {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                ui.spinner();
                ui.add_space(8.0);
                ui.label(RichText::new("Analyzing causal relationships...").size(13.0).color(TEXT_MUTED));
            });
            return;
        }
        
        // Empty state
        if self.causal_chains.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(60.0);
                ui.label(RichText::new("🔗").size(40.0));
                ui.add_space(16.0);
                ui.label(RichText::new("Root Cause Analysis").size(18.0).color(TEXT_PRIMARY).strong());
                ui.add_space(8.0);
                ui.label(RichText::new("Trace effects back to their origins across the graph.").size(14.0).color(TEXT_MUTED));
                ui.add_space(24.0);
                
                egui::Frame::none()
                    .fill(BG_WIDGET)
                    .rounding(Rounding::same(8.0))
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        ui.label(RichText::new("Try analyzing:").size(12.0).color(TEXT_SECONDARY));
                        ui.add_space(4.0);
                        ui.monospace("Effect: \"System Crash\"");
                        ui.add_space(8.0);
                        ui.label(RichText::new("Or teach causal links:").size(12.0).color(TEXT_SECONDARY));
                        ui.add_space(4.0);
                        ui.monospace("/learn Memory leak causes system crash");
                    });
            });
            return;
        }
        
        match self.view_mode {
            CausalViewMode::ChainList => self.render_chain_list(ui, action),
            CausalViewMode::Tree => self.render_tree_view(ui, action),
            CausalViewMode::Graph => self.render_graph_view(ui, action),
        }
    }
    
    fn render_chain_list(&mut self, ui: &mut egui::Ui, action: &mut Option<CausalViewAction>) {
        ui.label(RichText::new(format!("Found {} causal chains", self.causal_chains.len()))
            .size(13.0).color(TEXT_PRIMARY).strong());
        
        ui.add_space(8.0);
        
        ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                for (i, chain) in self.causal_chains.iter().enumerate() {
                    let is_selected = self.selected_chain == Some(i);
                    let bg = if is_selected { ACCENT.gamma_multiply(0.2) } else { BG_WIDGET.gamma_multiply(0.5) };
                    
                    egui::Frame::none()
                        .fill(bg)
                        .stroke(Stroke::new(1.0, if is_selected { ACCENT.gamma_multiply(0.5) } else { Color32::TRANSPARENT }))
                        .rounding(Rounding::same(8.0))
                        .inner_margin(12.0)
                        .show(ui, |ui| {
                            let response = ui.horizontal(|ui| {
                                // Chain number
                                egui::Frame::none()
                                    .fill(ACCENT.gamma_multiply(0.3))
                                    .rounding(Rounding::same(6.0))
                                    .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                                    .show(ui, |ui| {
                                        ui.label(RichText::new(format!("#{}", i + 1)).size(11.0).color(ACCENT).strong());
                                    });
                                
                                ui.add_space(8.0);
                                
                                ui.vertical(|ui| {
                                    // Chain representation
                                    ui.horizontal(|ui| {
                                        for (j, node) in chain.nodes.iter().enumerate() {
                                            if j > 0 {
                                                ui.label(RichText::new(" → ").size(11.0).color(TEXT_MUTED));
                                            }
                                            
                                            let node_color = if node.is_root_cause {
                                                ERROR
                                            } else if j == chain.nodes.len() - 1 {
                                                PRIMARY
                                            } else {
                                                TEXT_SECONDARY
                                            };
                                            
                                            let short_label = if node.label.len() > 20 {
                                                format!("{}...", &node.label[..20])
                                            } else {
                                                node.label.clone()
                                            };
                                            
                                            ui.label(RichText::new(&short_label).size(11.0).color(node_color));
                                        }
                                    });
                                    
                                    // Metadata
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(format!("{} hops", chain.nodes.len() - 1))
                                            .size(10.0).color(TEXT_MUTED));
                                        
                                        if self.show_confidence {
                                            ui.add_space(8.0);
                                            let conf_color = if chain.confidence > 0.8 { SUCCESS } else if chain.confidence > 0.5 { ACCENT } else { WARNING };
                                            ui.label(RichText::new(format!("{:.0}% conf", chain.confidence * 100.0))
                                                .size(10.0).color(conf_color));
                                        }
                                    });
                                });
                            });
                            
                            if response.response.interact(egui::Sense::click()).clicked() {
                                self.selected_chain = Some(i);
                            }
                        });
                    
                    ui.add_space(6.0);
                }
            });
    }
    
    fn render_tree_view(&mut self, ui: &mut egui::Ui, action: &mut Option<CausalViewAction>) {
        if self.causal_chains.is_empty() {
            return;
        }
        
        ui.label(RichText::new("Causal Tree View").size(13.0).color(TEXT_PRIMARY).strong());
        ui.add_space(8.0);
        
        ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                // Build tree from chains (simplified - take first chain)
                // Clone nodes to avoid borrow conflict with self.render_tree_node
                if let Some(nodes) = self.causal_chains.first().map(|c| c.nodes.clone()) {
                    self.render_tree_node(ui, &nodes, 0, action);
                }
            });
    }
    
    fn render_tree_node(&mut self, ui: &mut egui::Ui, nodes: &[CausalNode], index: usize, action: &mut Option<CausalViewAction>) {
        if index >= nodes.len() {
            return;
        }
        
        let node = &nodes[index];
        let indent = index * 24;
        
        ui.horizontal(|ui| {
            ui.add_space(indent as f32);
            
            // Expand/collapse button
            let has_children = index + 1 < nodes.len();
            let is_expanded = self.expanded_nodes.contains(&node.id);
            
            if has_children {
                let symbol = if is_expanded { "▼" } else { "▶" };
                if ui.add(egui::Label::new(RichText::new(symbol).size(10.0).color(TEXT_MUTED))
                    .sense(egui::Sense::click())).clicked() {
                    if is_expanded {
                        self.expanded_nodes.remove(&node.id);
                    } else {
                        self.expanded_nodes.insert(node.id.clone());
                    }
                }
            } else {
                ui.add_space(12.0);
            }
            
            // Node icon
            let icon = if node.is_root_cause {
                "🎯"
            } else {
                match node.relation_type {
                    CausalRelationType::DirectCause => "↑",
                    CausalRelationType::IndirectCause => "↗",
                    CausalRelationType::Contributing => "•",
                    CausalRelationType::Correlation => "~",
                }
            };
            
            let node_color = if node.is_root_cause {
                ERROR
            } else if self.highlight_root_causes && node.is_root_cause {
                ERROR
            } else {
                TEXT_PRIMARY
            };
            
            ui.label(RichText::new(icon).size(12.0));
            ui.add_space(4.0);
            
            // Node content
            egui::Frame::none()
                .fill(if node.is_root_cause { ERROR.gamma_multiply(0.15) } else { BG_WIDGET.gamma_multiply(0.5) })
                .rounding(Rounding::same(6.0))
                .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&node.label).size(12.0).color(node_color));
                        
                        if self.show_confidence {
                            ui.add_space(8.0);
                            ui.label(RichText::new(format!("{:.0}%", node.confidence * 100.0))
                                .size(10.0).color(TEXT_MUTED));
                        }
                    });
                });
            
            // Action button
            if ui.add(egui::Label::new(RichText::new("→").size(10.0).color(PRIMARY))
                .sense(egui::Sense::click())).clicked() {
                *action = Some(CausalViewAction::ExploreNode(node.id.clone()));
            }
        });
        
        // Render children
        if self.expanded_nodes.contains(&node.id) || index == 0 {
            self.render_tree_node(ui, nodes, index + 1, action);
        }
    }
    
    fn render_graph_view(&mut self, ui: &mut egui::Ui, _action: &mut Option<CausalViewAction>) {
        let available_size = ui.available_size();
        let graph_height = (available_size.y - 50.0).max(200.0);
        
        egui::Frame::none()
            .fill(BG_PANEL.gamma_multiply(0.8))
            .rounding(Rounding::same(8.0))
            .inner_margin(12.0)
            .show(ui, |ui| {
                let painter = ui.painter();
                let rect = ui.available_rect_before_wrap();
                
                if self.causal_chains.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("No causal data").size(13.0).color(TEXT_MUTED));
                    });
                    ui.allocate_space(Vec2::new(rect.width(), graph_height));
                    return;
                }
                
                // Collect unique nodes
                let mut all_nodes: Vec<&CausalNode> = Vec::new();
                for chain in &self.causal_chains {
                    for node in &chain.nodes {
                        if !all_nodes.iter().any(|n| n.id == node.id) {
                            all_nodes.push(node);
                        }
                    }
                }
                
                // Simple layout: arrange in a circle
                let center = rect.center();
                let radius = (rect.width().min(graph_height) / 2.0 - 40.0).max(50.0);
                
                // Draw edges first
                for chain in &self.causal_chains {
                    for pair in chain.nodes.windows(2) {
                        let from_idx = all_nodes.iter().position(|n| n.id == pair[0].id);
                        let to_idx = all_nodes.iter().position(|n| n.id == pair[1].id);
                        
                        if let (Some(fi), Some(ti)) = (from_idx, to_idx) {
                            let from_angle = (fi as f32) * std::f32::consts::TAU / (all_nodes.len() as f32);
                            let to_angle = (ti as f32) * std::f32::consts::TAU / (all_nodes.len() as f32);
                            
                            let from_pos = Pos2::new(
                                center.x + from_angle.cos() * radius,
                                center.y + from_angle.sin() * radius
                            );
                            let to_pos = Pos2::new(
                                center.x + to_angle.cos() * radius,
                                center.y + to_angle.sin() * radius
                            );
                            
                            // Draw arrow
                            painter.line_segment(
                                [from_pos, to_pos],
                                Stroke::new(2.0, ACCENT.gamma_multiply(0.5))
                            );
                            
                            // Arrowhead
                            let dir = (to_pos - from_pos).normalized();
                            let arrow_pos = to_pos - dir * 15.0;
                            let perp = egui::vec2(-dir.y, dir.x);
                            painter.add(egui::Shape::convex_polygon(
                                vec![
                                    to_pos,
                                    arrow_pos + perp * 5.0,
                                    arrow_pos - perp * 5.0,
                                ],
                                ACCENT.gamma_multiply(0.5),
                                Stroke::NONE
                            ));
                        }
                    }
                }
                
                // Draw nodes
                for (i, node) in all_nodes.iter().enumerate() {
                    let angle = (i as f32) * std::f32::consts::TAU / (all_nodes.len() as f32);
                    let pos = Pos2::new(
                        center.x + angle.cos() * radius,
                        center.y + angle.sin() * radius
                    );
                    
                    let node_radius = if node.is_root_cause { 18.0 } else { 14.0 };
                    let node_color = if node.is_root_cause { ERROR } else { SECONDARY };
                    
                    painter.circle_filled(pos, node_radius, node_color);
                    painter.circle_stroke(pos, node_radius, Stroke::new(2.0, Color32::WHITE.gamma_multiply(0.3)));
                    
                    // Label
                    let label = if node.label.len() > 15 {
                        format!("{}...", &node.label[..15])
                    } else {
                        node.label.clone()
                    };
                    
                    painter.text(
                        Pos2::new(pos.x, pos.y + node_radius + 12.0),
                        egui::Align2::CENTER_CENTER,
                        &label,
                        egui::FontId::proportional(10.0),
                        TEXT_SECONDARY
                    );
                }
                
                ui.allocate_space(Vec2::new(rect.width(), graph_height));
            });
    }
    
    fn render_root_causes(&mut self, ui: &mut egui::Ui, action: &mut Option<CausalViewAction>) {
        egui::Frame::none()
            .fill(ERROR.gamma_multiply(0.1))
            .stroke(Stroke::new(1.0, ERROR.gamma_multiply(0.3)))
            .rounding(Rounding::same(10.0))
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.label(RichText::new("🎯 Root Causes").size(13.0).color(ERROR).strong());
                ui.add_space(8.0);
                
                if self.root_causes.is_empty() {
                    ui.label(RichText::new("No root causes identified").size(11.0).color(TEXT_MUTED));
                } else {
                    for cause in &self.root_causes {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("•").color(ERROR));
                            ui.label(RichText::new(&cause.label).size(12.0).color(TEXT_PRIMARY));
                            
                            if self.show_confidence {
                                ui.label(RichText::new(format!("{:.0}%", cause.confidence * 100.0))
                                    .size(10.0).color(TEXT_MUTED));
                            }
                        });
                    }
                }
            });
    }
    
    fn render_impact_analysis(&mut self, ui: &mut egui::Ui, _action: &mut Option<CausalViewAction>) {
        egui::Frame::none()
            .fill(WARNING.gamma_multiply(0.1))
            .stroke(Stroke::new(1.0, WARNING.gamma_multiply(0.3)))
            .rounding(Rounding::same(10.0))
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.label(RichText::new("📊 Impact Analysis").size(13.0).color(WARNING).strong());
                ui.add_space(8.0);
                
                if self.impact_analysis.is_empty() {
                    ui.label(RichText::new("Run analysis to see impacts").size(11.0).color(TEXT_MUTED));
                } else {
                    for (node, score) in &self.impact_analysis {
                        ui.horizontal(|ui| {
                            // Impact bar
                            let bar_width = 60.0 * score;
                            let bar_color = if *score > 0.7 { ERROR } else if *score > 0.4 { WARNING } else { SUCCESS };
                            
                            egui::Frame::none()
                                .fill(bar_color.gamma_multiply(0.3))
                                .rounding(Rounding::same(3.0))
                                .show(ui, |ui| {
                                    ui.add_space(bar_width);
                                    ui.set_min_height(12.0);
                                });
                            
                            ui.add_space(8.0);
                            ui.label(RichText::new(&node.label).size(11.0).color(TEXT_SECONDARY));
                        });
                    }
                }
            });
    }
    
    /// Set analysis results
    pub fn set_results(&mut self, chains: Vec<CausalChain>, root_causes: Vec<CausalNode>) {
        self.causal_chains = chains;
        self.root_causes = root_causes;
        self.is_analyzing = false;
        self.error_message = None;
        
        // Calculate impact analysis (simplified)
        self.impact_analysis = self.causal_chains.iter()
            .flat_map(|c| c.nodes.iter())
            .map(|n| (n.clone(), n.confidence))
            .take(5)
            .collect();
        
        // Auto-select first chain
        if !self.causal_chains.is_empty() {
            self.selected_chain = Some(0);
        }
    }
    
    /// Analyze causal chains for an effect
    pub fn analyze(&mut self, storage: &sutra_storage::ConcurrentMemory, effect_id: sutra_storage::ConceptId, max_hops: usize) {
        self.is_analyzing = true;
        self.error_message = None;
        self.causal_chains.clear();
        self.root_causes.clear();
        
        // Build causal chain by traversing backwards from effect
        let mut visited = HashSet::new();
        let mut chain_nodes = Vec::new();
        let mut current_id = effect_id;
        
        // Get concept content for effect
        let snapshot = storage.get_snapshot();
        {
            if let Some(node) = snapshot.get_concept(&effect_id) {
                let content = String::from_utf8_lossy(&node.content).to_string();
                chain_nodes.push(CausalNode {
                    id: format!("{:?}", effect_id),
                    label: content.chars().take(50).collect::<String>(),
                    content: content,
                    confidence: 1.0,
                    relation_type: CausalRelationType::DirectCause,
                    is_root_cause: false,
                });
            }
            
            // Walk backwards through neighbors (simplified causal analysis)
            for hop in 0..max_hops {
                if visited.contains(&current_id) {
                    break;
                }
                visited.insert(current_id);
                
                let neighbors = storage.query_neighbors_weighted(&current_id);
                if neighbors.is_empty() {
                    // This is a root cause
                    if let Some(last) = chain_nodes.last_mut() {
                        last.is_root_cause = true;
                    }
                    break;
                }
                
                // Take strongest neighbor as cause (simplified)
                if let Some((neighbor_id, weight)) = neighbors.into_iter()
                    .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                {
                    if let Some(node) = snapshot.get_concept(&neighbor_id) {
                        let is_root = hop == max_hops - 1;
                        let relation = if hop == 0 {
                            CausalRelationType::DirectCause
                        } else if weight > 0.7 {
                            CausalRelationType::IndirectCause
                        } else {
                            CausalRelationType::Contributing
                        };
                        
                        let content = String::from_utf8_lossy(&node.content).to_string();
                        chain_nodes.push(CausalNode {
                            id: format!("{:?}", neighbor_id),
                            label: content.chars().take(50).collect::<String>(),
                            content: content,
                            confidence: weight,
                            relation_type: relation,
                            is_root_cause: is_root,
                        });
                        
                        current_id = neighbor_id;
                    }
                }
            }
            
            // Mark terminal node as root cause
            if chain_nodes.len() > 1 {
                if let Some(last) = chain_nodes.last_mut() {
                    last.is_root_cause = true;
                }
            }
            
            // Create chain
            if !chain_nodes.is_empty() {
                // Calculate overall confidence
                let avg_confidence = chain_nodes.iter()
                    .map(|n| n.confidence)
                    .sum::<f32>() / chain_nodes.len() as f32;
                    
                self.causal_chains.push(CausalChain {
                    nodes: chain_nodes.clone(),
                    confidence: avg_confidence,
                    depth: chain_nodes.len(),
                });
                
                // Extract root causes
                self.root_causes = chain_nodes.into_iter()
                    .filter(|n| n.is_root_cause)
                    .collect();
            }
        }
        
        self.is_analyzing = false;
        
        // Auto-select first chain
        if !self.causal_chains.is_empty() {
            self.selected_chain = Some(0);
        }
    }
    
    /// Analyze causal chains for an effect (Static version)
    pub fn analyze_static(storage: &sutra_storage::ConcurrentMemory, effect_id: sutra_storage::ConceptId, max_hops: usize) -> (Vec<CausalChain>, Vec<CausalNode>) {
        let mut causal_chains = Vec::new();
        let mut root_causes = Vec::new();
        
        // Build causal chain by traversing backwards from effect
        let mut visited = HashSet::new();
        let mut chain_nodes = Vec::new();
        let mut current_id = effect_id;
        
        // Get concept content for effect
        let snapshot = storage.get_snapshot();
        {
            if let Some(node) = snapshot.get_concept(&effect_id) {
                let content = String::from_utf8_lossy(&node.content).to_string();
                chain_nodes.push(CausalNode {
                    id: format!("{:?}", effect_id),
                    label: content.chars().take(50).collect::<String>(),
                    content: content,
                    confidence: 1.0,
                    relation_type: CausalRelationType::DirectCause,
                    is_root_cause: false,
                });
            }
            
            // Walk backwards through neighbors (simplified causal analysis)
            for hop in 0..max_hops {
                if visited.contains(&current_id) {
                    break;
                }
                visited.insert(current_id);
                
                let neighbors = storage.query_neighbors_weighted(&current_id);
                if neighbors.is_empty() {
                    // This is a root cause
                    if let Some(last) = chain_nodes.last_mut() {
                        last.is_root_cause = true;
                    }
                    break;
                }
                
                // Take strongest neighbor as cause (simplified)
                if let Some((neighbor_id, weight)) = neighbors.into_iter()
                    .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                {
                    if let Some(node) = snapshot.get_concept(&neighbor_id) {
                        let is_root = hop == max_hops - 1;
                        let relation = if hop == 0 {
                            CausalRelationType::DirectCause
                        } else if weight > 0.7 {
                            CausalRelationType::IndirectCause
                        } else {
                            CausalRelationType::Contributing
                        };
                        
                        let content = String::from_utf8_lossy(&node.content).to_string();
                        chain_nodes.push(CausalNode {
                            id: format!("{:?}", neighbor_id),
                            label: content.chars().take(50).collect::<String>(),
                            content: content,
                            confidence: weight,
                            relation_type: relation,
                            is_root_cause: is_root,
                        });
                        
                        current_id = neighbor_id;
                    }
                }
            }
            
            // Mark terminal node as root cause
            if chain_nodes.len() > 1 {
                if let Some(last) = chain_nodes.last_mut() {
                    last.is_root_cause = true;
                }
            }
            
            // Create chain
            if !chain_nodes.is_empty() {
                // Calculate overall confidence
                let avg_confidence = chain_nodes.iter()
                    .map(|n| n.confidence)
                    .sum::<f32>() / chain_nodes.len() as f32;
                    
                causal_chains.push(CausalChain {
                    nodes: chain_nodes.clone(),
                    confidence: avg_confidence,
                    depth: chain_nodes.len(),
                });
                
                // Extract root causes
                root_causes = chain_nodes.into_iter()
                    .filter(|n| n.is_root_cause)
                    .collect();
            }
        }
        
        (causal_chains, root_causes)
    }
    
    /// Set error message
    pub fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
        self.is_analyzing = false;
    }
}

/// Actions from causal view
#[derive(Debug, Clone)]
pub enum CausalViewAction {
    AnalyzeCause {
        effect: String,
        max_hops: usize,
    },
    ExploreNode(String),
    ExportChains,
}

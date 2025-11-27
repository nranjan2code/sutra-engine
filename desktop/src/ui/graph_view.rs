//! Graph Visualization Panel
//!
//! Interactive force-directed graph visualization of the knowledge graph.
//! Features: zoom/pan, node selection, edge coloring by type, filters.

use std::collections::HashMap;
use eframe::egui::{self, Color32, Pos2, Rect, RichText, Rounding, Sense, Stroke, Vec2};
use sutra_storage::{ConcurrentMemory, ConceptId};
use crate::theme::*;
use crate::types::{GraphNode, GraphEdge, EdgeType};

/// Camera for graph viewport
#[derive(Debug, Clone)]
pub struct Camera {
    pub offset: Vec2,
    pub zoom: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            offset: Vec2::ZERO,
            zoom: 1.0,
        }
    }
}

impl Camera {
    /// Transform world coordinates to screen coordinates
    pub fn world_to_screen(&self, world: Pos2, rect: Rect) -> Pos2 {
        let center = rect.center();
        Pos2::new(
            center.x + (world.x + self.offset.x) * self.zoom,
            center.y + (world.y + self.offset.y) * self.zoom,
        )
    }
    
    /// Transform screen coordinates to world coordinates
    pub fn screen_to_world(&self, screen: Pos2, rect: Rect) -> Pos2 {
        let center = rect.center();
        Pos2::new(
            (screen.x - center.x) / self.zoom - self.offset.x,
            (screen.y - center.y) / self.zoom - self.offset.y,
        )
    }
}

/// Graph filters
#[derive(Debug, Clone, Default)]
pub struct GraphFilters {
    pub min_confidence: f32,
    pub show_semantic: bool,
    pub show_causal: bool,
    pub show_temporal: bool,
    pub show_hierarchical: bool,
    pub show_similar: bool,
}

impl GraphFilters {
    pub fn new() -> Self {
        Self {
            min_confidence: 0.0,
            show_semantic: true,
            show_causal: true,
            show_temporal: true,
            show_hierarchical: true,
            show_similar: true,
        }
    }
}

/// Layout selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LayoutType {
    #[default]
    ForceDirected,
    Radial,
    Grid,
}

/// Graph Visualization Panel
pub struct GraphView {
    pub nodes: HashMap<ConceptId, GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub camera: Camera,
    pub selected: Option<ConceptId>,
    pub hovered: Option<ConceptId>,
    pub filters: GraphFilters,
    pub layout_type: LayoutType,
    pub search_query: String,
    
    // Force-directed layout parameters
    pub simulation_running: bool,
    repulsion: f32,
    attraction: f32,
    damping: f32,
    
    // Interaction state
    is_dragging: bool,
    drag_node: Option<ConceptId>,
    last_mouse_pos: Pos2,
}

impl Default for GraphView {
    fn default() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            camera: Camera::default(),
            selected: None,
            hovered: None,
            filters: GraphFilters::new(),
            layout_type: LayoutType::ForceDirected,
            search_query: String::new(),
            simulation_running: true,
            repulsion: 5000.0,
            attraction: 0.01,
            damping: 0.85,
            is_dragging: false,
            drag_node: None,
            last_mouse_pos: Pos2::ZERO,
        }
    }
}

impl GraphView {
    /// Load graph data from storage
    pub fn load_from_storage(&mut self, storage: &ConcurrentMemory) {
        self.nodes.clear();
        self.edges.clear();
        
        let snapshot = storage.get_snapshot();
        let concepts = snapshot.all_concepts();
        
        // Create nodes
        for concept in &concepts {
            let content = String::from_utf8_lossy(&concept.content).to_string();
            let mut node = GraphNode::new(
                concept.id,
                content,
                concept.confidence,
                concept.strength,
            );
            node.neighbor_count = concept.neighbors.len();
            self.nodes.insert(concept.id, node);
        }
        
        // Create edges
        for concept in &concepts {
            for neighbor_id in &concept.neighbors {
                // Only add edge once (from lower to higher id to avoid duplicates)
                if concept.id.to_hex() < neighbor_id.to_hex() {
                    self.edges.push(GraphEdge {
                        from: concept.id,
                        to: *neighbor_id,
                        strength: concept.strength,
                        edge_type: self.infer_edge_type(&concept.content),
                    });
                }
            }
        }
        
        self.simulation_running = true;
    }

    /// Load graph data from pre-computed nodes and edges
    pub fn load_from_data(&mut self, nodes: HashMap<ConceptId, GraphNode>, edges: Vec<GraphEdge>) {
        self.nodes = nodes;
        self.edges = edges;
        self.simulation_running = true;
    }
    
    /// Infer edge type from content (simple heuristic)
    fn infer_edge_type(&self, content: &[u8]) -> EdgeType {
        let text = String::from_utf8_lossy(content).to_lowercase();
        
        if text.contains("causes") || text.contains("leads to") || text.contains("results in") {
            EdgeType::Causal
        } else if text.contains("before") || text.contains("after") || text.contains("during") {
            EdgeType::Temporal
        } else if text.contains("is a") || text.contains("part of") || text.contains("contains") {
            EdgeType::Hierarchical
        } else if text.contains("similar") || text.contains("like") || text.contains("same") {
            EdgeType::Similar
        } else {
            EdgeType::Semantic
        }
    }
    
    /// Run one step of force-directed layout simulation
    pub fn simulate_step(&mut self) {
        if !self.simulation_running || self.nodes.len() < 2 {
            return;
        }
        
        let node_ids: Vec<ConceptId> = self.nodes.keys().cloned().collect();
        
        // Apply repulsion between all node pairs
        for i in 0..node_ids.len() {
            for j in (i + 1)..node_ids.len() {
                let id_a = node_ids[i];
                let id_b = node_ids[j];
                
                let (ax, ay, bx, by) = {
                    let a = &self.nodes[&id_a];
                    let b = &self.nodes[&id_b];
                    (a.x, a.y, b.x, b.y)
                };
                
                let dx = bx - ax;
                let dy = by - ay;
                let dist_sq = (dx * dx + dy * dy).max(1.0);
                let dist = dist_sq.sqrt();
                
                // Repulsion force (inverse square)
                let force = self.repulsion / dist_sq;
                let fx = (dx / dist) * force;
                let fy = (dy / dist) * force;
                
                if let Some(a) = self.nodes.get_mut(&id_a) {
                    a.vx -= fx;
                    a.vy -= fy;
                }
                if let Some(b) = self.nodes.get_mut(&id_b) {
                    b.vx += fx;
                    b.vy += fy;
                }
            }
        }
        
        // Apply attraction along edges
        for edge in &self.edges {
            let (ax, ay, bx, by) = {
                let a = match self.nodes.get(&edge.from) {
                    Some(n) => n,
                    None => continue,
                };
                let b = match self.nodes.get(&edge.to) {
                    Some(n) => n,
                    None => continue,
                };
                (a.x, a.y, b.x, b.y)
            };
            
            let dx = bx - ax;
            let dy = by - ay;
            let dist = (dx * dx + dy * dy).sqrt().max(1.0);
            
            // Attraction force (spring)
            let force = dist * self.attraction * edge.strength;
            let fx = (dx / dist) * force;
            let fy = (dy / dist) * force;
            
            if let Some(a) = self.nodes.get_mut(&edge.from) {
                a.vx += fx;
                a.vy += fy;
            }
            if let Some(b) = self.nodes.get_mut(&edge.to) {
                b.vx -= fx;
                b.vy -= fy;
            }
        }
        
        // Apply velocities with damping
        let mut total_movement = 0.0;
        for node in self.nodes.values_mut() {
            if self.drag_node == Some(node.id) {
                // Don't move dragged node
                node.vx = 0.0;
                node.vy = 0.0;
                continue;
            }
            
            node.vx *= self.damping;
            node.vy *= self.damping;
            
            node.x += node.vx;
            node.y += node.vy;
            
            total_movement += node.vx.abs() + node.vy.abs();
        }
        
        // Stop simulation when movement is minimal
        if total_movement < 0.5 {
            self.simulation_running = false;
        }
    }
    
    /// Render the graph visualization
    pub fn ui(&mut self, ui: &mut egui::Ui) -> Option<GraphAction> {
        let mut action = None;
        
        // Header
        self.render_header(ui, &mut action);
        
        ui.add_space(12.0);
        
        // Main graph area
        let available_rect = ui.available_rect_before_wrap();
        let graph_rect = Rect::from_min_size(
            available_rect.min,
            Vec2::new(available_rect.width(), available_rect.height() - 60.0),
        );
        
        // Background
        ui.painter().rect_filled(graph_rect, Rounding::same(12.0), BG_DARK);
        ui.painter().rect_stroke(graph_rect, Rounding::same(12.0), Stroke::new(1.0, BG_WIDGET));
        
        // Smart Empty State
        if self.nodes.is_empty() {
            ui.allocate_rect(graph_rect, Sense::hover());
            let center = graph_rect.center();
            
            ui.painter().text(
                center - Vec2::new(0.0, 30.0),
                egui::Align2::CENTER_CENTER,
                "🕸️ Knowledge Graph is Empty",
                egui::FontId::proportional(24.0),
                TEXT_MUTED,
            );
            
            ui.painter().text(
                center + Vec2::new(0.0, 10.0),
                egui::Align2::CENTER_CENTER,
                "Start by teaching Sutra some concepts.",
                egui::FontId::proportional(16.0),
                TEXT_SECONDARY,
            );
            
            ui.painter().text(
                center + Vec2::new(0.0, 40.0),
                egui::Align2::CENTER_CENTER,
                "Try: /learn Rust is a systems programming language",
                egui::FontId::monospace(12.0),
                ACCENT,
            );
            
            // Controls bar (still show it so user can see layout options etc)
            ui.add_space(8.0);
            self.render_controls(ui, &mut action);
            
            return action;
        }

        // Handle interactions
        let response = ui.allocate_rect(graph_rect, Sense::click_and_drag());
        
        // Zoom with scroll
        let scroll_delta = ui.input(|i| i.smooth_scroll_delta);
        if scroll_delta.y != 0.0 && graph_rect.contains(ui.input(|i| i.pointer.hover_pos().unwrap_or_default())) {
            let zoom_factor = 1.0 + scroll_delta.y * 0.001;
            self.camera.zoom = (self.camera.zoom * zoom_factor).clamp(0.1, 5.0);
        }
        
        // Pan with drag (when not dragging a node)
        if response.dragged() && self.drag_node.is_none() {
            let delta = response.drag_delta();
            self.camera.offset += delta / self.camera.zoom;
        }
        
        // Node dragging
        let mouse_pos = ui.input(|i| i.pointer.hover_pos().unwrap_or_default());
        let world_pos = self.camera.screen_to_world(mouse_pos, graph_rect);
        
        // Find hovered node
        self.hovered = None;
        for (id, node) in &self.nodes {
            let node_screen_pos = self.camera.world_to_screen(Pos2::new(node.x, node.y), graph_rect);
            let node_radius = self.node_radius(node);
            
            if (mouse_pos - node_screen_pos).length() < node_radius * self.camera.zoom {
                self.hovered = Some(*id);
                break;
            }
        }
        
        // Handle node interaction
        if response.drag_started() {
            if let Some(id) = self.hovered {
                self.drag_node = Some(id);
            }
        }
        
        if response.drag_stopped() {
            self.drag_node = None;
        }
        
        // Move dragged node
        if let Some(id) = self.drag_node {
            if let Some(node) = self.nodes.get_mut(&id) {
                node.x = world_pos.x;
                node.y = world_pos.y;
            }
        }
        
        // Select on click
        if response.clicked() {
            self.selected = self.hovered;
            if let Some(id) = self.selected {
                action = Some(GraphAction::SelectNode(id));
            }
        }
        
        // Run simulation
        if self.simulation_running {
            self.simulate_step();
            ui.ctx().request_repaint();
        }
        
        // Render edges
        for edge in &self.edges {
            if !self.should_show_edge(edge) {
                continue;
            }
            
            let from_node = match self.nodes.get(&edge.from) {
                Some(n) => n,
                None => continue,
            };
            let to_node = match self.nodes.get(&edge.to) {
                Some(n) => n,
                None => continue,
            };
            
            let from_pos = self.camera.world_to_screen(Pos2::new(from_node.x, from_node.y), graph_rect);
            let to_pos = self.camera.world_to_screen(Pos2::new(to_node.x, to_node.y), graph_rect);
            
            // Skip if outside view
            if !graph_rect.expand(50.0).contains(from_pos) && !graph_rect.expand(50.0).contains(to_pos) {
                continue;
            }
            
            let (r, g, b) = edge.edge_type.color();
            let alpha = (edge.strength * 200.0) as u8;
            let color = Color32::from_rgba_unmultiplied(r, g, b, alpha.max(80));
            
            // Highlight edges connected to selected node
            let is_highlighted = self.selected == Some(edge.from) || self.selected == Some(edge.to);
            let stroke_width = if is_highlighted { 3.0 } else { 1.5 };
            
            ui.painter().line_segment([from_pos, to_pos], Stroke::new(stroke_width, color));
        }
        
        // Render nodes
        for (id, node) in &self.nodes {
            if node.confidence < self.filters.min_confidence {
                continue;
            }
            
            let screen_pos = self.camera.world_to_screen(Pos2::new(node.x, node.y), graph_rect);
            
            // Skip if outside view
            if !graph_rect.expand(50.0).contains(screen_pos) {
                continue;
            }
            
            let radius = self.node_radius(node) * self.camera.zoom;
            let is_selected = self.selected == Some(*id);
            let is_hovered = self.hovered == Some(*id);
            
            // Node colors based on confidence
            let base_color = if node.confidence > 0.8 {
                PRIMARY
            } else if node.confidence > 0.5 {
                SECONDARY
            } else {
                TEXT_MUTED
            };
            
            // Draw outer glow for selected/hovered
            if is_selected {
                ui.painter().circle_filled(screen_pos, radius + 8.0, base_color.gamma_multiply(0.3));
                ui.painter().circle_filled(screen_pos, radius + 4.0, base_color.gamma_multiply(0.4));
            } else if is_hovered {
                ui.painter().circle_filled(screen_pos, radius + 4.0, base_color.gamma_multiply(0.2));
            }
            
            // Main node
            let fill = if is_selected {
                base_color
            } else if is_hovered {
                base_color.gamma_multiply(0.8)
            } else {
                base_color.gamma_multiply(0.6)
            };
            
            ui.painter().circle_filled(screen_pos, radius, fill);
            ui.painter().circle_stroke(screen_pos, radius, Stroke::new(2.0, base_color));
            
            // Inner highlight
            ui.painter().circle_filled(
                screen_pos - Vec2::new(radius * 0.3, radius * 0.3),
                radius * 0.3,
                Color32::from_white_alpha(40),
            );
            
            // Label (show only when zoomed in enough or selected/hovered)
            if self.camera.zoom > 0.5 || is_selected || is_hovered {
                let label = if node.label.len() > 20 && !is_selected && !is_hovered {
                    format!("{}...", &node.label[..20])
                } else {
                    node.label.clone()
                };
                
                ui.painter().text(
                    screen_pos + Vec2::new(0.0, radius + 10.0),
                    egui::Align2::CENTER_TOP,
                    &label,
                    egui::FontId::proportional(11.0 * self.camera.zoom.max(0.7)),
                    TEXT_PRIMARY,
                );
            }
        }
        
        // Legend
        self.render_legend(ui, graph_rect);
        
        // Controls bar
        ui.add_space(8.0);
        self.render_controls(ui, &mut action);
        
        // Selected node info panel
        if let Some(id) = self.selected {
            if let Some(node) = self.nodes.get(&id) {
                self.render_node_info(ui, node);
            }
        }
        
        action
    }
    
    fn render_header(&mut self, ui: &mut egui::Ui, action: &mut Option<GraphAction>) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("🧠").size(20.0));
            ui.add_space(4.0);
            ui.label(RichText::new("Knowledge Graph").size(22.0).color(TEXT_PRIMARY).strong());
            
            // Node count
            ui.add_space(12.0);
            egui::Frame::none()
                .fill(BG_WIDGET)
                .rounding(Rounding::same(10.0))
                .inner_margin(egui::Margin::symmetric(8.0, 2.0))
                .show(ui, |ui| {
                    ui.label(RichText::new(format!("{} nodes", self.nodes.len())).size(12.0).color(TEXT_SECONDARY));
                });
            
            // Edge count
            egui::Frame::none()
                .fill(BG_WIDGET)
                .rounding(Rounding::same(10.0))
                .inner_margin(egui::Margin::symmetric(8.0, 2.0))
                .show(ui, |ui| {
                    ui.label(RichText::new(format!("{} edges", self.edges.len())).size(12.0).color(TEXT_SECONDARY));
                });
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Refresh button
                if ui.button(RichText::new("↻ Refresh").size(13.0)).clicked() {
                    *action = Some(GraphAction::Refresh);
                }
                
                // Filter dropdown
                egui::ComboBox::from_id_salt("graph_filters")
                    .selected_text("Filters")
                    .width(80.0)
                    .show_ui(ui, |ui| {
                        ui.checkbox(&mut self.filters.show_semantic, "Semantic");
                        ui.checkbox(&mut self.filters.show_causal, "Causal");
                        ui.checkbox(&mut self.filters.show_temporal, "Temporal");
                        ui.checkbox(&mut self.filters.show_hierarchical, "Hierarchical");
                        ui.checkbox(&mut self.filters.show_similar, "Similar");
                        ui.add_space(8.0);
                        ui.add(egui::Slider::new(&mut self.filters.min_confidence, 0.0..=1.0)
                            .text("Min confidence"));
                    });
            });
        });
    }
    
    fn render_legend(&self, ui: &mut egui::Ui, rect: Rect) {
        let legend_rect = Rect::from_min_size(
            rect.min + Vec2::new(12.0, rect.height() - 80.0),
            Vec2::new(150.0, 70.0),
        );
        
        ui.painter().rect_filled(legend_rect, Rounding::same(8.0), BG_PANEL.gamma_multiply(0.9));
        
        let edge_types = [
            (EdgeType::Semantic, "Semantic"),
            (EdgeType::Causal, "Causal"),
            (EdgeType::Temporal, "Temporal"),
        ];
        
        for (i, (edge_type, name)) in edge_types.iter().enumerate() {
            let y = legend_rect.min.y + 12.0 + (i as f32 * 18.0);
            let (r, g, b) = edge_type.color();
            let color = Color32::from_rgb(r, g, b);
            
            ui.painter().line_segment(
                [Pos2::new(legend_rect.min.x + 10.0, y + 4.0), Pos2::new(legend_rect.min.x + 30.0, y + 4.0)],
                Stroke::new(2.0, color),
            );
            
            ui.painter().text(
                Pos2::new(legend_rect.min.x + 38.0, y),
                egui::Align2::LEFT_TOP,
                name,
                egui::FontId::proportional(11.0),
                TEXT_SECONDARY,
            );
        }
    }
    
    fn render_controls(&mut self, ui: &mut egui::Ui, action: &mut Option<GraphAction>) {
        ui.horizontal(|ui| {
            // Center button
            if ui.button(RichText::new("⌖ Center").size(13.0)).clicked() {
                self.camera.offset = Vec2::ZERO;
                self.camera.zoom = 1.0;
            }
            
            // Zoom controls
            ui.add_space(8.0);
            if ui.button(RichText::new("−").size(14.0)).clicked() {
                self.camera.zoom = (self.camera.zoom * 0.8).max(0.1);
            }
            ui.label(RichText::new(format!("{:.0}%", self.camera.zoom * 100.0)).size(12.0).color(TEXT_SECONDARY));
            if ui.button(RichText::new("+").size(14.0)).clicked() {
                self.camera.zoom = (self.camera.zoom * 1.25).min(5.0);
            }
            
            ui.add_space(16.0);
            
            // Layout selector
            ui.label(RichText::new("Layout:").size(12.0).color(TEXT_MUTED));
            egui::ComboBox::from_id_salt("layout_type")
                .selected_text(match self.layout_type {
                    LayoutType::ForceDirected => "Force",
                    LayoutType::Radial => "Radial",
                    LayoutType::Grid => "Grid",
                })
                .width(70.0)
                .show_ui(ui, |ui| {
                    if ui.selectable_label(self.layout_type == LayoutType::ForceDirected, "Force Directed").clicked() {
                        self.layout_type = LayoutType::ForceDirected;
                        self.simulation_running = true;
                    }
                    if ui.selectable_label(self.layout_type == LayoutType::Radial, "Radial").clicked() {
                        self.layout_type = LayoutType::Radial;
                        self.apply_radial_layout();
                    }
                    if ui.selectable_label(self.layout_type == LayoutType::Grid, "Grid").clicked() {
                        self.layout_type = LayoutType::Grid;
                        self.apply_grid_layout();
                    }
                });
            
            // Simulation toggle
            ui.add_space(16.0);
            let sim_text = if self.simulation_running { "⏸ Pause" } else { "▶ Run" };
            if ui.button(RichText::new(sim_text).size(13.0)).clicked() {
                self.simulation_running = !self.simulation_running;
            }
            
            // Selected node info
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(id) = self.selected {
                    if let Some(node) = self.nodes.get(&id) {
                        ui.label(RichText::new(format!("Selected: {} | Neighbors: {} | Confidence: {:.0}%",
                            &node.label[..node.label.len().min(30)],
                            node.neighbor_count,
                            node.confidence * 100.0
                        )).size(12.0).color(TEXT_SECONDARY));
                    }
                }
            });
        });
    }
    
    fn render_node_info(&self, ui: &mut egui::Ui, node: &GraphNode) {
        // This could be shown in a side panel or tooltip
        // For now, info is shown in the controls bar
    }
    
    fn node_radius(&self, node: &GraphNode) -> f32 {
        let base = 12.0;
        let confidence_bonus = node.confidence * 8.0;
        let neighbor_bonus = (node.neighbor_count as f32).sqrt() * 2.0;
        base + confidence_bonus + neighbor_bonus
    }
    
    fn should_show_edge(&self, edge: &GraphEdge) -> bool {
        match edge.edge_type {
            EdgeType::Semantic => self.filters.show_semantic,
            EdgeType::Causal => self.filters.show_causal,
            EdgeType::Temporal => self.filters.show_temporal,
            EdgeType::Hierarchical => self.filters.show_hierarchical,
            EdgeType::Similar => self.filters.show_similar,
        }
    }
    
    fn apply_radial_layout(&mut self) {
        let center = Pos2::ZERO;
        let node_ids: Vec<ConceptId> = self.nodes.keys().cloned().collect();
        let count = node_ids.len();
        
        if count == 0 {
            return;
        }
        
        let radius = 200.0 + (count as f32).sqrt() * 30.0;
        
        for (i, id) in node_ids.iter().enumerate() {
            let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
            if let Some(node) = self.nodes.get_mut(id) {
                node.x = center.x + angle.cos() * radius;
                node.y = center.y + angle.sin() * radius;
                node.vx = 0.0;
                node.vy = 0.0;
            }
        }
    }
    
    fn apply_grid_layout(&mut self) {
        let node_ids: Vec<ConceptId> = self.nodes.keys().cloned().collect();
        let count = node_ids.len();
        
        if count == 0 {
            return;
        }
        
        let cols = (count as f32).sqrt().ceil() as usize;
        let spacing = 100.0;
        let start_x = -((cols as f32 - 1.0) * spacing) / 2.0;
        let start_y = -((count as f32 / cols as f32).ceil() * spacing) / 2.0;
        
        for (i, id) in node_ids.iter().enumerate() {
            let col = i % cols;
            let row = i / cols;
            
            if let Some(node) = self.nodes.get_mut(id) {
                node.x = start_x + (col as f32 * spacing);
                node.y = start_y + (row as f32 * spacing);
                node.vx = 0.0;
                node.vy = 0.0;
            }
        }
    }
}

/// Actions from graph view
#[derive(Debug, Clone)]
pub enum GraphAction {
    SelectNode(ConceptId),
    Refresh,
    ExportImage,
}

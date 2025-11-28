//! Sidebar navigation - Premium design with animated elements

use eframe::egui::{self, Color32, RichText, Rounding, Sense, Stroke, Vec2};
use crate::theme::{PRIMARY, PRIMARY_DIM, PRIMARY_LIGHT, SECONDARY, ACCENT, SUCCESS, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_MUTED, BG_HOVER, BG_WIDGET, BG_SIDEBAR, BG_ELEVATED};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SidebarView {
    #[default]
    Home,
    Chat,
    Knowledge,
    Search,
    // Enhanced views
    Graph,
    Paths,
    Timeline,
    Causal,
    Analytics,
    Query,
    Export,
    // Settings
    Settings,
}

impl SidebarView {
    /// Get display info for the view
    pub fn info(&self) -> (&'static str, &'static str, &'static str) {
        match self {
            SidebarView::Home => ("🏠", "Home", "Dashboard overview"),
            SidebarView::Chat => ("💬", "Chat", "Have a conversation"),
            SidebarView::Knowledge => ("📚", "Knowledge", "Browse concepts"),
            SidebarView::Search => ("⚡", "Quick Learn", "Fast knowledge entry"),
            SidebarView::Graph => ("🕸️", "Graph View", "Visualize connections"),
            SidebarView::Paths => ("🛤️", "Reasoning", "Explore paths"),
            SidebarView::Timeline => ("⏱️", "Timeline", "Temporal analysis"),
            SidebarView::Causal => ("🔗", "Causality", "Root cause analysis"),
            SidebarView::Analytics => ("📊", "Analytics", "Performance metrics"),
            SidebarView::Query => ("🔎", "Query Builder", "Advanced search"),
            SidebarView::Export => ("📤", "Export/Import", "Data portability"),
            SidebarView::Settings => ("⚙️", "Settings", "Configure app"),
        }
    }
    
    /// Check if this is an analysis view (for grouping)
    pub fn is_analysis(&self) -> bool {
        matches!(self, SidebarView::Graph | SidebarView::Paths | SidebarView::Timeline | SidebarView::Causal)
    }
    
    /// Check if this is a tools view
    pub fn is_tools(&self) -> bool {
        matches!(self, SidebarView::Analytics | SidebarView::Query | SidebarView::Export)
    }
}

pub struct Sidebar {
    pub current_view: SidebarView,
    /// Track collapsed sections
    pub analysis_collapsed: bool,
    pub tools_collapsed: bool,
}

impl Default for Sidebar {
    fn default() -> Self {
        Self { 
            current_view: SidebarView::Chat,
            analysis_collapsed: false,
            tools_collapsed: false,
        }
    }
}

impl Sidebar {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let sidebar_width = ui.available_width();
        let sidebar_height = ui.available_height();
        
        // Sidebar background with subtle gradient effect
        let rect = ui.available_rect_before_wrap();
        ui.painter().rect_filled(rect, 0.0, BG_SIDEBAR);
        
        // Subtle right border
        ui.painter().line_segment(
            [rect.right_top(), rect.right_bottom()],
            Stroke::new(1.0, Color32::from_rgb(45, 45, 70))
        );
        
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(20.0);
                    
                    // Logo with glow effect
                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        self.draw_logo(ui);
                    });
                    
                    ui.add_space(4.0);
                    
                    // Subtitle with version badge
                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        ui.label(RichText::new("Desktop").size(12.0).color(TEXT_MUTED));
                        ui.add_space(6.0);
                        // Version badge - pill style
                        egui::Frame::none()
                            .fill(PRIMARY_DIM.gamma_multiply(0.2))
                            .rounding(Rounding::same(8.0))
                            .inner_margin(egui::Margin::symmetric(8.0, 3.0))
                            .show(ui, |ui| {
                                ui.label(RichText::new("v3.3").size(11.0).color(PRIMARY));
                            });
                    });
                    
                    ui.add_space(24.0);
                    
                    // ========================================
                    // MAIN section
                    // ========================================
                    self.section_header(ui, "MAIN");
                    ui.add_space(8.0);
                    
                    self.nav_item(ui, SidebarView::Home);
                    ui.add_space(2.0);
                    self.nav_item(ui, SidebarView::Chat);
                    ui.add_space(2.0);
                    self.nav_item(ui, SidebarView::Knowledge);
                    ui.add_space(2.0);
                    self.nav_item(ui, SidebarView::Search);
                    
                    ui.add_space(16.0);
                    
                    // ========================================
                    // ANALYSIS section (collapsible)
                    // ========================================
                    self.collapsible_section(ui, "ANALYSIS", &mut self.analysis_collapsed.clone(), |sidebar, ui| {
                        sidebar.nav_item(ui, SidebarView::Graph);
                        ui.add_space(2.0);
                        sidebar.nav_item(ui, SidebarView::Paths);
                        ui.add_space(2.0);
                        sidebar.nav_item(ui, SidebarView::Timeline);
                        ui.add_space(2.0);
                        sidebar.nav_item(ui, SidebarView::Causal);
                    });
                    
                    ui.add_space(16.0);
                    
                    // ========================================
                    // TOOLS section (collapsible)
                    // ========================================
                    self.collapsible_section(ui, "TOOLS", &mut self.tools_collapsed.clone(), |sidebar, ui| {
                        sidebar.nav_item(ui, SidebarView::Analytics);
                        ui.add_space(2.0);
                        sidebar.nav_item(ui, SidebarView::Query);
                        ui.add_space(2.0);
                        sidebar.nav_item(ui, SidebarView::Export);
                    });
                    
                    ui.add_space(24.0);
                    
                    // Divider before settings
                    self.draw_divider(ui, sidebar_width);
                    ui.add_space(12.0);
                    
                    // Settings always visible
                    self.nav_item(ui, SidebarView::Settings);
                    
                    ui.add_space(20.0);
                });
            });
    }
    
    fn section_header(&self, ui: &mut egui::Ui, label: &str) {
        ui.horizontal(|ui| {
            ui.add_space(16.0);
            ui.label(RichText::new(label).size(11.0).color(TEXT_MUTED).strong());
        });
    }
    
    fn collapsible_section<F>(&mut self, ui: &mut egui::Ui, label: &str, collapsed: &mut bool, content: F)
    where
        F: FnOnce(&mut Self, &mut egui::Ui),
    {
        // Clone the collapsed state to avoid borrow issues
        let is_collapsed = *collapsed;
        
        ui.horizontal(|ui| {
            ui.add_space(16.0);
            
            // Collapse/expand button
            let symbol = if is_collapsed { "▶" } else { "▼" };
            let response = ui.add(
                egui::Label::new(RichText::new(symbol).size(10.0).color(TEXT_MUTED))
                    .sense(Sense::click())
            );
            
            ui.add_space(4.0);
            
            let label_response = ui.add(
                egui::Label::new(RichText::new(label).size(11.0).color(TEXT_MUTED).strong())
                    .sense(Sense::click())
            );
            
            if response.clicked() || label_response.clicked() {
                // Toggle the actual field
                if label == "ANALYSIS" {
                    self.analysis_collapsed = !self.analysis_collapsed;
                } else if label == "TOOLS" {
                    self.tools_collapsed = !self.tools_collapsed;
                }
            }
        });
        
        if !is_collapsed {
            ui.add_space(8.0);
            content(self, ui);
        }
    }
    
    fn draw_logo(&self, ui: &mut egui::Ui) {
        let (rect, _) = ui.allocate_exact_size(Vec2::new(160.0, 40.0), Sense::hover());
        
        // Logo icon - neural/brain symbol
        let icon_center = rect.min + Vec2::new(18.0, 20.0);
        let icon_radius = 14.0;
        
        // Outer glow (larger, softer)
        ui.painter().circle_filled(icon_center, icon_radius + 6.0, PRIMARY.gamma_multiply(0.08));
        ui.painter().circle_filled(icon_center, icon_radius + 3.0, PRIMARY.gamma_multiply(0.12));
        
        // Main gradient circle
        ui.painter().circle_filled(icon_center, icon_radius, PRIMARY_DIM.gamma_multiply(0.4));
        ui.painter().circle_stroke(icon_center, icon_radius, Stroke::new(2.0, PRIMARY));
        
        // Inner neural pattern - 6 points hexagon
        let inner = icon_radius * 0.55;
        for i in 0..6 {
            let angle = std::f32::consts::PI * 2.0 / 6.0 * i as f32 - std::f32::consts::PI / 2.0;
            let point = icon_center + Vec2::new(angle.cos() * inner, angle.sin() * inner);
            ui.painter().circle_filled(point, 1.8, PRIMARY.gamma_multiply(0.8));
            // Connect to center
            ui.painter().line_segment([icon_center, point], Stroke::new(0.8, PRIMARY.gamma_multiply(0.4)));
        }
        // Bright center
        ui.painter().circle_filled(icon_center, 3.5, Color32::WHITE);
        ui.painter().circle_filled(icon_center, 2.5, PRIMARY);
        
        // Brand text - Sutra
        ui.painter().text(
            rect.min + Vec2::new(40.0, 8.0),
            egui::Align2::LEFT_TOP,
            "Sutra",
            egui::FontId::proportional(24.0),
            TEXT_PRIMARY,
        );
        
        // AI badge with background
        let badge_pos = rect.min + Vec2::new(108.0, 10.0);
        ui.painter().rect_filled(
            egui::Rect::from_min_size(badge_pos - Vec2::new(2.0, 1.0), Vec2::new(22.0, 16.0)),
            Rounding::same(4.0),
            SECONDARY.gamma_multiply(0.2)
        );
        ui.painter().text(
            badge_pos,
            egui::Align2::LEFT_TOP,
            "AI",
            egui::FontId::proportional(11.0),
            SECONDARY,
        );
    }
    
    fn draw_divider(&self, ui: &mut egui::Ui, width: f32) {
        let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 1.0), Sense::hover());
        let start = rect.min + Vec2::new(16.0, 0.0);
        let end = rect.max - Vec2::new(16.0, 0.0);
        ui.painter().line_segment([start, end], Stroke::new(1.0, BG_WIDGET));
    }
    
    fn nav_item(&mut self, ui: &mut egui::Ui, view: SidebarView) {
        let (icon, label, hint) = view.info();
        let is_selected = self.current_view == view;
        
        let margin_h = 12.0;
        let item_width = ui.available_width() - margin_h * 2.0;
        let item_height = 56.0; // Slightly taller for better touch targets
        
        ui.horizontal(|ui| {
            ui.add_space(margin_h);
            
            let (rect, response) = ui.allocate_exact_size(Vec2::new(item_width, item_height), Sense::click());
            let is_hovered = response.hovered();
            
            // Background with smooth transition
            let bg_color = if is_selected {
                PRIMARY.gamma_multiply(0.20)
            } else if is_hovered {
                BG_ELEVATED.gamma_multiply(1.1)
            } else {
                Color32::TRANSPARENT
            };
            
            // Border for selected/hovered - more prominent
            let border_width = if is_selected { 1.5 } else if is_hovered { 1.0 } else { 0.0 };
            let border_color = if is_selected {
                PRIMARY.gamma_multiply(0.5)
            } else if is_hovered {
                Color32::from_rgb(60, 60, 90)
            } else {
                Color32::TRANSPARENT
            };
            
            // Draw background with border
            ui.painter().rect(
                rect,
                Rounding::same(12.0),
                bg_color,
                Stroke::new(border_width, border_color)
            );
            
            // Left accent bar for selected - thicker and more prominent
            if is_selected {
                let indicator = egui::Rect::from_min_size(
                    rect.min + Vec2::new(0.0, 12.0),
                    Vec2::new(4.0, rect.height() - 24.0)
                );
                ui.painter().rect_filled(indicator, Rounding::same(2.0), PRIMARY);
                
                // Add subtle glow effect
                let glow_indicator = egui::Rect::from_min_size(
                    rect.min + Vec2::new(-1.0, 12.0),
                    Vec2::new(6.0, rect.height() - 24.0)
                );
                ui.painter().rect_filled(glow_indicator, Rounding::same(3.0), PRIMARY.gamma_multiply(0.15));
            }
            
            // Icon with background circle - enhanced
            let icon_pos = rect.min + Vec2::new(18.0, (item_height - 32.0) / 2.0);
            let icon_bg_color = if is_selected {
                PRIMARY.gamma_multiply(0.30)
            } else if is_hovered {
                BG_WIDGET.gamma_multiply(1.2)
            } else {
                BG_WIDGET
            };
            
            // Icon background with subtle shadow for selected
            if is_selected {
                ui.painter().rect_filled(
                    egui::Rect::from_min_size(icon_pos - Vec2::splat(1.0), Vec2::splat(34.0)),
                    Rounding::same(10.0),
                    PRIMARY.gamma_multiply(0.10)
                );
            }
            
            ui.painter().rect_filled(
                egui::Rect::from_min_size(icon_pos, Vec2::splat(32.0)),
                Rounding::same(9.0),
                icon_bg_color
            );
            
            // Icon with better contrast
            let icon_color = if is_selected { 
                PRIMARY_LIGHT 
            } else if is_hovered { 
                TEXT_PRIMARY 
            } else { 
                TEXT_SECONDARY 
            };
            
            ui.painter().text(
                icon_pos + Vec2::new(16.0, 16.0),
                egui::Align2::CENTER_CENTER,
                icon,
                egui::FontId::proportional(16.0),
                icon_color,
            );
            
            // Label with better typography
            let text_color = if is_selected { 
                TEXT_PRIMARY 
            } else if is_hovered { 
                TEXT_PRIMARY 
            } else { 
                TEXT_SECONDARY 
            };
            
            ui.painter().text(
                rect.min + Vec2::new(58.0, 12.0),
                egui::Align2::LEFT_TOP,
                label,
                egui::FontId::proportional(14.0),
                text_color,
            );
            
            // Hint with better visibility
            let hint_color = if is_selected || is_hovered {
                TEXT_SECONDARY
            } else {
                TEXT_MUTED
            };
            
            ui.painter().text(
                rect.min + Vec2::new(58.0, 32.0),
                egui::Align2::LEFT_TOP,
                hint,
                egui::FontId::proportional(11.5),
                hint_color,
            );
            
            if response.clicked() {
                self.current_view = view;
            }
            
            // Tooltip on hover for additional context
            if is_hovered {
                response.on_hover_text_at_pointer(format!("{} - {}", label, hint));
            }
        });
    }
}

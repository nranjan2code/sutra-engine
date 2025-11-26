//! Sidebar navigation - Premium design with animated elements

use eframe::egui::{self, Color32, RichText, Rounding, Sense, Stroke, Vec2};
use crate::theme::{PRIMARY, PRIMARY_DIM, SECONDARY, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_MUTED, BG_HOVER, BG_WIDGET, BG_SIDEBAR, BG_ELEVATED};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SidebarView {
    #[default]
    Chat,
    Knowledge,
    Search,
    Settings,
}

pub struct Sidebar {
    pub current_view: SidebarView,
}

impl Default for Sidebar {
    fn default() -> Self {
        Self { current_view: SidebarView::Chat }
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
                ui.label(RichText::new("Desktop").size(11.0).color(TEXT_MUTED));
                ui.add_space(6.0);
                // Version badge - pill style
                egui::Frame::none()
                    .fill(PRIMARY_DIM.gamma_multiply(0.2))
                    .rounding(Rounding::same(8.0))
                    .inner_margin(egui::Margin::symmetric(8.0, 3.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new("v3.3").size(10.0).color(PRIMARY));
                    });
            });
            
            ui.add_space(24.0);
            
            // Section label with line
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.label(RichText::new("MENU").size(10.0).color(TEXT_MUTED).strong());
            });
            ui.add_space(12.0);
            
            // Main navigation items
            self.nav_item(ui, "💬", "Chat", "Have a conversation", SidebarView::Chat);
            ui.add_space(2.0);
            self.nav_item(ui, "📚", "Knowledge", "Browse stored concepts", SidebarView::Knowledge);
            ui.add_space(2.0);
            self.nav_item(ui, "🔍", "Search", "Find information", SidebarView::Search);
            
            // Spacer to push settings to bottom
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.add_space(20.0);
                
                // Settings at bottom
                self.nav_item(ui, "⚙️", "Settings", "Configure app", SidebarView::Settings);
                
                ui.add_space(12.0);
                
                // Bottom divider
                self.draw_divider(ui, sidebar_width);
            });
        });
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
    
    fn nav_item(&mut self, ui: &mut egui::Ui, icon: &str, label: &str, hint: &str, view: SidebarView) {
        let is_selected = self.current_view == view;
        
        let margin_h = 12.0;
        let item_width = ui.available_width() - margin_h * 2.0;
        let item_height = 52.0;
        
        ui.horizontal(|ui| {
            ui.add_space(margin_h);
            
            let (rect, response) = ui.allocate_exact_size(Vec2::new(item_width, item_height), Sense::click());
            let is_hovered = response.hovered();
            
            // Background - selected has gradient-like fill
            let bg_color = if is_selected {
                PRIMARY.gamma_multiply(0.18)
            } else if is_hovered {
                BG_ELEVATED
            } else {
                Color32::TRANSPARENT
            };
            
            // Border for selected/hovered
            let border_color = if is_selected {
                PRIMARY.gamma_multiply(0.4)
            } else if is_hovered {
                Color32::from_rgb(55, 55, 80)
            } else {
                Color32::TRANSPARENT
            };
            
            // Draw background with border
            ui.painter().rect(
                rect,
                Rounding::same(12.0),
                bg_color,
                Stroke::new(1.0, border_color)
            );
            
            // Left accent bar for selected
            if is_selected {
                let indicator = egui::Rect::from_min_size(
                    rect.min + Vec2::new(0.0, 10.0),
                    Vec2::new(3.0, rect.height() - 20.0)
                );
                ui.painter().rect_filled(indicator, Rounding::same(2.0), PRIMARY);
            }
            
            // Icon with background circle
            let icon_pos = rect.min + Vec2::new(16.0, (item_height - 28.0) / 2.0);
            let icon_bg_color = if is_selected {
                PRIMARY.gamma_multiply(0.25)
            } else {
                BG_WIDGET
            };
            ui.painter().rect_filled(
                egui::Rect::from_min_size(icon_pos, Vec2::splat(28.0)),
                Rounding::same(8.0),
                icon_bg_color
            );
            ui.painter().text(
                icon_pos + Vec2::new(14.0, 14.0),
                egui::Align2::CENTER_CENTER,
                icon,
                egui::FontId::proportional(15.0),
                if is_selected { PRIMARY } else { TEXT_SECONDARY },
            );
            
            // Label and hint
            let text_color = if is_selected { TEXT_PRIMARY } else { TEXT_SECONDARY };
            ui.painter().text(
                rect.min + Vec2::new(52.0, 11.0),
                egui::Align2::LEFT_TOP,
                label,
                egui::FontId::proportional(14.0),
                text_color,
            );
            
            // Always show hint for clarity
            ui.painter().text(
                rect.min + Vec2::new(52.0, 29.0),
                egui::Align2::LEFT_TOP,
                hint,
                egui::FontId::proportional(11.0),
                TEXT_MUTED,
            );
            
            if response.clicked() {
                self.current_view = view;
            }
        });
    }
}

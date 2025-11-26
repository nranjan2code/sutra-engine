//! Modern dark theme for Sutra Desktop
//! Premium design with rich purple accent

use eframe::egui::{self, Color32, Frame, Margin, Rounding, Stroke, Visuals};

// Primary palette - Vibrant and modern
pub const PRIMARY: Color32 = Color32::from_rgb(167, 139, 250);    // Vibrant Purple
pub const PRIMARY_DIM: Color32 = Color32::from_rgb(139, 92, 246); // Deep Purple
pub const SECONDARY: Color32 = Color32::from_rgb(96, 165, 250);   // Sky Blue
pub const ACCENT: Color32 = Color32::from_rgb(251, 191, 36);      // Amber/Gold
pub const SUCCESS: Color32 = Color32::from_rgb(52, 211, 153);     // Emerald
pub const WARNING: Color32 = Color32::from_rgb(251, 146, 60);     // Orange
pub const ERROR: Color32 = Color32::from_rgb(248, 113, 113);      // Red

// Background colors - Deep, rich tones
pub const BG_DARK: Color32 = Color32::from_rgb(15, 15, 25);       // Darkest
pub const BG_PANEL: Color32 = Color32::from_rgb(22, 22, 35);      // Panels
pub const BG_SIDEBAR: Color32 = Color32::from_rgb(18, 18, 30);    // Sidebar
pub const BG_WIDGET: Color32 = Color32::from_rgb(35, 35, 55);     // Inputs/cards
pub const BG_HOVER: Color32 = Color32::from_rgb(45, 45, 70);      // Hover state
pub const BG_ELEVATED: Color32 = Color32::from_rgb(40, 40, 62);   // Elevated cards

// Text colors - High contrast
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(248, 250, 252);   // Almost white
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(148, 163, 184); // Slate
pub const TEXT_MUTED: Color32 = Color32::from_rgb(100, 116, 139);     // Dimmed

/// Apply custom theme to egui context
pub fn setup_custom_theme(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();
    
    // Window - subtle border for depth
    visuals.window_fill = BG_PANEL;
    visuals.window_stroke = Stroke::new(1.0, Color32::from_rgb(50, 50, 75));
    visuals.window_shadow = egui::Shadow::NONE;
    visuals.window_rounding = Rounding::same(16.0);
    
    // Panel - clean background
    visuals.panel_fill = BG_PANEL;
    
    // Widgets - polished with subtle borders
    visuals.widgets.noninteractive.bg_fill = BG_WIDGET;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_SECONDARY);
    visuals.widgets.noninteractive.rounding = Rounding::same(10.0);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(55, 55, 80));
    
    visuals.widgets.inactive.bg_fill = BG_WIDGET;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.inactive.rounding = Rounding::same(10.0);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(55, 55, 80));
    
    visuals.widgets.hovered.bg_fill = BG_HOVER;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.hovered.rounding = Rounding::same(10.0);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, PRIMARY.gamma_multiply(0.4));
    
    visuals.widgets.active.bg_fill = PRIMARY.gamma_multiply(0.25);
    visuals.widgets.active.fg_stroke = Stroke::new(1.5, PRIMARY);
    visuals.widgets.active.rounding = Rounding::same(10.0);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, PRIMARY.gamma_multiply(0.6));
    
    // Selection - vibrant highlight
    visuals.selection.bg_fill = PRIMARY.gamma_multiply(0.25);
    visuals.selection.stroke = Stroke::new(1.0, PRIMARY);
    
    // Extreme background
    visuals.extreme_bg_color = BG_DARK;
    
    // Hyperlinks - use secondary color
    visuals.hyperlink_color = SECONDARY;
    
    // Text cursor - prominent
    visuals.text_cursor.stroke = Stroke::new(2.0, PRIMARY);
    
    ctx.set_visuals(visuals);
    
    // Set default font sizes
    let mut style = (*ctx.style()).clone();
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::proportional(14.0),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::proportional(14.0),
    );
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::proportional(20.0),
    );
    style.text_styles.insert(
        egui::TextStyle::Monospace,
        egui::FontId::monospace(13.0),
    );
    ctx.set_style(style);
}

/// Create a styled card frame
pub fn card_frame() -> Frame {
    Frame::none()
        .fill(BG_ELEVATED)
        .inner_margin(Margin::same(16.0))
        .rounding(Rounding::same(12.0))
        .stroke(Stroke::new(1.0, Color32::from_rgb(55, 55, 80)))
}

/// Create an elevated card with more prominence
pub fn elevated_card() -> Frame {
    Frame::none()
        .fill(BG_WIDGET)
        .inner_margin(Margin::same(20.0))
        .rounding(Rounding::same(14.0))
        .stroke(Stroke::new(1.0, Color32::from_rgb(60, 60, 90)))
}

/// Create a subtle section frame
pub fn section_frame() -> Frame {
    Frame::none()
        .fill(Color32::TRANSPARENT)
        .inner_margin(Margin::same(0.0))
        .rounding(Rounding::same(0.0))
}

/// Create a styled button frame
pub fn button_frame(selected: bool) -> Frame {
    Frame::none()
        .fill(if selected { PRIMARY.gamma_multiply(0.25) } else { BG_WIDGET })
        .inner_margin(Margin::symmetric(16.0, 10.0))
        .rounding(Rounding::same(10.0))
        .stroke(Stroke::new(1.0, if selected { PRIMARY.gamma_multiply(0.5) } else { Color32::from_rgb(55, 55, 80) }))
}

/// Premium gradient-like highlight color
pub fn highlight_color(base: Color32, intensity: f32) -> Color32 {
    base.gamma_multiply(intensity)
}

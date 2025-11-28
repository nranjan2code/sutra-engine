//! Modern dark theme for Sutra Desktop
//! Premium design with rich purple accent
//! Supports multiple theme variants including high contrast

use eframe::egui::{self, Color32, Frame, Margin, Rounding, Stroke, Visuals};
use std::sync::atomic::{AtomicU8, Ordering};

// ============================================================================
// Theme Selection
// ============================================================================

/// Current theme mode (stored atomically for thread safety)
static CURRENT_THEME: AtomicU8 = AtomicU8::new(0); // 0 = Dark, 1 = Light, 2 = HighContrast

/// Available theme modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
    HighContrast,
}

impl ThemeMode {
    pub fn name(&self) -> &'static str {
        match self {
            ThemeMode::Dark => "Dark",
            ThemeMode::Light => "Light", 
            ThemeMode::HighContrast => "High Contrast",
        }
    }
    
    pub fn description(&self) -> &'static str {
        match self {
            ThemeMode::Dark => "Default dark theme with purple accents",
            ThemeMode::Light => "Light theme for bright environments",
            ThemeMode::HighContrast => "Maximum contrast for accessibility",
        }
    }
}

/// Get current theme mode
pub fn current_theme() -> ThemeMode {
    match CURRENT_THEME.load(Ordering::Relaxed) {
        1 => ThemeMode::Light,
        2 => ThemeMode::HighContrast,
        _ => ThemeMode::Dark,
    }
}

/// Set theme mode
pub fn set_theme(mode: ThemeMode) {
    let value = match mode {
        ThemeMode::Dark => 0,
        ThemeMode::Light => 1,
        ThemeMode::HighContrast => 2,
    };
    CURRENT_THEME.store(value, Ordering::Relaxed);
}

// ============================================================================
// Dark Theme Colors (Default)
// ============================================================================

// Primary palette - Vibrant and modern
pub const PRIMARY: Color32 = Color32::from_rgb(167, 139, 250);    // Vibrant Purple
pub const PRIMARY_DIM: Color32 = Color32::from_rgb(139, 92, 246); // Deep Purple
pub const PRIMARY_LIGHT: Color32 = Color32::from_rgb(196, 181, 253); // Light Purple
pub const SECONDARY: Color32 = Color32::from_rgb(96, 165, 250);   // Sky Blue
pub const ACCENT: Color32 = Color32::from_rgb(251, 191, 36);      // Amber/Gold
pub const SUCCESS: Color32 = Color32::from_rgb(52, 211, 153);     // Emerald
pub const WARNING: Color32 = Color32::from_rgb(251, 146, 60);     // Orange
pub const ERROR: Color32 = Color32::from_rgb(248, 113, 113);      // Red
pub const INFO: Color32 = Color32::from_rgb(96, 165, 250);        // Blue

// Background colors - Deep, rich tones
pub const BG_DARK: Color32 = Color32::from_rgb(15, 15, 25);       // Darkest
pub const BG_PANEL: Color32 = Color32::from_rgb(22, 22, 35);      // Panels
pub const BG_SIDEBAR: Color32 = Color32::from_rgb(18, 18, 30);    // Sidebar
pub const BG_WIDGET: Color32 = Color32::from_rgb(35, 35, 55);     // Inputs/cards
pub const BG_HOVER: Color32 = Color32::from_rgb(45, 45, 70);      // Hover state
pub const BG_ELEVATED: Color32 = Color32::from_rgb(40, 40, 62);   // Elevated cards

// Text colors - High contrast (WCAG AA Compliant)
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(248, 250, 252);   // Almost white (~15:1 contrast)
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(160, 174, 192); // Lighter Slate (~7:1 contrast)
pub const TEXT_MUTED: Color32 = Color32::from_rgb(125, 140, 165);     // Lightened Dimmed (~5:1 contrast)

// ============================================================================
// Light Theme Colors
// ============================================================================

pub mod light {
    use super::*;
    
    pub const PRIMARY: Color32 = Color32::from_rgb(124, 58, 237);     // Deeper purple for light bg
    pub const PRIMARY_DIM: Color32 = Color32::from_rgb(109, 40, 217);
    pub const PRIMARY_LIGHT: Color32 = Color32::from_rgb(167, 139, 250);
    pub const SECONDARY: Color32 = Color32::from_rgb(59, 130, 246);
    pub const ACCENT: Color32 = Color32::from_rgb(217, 119, 6);       // Darker amber
    pub const SUCCESS: Color32 = Color32::from_rgb(16, 185, 129);
    pub const WARNING: Color32 = Color32::from_rgb(245, 158, 11);
    pub const ERROR: Color32 = Color32::from_rgb(239, 68, 68);
    pub const INFO: Color32 = Color32::from_rgb(59, 130, 246);
    
    pub const BG_DARK: Color32 = Color32::from_rgb(249, 250, 251);    // Lightest
    pub const BG_PANEL: Color32 = Color32::from_rgb(255, 255, 255);   // White
    pub const BG_SIDEBAR: Color32 = Color32::from_rgb(243, 244, 246); // Light gray
    pub const BG_WIDGET: Color32 = Color32::from_rgb(229, 231, 235);  // Input bg
    pub const BG_HOVER: Color32 = Color32::from_rgb(209, 213, 219);   // Hover
    pub const BG_ELEVATED: Color32 = Color32::from_rgb(255, 255, 255);// Cards
    
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(17, 24, 39);  // Near black
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(75, 85, 99);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(107, 114, 128);
}

// ============================================================================
// High Contrast Theme Colors (WCAG AAA Compliant)
// ============================================================================

pub mod high_contrast {
    use super::*;
    
    // Maximum contrast colors
    pub const PRIMARY: Color32 = Color32::from_rgb(255, 255, 0);      // Yellow on black
    pub const PRIMARY_DIM: Color32 = Color32::from_rgb(255, 215, 0);  // Gold
    pub const PRIMARY_LIGHT: Color32 = Color32::from_rgb(255, 255, 150);
    pub const SECONDARY: Color32 = Color32::from_rgb(0, 255, 255);    // Cyan
    pub const ACCENT: Color32 = Color32::from_rgb(255, 165, 0);       // Orange
    pub const SUCCESS: Color32 = Color32::from_rgb(0, 255, 0);        // Bright green
    pub const WARNING: Color32 = Color32::from_rgb(255, 165, 0);      // Orange
    pub const ERROR: Color32 = Color32::from_rgb(255, 0, 0);          // Red
    pub const INFO: Color32 = Color32::from_rgb(0, 191, 255);         // Deep sky blue
    
    // Pure black backgrounds for maximum contrast
    pub const BG_DARK: Color32 = Color32::from_rgb(0, 0, 0);
    pub const BG_PANEL: Color32 = Color32::from_rgb(0, 0, 0);
    pub const BG_SIDEBAR: Color32 = Color32::from_rgb(0, 0, 0);
    pub const BG_WIDGET: Color32 = Color32::from_rgb(20, 20, 20);
    pub const BG_HOVER: Color32 = Color32::from_rgb(40, 40, 40);
    pub const BG_ELEVATED: Color32 = Color32::from_rgb(30, 30, 30);
    
    // Pure white text for maximum contrast
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(255, 255, 255);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(255, 255, 255);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(200, 200, 200);
}

// ============================================================================
// Theme-aware color getters
// ============================================================================

/// Get primary color for current theme
pub fn primary() -> Color32 {
    match current_theme() {
        ThemeMode::Dark => PRIMARY,
        ThemeMode::Light => light::PRIMARY,
        ThemeMode::HighContrast => high_contrast::PRIMARY,
    }
}

/// Get background panel color for current theme
pub fn bg_panel() -> Color32 {
    match current_theme() {
        ThemeMode::Dark => BG_PANEL,
        ThemeMode::Light => light::BG_PANEL,
        ThemeMode::HighContrast => high_contrast::BG_PANEL,
    }
}

/// Get text primary color for current theme
pub fn text_primary() -> Color32 {
    match current_theme() {
        ThemeMode::Dark => TEXT_PRIMARY,
        ThemeMode::Light => light::TEXT_PRIMARY,
        ThemeMode::HighContrast => high_contrast::TEXT_PRIMARY,
    }
}

/// Get text secondary color for current theme
pub fn text_secondary() -> Color32 {
    match current_theme() {
        ThemeMode::Dark => TEXT_SECONDARY,
        ThemeMode::Light => light::TEXT_SECONDARY,
        ThemeMode::HighContrast => high_contrast::TEXT_SECONDARY,
    }
}

/// Apply custom theme to egui context (initial setup)
pub fn setup_custom_theme(ctx: &egui::Context) {
    apply_theme(ctx, ThemeMode::Dark);
}

/// Apply a specific theme to the context
pub fn apply_theme(ctx: &egui::Context, mode: ThemeMode) {
    set_theme(mode);
    
    let (bg_panel, bg_widget, bg_hover, bg_dark, text_primary, text_secondary, primary, secondary) = match mode {
        ThemeMode::Dark => (
            BG_PANEL, BG_WIDGET, BG_HOVER, BG_DARK, 
            TEXT_PRIMARY, TEXT_SECONDARY, PRIMARY, SECONDARY
        ),
        ThemeMode::Light => (
            light::BG_PANEL, light::BG_WIDGET, light::BG_HOVER, light::BG_DARK,
            light::TEXT_PRIMARY, light::TEXT_SECONDARY, light::PRIMARY, light::SECONDARY
        ),
        ThemeMode::HighContrast => (
            high_contrast::BG_PANEL, high_contrast::BG_WIDGET, high_contrast::BG_HOVER, high_contrast::BG_DARK,
            high_contrast::TEXT_PRIMARY, high_contrast::TEXT_SECONDARY, high_contrast::PRIMARY, high_contrast::SECONDARY
        ),
    };
    
    let mut visuals = match mode {
        ThemeMode::Light => Visuals::light(),
        _ => Visuals::dark(),
    };
    
    // Window
    visuals.window_fill = bg_panel;
    visuals.window_stroke = Stroke::new(1.0, if mode == ThemeMode::HighContrast { 
        Color32::WHITE 
    } else { 
        Color32::from_rgb(50, 50, 75) 
    });
    visuals.window_shadow = egui::Shadow::NONE;
    visuals.window_rounding = Rounding::same(16.0);
    
    // Panel
    visuals.panel_fill = bg_panel;
    
    // Widgets
    visuals.widgets.noninteractive.bg_fill = bg_widget;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, text_secondary);
    visuals.widgets.noninteractive.rounding = Rounding::same(10.0);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(
        if mode == ThemeMode::HighContrast { 2.0 } else { 1.0 }, 
        if mode == ThemeMode::HighContrast { Color32::WHITE } else { Color32::from_rgb(55, 55, 80) }
    );
    
    visuals.widgets.inactive.bg_fill = bg_widget;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, text_primary);
    visuals.widgets.inactive.rounding = Rounding::same(10.0);
    visuals.widgets.inactive.bg_stroke = Stroke::new(
        if mode == ThemeMode::HighContrast { 2.0 } else { 1.0 },
        if mode == ThemeMode::HighContrast { Color32::WHITE } else { Color32::from_rgb(55, 55, 80) }
    );
    
    visuals.widgets.hovered.bg_fill = bg_hover;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, text_primary);
    visuals.widgets.hovered.rounding = Rounding::same(10.0);
    visuals.widgets.hovered.bg_stroke = Stroke::new(
        if mode == ThemeMode::HighContrast { 3.0 } else { 1.0 }, 
        primary
    );
    
    visuals.widgets.active.bg_fill = primary.gamma_multiply(0.25);
    visuals.widgets.active.fg_stroke = Stroke::new(1.5, primary);
    visuals.widgets.active.rounding = Rounding::same(10.0);
    visuals.widgets.active.bg_stroke = Stroke::new(
        if mode == ThemeMode::HighContrast { 3.0 } else { 1.0 }, 
        primary
    );
    
    // Selection
    visuals.selection.bg_fill = primary.gamma_multiply(if mode == ThemeMode::HighContrast { 0.5 } else { 0.25 });
    visuals.selection.stroke = Stroke::new(if mode == ThemeMode::HighContrast { 2.0 } else { 1.0 }, primary);
    
    // Extreme background
    visuals.extreme_bg_color = bg_dark;
    
    // Hyperlinks
    visuals.hyperlink_color = secondary;
    
    // Text cursor
    visuals.text_cursor.stroke = Stroke::new(2.0, primary);
    
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

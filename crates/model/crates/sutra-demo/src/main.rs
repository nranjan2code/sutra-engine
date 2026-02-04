/// SutraWorks Interactive AI Demo
/// 
/// A hands-on interface where users can:
/// - Chat with RWKV models in real-time
/// - See Mamba processing text step-by-step  
/// - Watch quantization compress models live
/// - Experiment with neuro-symbolic reasoning
/// - Compare architecture performance
mod app;
mod chat;
mod comparison;
mod models;
mod qa;
mod quantization_demo;

use eframe::egui;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([1000.0, 600.0])
            .with_title("SutraWorks AI Demo - Pure Rust Intelligence"),
        ..Default::default()
    };

    eframe::run_native(
        "SutraWorks AI Demo",
        options,
        Box::new(|cc| {
            // Setup custom styling for demo
            setup_demo_style(&cc.egui_ctx);
            
            Ok(Box::new(app::DemoApp::new(cc)))
        }),
    )
}

fn setup_demo_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    
    // Enhanced spacing for better readability
    style.spacing.item_spacing = egui::vec2(10.0, 8.0);
    style.spacing.button_padding = egui::vec2(16.0, 10.0);
    style.spacing.indent = 24.0;
    
    // Larger, more readable text
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(15.0, egui::FontFamily::Proportional),
    );
    
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(15.0, egui::FontFamily::Proportional),
    );
    
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::new(20.0, egui::FontFamily::Proportional),
    );
    
    style.text_styles.insert(
        egui::TextStyle::Small,
        egui::FontId::new(13.0, egui::FontFamily::Proportional),
    );
    
    // High contrast color scheme inspired by terminal themes
    let visuals = &mut style.visuals;
    
    // Dark theme with high contrast
    *visuals = egui::Visuals::dark();
    
    // Terminal-inspired colors
    visuals.override_text_color = Some(egui::Color32::from_rgb(240, 240, 240)); // Bright white text
    visuals.panel_fill = egui::Color32::from_rgb(25, 25, 35); // Dark blue-gray background
    visuals.window_fill = egui::Color32::from_rgb(30, 30, 40); // Slightly lighter for windows
    
    // Button colors with terminal theme
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(45, 45, 55); // Dark gray for buttons
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(60, 80, 100); // Blue on hover
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(40, 120, 80); // Green when active
    
    // Selection colors - terminal green
    visuals.selection.bg_fill = egui::Color32::from_rgb(0, 150, 80);
    visuals.selection.stroke.color = egui::Color32::from_rgb(0, 200, 100);
    
    // Hyperlinks - cyan like terminals
    visuals.hyperlink_color = egui::Color32::from_rgb(100, 200, 255);
    
    // Error color - terminal red
    visuals.error_fg_color = egui::Color32::from_rgb(255, 80, 80);
    
    // Warning color - terminal yellow
    visuals.warn_fg_color = egui::Color32::from_rgb(255, 200, 100);
    
    // Borders and separators
    visuals.widgets.noninteractive.bg_stroke.color = egui::Color32::from_rgb(80, 80, 90);
    visuals.widgets.inactive.bg_stroke.color = egui::Color32::from_rgb(100, 100, 110);
    
    // Window border
    visuals.window_stroke.color = egui::Color32::from_rgb(120, 120, 130);
    visuals.window_stroke.width = 1.0;
    
    ctx.set_style(style);
}
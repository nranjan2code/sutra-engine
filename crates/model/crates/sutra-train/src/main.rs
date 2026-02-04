/// SutraWorks Training GUI
/// 
/// A user-friendly interface for training AI models without ML expertise required.
/// Features:
/// - Drag & drop data loading
/// - Visual training configuration
/// - Real-time progress monitoring
/// - Model templates for common tasks
/// - One-click export and deployment
mod app;
mod config;
mod data;
mod models;
mod progress;
mod templates;
mod ui;
mod utils;

use eframe::egui;

fn main() -> eframe::Result<()> {
    env_logger::init(); // Initialize logging

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "SutraWorks Training Studio",
        options,
        Box::new(|cc| {
            // Setup custom font if needed
            setup_custom_fonts(&cc.egui_ctx);
            
            Ok(Box::new(app::TrainingApp::new(cc)))
        }),
    )
}

fn setup_custom_fonts(_ctx: &egui::Context) {
    // Custom fonts will be added later
    // For now, use default fonts
}
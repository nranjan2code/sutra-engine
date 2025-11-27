//! Sutra Desktop - Native GUI for the Knowledge Reasoning Engine
//!
//! This is a THIN GUI layer that directly uses `sutra-storage` crate.
//! NO duplication of storage logic - we embed the same engine used by Docker deployments.
//!
//! ENHANCED UI (v3.3) features:
//! - Interactive graph visualization with force-directed layout
//! - MPPA-style reasoning path exploration
//! - Temporal and causal analysis views
//! - Real-time analytics dashboard
//! - Advanced query builder
//! - Export/Import functionality

mod app;
mod ui;
mod theme;
mod types;
mod local_embedding; // 🔥 NEW: Local AI provider

pub use app::SutraApp;

/// Application version (matches workspace)
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application name
pub const APP_NAME: &str = "Sutra AI";

use anyhow::Result;
use eframe::egui;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting {} Desktop v{}", APP_NAME, VERSION);

    // Configure native options for modern look with native decorations
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(format!("{} - Desktop Edition", APP_NAME))
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_icon(load_icon())
            .with_active(true),
        centered: true,
        ..Default::default()
    };

    // Run the application
    eframe::run_native(
        APP_NAME,
        options,
        Box::new(|cc| {
            // Apply custom theme
            theme::setup_custom_theme(&cc.egui_ctx);
            
            // Enable image loading
            egui_extras::install_image_loaders(&cc.egui_ctx);
            
            // Setup native menu bar (macOS)
            setup_menu_bar(&cc.egui_ctx);
            
            Ok(Box::new(SutraApp::new(cc)))
        }),
    ).map_err(|e| anyhow::anyhow!("Failed to start application: {}", e))
}

/// Setup native menu bar for better macOS integration
fn setup_menu_bar(ctx: &egui::Context) {
    // Note: egui doesn't support native menu bars directly yet,
    // but we can prepare for it and add a menu bar in the UI
    // For now, we'll add a custom menu bar in the app UI
    let _ = ctx; // Use to avoid warning
}

/// Load application icon
fn load_icon() -> egui::IconData {
    let size = 64;
    let mut rgba = vec![0u8; size * size * 4];
    
    for y in 0..size {
        for x in 0..size {
            let idx = (y * size + x) * 4;
            let cx = (x as f32 - size as f32 / 2.0) / (size as f32 / 2.0);
            let cy = (y as f32 - size as f32 / 2.0) / (size as f32 / 2.0);
            let dist = (cx * cx + cy * cy).sqrt();
            
            if dist < 0.9 {
                let t = dist / 0.9;
                rgba[idx] = (138.0 + (59.0 - 138.0) * t) as u8;
                rgba[idx + 1] = (43.0 + (130.0 - 43.0) * t) as u8;
                rgba[idx + 2] = (226.0 + (246.0 - 226.0) * t) as u8;
                rgba[idx + 3] = 255;
            }
        }
    }
    
    egui::IconData {
        rgba,
        width: size as u32,
        height: size as u32,
    }
}

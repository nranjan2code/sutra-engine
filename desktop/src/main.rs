//! Sutra Desktop - Native GUI for the Knowledge Reasoning Engine
//!
//! This is a THIN GUI layer that directly uses `sutra-storage` crate.
//! NO duplication of storage logic - we embed the same engine used by Docker deployments.

mod app;
mod ui;
mod theme;

use anyhow::Result;
use eframe::egui;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use app::SutraApp;

/// Application version (matches workspace)
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application name
pub const APP_NAME: &str = "Sutra AI";

fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting {} Desktop v{}", APP_NAME, VERSION);

    // Configure native options for modern look
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(format!("{} - Desktop Edition", APP_NAME))
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_icon(load_icon()),
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
            
            Ok(Box::new(SutraApp::new(cc)))
        }),
    ).map_err(|e| anyhow::anyhow!("Failed to start application: {}", e))
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

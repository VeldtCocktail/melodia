// melodia/src/main.rs
// Cross-platform music player — Windows & Linux

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // No console on Windows release

mod app;
mod audio;
mod library;
mod playlist;
mod queue;
mod ui;
mod metadata;
mod config;
mod theme;

use eframe::egui;

fn main() -> eframe::Result<()> {
    // Logging (debug builds only, requires --features debug-logging)
    #[cfg(all(debug_assertions, feature = "debug-logging"))]
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Melodia")
            .with_inner_size([1100.0, 700.0])
            .with_min_inner_size([800.0, 500.0])
            .with_icon(load_app_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "melodia",
        options,
        Box::new(|cc| Box::new(app::MelodiaApp::new(cc))),
    )
}

fn load_app_icon() -> egui::IconData {
    let icon_bytes = include_bytes!("ui/logo.png");
    let image = image::load_from_memory(icon_bytes)
        .expect("Failed to load icon")
        .to_rgba8();
    let (width, height) = image.dimensions();
    egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    }
}

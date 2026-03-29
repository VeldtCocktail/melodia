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
        "Melodia",
        options,
        Box::new(|cc| Box::new(app::MelodiaApp::new(cc))),
    )
}

fn load_app_icon() -> egui::IconData {
    // Embedded minimal icon (16x16 RGBA)
    let size = 32u32;
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let cx = x as f32 - size as f32 / 2.0;
            let cy = y as f32 - size as f32 / 2.0;
            let dist = (cx * cx + cy * cy).sqrt();
            if dist < size as f32 / 2.0 - 1.0 {
                rgba.extend_from_slice(&[138, 43, 226, 255]); // purple
            } else {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }
    egui::IconData { rgba, width: size, height: size }
}

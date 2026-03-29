// melodia/src/ui/sidebar.rs

use egui::*;
use crate::theme::*;
use crate::app::{MelodiaApp, Panel};

pub fn show(app: &mut MelodiaApp, ui: &mut Ui) {
    ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
        // App title
        ui.add_space(16.0);
        ui.horizontal(|ui| {
            ui.add_space(12.0);
            ui.colored_label(ACCENT_LIGHT, RichText::new("🎵 Melodia").size(20.0).strong());
        });
        ui.add_space(16.0);
        ui.add(Separator::default().spacing(0.0));
        ui.add_space(8.0);

        // Navigation items
        nav_item(ui, app, "📚  Library", Panel::Library);
        nav_item(ui, app, "📋  Queue", Panel::Queue);

        ui.add_space(8.0);
        ui.add(Separator::default().spacing(0.0));
        ui.add_space(8.0);

        // Playlists section
        ui.horizontal(|ui| {
            ui.add_space(12.0);
            ui.colored_label(TEXT_DIM, RichText::new("PLAYLISTS").size(11.0));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(8.0);
                if ui.add(
                    Button::new(RichText::new("+").size(16.0).color(ACCENT_LIGHT))
                        .frame(false)
                ).on_hover_text("New Playlist").clicked() {
                    app.show_new_playlist_dialog = true;
                }
            });
        });
        ui.add_space(4.0);

        // List playlists
        let playlist_ids: Vec<(String, String)> = app.playlist_store
            .playlists
            .iter()
            .map(|p| (p.id.clone(), p.name.clone()))
            .collect();

        for (id, name) in playlist_ids {
            let is_selected = matches!(&app.active_panel, Panel::Playlist(pid) if pid == &id);
            let response = ui.add_sized(
                [ui.available_width(), 30.0],
                SelectableLabel::new(is_selected, format!("  🎵 {}", name))
            );
            if response.clicked() {
                app.active_panel = Panel::Playlist(id);
            }
            response.context_menu(|ui| {
                if ui.button("🗑 Delete Playlist").clicked() {
                    // handled in app
                    ui.close_menu();
                }
            });
        }

        // Spacer + folder button at bottom
        ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
            ui.add_space(8.0);
            ui.add(Separator::default().spacing(0.0));
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.add_space(8.0);
                if ui.add(
                    Button::new(RichText::new("📂  Open Folder").size(13.0))
                        .fill(BG_CARD)
                ).clicked() {
                    app.open_folder_dialog();
                }
            });
            ui.add_space(4.0);

            if let Some(ref folder) = app.current_folder.clone() {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.add_space(12.0);
                    let name = folder.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| folder.to_string_lossy().to_string());
                    ui.colored_label(TEXT_DIM, RichText::new(format!("📁 {}", name)).size(11.0));
                });
            }

            // Audio device info
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add_space(12.0);
                ui.colored_label(TEXT_DIM, RichText::new(format!("🔊 {}", app.player.device_name)).size(10.0));
            });
            ui.add_space(4.0);
        });
    });
}

fn nav_item(ui: &mut Ui, app: &mut MelodiaApp, label: &str, panel: Panel) {
    let is_selected = std::mem::discriminant(&app.active_panel) == std::mem::discriminant(&panel);
    let resp = ui.add_sized(
        [ui.available_width(), 32.0],
        SelectableLabel::new(is_selected, RichText::new(label).size(14.0))
    );
    if resp.clicked() {
        app.active_panel = panel;
    }
}

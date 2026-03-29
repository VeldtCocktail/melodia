// melodia/src/ui/dialogs.rs

use egui::*;
use crate::theme::*;
use crate::app::MelodiaApp;
use crate::playlist::Playlist;

pub fn show_new_playlist_dialog(app: &mut MelodiaApp, ctx: &Context) {
    if !app.show_new_playlist_dialog {
        return;
    }

    let mut open = true;
    let mut confirmed = false;

    Window::new("New Playlist")
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .fixed_size(vec2(320.0, 130.0))
        .open(&mut open)
        .show(ctx, |ui| {
            ui.add_space(8.0);
            ui.label("Playlist name:");
            ui.add_space(4.0);

            let resp = ui.add(
                TextEdit::singleline(&mut app.new_playlist_name)
                    .hint_text("My Playlist")
                    .desired_width(f32::INFINITY)
            );
            // Auto-focus
            resp.request_focus();

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.add(Button::new("Cancel").fill(BG_CARD)).clicked() {
                    open = false;
                }
                ui.add_space(8.0);
                let can_create = !app.new_playlist_name.trim().is_empty();
                if ui.add(
                    Button::new(RichText::new("Create").color(Color32::WHITE))
                        .fill(if can_create { ACCENT } else { BG_CARD })
                ).clicked() && can_create {
                    confirmed = true;
                }
            });

            // Enter key confirms
            if ui.input(|i| i.key_pressed(Key::Enter)) && !app.new_playlist_name.trim().is_empty() {
                confirmed = true;
            }
        });

    if confirmed {
        let name = app.new_playlist_name.trim().to_string();
        if !name.is_empty() {
            let pl = Playlist::new(name);
            let id = pl.id.clone();
            app.playlist_store.add_playlist(pl);
            app.save_playlists();
            app.active_panel = crate::app::Panel::Playlist(id);
        }
        app.new_playlist_name.clear();
        app.show_new_playlist_dialog = false;
    } else if !open {
        app.new_playlist_name.clear();
        app.show_new_playlist_dialog = false;
    }
}

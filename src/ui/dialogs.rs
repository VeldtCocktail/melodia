// melodia/src/ui/dialogs.rs

use egui::*;
use crate::theme::*;
use crate::app::MelodiaApp;
use crate::playlist::Playlist;

pub fn show_new_playlist_dialog(app: &mut MelodiaApp, ctx: &Context) {
    if !app.show_new_playlist_dialog {
        return;
    }

    // egui Window::open() takes &mut bool but then the closure also needs
    // to mutate it — avoid the double-borrow by not using .open() and
    // instead tracking close intent with a local flag.
    let mut close  = false;
    let mut confirm = false;

    Window::new("New Playlist")
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .fixed_size(vec2(320.0, 130.0))
        .show(ctx, |ui| {
            ui.add_space(8.0);
            ui.label("Playlist name:");
            ui.add_space(4.0);

            let resp = ui.add(
                TextEdit::singleline(&mut app.new_playlist_name)
                    .hint_text("My Playlist")
                    .desired_width(f32::INFINITY)
            );
            resp.request_focus();

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.add(Button::new("Cancel").fill(BG_CARD)).clicked() {
                    close = true;
                }
                ui.add_space(8.0);
                let can_create = !app.new_playlist_name.trim().is_empty();
                let btn_fill   = if can_create { ACCENT } else { BG_CARD };
                if ui.add(
                    Button::new(RichText::new("Create").color(Color32::WHITE)).fill(btn_fill)
                ).clicked() && can_create {
                    confirm = true;
                }
            });

            if ui.input(|i| i.key_pressed(Key::Enter))
                && !app.new_playlist_name.trim().is_empty()
            {
                confirm = true;
            }
            if ui.input(|i| i.key_pressed(Key::Escape)) {
                close = true;
            }
        });

    if confirm {
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
    } else if close {
        app.new_playlist_name.clear();
        app.show_new_playlist_dialog = false;
    }
}

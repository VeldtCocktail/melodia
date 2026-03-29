// melodia/src/ui/playlist_panel.rs

use egui::*;
use crate::theme::*;
use crate::app::MelodiaApp;

pub fn show(app: &mut MelodiaApp, ui: &mut Ui, playlist_id: &str) {
    let playlist_id = playlist_id.to_string();

    // Clone data we need for display (avoids borrow conflict)
    let (pl_name, pl_track_ids) = match app.playlist_store.get(&playlist_id) {
        Some(pl) => (pl.name.clone(), pl.track_ids.clone()),
        None => {
            ui.colored_label(TEXT_SECONDARY, "Playlist not found.");
            return;
        }
    };

    // Header
    ui.horizontal(|ui| {
        ui.colored_label(TEXT_PRIMARY, RichText::new(&pl_name).size(20.0).strong());
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.colored_label(TEXT_SECONDARY, format!("{} tracks", pl_track_ids.len()));

            if !pl_track_ids.is_empty() {
                if ui.add(
                    Button::new(RichText::new("▶  Play All").size(12.0))
                        .fill(ACCENT)
                ).clicked() {
                    app.play_playlist(&playlist_id);
                }
            }
        });
    });
    ui.add_space(8.0);

    if pl_track_ids.is_empty() {
        ui.add_space(40.0);
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            ui.colored_label(TEXT_DIM, RichText::new("🎵").size(48.0));
            ui.add_space(8.0);
            ui.colored_label(TEXT_SECONDARY, "This playlist is empty.");
            ui.colored_label(TEXT_DIM, "Right-click tracks in the Library to add them.");
        });
        return;
    }

    // Resolve track IDs to tracks
    let resolved: Vec<(usize, String, String, String, String)> = pl_track_ids
        .iter()
        .enumerate()
        .filter_map(|(pl_pos, tid)| {
            app.library.iter().enumerate().find(|(_, t)| &t.id == tid)
                .map(|(lib_idx, t)| (
                    pl_pos,
                    tid.clone(),
                    t.display_title().to_string(),
                    t.display_artist().to_string(),
                    t.format_duration(),
                ))
        })
        .collect();

    let row_height = 42.0;

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let mut remove_idx: Option<usize> = None;
            let mut move_from: Option<usize> = None;
            let mut move_to: Option<usize> = None;

            for (pl_pos, tid, title, artist, dur) in &resolved {
                let is_current = app.queue.current_track_id() == Some(tid.as_str());
                let bg = if is_current { BG_SELECTED_DIM } else if *pl_pos % 2 == 0 { BG_DARK } else { BG_PANEL };

                let row_rect = ui.available_rect_before_wrap();
                let row_rect = Rect::from_min_size(row_rect.min, vec2(row_rect.width(), row_height));

                let resp = ui.allocate_rect(row_rect, Sense::click_and_drag());
                ui.painter().rect_filled(row_rect, 0.0, bg);
                if resp.hovered() {
                    ui.painter().rect_filled(row_rect, 0.0, BG_HOVER.linear_multiply(0.4));
                }

                ui.allocate_ui_at_rect(row_rect, |ui| {
                    ui.horizontal_centered(|ui| {
                        ui.add_space(8.0);
                        let num_str = if is_current { "▶".to_string() } else { format!("{}", pl_pos + 1) };
                        let num_col = if is_current { ACCENT_LIGHT } else { TEXT_DIM };
                        ui.add_sized([32.0, row_height], Label::new(RichText::new(num_str).color(num_col).size(12.0)));

                        let title_col = if is_current { ACCENT_LIGHT } else { TEXT_PRIMARY };
                        ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                            ui.add(Label::new(RichText::new(title).color(title_col).size(13.0)).truncate(true));
                            ui.colored_label(TEXT_DIM, RichText::new(artist).size(11.0));
                        });

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.add_space(8.0);
                            if ui.add(Button::new(RichText::new("✕").color(TEXT_DIM).size(11.0)).frame(false)).clicked() {
                                remove_idx = Some(*pl_pos);
                            }
                            ui.add_space(8.0);
                            ui.colored_label(TEXT_DIM, dur);
                        });
                    });
                });

                if resp.double_clicked() {
                    // Find lib index
                    if let Some(lib_idx) = app.library.iter().position(|t| &t.id == tid) {
                        app.play_playlist_from(&playlist_id, *pl_pos);
                    }
                }

                // Drag to reorder
                if resp.hovered() && ui.memory(|m| m.is_anything_being_dragged()) {
                    move_to = Some(*pl_pos);
                }
                if resp.drag_started() {
                    move_from = Some(*pl_pos);
                }
            }

            // Apply mutations
            if let Some(idx) = remove_idx {
                if let Some(pl) = app.playlist_store.get_mut(&playlist_id) {
                    pl.track_ids.remove(idx);
                }
                app.save_playlists();
            }
            if let (Some(f), Some(t)) = (move_from, move_to) {
                if f != t {
                    if let Some(pl) = app.playlist_store.get_mut(&playlist_id) {
                        pl.move_track(f, t);
                    }
                    app.save_playlists();
                }
            }
        });
}

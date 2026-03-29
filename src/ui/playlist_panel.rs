// melodia/src/ui/playlist_panel.rs

use egui::*;
use crate::theme::*;
use crate::app::MelodiaApp;

pub fn show(app: &mut MelodiaApp, ui: &mut Ui, playlist_id: &str) {
    let playlist_id = playlist_id.to_string();

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
                    app.play_playlist(&playlist_id.clone());
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

    // Resolve track IDs → display data (owned, no borrow on app later)
    struct RowData {
        pl_pos:     usize,
        track_id:   String,
        title:      String,
        artist:     String,
        duration:   String,
        is_current: bool,
    }

    let current_id = app.queue.current_track_id().map(|s| s.to_string());

    let rows: Vec<RowData> = pl_track_ids.iter().enumerate().filter_map(|(pl_pos, tid)| {
        app.library.iter().find(|t| &t.id == tid).map(|t| RowData {
            pl_pos,
            track_id:   tid.clone(),
            title:      t.display_title().to_string(),
            artist:     t.display_artist().to_string(),
            duration:   t.format_duration(),
            is_current: current_id.as_deref() == Some(tid.as_str()),
        })
    }).collect();

    let row_height = 42.0;

    let mut action_remove:    Option<usize>         = None;
    let mut action_play_from: Option<usize>         = None;
    let mut action_move:      Option<(usize, usize)>= None;

    let drag_id_base = Id::new("pl_drag");

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let dragged_id  = ui.ctx().dragged_id();
            let pointer_pos = ui.ctx().pointer_hover_pos();

            for row in &rows {
                let i  = row.pl_pos;
                let bg = if row.is_current { BG_SELECTED_DIM }
                         else if i % 2 == 0 { BG_DARK }
                         else { BG_PANEL };

                let row_rect = ui.available_rect_before_wrap();
                let row_rect = Rect::from_min_size(row_rect.min, vec2(row_rect.width(), row_height));

                let resp = ui.allocate_rect(row_rect, Sense::click_and_drag());
                ui.painter().rect_filled(row_rect, 0.0, bg);
                if resp.hovered() {
                    ui.painter().rect_filled(row_rect, 0.0, BG_HOVER.linear_multiply(0.4));
                }

                // Drop indicator
                if dragged_id.is_some() && resp.hovered() {
                    ui.painter().hline(
                        row_rect.x_range(),
                        row_rect.top(),
                        Stroke::new(2.0, ACCENT),
                    );
                }

                ui.allocate_ui_at_rect(row_rect, |ui| {
                    ui.horizontal_centered(|ui| {
                        ui.add_space(8.0);
                        let num_str  = if row.is_current { "▶".to_string() } else { format!("{}", i + 1) };
                        let num_col  = if row.is_current { ACCENT_LIGHT } else { TEXT_DIM };
                        ui.add_sized([32.0, row_height],
                            Label::new(RichText::new(num_str).color(num_col).size(12.0)));

                        let title_col = if row.is_current { ACCENT_LIGHT } else { TEXT_PRIMARY };
                        ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                            ui.add(Label::new(RichText::new(&row.title).color(title_col).size(13.0))
                                .truncate(true));
                            ui.colored_label(TEXT_DIM, RichText::new(&row.artist).size(11.0));
                        });

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.add_space(8.0);
                            if ui.add(Button::new(
                                RichText::new("✕").color(TEXT_DIM).size(11.0)).frame(false)
                            ).clicked() {
                                action_remove = Some(i);
                            }
                            ui.add_space(8.0);
                            ui.colored_label(TEXT_DIM, &row.duration);
                        });
                    });
                });

                if resp.double_clicked() { action_play_from = Some(i); }

                if resp.drag_started() {
                    ui.ctx().set_dragged_id(drag_id_base.with(i));
                    ui.ctx().data_mut(|d| d.insert_temp::<usize>(Id::new("pl_drag_src"), i));
                }
            }

            // Resolve drop
            if ui.input(|inp| inp.pointer.any_released()) {
                let src = ui.ctx().data_mut(|d| d.remove_temp::<usize>(Id::new("pl_drag_src")));
                if let (Some(src_i), Some(pos)) = (src, pointer_pos) {
                    for j in 0..rows.len() {
                        let check_y_top = j as f32 * (row_height + 2.0);
                        let check_y_bot = check_y_top + row_height + 2.0;
                        if pos.y > check_y_top && pos.y <= check_y_bot && src_i != j {
                            action_move = Some((src_i, j));
                            break;
                        }
                    }
                }
            }
        });

    // Apply deferred mutations
    if let Some(idx) = action_remove {
        if let Some(pl) = app.playlist_store.get_mut(&playlist_id) {
            pl.track_ids.remove(idx);
        }
        app.save_playlists();
    }
    if let Some((f, t)) = action_move {
        if let Some(pl) = app.playlist_store.get_mut(&playlist_id) {
            pl.move_track(f, t);
        }
        app.save_playlists();
    }
    if let Some(start) = action_play_from {
        app.play_playlist_from(&playlist_id, start);
    }
}

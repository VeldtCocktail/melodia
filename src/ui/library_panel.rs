// melodia/src/ui/library_panel.rs

use egui::*;
use crate::theme::*;
use crate::app::MelodiaApp;

pub fn show(app: &mut MelodiaApp, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.colored_label(TEXT_PRIMARY, RichText::new("Library").size(20.0).strong());
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if !app.library.is_empty() {
                ui.colored_label(TEXT_SECONDARY, format!("{} tracks", app.library.len()));
            }
        });
    });
    ui.add_space(8.0);

    ui.horizontal(|ui| {
        ui.add(
            TextEdit::singleline(&mut app.search_query)
                .hint_text("🔍  Search tracks, artists, albums…")
                .desired_width(f32::INFINITY)
        );
    });
    ui.add_space(8.0);

    if app.library.is_empty() {
        ui.add_space(40.0);
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            ui.colored_label(TEXT_DIM, RichText::new("📂").size(48.0));
            ui.add_space(8.0);
            ui.colored_label(TEXT_SECONDARY, "No music loaded.");
            ui.add_space(4.0);
            ui.colored_label(TEXT_DIM, "Open a folder with the button in the sidebar.");
        });
        return;
    }

    if app.loading_metadata {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.spinner();
            ui.colored_label(TEXT_SECONDARY, " Loading metadata…");
        });
        ui.add_space(4.0);
    }

    let col_widths = column_widths(ui.available_width());
    ui.horizontal(|ui| {
        ui.add_space(8.0);
        header_col(ui, "#",      col_widths[0]);
        header_col(ui, "Title",  col_widths[1]);
        header_col(ui, "Artist", col_widths[2]);
        header_col(ui, "Album",  col_widths[3]);
        header_col(ui, "Time",   col_widths[4]);
    });
    ui.add(Separator::default().spacing(0.0));

    // ── Pre-extract all row data so we hold no borrow on `app` inside the loop
    let query = app.search_query.to_lowercase();
    let current_track_id = app.queue.current_track_id().map(|s| s.to_string());
    let selected_id      = app.selected_track_id.clone();

    struct RowData {
        lib_idx:    usize,
        track_id:   String,
        is_playing: bool,
        is_selected:bool,
        num_str:    String,
        title:      String,
        artist:     String,
        album:      String,
        duration:   String,
    }

    let rows: Vec<RowData> = (0..app.library.len())
        .filter(|&i| {
            if query.is_empty() { return true; }
            let t = &app.library[i];
            t.display_title().to_lowercase().contains(&query)
                || t.display_artist().to_lowercase().contains(&query)
                || t.display_album().to_lowercase().contains(&query)
        })
        .enumerate()
        .map(|(row_idx, lib_idx)| {
            let t = &app.library[lib_idx];
            let is_playing  = current_track_id.as_deref() == Some(t.id.as_str());
            let is_selected = selected_id.as_deref()      == Some(t.id.as_str());
            let num_str = if is_playing {
                "▶".to_string()
            } else {
                t.track_number
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| (row_idx + 1).to_string())
            };
            RowData {
                lib_idx,
                track_id:   t.id.clone(),
                is_playing,
                is_selected,
                num_str,
                title:    t.display_title().to_string(),
                artist:   t.display_artist().to_string(),
                album:    t.display_album().to_string(),
                duration: t.format_duration(),
            }
        })
        .collect();

    // Playlist list for context menus (owned, so no borrow on app later)
    let playlists: Vec<(String, String)> = app.playlist_store.playlists
        .iter()
        .map(|p| (p.id.clone(), p.name.clone()))
        .collect();

    let row_height = 36.0;

    // Deferred actions — applied after scroll area to avoid borrow conflicts
    let mut action_play:             Option<usize>          = None;
    let mut action_enqueue_next:     Option<usize>          = None;
    let mut action_enqueue:          Option<usize>          = None;
    let mut action_select:           Option<String>         = None;
    let mut action_add_to_playlist:  Option<(String,String)>= None;

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show_rows(ui, row_height, rows.len(), |ui, row_range| {
            for row_idx in row_range {
                let row = &rows[row_idx];

                let bg = if row.is_playing      { BG_SELECTED_DIM }
                         else if row.is_selected { BG_HOVER }
                         else if row_idx % 2 == 0 { BG_DARK }
                         else { BG_PANEL };

                let row_rect = ui.available_rect_before_wrap();
                let row_rect = Rect::from_min_size(row_rect.min, vec2(row_rect.width(), row_height));

                let resp = ui.allocate_rect(row_rect, Sense::click());
                ui.painter().rect_filled(row_rect, 0.0, bg);
                if resp.hovered() {
                    ui.painter().rect_filled(row_rect, 0.0, BG_HOVER.linear_multiply(0.5));
                }

                ui.allocate_ui_at_rect(row_rect, |ui| {
                    ui.horizontal_centered(|ui| {
                        ui.add_space(8.0);

                        let num_col     = if row.is_playing { ACCENT_LIGHT } else { TEXT_DIM };
                        let title_color = if row.is_playing { ACCENT_LIGHT } else { TEXT_PRIMARY };

                        ui.add_sized([col_widths[0], row_height],
                            Label::new(RichText::new(&row.num_str).color(num_col).size(12.0)));
                        ui.add_sized([col_widths[1], row_height],
                            Label::new(RichText::new(&row.title).color(title_color).size(13.0))
                                .truncate(true));
                        ui.add_sized([col_widths[2], row_height],
                            Label::new(RichText::new(&row.artist).color(TEXT_SECONDARY).size(13.0))
                                .truncate(true));
                        ui.add_sized([col_widths[3], row_height],
                            Label::new(RichText::new(&row.album).color(TEXT_DIM).size(12.0))
                                .truncate(true));
                        ui.add_sized([col_widths[4], row_height],
                            Label::new(RichText::new(&row.duration).color(TEXT_DIM).size(12.0)));
                    });
                });

                if resp.double_clicked() { action_play   = Some(row.lib_idx); }
                if resp.clicked()        { action_select = Some(row.track_id.clone()); }

                // Context menu — capture only owned values, no reference to app
                let lib_idx        = row.lib_idx;
                let track_id       = row.track_id.clone();
                let playlists_copy = playlists.clone();

                resp.context_menu(|ui| {
                    if ui.button("▶  Play Now").clicked() {
                        action_play = Some(lib_idx); ui.close_menu();
                    }
                    if ui.button("⏭  Play Next").clicked() {
                        action_enqueue_next = Some(lib_idx); ui.close_menu();
                    }
                    if ui.button("➕  Add to Queue").clicked() {
                        action_enqueue = Some(lib_idx); ui.close_menu();
                    }
                    ui.add(Separator::default());
                    ui.menu_button("🎵  Add to Playlist", |ui| {
                        if playlists_copy.is_empty() {
                            ui.colored_label(TEXT_DIM, "No playlists yet");
                        }
                        for (pid, pname) in &playlists_copy {
                            if ui.button(pname).clicked() {
                                action_add_to_playlist = Some((pid.clone(), track_id.clone()));
                                ui.close_menu();
                            }
                        }
                    });
                });
            }
        });

    // ── Apply deferred mutations (no conflicting borrows) ─────────────────
    if let Some(idx) = action_play            { app.play_from_library(idx); }
    if let Some(idx) = action_enqueue_next    { app.enqueue_next(idx); }
    if let Some(idx) = action_enqueue         { app.enqueue_track(idx); }
    if let Some(id)  = action_select          { app.selected_track_id = Some(id); }
    if let Some((pid, tid)) = action_add_to_playlist {
        if let Some(pl) = app.playlist_store.get_mut(&pid) { pl.add_track(tid); }
        app.save_playlists();
    }
}

fn column_widths(total: f32) -> [f32; 5] {
    let num_w  = 36.0;
    let dur_w  = 52.0;
    let rest   = total - num_w - dur_w - 24.0;
    [num_w, rest * 0.40, rest * 0.30, rest * 0.30, dur_w]
}

fn header_col(ui: &mut Ui, text: &str, width: f32) {
    ui.add_sized([width, 20.0],
        Label::new(RichText::new(text).color(TEXT_DIM).size(11.0)));
}

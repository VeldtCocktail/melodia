// melodia/src/ui/library_panel.rs

use egui::*;
use crate::theme::*;
use crate::app::MelodiaApp;
use crate::library::format_duration;

pub fn show(app: &mut MelodiaApp, ui: &mut Ui) {
    // Header row
    ui.horizontal(|ui| {
        ui.colored_label(TEXT_PRIMARY, RichText::new("Library").size(20.0).strong());
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if !app.library.is_empty() {
                ui.colored_label(
                    TEXT_SECONDARY,
                    format!("{} tracks", app.library.len())
                );
            }
        });
    });
    ui.add_space(8.0);

    // Search bar
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

    // Column headers
    let col_widths = column_widths(ui.available_width());

    ui.horizontal(|ui| {
        ui.add_space(8.0);
        header_col(ui, "#", col_widths[0]);
        header_col(ui, "Title", col_widths[1]);
        header_col(ui, "Artist", col_widths[2]);
        header_col(ui, "Album", col_widths[3]);
        header_col(ui, "Time", col_widths[4]);
    });
    ui.add(Separator::default().spacing(0.0));

    // Filtered indices
    let query = app.search_query.to_lowercase();
    let indices: Vec<usize> = (0..app.library.len())
        .filter(|&i| {
            if query.is_empty() { return true; }
            let t = &app.library[i];
            t.display_title().to_lowercase().contains(&query)
                || t.display_artist().to_lowercase().contains(&query)
                || t.display_album().to_lowercase().contains(&query)
        })
        .collect();

    let row_height = 36.0;
    let available_height = ui.available_height();

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show_rows(ui, row_height, indices.len(), |ui, row_range| {
            for row_idx in row_range {
                let lib_idx = indices[row_idx];
                let track = &app.library[lib_idx];
                let is_playing = app.queue.current_track_id() == Some(&track.id);
                let is_selected = app.selected_track_id.as_deref() == Some(&track.id);

                let bg = if is_playing {
                    BG_SELECTED_DIM
                } else if is_selected {
                    BG_HOVER
                } else if row_idx % 2 == 0 {
                    BG_DARK
                } else {
                    BG_PANEL
                };

                let row_rect = ui.available_rect_before_wrap();
                let row_rect = Rect::from_min_size(
                    row_rect.min,
                    vec2(row_rect.width(), row_height)
                );

                let resp = ui.allocate_rect(row_rect, Sense::click());
                ui.painter().rect_filled(row_rect, 0.0, bg);

                if resp.hovered() {
                    ui.painter().rect_filled(row_rect, 0.0, BG_HOVER.linear_multiply(0.5));
                }

                // Row content
                ui.allocate_ui_at_rect(row_rect, |ui| {
                    ui.horizontal_centered(|ui| {
                        ui.add_space(8.0);

                        // Track number / playing indicator
                        let num_str = if is_playing {
                            "▶".to_string()
                        } else {
                            track.track_number
                                .map(|n| n.to_string())
                                .unwrap_or_else(|| (lib_idx + 1).to_string())
                        };
                        let num_col = if is_playing { ACCENT_LIGHT } else { TEXT_DIM };
                        ui.add_sized(
                            [col_widths[0], row_height],
                            Label::new(RichText::new(num_str).color(num_col).size(12.0))
                        );

                        // Title
                        let title_color = if is_playing { ACCENT_LIGHT } else { TEXT_PRIMARY };
                        ui.add_sized(
                            [col_widths[1], row_height],
                            Label::new(RichText::new(track.display_title()).color(title_color).size(13.0))
                                .truncate(true)
                        );

                        // Artist
                        ui.add_sized(
                            [col_widths[2], row_height],
                            Label::new(RichText::new(track.display_artist()).color(TEXT_SECONDARY).size(13.0))
                                .truncate(true)
                        );

                        // Album
                        ui.add_sized(
                            [col_widths[3], row_height],
                            Label::new(RichText::new(track.display_album()).color(TEXT_DIM).size(12.0))
                                .truncate(true)
                        );

                        // Duration
                        ui.add_sized(
                            [col_widths[4], row_height],
                            Label::new(RichText::new(track.format_duration()).color(TEXT_DIM).size(12.0))
                        );
                    });
                });

                // Interactions
                if resp.double_clicked() {
                    app.play_from_library(lib_idx);
                }
                if resp.clicked() {
                    app.selected_track_id = Some(track.id.clone());
                }

                // Right-click context menu
                resp.context_menu(|ui| {
                    let track_id = track.id.clone();
                    let track_title = track.display_title().to_string();
                    let track_artist = track.display_artist().to_string();
                    let track_dur = track.format_duration();

                    if ui.button("▶  Play Now").clicked() {
                        app.play_from_library(lib_idx);
                        ui.close_menu();
                    }
                    if ui.button("⏭  Play Next").clicked() {
                        app.enqueue_next(lib_idx);
                        ui.close_menu();
                    }
                    if ui.button("➕  Add to Queue").clicked() {
                        app.enqueue_track(lib_idx);
                        ui.close_menu();
                    }

                    ui.add(Separator::default());

                    // Add to playlist submenu
                    ui.menu_button("🎵  Add to Playlist", |ui| {
                        let playlists: Vec<(String, String)> = app.playlist_store
                            .playlists
                            .iter()
                            .map(|p| (p.id.clone(), p.name.clone()))
                            .collect();
                        for (pid, pname) in playlists {
                            if ui.button(&pname).clicked() {
                                if let Some(pl) = app.playlist_store.get_mut(&pid) {
                                    pl.add_track(track_id.clone());
                                }
                                app.save_playlists();
                                ui.close_menu();
                            }
                        }
                        if app.playlist_store.playlists.is_empty() {
                            ui.colored_label(TEXT_DIM, "No playlists yet");
                        }
                    });
                });
            }
        });
}

fn column_widths(total: f32) -> [f32; 5] {
    let num_w = 36.0;
    let dur_w = 52.0;
    let rest = total - num_w - dur_w - 24.0; // 24 for spacing/padding
    let title_w = rest * 0.40;
    let artist_w = rest * 0.30;
    let album_w = rest * 0.30;
    [num_w, title_w, artist_w, album_w, dur_w]
}

fn header_col(ui: &mut Ui, text: &str, width: f32) {
    ui.add_sized(
        [width, 20.0],
        Label::new(RichText::new(text).color(TEXT_DIM).size(11.0))
    );
}

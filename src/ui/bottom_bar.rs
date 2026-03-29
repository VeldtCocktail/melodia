// melodia/src/ui/bottom_bar.rs

use egui::*;
use crate::theme::*;
use crate::app::MelodiaApp;
use crate::audio::PlaybackState;
use crate::library::format_duration;
use std::time::Duration;

pub fn show(app: &mut MelodiaApp, ui: &mut Ui) {
    let total_w = ui.available_width();

    ui.painter().rect_filled(
        ui.available_rect_before_wrap(),
        0.0,
        BG_PANEL,
    );

    // Top separator line
    ui.add(Separator::default().spacing(0.0));
    ui.add_space(6.0);

    ui.horizontal(|ui| {
        // ── LEFT: Album art + track info ──────────────────────────────────
        let left_w = (total_w * 0.30).max(220.0);
        ui.allocate_ui(vec2(left_w, BOTTOM_BAR_HEIGHT), |ui| {
            ui.horizontal_centered(|ui| {
                // Album art thumbnail
                let art_size = vec2(52.0, 52.0);
                if let Some(tex) = &app.current_album_art_texture {
                    ui.add(Image::new(tex).fit_to_exact_size(art_size).rounding(Rounding::same(4.0)));
                } else {
                    // Placeholder
                    let (rect, _) = ui.allocate_exact_size(art_size, Sense::hover());
                    ui.painter().rect_filled(rect, 4.0, BG_CARD);
                    ui.painter().text(
                        rect.center(),
                        Align2::CENTER_CENTER,
                        "🎵",
                        FontId::proportional(24.0),
                        TEXT_DIM,
                    );
                }

                ui.add_space(8.0);

                ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                    if let Some(item) = app.queue.current_item() {
                        ui.add(
                            Label::new(
                                RichText::new(&item.display_title)
                                    .size(13.0)
                                    .color(TEXT_PRIMARY)
                                    .strong()
                            ).truncate(true)
                        );
                        ui.add(
                            Label::new(
                                RichText::new(&item.display_artist)
                                    .size(11.0)
                                    .color(TEXT_SECONDARY)
                            ).truncate(true)
                        );
                    } else {
                        ui.colored_label(TEXT_DIM, "No track playing");
                    }
                });
            });
        });

        // ── CENTER: Controls + seek bar ───────────────────────────────────
        let center_w = total_w * 0.40;
        ui.allocate_ui(vec2(center_w, BOTTOM_BAR_HEIGHT), |ui| {
            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                ui.add_space(4.0);

                // Playback buttons
                ui.horizontal(|ui| {
                    ui.add_space((center_w - 200.0).max(0.0) / 2.0);

                    // Shuffle
                    let shuffle_color = if app.shuffle { ACCENT_LIGHT } else { TEXT_DIM };
                    if ui.add(
                        Button::new(RichText::new("⇄").size(16.0).color(shuffle_color)).frame(false)
                    ).on_hover_text("Shuffle").clicked() {
                        app.shuffle = !app.shuffle;
                    }

                    ui.add_space(8.0);

                    // Previous
                    if ui.add(
                        Button::new(RichText::new("⏮").size(20.0).color(TEXT_PRIMARY)).frame(false)
                    ).on_hover_text("Previous / Restart").clicked() {
                        app.prev_track();
                    }

                    ui.add_space(8.0);

                    // Play/Pause — large button
                    let play_icon = match app.player.state() {
                        PlaybackState::Playing => "⏸",
                        _ => "▶",
                    };
                    let (play_rect, play_resp) = ui.allocate_exact_size(vec2(40.0, 40.0), Sense::click());
                    ui.painter().circle_filled(play_rect.center(), 20.0, ACCENT);
                    if play_resp.hovered() {
                        ui.painter().circle_stroke(play_rect.center(), 20.0, Stroke::new(2.0, ACCENT_LIGHT));
                    }
                    ui.painter().text(
                        play_rect.center(),
                        Align2::CENTER_CENTER,
                        play_icon,
                        FontId::proportional(18.0),
                        Color32::WHITE,
                    );
                    if play_resp.clicked() {
                        app.toggle_pause();
                    }

                    ui.add_space(8.0);

                    // Next
                    if ui.add(
                        Button::new(RichText::new("⏭").size(20.0).color(TEXT_PRIMARY)).frame(false)
                    ).on_hover_text("Next").clicked() {
                        app.next_track();
                    }

                    ui.add_space(8.0);

                    // Repeat
                    let repeat_color = if app.repeat { ACCENT_LIGHT } else { TEXT_DIM };
                    if ui.add(
                        Button::new(RichText::new("↺").size(16.0).color(repeat_color)).frame(false)
                    ).on_hover_text("Repeat").clicked() {
                        app.repeat = !app.repeat;
                    }
                });

                ui.add_space(4.0);

                // Seek bar + time
                ui.horizontal(|ui| {
                    let pos = app.player.position();
                    let dur = app.player.track_duration();

                    ui.colored_label(TEXT_DIM, RichText::new(format_duration(pos)).size(11.0));
                    ui.add_space(4.0);

                    let progress = app.player.progress();
                    let bar_w = center_w - 120.0;

                    let (bar_rect, bar_resp) = ui.allocate_exact_size(
                        vec2(bar_w.max(100.0), 6.0),
                        Sense::click_and_drag()
                    );

                    // Background
                    ui.painter().rect_filled(bar_rect, 3.0, PROGRESS_BG);
                    // Filled portion
                    let filled_w = bar_rect.width() * progress;
                    let filled_rect = Rect::from_min_size(bar_rect.min, vec2(filled_w, bar_rect.height()));
                    ui.painter().rect_filled(filled_rect, 3.0, ACCENT);

                    // Scrubber knob
                    let knob_x = bar_rect.left() + filled_w;
                    let knob_center = pos2(knob_x, bar_rect.center().y);
                    if bar_resp.hovered() || bar_resp.dragged() {
                        ui.painter().circle_filled(knob_center, 7.0, Color32::WHITE);
                    } else {
                        ui.painter().circle_filled(knob_center, 5.0, ACCENT_LIGHT);
                    }

                    // Seek on click/drag
                    if bar_resp.clicked() || bar_resp.dragged() {
                        if let Some(ptr) = bar_resp.interact_pointer_pos() {
                            let frac = ((ptr.x - bar_rect.left()) / bar_rect.width()).clamp(0.0, 1.0);
                            if let Some(d) = dur {
                                let target = Duration::from_secs_f32(d.as_secs_f32() * frac);
                                app.seek_to(target);
                            }
                        }
                    }

                    ui.add_space(4.0);
                    let dur_str = dur.map(|d| format_duration(d)).unwrap_or_else(|| "--:--".to_string());
                    ui.colored_label(TEXT_DIM, RichText::new(dur_str).size(11.0));
                });
            });
        });

        // ── RIGHT: Volume ─────────────────────────────────────────────────
        let right_w = total_w - left_w - center_w;
        ui.allocate_ui(vec2(right_w, BOTTOM_BAR_HEIGHT), |ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(16.0);

                // Volume slider
                let mut vol = app.player.volume();
                let vol_resp = ui.add(
                    Slider::new(&mut vol, 0.0..=1.5)
                        .show_value(false)
                        .desired_width(80.0)
                );
                if vol_resp.changed() {
                    app.player.set_volume(vol);
                }

                // Volume icon
                let vol_icon = if vol == 0.0 { "🔇" } else if vol < 0.5 { "🔈" } else { "🔊" };
                ui.colored_label(TEXT_SECONDARY, vol_icon);
                ui.add_space(8.0);

                // Queue toggle
                if ui.add(
                    Button::new(RichText::new("☰ Queue").size(12.0).color(TEXT_SECONDARY)).frame(false)
                ).clicked() {
                    app.toggle_queue_panel();
                }
            });
        });
    });
}

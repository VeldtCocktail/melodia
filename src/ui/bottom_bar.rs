// melodia/src/ui/bottom_bar.rs

use egui::*;
use crate::theme::*;
use crate::app::MelodiaApp;
use crate::audio::PlaybackState;
use crate::library::format_duration;
use std::time::Duration;

pub fn show(app: &mut MelodiaApp, ui: &mut Ui) {
    let total_rect = ui.available_rect_before_wrap();
    let total_w = total_rect.width();

    ui.painter().rect_filled(total_rect, 0.0, BG_PANEL);

    // Top separator line
    ui.painter().hline(
        total_rect.x_range(),
        total_rect.top(),
        Stroke::new(1.0, SEPARATOR),
    );

    // Fixed section widths
    let left_w = 320.0;
    let center_w = (total_w - left_w - 220.0).max(280.0);
    let right_w = total_w - left_w - center_w;

    let bar_top = total_rect.top() + 2.0; // small padding below separator
    let bar_height = total_rect.height() - 2.0;

    // ── LEFT: Album art + track info ──────────────────────────────────
    let left_rect = Rect::from_min_size(
        pos2(total_rect.left(), bar_top),
        vec2(left_w, bar_height),
    );
    ui.allocate_ui_at_rect(left_rect, |ui| {
        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
            ui.add_space(8.0);

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
                    "♪",
                    FontId::proportional(24.0),
                    TEXT_DIM,
                );
            }

            ui.add_space(8.0);

            // Track info — vertically centered
            let info_width = left_w - 52.0 - 32.0; // art + spacing
            ui.allocate_ui(vec2(info_width, art_size.y), |ui| {
                ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                    ui.add_space(6.0);
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
    });

    // ── CENTER: Controls + seek bar (FIXED POSITION) ──────────────────
    let center_rect = Rect::from_min_size(
        pos2(total_rect.left() + left_w, bar_top),
        vec2(center_w, bar_height),
    );
    ui.allocate_ui_at_rect(center_rect, |ui| {
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            ui.add_space(8.0);

            // Playback buttons — fixed width container
            let buttons_width = 240.0;
            ui.allocate_ui(vec2(buttons_width, 40.0), |ui| {
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    let btn_spacing = (buttons_width - 40.0 - 20.0 * 2.0 - 16.0 * 2.0) / 4.0;
                    let btn_spacing = btn_spacing.max(8.0).min(16.0);

                    // Shuffle — draw custom icon
                    let shuffle_color = if app.shuffle { ACCENT_LIGHT } else { TEXT_DIM };
                    let (shuf_rect, shuf_resp) = ui.allocate_exact_size(vec2(20.0, 20.0), Sense::click());
                    paint_shuffle_icon(ui.painter(), shuf_rect, shuffle_color);
                    if shuf_resp.on_hover_text("Shuffle").clicked() {
                        app.shuffle = !app.shuffle;
                    }

                    ui.add_space(btn_spacing);

                    // Previous
                    let (prev_rect, prev_resp) = ui.allocate_exact_size(vec2(20.0, 20.0), Sense::click());
                    paint_prev_icon(ui.painter(), prev_rect, TEXT_PRIMARY);
                    if prev_resp.on_hover_text("Previous / Restart").clicked() {
                        app.prev_track();
                    }

                    ui.add_space(btn_spacing);

                    // Play/Pause — large circle button
                    let (play_rect, play_resp) = ui.allocate_exact_size(vec2(40.0, 40.0), Sense::click());
                    ui.painter().circle_filled(play_rect.center(), 20.0, ACCENT);
                    if play_resp.hovered() {
                        ui.painter().circle_stroke(play_rect.center(), 20.0, Stroke::new(2.0, ACCENT_LIGHT));
                    }
                    if app.player.state() == PlaybackState::Playing {
                        paint_pause_icon(ui.painter(), play_rect.center(), Color32::WHITE);
                    } else {
                        paint_play_icon(ui.painter(), play_rect.center(), Color32::WHITE);
                    }
                    if play_resp.clicked() {
                        app.toggle_pause();
                    }

                    ui.add_space(btn_spacing);

                    // Next
                    let (next_rect, next_resp) = ui.allocate_exact_size(vec2(20.0, 20.0), Sense::click());
                    paint_next_icon(ui.painter(), next_rect, TEXT_PRIMARY);
                    if next_resp.on_hover_text("Next").clicked() {
                        app.next_track();
                    }

                    ui.add_space(btn_spacing);

                    // Repeat — draw custom icon
                    let repeat_color = if app.repeat { ACCENT_LIGHT } else { TEXT_DIM };
                    let (rep_rect, rep_resp) = ui.allocate_exact_size(vec2(20.0, 20.0), Sense::click());
                    paint_repeat_icon(ui.painter(), rep_rect, repeat_color);
                    if rep_resp.on_hover_text("Repeat").clicked() {
                        app.repeat = !app.repeat;
                    }
                });
            });

            ui.add_space(2.0);

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
    let right_rect = Rect::from_min_size(
        pos2(total_rect.left() + left_w + center_w, bar_top),
        vec2(right_w, bar_height),
    );
    ui.allocate_ui_at_rect(right_rect, |ui| {
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add_space(16.0);

            // Volume slider
            let mut vol = app.player.volume();
            let vol_resp = ui.add(
                Slider::new(&mut vol, 0.0..=1.5)
                    .show_value(false)
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

    // Allocate the full rect so egui knows the space is used
    ui.allocate_rect(total_rect, Sense::hover());
}

// ── Custom icon painters ─────────────────────────────────────────────────────

/// Draw a play triangle, properly centered in the circle
fn paint_play_icon(painter: &Painter, center: Pos2, color: Color32) {
    let size = 9.0;
    // Shift right slightly to visually center the triangle (triangles look left-heavy)
    let cx = center.x + 1.0;
    let cy = center.y;
    let points = vec![
        pos2(cx - size * 0.5, cy - size),
        pos2(cx + size * 0.7, cy),
        pos2(cx - size * 0.5, cy + size),
    ];
    painter.add(Shape::convex_polygon(points, color, Stroke::NONE));
}

/// Draw pause icon (two vertical bars)
fn paint_pause_icon(painter: &Painter, center: Pos2, color: Color32) {
    let bar_w = 3.5;
    let bar_h = 10.0;
    let gap = 3.0;
    // Left bar
    painter.rect_filled(
        Rect::from_center_size(pos2(center.x - gap, center.y), vec2(bar_w, bar_h * 2.0)),
        1.0, color,
    );
    // Right bar
    painter.rect_filled(
        Rect::from_center_size(pos2(center.x + gap, center.y), vec2(bar_w, bar_h * 2.0)),
        1.0, color,
    );
}

/// Draw previous track icon (bar + left-pointing triangle)
fn paint_prev_icon(painter: &Painter, rect: Rect, color: Color32) {
    let center = rect.center();
    let s = 6.0;
    // Bar on left
    painter.rect_filled(
        Rect::from_center_size(pos2(center.x - s, center.y), vec2(2.0, s * 2.0)),
        0.0, color,
    );
    // Triangle pointing left
    let points = vec![
        pos2(center.x + s, center.y - s),
        pos2(center.x - s + 2.0, center.y),
        pos2(center.x + s, center.y + s),
    ];
    painter.add(Shape::convex_polygon(points, color, Stroke::NONE));
}

/// Draw next track icon (right-pointing triangle + bar)
fn paint_next_icon(painter: &Painter, rect: Rect, color: Color32) {
    let center = rect.center();
    let s = 6.0;
    // Triangle pointing right
    let points = vec![
        pos2(center.x - s, center.y - s),
        pos2(center.x + s - 2.0, center.y),
        pos2(center.x - s, center.y + s),
    ];
    painter.add(Shape::convex_polygon(points, color, Stroke::NONE));
    // Bar on right
    painter.rect_filled(
        Rect::from_center_size(pos2(center.x + s, center.y), vec2(2.0, s * 2.0)),
        0.0, color,
    );
}

/// Draw shuffle icon (two crossing arrows)
fn paint_shuffle_icon(painter: &Painter, rect: Rect, color: Color32) {
    let c = rect.center();
    let w = 8.0;
    let h = 5.0;
    let stroke = Stroke::new(2.0, color);
    // Two crossing lines
    painter.line_segment([pos2(c.x - w, c.y - h), pos2(c.x + w, c.y + h)], stroke);
    painter.line_segment([pos2(c.x - w, c.y + h), pos2(c.x + w, c.y - h)], stroke);
    // Arrowhead top-right
    let ar = 4.0;
    painter.line_segment([pos2(c.x + w, c.y - h), pos2(c.x + w - ar, c.y - h - 0.5)], stroke);
    painter.line_segment([pos2(c.x + w, c.y - h), pos2(c.x + w - ar, c.y - h + ar)], stroke);
    // Arrowhead bottom-right
    painter.line_segment([pos2(c.x + w, c.y + h), pos2(c.x + w - ar, c.y + h + 0.5)], stroke);
    painter.line_segment([pos2(c.x + w, c.y + h), pos2(c.x + w - ar, c.y + h - ar)], stroke);
}

/// Draw repeat icon (rectangle with arrows)
fn paint_repeat_icon(painter: &Painter, rect: Rect, color: Color32) {
    let c = rect.center();
    let w = 7.0;
    let h = 4.0;
    let stroke = Stroke::new(1.5, color);

    // Rounded rectangle path (top-left → top-right → bottom-right → bottom-left)
    painter.line_segment([pos2(c.x - w, c.y - h), pos2(c.x + w, c.y - h)], stroke);
    painter.line_segment([pos2(c.x + w, c.y - h), pos2(c.x + w, c.y + h)], stroke);
    painter.line_segment([pos2(c.x + w, c.y + h), pos2(c.x - w, c.y + h)], stroke);
    painter.line_segment([pos2(c.x - w, c.y + h), pos2(c.x - w, c.y - h)], stroke);

    // Arrow on top edge pointing right
    let ar = 3.0;
    painter.line_segment([pos2(c.x + 2.0, c.y - h), pos2(c.x + 2.0 - ar, c.y - h - ar)], stroke);
    painter.line_segment([pos2(c.x + 2.0, c.y - h), pos2(c.x + 2.0 - ar, c.y - h + ar)], stroke);
}

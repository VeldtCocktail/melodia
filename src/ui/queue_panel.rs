// melodia/src/ui/queue_panel.rs

use egui::*;
use crate::theme::*;
use crate::app::MelodiaApp;

pub fn show(app: &mut MelodiaApp, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.colored_label(TEXT_PRIMARY, RichText::new("Queue").size(20.0).strong());
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if !app.queue.is_empty() {
                if ui.add(
                    Button::new(RichText::new("🗑 Clear").size(12.0).color(TEXT_SECONDARY))
                        .frame(false)
                ).clicked() {
                    app.queue.clear();
                    app.stop_playback();
                }
            }
        });
    });
    ui.add_space(8.0);

    if app.queue.is_empty() {
        ui.add_space(40.0);
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            ui.colored_label(TEXT_DIM, RichText::new("📋").size(48.0));
            ui.add_space(8.0);
            ui.colored_label(TEXT_SECONDARY, "Queue is empty.");
            ui.add_space(4.0);
            ui.colored_label(TEXT_DIM, "Double-click a track or right-click → Add to Queue.");
        });
        return;
    }

    let row_height = 52.0;
    let queue_len = app.queue.len();

    // Track drag state
    let mut drag_src: Option<usize> = None;
    let mut drag_dst: Option<usize> = None;

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for i in 0..queue_len {
                let item = match app.queue.items.get(i) {
                    Some(it) => it.clone(),
                    None => break,
                };
                let is_current = app.queue.current_index == Some(i);

                let bg = if is_current { BG_SELECTED_DIM } else { BG_PANEL };

                let row_id = Id::new("queue_row").with(i);
                let row_rect = ui.available_rect_before_wrap();
                let row_rect = Rect::from_min_size(row_rect.min, vec2(row_rect.width(), row_height));

                // Drop target highlight
                if ui.memory(|m| m.is_being_dragged(row_id)) {
                    drag_src = Some(i);
                }

                let resp = ui.allocate_rect(row_rect, Sense::click_and_drag());
                ui.painter().rect_filled(row_rect, 4.0, bg);

                if resp.hovered() && !is_current {
                    ui.painter().rect_filled(row_rect, 4.0, BG_HOVER);
                }

                // Drag handle + content
                ui.allocate_ui_at_rect(row_rect, |ui| {
                    ui.horizontal_centered(|ui| {
                        ui.add_space(8.0);

                        // Drag handle
                        ui.colored_label(TEXT_DIM, "⠿");
                        ui.add_space(4.0);

                        // Playing indicator
                        if is_current {
                            ui.colored_label(ACCENT_LIGHT, "▶");
                        } else {
                            ui.colored_label(TEXT_DIM, format!("{}", i + 1));
                        }
                        ui.add_space(8.0);

                        // Track info (vertical stack)
                        ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                            let title_color = if is_current { ACCENT_LIGHT } else { TEXT_PRIMARY };
                            ui.add(Label::new(
                                RichText::new(&item.display_title).color(title_color).size(13.0)
                            ).truncate(true));
                            ui.horizontal(|ui| {
                                ui.colored_label(TEXT_SECONDARY, RichText::new(&item.display_artist).size(11.0));
                                ui.colored_label(TEXT_DIM, " · ");
                                ui.colored_label(TEXT_DIM, RichText::new(&item.duration_str).size(11.0));
                            });
                        });

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.add_space(8.0);
                            // Remove button
                            if ui.add(
                                Button::new(RichText::new("✕").size(12.0).color(TEXT_DIM))
                                    .frame(false)
                            ).clicked() {
                                app.queue.remove(i);
                                return;
                            }
                        });
                    });
                });

                // Double-click to jump to this track
                if resp.double_clicked() {
                    app.jump_to_queue_item(i);
                }

                // Drag detection for reorder
                if resp.drag_started() {
                    drag_src = Some(i);
                }
                if resp.hovered() && ui.memory(|m| m.is_anything_being_dragged()) {
                    drag_dst = Some(i);
                    // Draw drop line
                    let line_y = row_rect.top();
                    ui.painter().hline(
                        row_rect.x_range(),
                        line_y,
                        Stroke::new(2.0, ACCENT),
                    );
                }

                ui.add_space(2.0);
            }
        });

    // Apply drag-and-drop reorder
    if let (Some(src), Some(dst)) = (drag_src, drag_dst) {
        if src != dst && ui.input(|i| i.pointer.any_released()) {
            app.queue.move_item(src, dst);
        }
    }
}

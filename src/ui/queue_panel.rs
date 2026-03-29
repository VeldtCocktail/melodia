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

    let row_height  = 52.0;
    let queue_len   = app.queue.len();

    // Pre-extract row data (avoids borrow on app inside the closure)
    struct RowData {
        title:      String,
        artist:     String,
        duration:   String,
        is_current: bool,
    }

    let rows: Vec<RowData> = (0..queue_len).map(|i| {
        let item       = &app.queue.items[i];
        let is_current = app.queue.current_index == Some(i);
        RowData {
            title:    item.display_title.clone(),
            artist:   item.display_artist.clone(),
            duration: item.duration_str.clone(),
            is_current,
        }
    }).collect();

    let mut action_jump:   Option<usize> = None;
    let mut action_remove: Option<usize> = None;
    let mut action_move:   Option<(usize, usize)> = None;

    // Track drag state outside scroll area
    let drag_id_base = Id::new("queue_drag");

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let pointer_pos   = ui.ctx().pointer_hover_pos();
            let dragged_id    = ui.ctx().dragged_id();

            for i in 0..rows.len() {
                let row        = &rows[i];
                let bg         = if row.is_current { BG_SELECTED_DIM } else { BG_PANEL };
                let row_id     = drag_id_base.with(i);

                let row_rect   = ui.available_rect_before_wrap();
                let row_rect   = Rect::from_min_size(row_rect.min, vec2(row_rect.width(), row_height));

                let resp = ui.allocate_rect(row_rect, Sense::click_and_drag());
                ui.painter().rect_filled(row_rect, 4.0, bg);
                if resp.hovered() && !row.is_current {
                    ui.painter().rect_filled(row_rect, 4.0, BG_HOVER);
                }

                // Drop-target highlight
                let is_dragging_something = dragged_id.is_some();
                if is_dragging_something && resp.hovered() {
                    ui.painter().hline(
                        row_rect.x_range(),
                        row_rect.top(),
                        Stroke::new(2.0, ACCENT),
                    );
                }

                ui.allocate_ui_at_rect(row_rect, |ui| {
                    ui.horizontal_centered(|ui| {
                        ui.add_space(8.0);
                        ui.colored_label(TEXT_DIM, "⠿");
                        ui.add_space(4.0);

                        if row.is_current {
                            ui.colored_label(ACCENT_LIGHT, "▶");
                        } else {
                            ui.colored_label(TEXT_DIM, format!("{}", i + 1));
                        }
                        ui.add_space(8.0);

                        ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                            let title_color = if row.is_current { ACCENT_LIGHT } else { TEXT_PRIMARY };
                            ui.add(Label::new(
                                RichText::new(&row.title).color(title_color).size(13.0)
                            ).truncate(true));
                            ui.horizontal(|ui| {
                                ui.colored_label(TEXT_SECONDARY, RichText::new(&row.artist).size(11.0));
                                ui.colored_label(TEXT_DIM, " · ");
                                ui.colored_label(TEXT_DIM, RichText::new(&row.duration).size(11.0));
                            });
                        });

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.add_space(8.0);
                            if ui.add(
                                Button::new(RichText::new("✕").size(12.0).color(TEXT_DIM))
                                    .frame(false)
                            ).clicked() {
                                action_remove = Some(i);
                            }
                        });
                    });
                });

                if resp.double_clicked() { action_jump = Some(i); }

                // Detect drag start / drop
                if resp.drag_started() {
                    ui.ctx().set_dragged_id(row_id);
                }
                if resp.drag_stopped() {
                    // Find the row we're hovering over
                    if let Some(pos) = pointer_pos {
                        // We'll resolve after loop; store drag src in ctx memory
                        ui.ctx().data_mut(|d| d.insert_temp::<usize>(Id::new("drag_src"), i));
                    }
                }
                if dragged_id == Some(row_id) && resp.drag_stopped() {
                    if let Some(pos) = pointer_pos {
                        ui.ctx().data_mut(|d| d.insert_temp::<usize>(Id::new("drag_src"), i));
                    }
                }
            }

            // Resolve drag-and-drop after all rows are rendered
            let pointer_released = ui.input(|inp| inp.pointer.any_released());
            if pointer_released {
                let src = ui.ctx().data_mut(|d| d.remove_temp::<usize>(Id::new("drag_src")));
                if let (Some(src_i), Some(pos)) = (src, pointer_pos) {
                    // Find which row the pointer is over
                    // Approximate: each row is row_height tall in the scroll area
                    // Use relative position within the scroll
                    for j in 0..rows.len() {
                        let check_rect = Rect::from_min_size(
                            pos2(0.0, j as f32 * (row_height + 2.0)),
                            vec2(f32::INFINITY, row_height + 2.0),
                        );
                        if pos.y > check_rect.min.y && pos.y <= check_rect.max.y {
                            if src_i != j {
                                action_move = Some((src_i, j));
                            }
                            break;
                        }
                    }
                }
            }
        });

    // Apply deferred mutations
    if let Some(i) = action_remove { app.queue.remove(i); }
    if let Some(i) = action_jump   { app.jump_to_queue_item(i); }
    if let Some((f, t)) = action_move { app.queue.move_item(f, t); }
}

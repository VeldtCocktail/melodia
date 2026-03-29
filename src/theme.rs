// melodia/src/theme.rs
// Dark theme colors and styling constants

use egui::{Color32, FontId, FontFamily, Rounding, Stroke, Style, Visuals};

pub const BG_DARK: Color32 = Color32::from_rgb(18, 18, 18);
pub const BG_PANEL: Color32 = Color32::from_rgb(28, 28, 28);
pub const BG_CARD: Color32 = Color32::from_rgb(38, 38, 38);
pub const BG_HOVER: Color32 = Color32::from_rgb(48, 48, 48);
pub const BG_SELECTED: Color32 = Color32::from_rgb(138, 43, 226);      // purple
pub const BG_SELECTED_DIM: Color32 = Color32::from_rgb(80, 20, 130);   // dark purple
pub const ACCENT: Color32 = Color32::from_rgb(138, 43, 226);
pub const ACCENT_LIGHT: Color32 = Color32::from_rgb(180, 100, 255);
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(240, 240, 240);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(160, 160, 160);
pub const TEXT_DIM: Color32 = Color32::from_rgb(100, 100, 100);
pub const SEPARATOR: Color32 = Color32::from_rgb(50, 50, 50);
pub const PROGRESS_BG: Color32 = Color32::from_rgb(55, 55, 55);
pub const DANGER: Color32 = Color32::from_rgb(220, 60, 60);

pub const SIDEBAR_WIDTH: f32 = 220.0;
pub const BOTTOM_BAR_HEIGHT: f32 = 90.0;
pub const ALBUM_ART_SIZE: f32 = 64.0;
pub const ALBUM_ART_LARGE: f32 = 240.0;

pub fn apply_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    style.visuals = Visuals::dark();
    style.visuals.panel_fill = BG_PANEL;
    style.visuals.window_fill = BG_DARK;
    style.visuals.faint_bg_color = BG_CARD;
    style.visuals.extreme_bg_color = BG_DARK;
    style.visuals.override_text_color = Some(TEXT_PRIMARY);
    style.visuals.selection.bg_fill = ACCENT;
    style.visuals.selection.stroke = Stroke::new(1.0, ACCENT_LIGHT);
    style.visuals.widgets.noninteractive.bg_fill = BG_CARD;
    style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_SECONDARY);
    style.visuals.widgets.noninteractive.rounding = Rounding::same(6.0);
    style.visuals.widgets.inactive.bg_fill = BG_CARD;
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_SECONDARY);
    style.visuals.widgets.inactive.rounding = Rounding::same(6.0);
    style.visuals.widgets.hovered.bg_fill = BG_HOVER;
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
    style.visuals.widgets.active.bg_fill = ACCENT;
    style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    style.visuals.widgets.open.bg_fill = BG_CARD;

    style.spacing.item_spacing = egui::vec2(8.0, 4.0);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);

    ctx.set_style(style);
}

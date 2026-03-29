// melodia/src/ui/album_art.rs
// Large album art view shown when clicking on the art thumbnail

use egui::*;
use crate::theme::*;
use crate::app::MelodiaApp;

pub fn show_large(app: &MelodiaApp, ui: &mut Ui) {
    if let Some(tex) = &app.current_album_art_texture {
        let size = ui.available_size();
        let sq = size.x.min(size.y).min(ALBUM_ART_LARGE);
        ui.centered_and_justified(|ui| {
            ui.add(
                Image::new(tex)
                    .fit_to_exact_size(vec2(sq, sq))
                    .rounding(Rounding::same(8.0))
            );
        });
    } else {
        // No album art — show placeholder
        let (rect, _) = ui.allocate_exact_size(
            vec2(ALBUM_ART_LARGE, ALBUM_ART_LARGE),
            Sense::hover()
        );
        ui.painter().rect_filled(rect, 8.0, BG_CARD);
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            "🎵",
            FontId::proportional(64.0),
            TEXT_DIM,
        );
    }
}

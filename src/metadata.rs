// melodia/src/metadata.rs
// Reads ID3/Vorbis/FLAC tags and extracts album art

use lofty::prelude::*;
use lofty::probe::Probe;
use lofty::picture::PictureType;
use std::path::Path;
use std::time::Duration;
use image::DynamicImage;

#[derive(Debug, Clone)]
pub struct TrackMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub track_number: Option<u32>,
    pub duration: Option<Duration>,
    pub year: Option<u32>,
    pub genre: Option<String>,
    pub album_art: Option<Vec<u8>>, // raw JPEG/PNG bytes
}

impl Default for TrackMetadata {
    fn default() -> Self {
        Self {
            title: None,
            artist: None,
            album: None,
            track_number: None,
            duration: None,
            year: None,
            genre: None,
            album_art: None,
        }
    }
}

/// Read metadata from an audio file. Never panics; returns defaults on error.
pub fn read_metadata(path: &Path) -> TrackMetadata {
    let mut meta = TrackMetadata::default();

    let tagged_file = match Probe::open(path)
        .and_then(|p| p.read())
    {
        Ok(f) => f,
        Err(_) => return meta,
    };

    // Duration
    meta.duration = Some(tagged_file.properties().duration());

    // Tags
    if let Some(tag) = tagged_file.primary_tag() {
        meta.title = tag.title().map(|s| s.to_string());
        meta.artist = tag.artist().map(|s| s.to_string());
        meta.album = tag.album().map(|s| s.to_string());
        meta.track_number = tag.track();
        meta.year = tag.year();
        meta.genre = tag.genre().map(|s| s.to_string());

        // Album art: prefer front cover, fall back to any picture
        let picture = tag
            .pictures()
            .iter()
            .find(|p| p.pic_type() == PictureType::CoverFront)
            .or_else(|| tag.pictures().first());

        if let Some(pic) = picture {
            meta.album_art = Some(pic.data().to_vec());
        }
    }

    // If no primary tag, try any tag
    if meta.title.is_none() {
        if let Some(tag) = tagged_file.tags().first() {
            meta.title = tag.title().map(|s| s.to_string());
            meta.artist = tag.artist().map(|s| s.to_string());
            meta.album = tag.album().map(|s| s.to_string());
            meta.track_number = tag.track();
            meta.year = tag.year();
            meta.genre = tag.genre().map(|s| s.to_string());

            if meta.album_art.is_none() {
                if let Some(pic) = tag.pictures().first() {
                    meta.album_art = Some(pic.data().to_vec());
                }
            }
        }
    }

    meta
}

/// Decode raw image bytes to an egui-compatible image.
pub fn decode_album_art(data: &[u8]) -> Option<DynamicImage> {
    image::load_from_memory(data).ok()
}

/// Convert a DynamicImage to egui ColorImage.
pub fn to_egui_image(img: DynamicImage) -> egui::ColorImage {
    let img = img.to_rgba8();
    let size = [img.width() as usize, img.height() as usize];
    let pixels = img
        .pixels()
        .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
        .collect();
    egui::ColorImage { size, pixels }
}

use egui;

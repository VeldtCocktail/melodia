// melodia/src/library.rs
// Track model and folder scanning

use std::path::{Path, PathBuf};
use std::time::Duration;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use walkdir::WalkDir;
use crate::metadata;

pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "mp3", "flac", "ogg", "wav", "aac", "m4a", "opus", "wma",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub path: PathBuf,
    pub file_name: String,

    // Metadata (populated lazily or eagerly)
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub track_number: Option<u32>,
    pub duration: Option<Duration>,
    pub year: Option<u32>,
    pub genre: Option<String>,

    #[serde(skip)]
    pub album_art_bytes: Option<Vec<u8>>,
    pub metadata_loaded: bool,
}

impl Track {
    pub fn from_path(path: PathBuf) -> Self {
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        Track {
            id: Uuid::new_v4().to_string(),
            path,
            file_name,
            title: None,
            artist: None,
            album: None,
            track_number: None,
            duration: None,
            year: None,
            genre: None,
            album_art_bytes: None,
            metadata_loaded: false,
        }
    }

    /// Eagerly load metadata from disk.
    pub fn load_metadata(&mut self) {
        if self.metadata_loaded {
            return;
        }
        let meta = metadata::read_metadata(&self.path);
        self.title = meta.title;
        self.artist = meta.artist;
        self.album = meta.album;
        self.track_number = meta.track_number;
        self.duration = meta.duration;
        self.year = meta.year;
        self.genre = meta.genre;
        self.album_art_bytes = meta.album_art;
        self.metadata_loaded = true;
    }

    /// Display title: tag title → filename without extension.
    pub fn display_title(&self) -> &str {
        if let Some(ref t) = self.title {
            return t.as_str();
        }
        // Strip extension from file_name
        if let Some(stem) = Path::new(&self.file_name).file_stem() {
            return stem.to_str().unwrap_or(&self.file_name);
        }
        &self.file_name
    }

    pub fn display_artist(&self) -> &str {
        self.artist.as_deref().unwrap_or("Unknown Artist")
    }

    pub fn display_album(&self) -> &str {
        self.album.as_deref().unwrap_or("Unknown Album")
    }

    pub fn format_duration(&self) -> String {
        match self.duration {
            Some(d) => format_duration(d),
            None => "--:--".to_string(),
        }
    }
}

pub fn format_duration(d: Duration) -> String {
    let total_secs = d.as_secs();
    let mins = total_secs / 60;
    let secs = total_secs % 60;
    if mins >= 60 {
        let hours = mins / 60;
        let mins = mins % 60;
        format!("{:02}:{:02}:{:02}", hours, mins, secs)
    } else {
        format!("{:02}:{:02}", mins, secs)
    }
}

/// Scan a folder recursively and return all audio tracks.
pub fn scan_folder(folder: &Path) -> Vec<Track> {
    let mut tracks = Vec::new();

    for entry in WalkDir::new(folder)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path().to_path_buf();
        if let Some(ext) = path.extension() {
            let ext_lower = ext.to_string_lossy().to_lowercase();
            if SUPPORTED_EXTENSIONS.contains(&ext_lower.as_str()) {
                tracks.push(Track::from_path(path));
            }
        }
    }

    // Sort by folder/filename for consistent listing
    tracks.sort_by(|a, b| a.path.cmp(&b.path));
    tracks
}

/// Load metadata for all tracks (used in background thread).
pub fn load_all_metadata(tracks: &mut Vec<Track>) {
    for track in tracks.iter_mut() {
        track.load_metadata();
    }
}

// melodia/src/config.rs
// App configuration — window state, last folder, volume, etc.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub last_folder: Option<PathBuf>,
    pub volume: f32,
    pub repeat: bool,
    pub shuffle: bool,
    pub last_playlist_id: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            last_folder: None,
            volume: 0.8,
            repeat: false,
            shuffle: false,
            last_playlist_id: None,
        }
    }
}

impl Config {
    fn config_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Melodia")
            .join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(data) = std::fs::read_to_string(&path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, json);
        }
    }

    pub fn playlists_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Melodia")
            .join("playlists.json")
    }
}

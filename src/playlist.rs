// melodia/src/playlist.rs
// Playlist creation, management, and persistence

use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    /// Track IDs (references into library)
    pub track_ids: Vec<String>,
    pub created_at: u64,
}

impl Playlist {
    pub fn new(name: impl Into<String>) -> Self {
        Playlist {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            track_ids: Vec::new(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    pub fn add_track(&mut self, track_id: String) {
        if !self.track_ids.contains(&track_id) {
            self.track_ids.push(track_id);
        }
    }

    pub fn remove_track(&mut self, track_id: &str) {
        self.track_ids.retain(|id| id != track_id);
    }

    pub fn move_track(&mut self, from: usize, to: usize) {
        if from < self.track_ids.len() && to < self.track_ids.len() {
            let item = self.track_ids.remove(from);
            self.track_ids.insert(to, item);
        }
    }

    pub fn track_count(&self) -> usize {
        self.track_ids.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlaylistStore {
    pub playlists: Vec<Playlist>,
}

impl PlaylistStore {
    pub fn add_playlist(&mut self, playlist: Playlist) {
        self.playlists.push(playlist);
    }

    pub fn remove_playlist(&mut self, id: &str) {
        self.playlists.retain(|p| p.id != id);
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Playlist> {
        self.playlists.iter_mut().find(|p| p.id == id)
    }

    pub fn get(&self, id: &str) -> Option<&Playlist> {
        self.playlists.iter().find(|p| p.id == id)
    }

    pub fn save(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load(path: &PathBuf) -> Self {
        if let Ok(data) = std::fs::read_to_string(path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Self::default()
        }
    }
}

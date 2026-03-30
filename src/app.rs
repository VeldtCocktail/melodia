// melodia/src/app.rs
// Central application state, logic, and eframe integration

use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use egui::{Context, TextureHandle, TextureOptions};

use crate::audio::{AudioPlayer, PlaybackState};
use crate::config::Config;
use crate::library::{self, Track};
use crate::metadata;
use crate::playlist::PlaylistStore;
use crate::queue::{Queue, QueueItem};
use crate::theme;
use crate::ui;

// ── Panel enum ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Panel {
    Library,
    Queue,
    Playlist(String),
}

// ── Background metadata loading ───────────────────────────────────────────────

struct MetadataResult {
    tracks: Vec<Track>,
}

// ── Main app struct ───────────────────────────────────────────────────────────

pub struct MelodiaApp {
    // Audio
    pub player: AudioPlayer,

    // Library
    pub library: Vec<Track>,
    pub current_folder: Option<PathBuf>,
    pub loading_metadata: bool,
    metadata_rx: Option<std::sync::mpsc::Receiver<MetadataResult>>,

    // Playback state
    pub queue: Queue,
    pub current_track_path: Option<PathBuf>,
    pub shuffle: bool,
    pub repeat: bool,
    pub shuffle_history: Vec<usize>,  // for un-shuffle
    pub playback_history: Vec<usize>, // Stack for backward navigation
    pub forward_history: Vec<usize>,  // Stack for forward navigation

    // Back-button logic: if >10s elapsed, restart; else go to previous
    pub track_start_time: Option<Instant>,

    // UI state
    pub active_panel: Panel,
    pub selected_track_id: Option<String>,
    pub search_query: String,
    pub show_new_playlist_dialog: bool,
    pub new_playlist_name: String,
    pub show_queue_panel: bool,

    // Album art texture (for current track)
    pub current_album_art_texture: Option<TextureHandle>,
    current_album_art_track_id: Option<String>,

    // Playlists
    pub playlist_store: PlaylistStore,

    // Config
    pub config: Config,

    // Frame counter for periodic tasks
    frame_count: u64,
}

impl MelodiaApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply_theme(&cc.egui_ctx);

        // Load custom font if desired (uses default egui fonts otherwise)
        let config = Config::load();
        let playlist_store = PlaylistStore::load(&Config::playlists_path());

        let player = AudioPlayer::new().expect("Failed to initialize audio output");
        player.set_volume(config.volume);

        let mut app = Self {
            player,
            library: Vec::new(),
            current_folder: config.last_folder.clone(),
            loading_metadata: false,
            metadata_rx: None,
            queue: Queue::new(),
            current_track_path: None,
            shuffle: config.shuffle,
            repeat: config.repeat,
            shuffle_history: Vec::new(),
            playback_history: Vec::new(),
            forward_history: Vec::new(),
            track_start_time: None,
            active_panel: Panel::Library,
            selected_track_id: None,
            search_query: String::new(),
            show_new_playlist_dialog: false,
            new_playlist_name: String::new(),
            show_queue_panel: false,
            current_album_art_texture: None,
            current_album_art_track_id: None,
            playlist_store,
            config,
            frame_count: 0,
        };

        // Auto-load last folder
        if let Some(ref folder) = app.current_folder.clone() {
            let folder = folder.clone();
            app.load_folder(&folder);
        }

        app
    }

    // ── Folder loading ────────────────────────────────────────────────────

    pub fn open_folder_dialog(&mut self) {
        // Use rfd (Rusty File Dialog) on both platforms
        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
            self.load_folder(&folder);
        }
    }

    pub fn load_folder(&mut self, folder: &PathBuf) {
        self.current_folder = Some(folder.clone());
        self.config.last_folder = Some(folder.clone());
        self.config.save();

        // Quick scan (no metadata yet)
        let mut tracks = library::scan_folder(folder);
        self.library = tracks.clone();
        self.loading_metadata = true;

        // Spawn background thread to load metadata
        let (tx, rx) = std::sync::mpsc::channel();
        self.metadata_rx = Some(rx);

        thread::spawn(move || {
            library::load_all_metadata(&mut tracks);
            let _ = tx.send(MetadataResult { tracks });
        });
    }

    // ── Playback ──────────────────────────────────────────────────────────

    /// Play from library at given index; replaces queue with full library starting there.
    pub fn play_from_library(&mut self, lib_idx: usize) {
        let items: Vec<QueueItem> = self
            .library
            .iter()
            .map(|t| QueueItem {
                track_id: t.id.clone(),
                display_title: t.display_title().to_string(),
                display_artist: t.display_artist().to_string(),
                duration_str: t.format_duration(),
            })
            .collect();
        self.queue.set(items, lib_idx);
        self.play_current_queue_item();
    }

    /// Enqueue a track at the end.
    pub fn enqueue_track(&mut self, lib_idx: usize) {
        if let Some(t) = self.library.get(lib_idx) {
            self.queue.enqueue(QueueItem {
                track_id: t.id.clone(),
                display_title: t.display_title().to_string(),
                display_artist: t.display_artist().to_string(),
                duration_str: t.format_duration(),
            });
        }
    }

    /// Insert a track right after the current one.
    pub fn enqueue_next(&mut self, lib_idx: usize) {
        if let Some(t) = self.library.get(lib_idx) {
            self.queue.enqueue_next(QueueItem {
                track_id: t.id.clone(),
                display_title: t.display_title().to_string(),
                display_artist: t.display_artist().to_string(),
                duration_str: t.format_duration(),
            });
        }
    }

    /// Play a whole playlist (replaces queue).
    pub fn play_playlist(&mut self, playlist_id: &str) {
        self.play_playlist_from(playlist_id, 0);
    }

    pub fn play_playlist_from(&mut self, playlist_id: &str, start: usize) {
        let track_ids = match self.playlist_store.get(playlist_id) {
            Some(pl) => pl.track_ids.clone(),
            None => return,
        };

        let items: Vec<QueueItem> = track_ids
            .iter()
            .filter_map(|tid| {
                self.library
                    .iter()
                    .find(|t| &t.id == tid)
                    .map(|t| QueueItem {
                        track_id: t.id.clone(),
                        display_title: t.display_title().to_string(),
                        display_artist: t.display_artist().to_string(),
                        duration_str: t.format_duration(),
                    })
            })
            .collect();

        if items.is_empty() {
            return;
        }
        self.queue.set(items, start);
        self.play_current_queue_item();
    }

    pub fn play_current_queue_item(&mut self) {
        let track_id = match self.queue.current_track_id() {
            Some(id) => id.to_string(),
            None => return,
        };

        let track = match self.library.iter().find(|t| t.id == track_id) {
            Some(t) => t.clone(),
            None => return,
        };

        // Push current queue index to playback history before playing
        if let Some(current_idx) = self.queue.current_index {
            self.playback_history.push(current_idx);
            // Clear forward history when playing a new track
            self.forward_history.clear();
        }

        if let Err(e) = self.player.play(&track.path, track.duration) {
            eprintln!("Playback error: {e}");
            return;
        }

        self.current_track_path = Some(track.path.clone());
        self.track_start_time = Some(Instant::now());
        self.update_album_art_for_track(&track);
    }

    pub fn toggle_pause(&mut self) {
        self.player.toggle_pause();
    }

    pub fn stop_playback(&mut self) {
        self.player.stop();
        self.current_track_path = None;
        self.track_start_time = None;
    }

    pub fn next_track(&mut self) {
        if !self.forward_history.is_empty() {
            // Go to next track in forward history
            let next_idx = self.forward_history.pop();
            if let Some(idx) = next_idx {
                self.playback_history.push(idx);
                self.queue.jump_to(idx);
                self.play_current_queue_item();
            }
        } else if self.shuffle {
            self.play_shuffle_next();
        } else if self.queue.has_next() {
            self.queue.advance();
            self.play_current_queue_item();
        } else if self.repeat {
            self.queue.jump_to(0);
            self.play_current_queue_item();
        }
    }

    /// Previous: if >10s elapsed, restart; else go to previous track.
    pub fn prev_track(&mut self) {
        let elapsed = self
            .track_start_time
            .map(|t| t.elapsed())
            .unwrap_or(Duration::ZERO);

        if elapsed > Duration::from_secs(10) {
            // Restart current track
            if let Some(path) = self.current_track_path.clone() {
                let dur = self.player.track_duration();
                if let Err(e) = self.player.play(&path, dur) {
                    eprintln!("Restart error: {e}");
                }
                self.track_start_time = Some(Instant::now());
            }
        } else if !self.playback_history.is_empty() {
            // Go to previous track in history
            if let Some(current_idx) = self.queue.current_index {
                self.forward_history.push(current_idx);
            }
            let prev_idx = self.playback_history.pop();
            if let Some(idx) = prev_idx {
                self.queue.jump_to(idx);
                self.play_current_queue_item();
            }
        } else {
            // At start — restart regardless
            if let Some(path) = self.current_track_path.clone() {
                let dur = self.player.track_duration();
                let _ = self.player.play(&path, dur);
                self.track_start_time = Some(Instant::now());
            }
        }
    }

    fn play_shuffle_next(&mut self) {
        if self.library.is_empty() {
            return;
        }
        let idx = (rand_usize() % self.library.len()) as usize;
        self.queue.jump_to(idx);
        self.play_current_queue_item();
    }

    pub fn jump_to_queue_item(&mut self, index: usize) {
        self.queue.jump_to(index);
        self.play_current_queue_item();
    }

    pub fn seek_to(&mut self, target: Duration) {
        if let Some(ref path) = self.current_track_path.clone() {
            let dur = self.player.track_duration();
            self.player.seek_to(path, target, dur);
            self.track_start_time = Some(Instant::now() - target);
        }
    }

    pub fn toggle_queue_panel(&mut self) {
        self.show_queue_panel = !self.show_queue_panel;
        if self.show_queue_panel {
            self.active_panel = Panel::Queue;
        }
    }

    // ── Album art ─────────────────────────────────────────────────────────

    fn update_album_art_for_track(&mut self, _track: &Track) {
        // Texture is created lazily in maybe_update_album_art() each frame
    }

    fn load_album_art_texture(&mut self, ctx: &Context, art_bytes: &[u8]) {
        if let Some(img) = metadata::decode_album_art(art_bytes) {
            let color_image = metadata::to_egui_image(img);
            self.current_album_art_texture =
                Some(ctx.load_texture("album_art", color_image, TextureOptions::LINEAR));
            ctx.request_repaint(); // Ensure new art shows up immediately
        }
    }

    // ── Playlists ─────────────────────────────────────────────────────────

    pub fn save_playlists(&self) {
        let _ = self.playlist_store.save(&Config::playlists_path());
    }

    // ── Periodic update ───────────────────────────────────────────────────

    fn tick(&mut self, ctx: &Context) {
        self.frame_count += 1;

        // Poll metadata background thread
        if self.loading_metadata {
            if let Some(ref rx) = self.metadata_rx {
                if let Ok(result) = rx.try_recv() {
                    self.library = result.tracks;
                    self.loading_metadata = false;
                    self.metadata_rx = None;
                    ctx.request_repaint();
                }
            }
        }

        // Check if current track finished naturally
        if self.player.is_finished() && self.player.state() == PlaybackState::Playing {
            // Already finished — advance
            self.auto_advance(ctx);
        }

        // Update album art when track changes
        if self.frame_count % 10 == 0 {
            self.maybe_update_album_art(ctx);
        }

        // Auto-repaint during playback
        if self.player.state() == PlaybackState::Playing {
            ctx.request_repaint_after(Duration::from_millis(500));
        }
    }

    fn auto_advance(&mut self, ctx: &Context) {
        if self.repeat && self.queue.len() == 1 {
            // Repeat single
            if let Some(path) = self.current_track_path.clone() {
                let dur = self.player.track_duration();
                let _ = self.player.play(&path, dur);
                self.track_start_time = Some(Instant::now());
            }
        } else if self.shuffle {
            self.play_shuffle_next();
        } else if self.queue.has_next() {
            self.queue.advance();
            self.play_current_queue_item();
        } else if self.repeat {
            self.queue.jump_to(0);
            self.play_current_queue_item();
        } else {
            // End of queue
            self.player.stop();
            self.track_start_time = None;
        }
        ctx.request_repaint();
    }

    fn maybe_update_album_art(&mut self, ctx: &Context) {
        let current_id = self.queue.current_track_id().map(|s| s.to_string());
        if let Some(id) = current_id {
            // Check if track changed since last art load
            let art_stale = self.current_album_art_track_id.as_deref() != Some(id.as_str());
            if let Some(track) = self.library.iter().find(|t| t.id == id) {
                if let Some(ref art) = track.album_art_bytes.clone() {
                    if art_stale || self.current_album_art_texture.is_none() {
                        self.load_album_art_texture(ctx, art);
                        self.current_album_art_track_id = Some(id);
                        ctx.request_repaint();
                    }
                } else if !track.metadata_loaded {
                    // metadata not loaded yet — keep existing art
                } else {
                    if self.current_album_art_texture.is_some() {
                        self.current_album_art_texture = None;
                        ctx.request_repaint();
                    }
                    self.current_album_art_track_id = Some(id);
                }
            }
        } else {
            self.current_album_art_texture = None;
            self.current_album_art_track_id = None;
        }
    }
}

// ── Simple LCG pseudo-random (no dep) ────────────────────────────────────────

static RAND_STATE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(12345);

fn rand_usize() -> usize {
    let prev = RAND_STATE.load(std::sync::atomic::Ordering::Relaxed);
    let next = prev
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    RAND_STATE.store(next, std::sync::atomic::Ordering::Relaxed);
    next as usize
}

// ── eframe::App impl ─────────────────────────────────────────────────────────

impl eframe::App for MelodiaApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.tick(ctx);

        // ── Keyboard shortcuts ────────────────────────────────────────────
        let has_focus = ctx.memory(|mem| mem.focused().is_some());
        if !has_focus {
            let space = ctx.input(|i| i.key_pressed(egui::Key::Space));
            let right = ctx.input(|i| i.key_pressed(egui::Key::ArrowRight));
            let left = ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft));
            let up = ctx.input(|i| i.key_pressed(egui::Key::ArrowUp));
            let down = ctx.input(|i| i.key_pressed(egui::Key::ArrowDown));

            if space {
                self.toggle_pause();
            }
            if right {
                self.next_track();
            }
            if left {
                self.prev_track();
            }
            if up {
                let v = (self.player.volume() + 0.05).min(1.5);
                self.player.set_volume(v);
            }
            if down {
                let v = (self.player.volume() - 0.05).max(0.0);
                self.player.set_volume(v);
            }
        }

        // Handle album art after track change (needs ctx)
        // done in tick via maybe_update_album_art

        // ── Dialogs (floating windows) ────────────────────────────────────
        ui::dialogs::show_new_playlist_dialog(self, ctx);

        // ── Sidebar ───────────────────────────────────────────────────────
        egui::SidePanel::left("sidebar")
            .exact_width(crate::theme::SIDEBAR_WIDTH)
            .resizable(false)
            .show(ctx, |ui| {
                ui::sidebar::show(self, ui);
            });

        // ── Bottom bar ────────────────────────────────────────────────────
        egui::TopBottomPanel::bottom("bottom_bar")
            .exact_height(crate::theme::BOTTOM_BAR_HEIGHT)
            .show(ctx, |ui| {
                ui::bottom_bar::show(self, ui);
            });

        // ── Central panel (main content area) ────────────────────────────
        egui::CentralPanel::default().show(ctx, |ui| match self.active_panel.clone() {
            Panel::Library => ui::library_panel::show(self, ui),
            Panel::Queue => ui::queue_panel::show(self, ui),
            Panel::Playlist(id) => ui::playlist_panel::show(self, ui, &id),
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Save config on exit
        self.config.volume = self.player.volume();
        self.config.shuffle = self.shuffle;
        self.config.repeat = self.repeat;
        self.config.save();
        self.save_playlists();
    }
}

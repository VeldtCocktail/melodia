// melodia/src/audio.rs
// Audio playback via rodio — thread-safe player handle

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use cpal::traits::{DeviceTrait, HostTrait};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

/// Thread-safe wrapper around rodio Sink + position tracking.
pub struct AudioPlayer {
    inner: Arc<Mutex<PlayerInner>>,
    // OutputStream must be kept alive for audio to play
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    pub device_name: String,
}

struct PlayerInner {
    sink: Option<Sink>,
    state: PlaybackState,
    /// Time when playback started (or resumed)
    play_started: Option<Instant>,
    /// Accumulated position before last pause/stop
    position_offset: Duration,
    /// Total duration of current track
    track_duration: Option<Duration>,
    volume: f32,
}

impl AudioPlayer {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Log available hosts and devices for diagnostics
        #[cfg(all(debug_assertions, feature = "debug-logging"))]
        {
            let host_ids = cpal::available_hosts();
            println!("Available audio hosts: {:?}", host_ids);
            for host_id in host_ids {
                let host = cpal::host_from_id(host_id)?;
                let devices = host.output_devices()?;
                for (i, device) in devices.enumerate() {
                    if let Ok(name) = device.name() {
                        println!("  Host {:?} Device {}: {}", host_id, i, name);
                    }
                }
            }
        }

        let (stream, stream_handle) = OutputStream::try_default().map_err(|e| {
            format!("Failed to open default audio output: {}", e)
        })?;

        // Try to get a human-readable name for the default device
        let device_name = match cpal::default_host().default_output_device() {
            Some(device) => device.name().unwrap_or_else(|_| "Unknown Device".to_string()),
            None => "No Device Found".to_string(),
        };

        println!("Selected Audio Device: {}", device_name);

        let inner = Arc::new(Mutex::new(PlayerInner {
            sink: None,
            state: PlaybackState::Stopped,
            play_started: None,
            position_offset: Duration::ZERO,
            track_duration: None,
            volume: 1.0,
        }));
        Ok(AudioPlayer { 
            inner, 
            _stream: stream, 
            stream_handle,
            device_name 
        })
    }

    /// Load and start playing a file immediately.
    pub fn play(&self, path: &PathBuf, duration: Option<Duration>) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let source = Decoder::new(BufReader::new(file))?;

        let mut inner = self.inner.lock().unwrap();

        // Stop existing sink
        if let Some(ref s) = inner.sink {
            s.stop();
        }

        let sink = Sink::try_new(&self.stream_handle)?;
        sink.set_volume(inner.volume);
        sink.append(source);

        inner.sink = Some(sink);
        inner.state = PlaybackState::Playing;
        inner.play_started = Some(Instant::now());
        inner.position_offset = Duration::ZERO;
        inner.track_duration = duration;

        Ok(())
    }

    pub fn pause(&self) {
        let mut inner = self.inner.lock().unwrap();
        if inner.state == PlaybackState::Playing {
            if let Some(ref s) = inner.sink {
                s.pause();
            }
            // Accumulate elapsed time
            if let Some(started) = inner.play_started.take() {
                inner.position_offset += started.elapsed();
            }
            inner.state = PlaybackState::Paused;
        }
    }

    pub fn resume(&self) {
        let mut inner = self.inner.lock().unwrap();
        if inner.state == PlaybackState::Paused {
            if let Some(ref s) = inner.sink {
                s.play();
            }
            inner.play_started = Some(Instant::now());
            inner.state = PlaybackState::Playing;
        }
    }

    pub fn stop(&self) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(ref s) = inner.sink {
            s.stop();
        }
        inner.sink = None;
        inner.state = PlaybackState::Stopped;
        inner.play_started = None;
        inner.position_offset = Duration::ZERO;
    }

    pub fn toggle_pause(&self) {
        let state = self.state();
        match state {
            PlaybackState::Playing => self.pause(),
            PlaybackState::Paused => self.resume(),
            _ => {}
        }
    }

    pub fn set_volume(&self, vol: f32) {
        let mut inner = self.inner.lock().unwrap();
        inner.volume = vol.clamp(0.0, 1.5);
        if let Some(ref s) = inner.sink {
            s.set_volume(inner.volume);
        }
    }

    pub fn volume(&self) -> f32 {
        self.inner.lock().unwrap().volume
    }

    pub fn state(&self) -> PlaybackState {
        self.inner.lock().unwrap().state.clone()
    }

    /// Returns approximate playback position.
    pub fn position(&self) -> Duration {
        let inner = self.inner.lock().unwrap();
        let base = inner.position_offset;
        if inner.state == PlaybackState::Playing {
            if let Some(started) = inner.play_started {
                return base + started.elapsed();
            }
        }
        base
    }

    pub fn track_duration(&self) -> Option<Duration> {
        self.inner.lock().unwrap().track_duration
    }

    /// Returns 0.0–1.0 progress fraction.
    pub fn progress(&self) -> f32 {
        let inner = self.inner.lock().unwrap();
        let dur = match inner.track_duration {
            Some(d) if d.as_secs_f32() > 0.0 => d.as_secs_f32(),
            _ => return 0.0,
        };
        let pos = match inner.state {
            PlaybackState::Playing => {
                let base = inner.position_offset;
                if let Some(s) = inner.play_started {
                    (base + s.elapsed()).as_secs_f32()
                } else {
                    base.as_secs_f32()
                }
            }
            _ => inner.position_offset.as_secs_f32(),
        };
        (pos / dur).clamp(0.0, 1.0)
    }

    /// Seek to an absolute position (approximate: re-open and skip).
    /// rodio doesn't support true seeking so we restart + skip frames.
    pub fn seek_to(&self, path: &PathBuf, target: Duration, track_duration: Option<Duration>) {
        // rodio 0.19+ supports Source::skip_duration
        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            if let Ok(source) = Decoder::new(reader) {
                let mut inner = self.inner.lock().unwrap();
                if let Some(ref s) = inner.sink {
                    s.stop();
                }
                if let Ok(sink) = Sink::try_new(&self.stream_handle) {
                    sink.set_volume(inner.volume);
                    use rodio::Source;
                    sink.append(source.skip_duration(target));
                    inner.sink = Some(sink);
                    inner.state = PlaybackState::Playing;
                    inner.play_started = Some(Instant::now());
                    inner.position_offset = target;
                    inner.track_duration = track_duration;
                }
            }
        }
    }

    /// Returns true if the current track has finished playing naturally.
    pub fn is_finished(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        if inner.state == PlaybackState::Playing {
            if let Some(ref s) = inner.sink {
                return s.empty();
            }
        }
        false
    }
}

unsafe impl Send for AudioPlayer {}
unsafe impl Sync for AudioPlayer {}

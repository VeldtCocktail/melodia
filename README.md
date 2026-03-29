# 🎵 Melodia

A high-performance, cross-platform music player written in Rust.

![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange)
![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux-blue)
![License](https://img.shields.io/badge/License-MIT-green)

---

## Features

| Feature | Details |
|---|---|
| **Library browser** | Scan folders recursively; lists all audio files with metadata |
| **Album art** | Displays embedded cover art (JPEG/PNG) from ID3/FLAC/Vorbis tags |
| **Playlists** | Create, rename, delete playlists; add tracks by right-clicking |
| **Queue** | Full queue management with drag-and-drop reordering |
| **Play Next** | Insert a track right after the current one |
| **Skip forward/back** | Previous button restarts track if >10s elapsed, else goes back |
| **Seek bar** | Click or drag to seek to any position |
| **Shuffle** | Random playback across the queue |
| **Repeat** | Repeat queue or single track |
| **Volume** | Adjustable volume slider (0–150%) |
| **Persistence** | Remembers last folder, playlists, volume, shuffle/repeat state |

## Supported Formats

`MP3` · `FLAC` · `OGG/Vorbis` · `WAV` · `AAC` · `M4A` · `Opus` · `WMA`

---

## Building

### Prerequisites

- **Rust 1.75+** (install via [rustup.rs](https://rustup.rs))

---

### Windows

**Option A — PowerShell script (recommended):**

```powershell
# From the melodia\ directory:
.\build_windows.ps1

# Debug build:
.\build_windows.ps1 -Debug

# Build + install to %LOCALAPPDATA%\Melodia + Start Menu shortcut:
.\build_windows.ps1 -Install
```

**Option B — Manual:**

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc
# Binary: target\x86_64-pc-windows-msvc\release\melodia.exe
```

> **Note:** On Windows you need the **MSVC** toolchain (installed with Visual Studio Build Tools or Visual Studio Community — select "Desktop development with C++"). The script will guide you.

---

### Linux (Debian/Ubuntu)

**Option A — Shell script (recommended, installs all apt dependencies automatically):**

```bash
chmod +x build_linux.sh
./build_linux.sh

# Debug build:
./build_linux.sh --debug

# Build + install to /usr/local/bin + .desktop file:
./build_linux.sh --install
```

**Option B — Manual:**

```bash
# Install system dependencies
sudo apt-get install -y \
    build-essential pkg-config \
    libasound2-dev libssl-dev \
    libfontconfig1-dev libxkbcommon-dev \
    libwayland-dev libxcb1-dev \
    libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
    libgtk-3-dev

# Build
rustup target add x86_64-unknown-linux-gnu
cargo build --release --target x86_64-unknown-linux-gnu

# Binary: target/x86_64-unknown-linux-gnu/release/melodia
```

---

### Cross-compile Windows EXE from Linux

```bash
# Install cross (Docker required)
cargo install cross

chmod +x cross_compile_windows.sh
./cross_compile_windows.sh
# Output: target/x86_64-pc-windows-gnu/release/melodia.exe
```

---

## Usage

### Keyboard Shortcuts

| Key | Action |
|---|---|
| `Space` | Play / Pause |
| `→` / `L` | Next track |
| `←` / `J` | Previous track / Restart |
| `S` | Toggle shuffle |
| `R` | Toggle repeat |
| `+` / `=` | Volume up |
| `-` | Volume down |
| `Ctrl+O` | Open folder |
| `Escape` | Close dialog |

### Getting Started

1. Launch Melodia
2. Click **📂 Open Folder** in the sidebar
3. Select a folder containing music files
4. Double-click any track to play it
5. Right-click tracks to add to queue, play next, or add to playlists

### Queue Management

- **Double-click** any track in the library → plays immediately, fills queue with library
- **Right-click → Add to Queue** → appends to end
- **Right-click → Play Next** → inserts after current track
- In the **Queue** panel: drag rows to reorder, click ✕ to remove

### Playlists

- Click **+** next to "PLAYLISTS" in the sidebar to create one
- Right-click any library track → **Add to Playlist**
- Click a playlist in the sidebar to view and manage it
- Drag tracks within a playlist to reorder them
- Click **▶ Play All** to play the whole playlist

---

## Architecture

```
melodia/
├── src/
│   ├── main.rs          # Entry point, window setup
│   ├── app.rs           # Central state, eframe::App impl
│   ├── audio.rs         # Audio playback engine (rodio wrapper)
│   ├── library.rs       # Track model, folder scanner
│   ├── metadata.rs      # Tag reading (lofty), album art decoding
│   ├── playlist.rs      # Playlist model + JSON persistence
│   ├── queue.rs         # Playback queue with reordering
│   ├── config.rs        # User config persistence
│   ├── theme.rs         # Dark theme colors + egui style
│   └── ui/
│       ├── mod.rs
│       ├── sidebar.rs        # Left navigation panel
│       ├── library_panel.rs  # Main track list
│       ├── queue_panel.rs    # Queue view with drag-drop
│       ├── playlist_panel.rs # Playlist view
│       ├── bottom_bar.rs     # Playback controls + seek bar
│       ├── album_art.rs      # Album art display
│       └── dialogs.rs        # New playlist dialog
├── build_windows.ps1         # Windows build script
├── build_linux.sh            # Linux build + dependency installer
├── cross_compile_windows.sh  # Cross-compile from Linux
└── Cargo.toml
```

## Data Stored

| Platform | Location |
|---|---|
| Windows | `%LOCALAPPDATA%\Melodia\` |
| Linux | `~/.local/share/Melodia/` |

Files: `config.json` (settings), `playlists.json` (playlists)

---

## Dependencies

| Crate | Purpose |
|---|---|
| `eframe` / `egui` | Cross-platform GUI framework |
| `rodio` | Audio decoding and playback |
| `lofty` | Audio tag reading (ID3, FLAC, Vorbis, etc.) |
| `image` | Album art image decoding |
| `rfd` | Native file picker dialogs |
| `walkdir` | Recursive directory scanning |
| `serde` / `serde_json` | Config and playlist serialization |
| `uuid` | Unique track/playlist IDs |
| `dirs` | Platform config directory paths |

---

## License

MIT

#!/usr/bin/env bash
# build_linux.sh
# Builds Melodia for Linux (Debian/Ubuntu and derivatives)
# Run: chmod +x build_linux.sh && ./build_linux.sh

set -euo pipefail

CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${CYAN}=== Melodia Linux Build ===${NC}"

DEBUG=0
INSTALL=0
for arg in "$@"; do
    case $arg in
        --debug)   DEBUG=1 ;;
        --install) INSTALL=1 ;;
    esac
done

# ── 1. Install system dependencies ──────────────────────────────────────────

echo -e "\n${YELLOW}Checking system dependencies...${NC}"

PKGS_NEEDED=()
check_pkg() {
    dpkg -s "$1" &>/dev/null || PKGS_NEEDED+=("$1")
}

check_pkg build-essential
check_pkg pkg-config
check_pkg libasound2-dev       # ALSA (rodio audio backend on Linux)
check_pkg libssl-dev           # TLS for any network features
check_pkg libfontconfig1-dev   # Font discovery (egui)
check_pkg libxkbcommon-dev     # Keyboard input (egui/winit)
check_pkg libwayland-dev       # Wayland support
check_pkg libxcb1-dev          # X11/XCB support
check_pkg libxcb-render0-dev
check_pkg libxcb-shape0-dev
check_pkg libxcb-xfixes0-dev
check_pkg libgtk-3-dev         # rfd file dialogs

if [ ${#PKGS_NEEDED[@]} -gt 0 ]; then
    echo -e "${YELLOW}Installing: ${PKGS_NEEDED[*]}${NC}"
    sudo apt-get update -q
    sudo apt-get install -y "${PKGS_NEEDED[@]}"
else
    echo -e "${GREEN}All system dependencies satisfied.${NC}"
fi

# ── 2. Install Rust if needed ────────────────────────────────────────────────

if ! command -v cargo &>/dev/null; then
    echo -e "\n${YELLOW}Rust not found. Installing via rustup...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    source "$HOME/.cargo/env"
fi

echo -e "${GREEN}Rust: $(rustc --version)${NC}"
echo -e "${GREEN}Cargo: $(cargo --version)${NC}"

# ── 3. Build ─────────────────────────────────────────────────────────────────

echo ""
if [ "$DEBUG" -eq 1 ]; then
    echo -e "${YELLOW}Building DEBUG build...${NC}"
    cargo build --target x86_64-unknown-linux-gnu
    OUT="target/x86_64-unknown-linux-gnu/debug/melodia"
else
    echo -e "${YELLOW}Building RELEASE build (optimized)...${NC}"
    cargo build --release --target x86_64-unknown-linux-gnu
    OUT="target/x86_64-unknown-linux-gnu/release/melodia"
fi

echo -e "\n${GREEN}Build succeeded!${NC}"
echo -e "Binary: $(realpath "$OUT")"
SIZE=$(du -sh "$OUT" | cut -f1)
echo -e "Size:   $SIZE"

# ── 4. Optional install ──────────────────────────────────────────────────────

if [ "$INSTALL" -eq 1 ]; then
    echo -e "\n${YELLOW}Installing to /usr/local/bin/melodia...${NC}"
    sudo install -m 755 "$OUT" /usr/local/bin/melodia

    # Create .desktop file for application menu
    DESKTOP_DIR="$HOME/.local/share/applications"
    mkdir -p "$DESKTOP_DIR"
    cat > "$DESKTOP_DIR/melodia.desktop" <<EOF
[Desktop Entry]
Name=Melodia
Comment=High-performance music player
Exec=/usr/local/bin/melodia
Icon=audio-x-generic
Terminal=false
Type=Application
Categories=Audio;Music;Player;
Keywords=music;player;audio;mp3;flac;
EOF
    chmod +x "$DESKTOP_DIR/melodia.desktop"
    update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true

    echo -e "${GREEN}Installed! Launch with: melodia${NC}"
    echo -e "${GREEN}Or find 'Melodia' in your application menu.${NC}"
fi

echo -e "\n${GREEN}Done! Run with: ./$OUT${NC}"

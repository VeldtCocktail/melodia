#!/usr/bin/env bash
# cross_compile_windows.sh
# Build a Windows .exe from a Linux host using cargo cross or mingw
# Requires: cargo install cross  OR  apt install mingw-w64

set -euo pipefail

CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${CYAN}=== Melodia Cross-Compile: Linux → Windows ===${NC}"

TARGET="x86_64-pc-windows-gnu"

# Check for cross tool first (Docker-based, easiest)
if command -v cross &>/dev/null; then
    echo -e "${GREEN}Using 'cross' (Docker-based cross compilation)${NC}"
    rustup target add "$TARGET" 2>/dev/null || true
    cross build --release --target "$TARGET"

elif command -v x86_64-w64-mingw32-gcc &>/dev/null; then
    echo -e "${GREEN}Using mingw-w64${NC}"
    rustup target add "$TARGET" 2>/dev/null || true
    export CC_x86_64_pc_windows_gnu=x86_64-w64-mingw32-gcc
    export CXX_x86_64_pc_windows_gnu=x86_64-w64-mingw32-g++
    export AR_x86_64_pc_windows_gnu=x86_64-w64-mingw32-ar
    export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc
    cargo build --release --target "$TARGET"

else
    echo -e "${YELLOW}Neither 'cross' nor mingw-w64 found.${NC}"
    echo ""
    echo "To install cross (recommended):"
    echo "  cargo install cross"
    echo "  (requires Docker running)"
    echo ""
    echo "Or install mingw-w64:"
    echo "  sudo apt-get install mingw-w64"
    exit 1
fi

OUT="target/$TARGET/release/melodia.exe"
echo -e "\n${GREEN}Cross-compilation succeeded!${NC}"
echo -e "Windows binary: $(realpath "$OUT")"
SIZE=$(du -sh "$OUT" | cut -f1)
echo -e "Size: $SIZE"

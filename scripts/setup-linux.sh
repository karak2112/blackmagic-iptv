#!/usr/bin/env bash
# Install Tauri 2 build dependencies on Debian/Ubuntu (including WSL).
# Run: bash scripts/setup-linux.sh

set -euo pipefail

if ! command -v apt-get >/dev/null 2>&1; then
  echo "This script supports Debian/Ubuntu only."
  echo "See https://tauri.app/start/prerequisites/#linux for other distros."
  exit 1
fi

echo "Installing IPTV Player / Tauri Linux dependencies..."
echo "(sudo password may be required)"
echo

sudo apt-get update
sudo apt-get install -y \
  pkg-config \
  libdbus-1-dev \
  libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev

echo
echo "Done. Verify with:"
echo "  pkg-config --version"
echo "  cargo test -p iptv-core"
echo "  npm run tauri dev"

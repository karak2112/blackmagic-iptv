#!/usr/bin/env bash
# Downloads ffmpeg.exe for Windows x64 into third_party/ffmpeg/win/
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEST="$ROOT/third_party/ffmpeg/win"
TMP="$(mktemp -d)"
URL="https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip"

cleanup() { rm -rf "$TMP"; }
trap cleanup EXIT

mkdir -p "$DEST"

echo "Downloading ffmpeg-release-essentials.zip..."
curl -fsSL "$URL" -o "$TMP/ffmpeg-release-essentials.zip"

if command -v unzip >/dev/null 2>&1; then
  unzip -q "$TMP/ffmpeg-release-essentials.zip" -d "$TMP/extract"
else
  echo "ERROR: unzip not found. Install unzip or run npm run fetch-ffmpeg from Windows PowerShell." >&2
  exit 1
fi

EXE="$(find "$TMP/extract" -name ffmpeg.exe -type f | head -n 1)"
if [[ -z "$EXE" ]]; then
  echo "ERROR: ffmpeg.exe not found inside archive" >&2
  exit 1
fi

cp -f "$EXE" "$DEST/ffmpeg.exe"
echo "ffmpeg-release-essentials" >"$DEST/VERSION"

echo "Installed:"
ls -lh "$DEST/ffmpeg.exe" "$DEST/VERSION"

#!/usr/bin/env bash
# Downloads libmpv-2.dll (and import libs) for Windows x64 into third_party/mpv/win/
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEST="$ROOT/third_party/mpv/win"
TOOLS="$ROOT/third_party/tools"
VERSION_FILE="$DEST/VERSION"
TMP="$(mktemp -d)"

cleanup() { rm -rf "$TMP"; }
trap cleanup EXIT

mkdir -p "$DEST" "$TOOLS"

resolve_asset_url() {
  python3 - <<'PY'
import json, urllib.request
data = json.load(urllib.request.urlopen(
    "https://api.github.com/repos/shinchiro/mpv-winbuild-cmake/releases/latest"
))
for asset in data["assets"]:
    name = asset["name"]
    if name.startswith("mpv-dev-x86_64-") and name.endswith(".7z") and "v3" not in name:
        print(asset["browser_download_url"])
        print(data["tag_name"])
        break
else:
    raise SystemExit("Could not find mpv-dev-x86_64 asset in latest release")
PY
}

mapfile -t META < <(resolve_asset_url)
URL="${META[0]}"
TAG="${META[1]}"
ASSET="${URL##*/}"

echo "Downloading ${ASSET} (release ${TAG})..."
curl -fsSL "$URL" -o "$TMP/archive.7z"

seven_zip() {
  if command -v 7z >/dev/null; then
    echo "7z"
    return
  fi
  if command -v 7za >/dev/null; then
    echo "7za"
    return
  fi
  local tool="$TOOLS/7zz"
  if [[ ! -x "$tool" ]]; then
    echo "Fetching portable 7-Zip (one-time)..." >&2
    curl -fsSL "https://github.com/ip7z/7zip/releases/download/24.09/7z2409-linux-x64.tar.xz" -o "$TMP/7z.tar.xz"
    tar -xf "$TMP/7z.tar.xz" -C "$TOOLS"
    chmod +x "$tool"
  fi
  echo "$tool"
}

EXTRACTOR="$(seven_zip)"
"$EXTRACTOR" x -y "-o$DEST" "$TMP/archive.7z" '*.dll' '*.lib' >/dev/null

find "$DEST" -mindepth 2 -type f \( -name '*.dll' -o -name '*.lib' \) -exec mv -t "$DEST" {} +
find "$DEST" -type d -empty -delete 2>/dev/null || true

if [[ ! -f "$DEST/libmpv-2.dll" ]]; then
  echo "ERROR: libmpv-2.dll not found after extract" >&2
  exit 1
fi

echo "$TAG" > "$VERSION_FILE"
echo "Installed to $DEST:"
ls -lh "$DEST"/*.{dll,lib} 2>/dev/null || ls -lh "$DEST"

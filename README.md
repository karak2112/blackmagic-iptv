# IPTV Player

A lightweight, open-source IPTV player for Windows built with **Tauri 2**, **Rust**, **Svelte**, and **libmpv**.

Load M3U playlists and XMLTV guides, browse channels by group, view now/next programme info, and watch streams with a TV-friendly interface.

## Features

- M3U/M3U8 playlists (local file or remote URL)
- XMLTV electronic program guide (local file or remote URL)
- Channel browse with groups, search, favorites, and pagination
- Now/next EPG display in browse and guide views
- libmpv playback with volume, mute, pause, and reload controls
- SQLite cache for fast startup
- Security-first: Rust-side fetching, URL allowlists, download size limits

## Architecture

```
iptv/
├── crates/iptv-core/   # Pure Rust: M3U/XMLTV parsers, EPG matching, validation
├── src-tauri/          # Tauri backend: SQLite, HTTP, playback, commands
└── src/                # Svelte frontend: TV-like UI
```

## Prerequisites

### WSL development (typical setup)

If you develop entirely inside **WSL Ubuntu** (all commands in the Linux shell), you are building and running a **Linux** Tauri app — usually displayed via **WSLg**. That is supported; it is not the same as running the Windows `.exe`.

One-time system deps:

```bash
bash scripts/setup-linux.sh
```

Also: Node.js 20+, Rust stable ([rustup](https://rustup.rs/)).

| Command (WSL) | What it does |
|---------------|--------------|
| `npm run tauri dev` | Linux dev build; **stub** playback (UI, playlists, EPG — no video) |
| `npm run fetch-mpv` | Downloads the **Windows** `libmpv-2.dll` into `third_party/mpv/win/` for Windows builds/installers — **not used** by the Linux dev binary |
| `cargo test -p iptv-core` | Parser/tests without Tauri or GTK |

**Video while staying in WSL:** install the Linux mpv library and rebuild with mpv enabled:

```bash
sudo apt install -y libmpv-dev
npm run tauri dev -- --features playback-mpv
```

**Video in the shipped Windows app:** build on Windows (or CI) — `tauri.windows.conf.json` enables `playback-mpv` and bundles the DLL automatically after `npm run fetch-mpv`.

### Windows (native build / release)

- [Node.js](https://nodejs.org/) 20+
- [Rust](https://rustup.rs/) (stable)
- [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (usually preinstalled on Windows 11)
- Visual Studio 2022 with C++ build tools
- **libmpv** for video playback — run `npm run fetch-mpv` from PowerShell or WSL, then `npm run tauri dev` / `npm run tauri build`

Without libmpv on Windows, channels load but video stays blank (stub engine).

### Fetch libmpv automatically (recommended)

From the repo root (works in **WSL** or **PowerShell**):

```bash
npm run fetch-mpv
```

This downloads `libmpv-2.dll` and generates `mpv.lib` (MSVC import library) into
`third_party/mpv/win/`. Windows builds copy the DLL next to the executable automatically.
See [third_party/NOTICES.md](third_party/NOTICES.md) for GPL licensing notes when distributing installers.

If you fetched from WSL (DLL only) and the Windows link step fails with `cannot open input file 'mpv.lib'`, run `npm run generate-mpv-lib` from **Windows PowerShell** in the repo, or re-run `npm run fetch-mpv` on Windows.

### Linux / WSL system libraries

On **Ubuntu / Debian / WSL**, if you have not run `setup-linux.sh`:

```bash
sudo apt update
sudo apt install -y pkg-config libdbus-1-dev libwebkit2gtk-4.1-dev \
  build-essential curl wget file libxdo-dev libssl-dev \
  libayatana-appindicator3-dev librsvg2-dev
```

## Development

All of this works from a WSL shell:

```bash
npm install
cargo test -p iptv-core
npm run tauri dev
```

That launches the **Linux** dev binary (stub playback unless you pass `--features playback-mpv` and have `libmpv-dev` installed).

## Building for Windows

From a **Windows** shell (or CI), in the same repo:

```bash
npm run fetch-mpv
npm run tauri build
```

From WSL alone, `npm run tauri build` produces a **Linux** binary, not a Windows installer. Installers (NSIS/MSI) come from a Windows build under `src-tauri/target/release/bundle/`.

## Usage

1. Open **Settings** and load your M3U playlist (file or URL).
2. Load your XMLTV guide (file or URL).
3. Browse channels by group or search.
4. Press Enter or click a channel to play.
5. Use arrow keys and Escape for remote-friendly navigation.

## Sample fixtures

Test fixtures for parsers live in `crates/iptv-core/tests/fixtures/`.

## License

MIT — see [LICENSE](LICENSE).

## Contributing

Parser changes should include fixture tests. Run `cargo test -p iptv-core` before submitting a PR.

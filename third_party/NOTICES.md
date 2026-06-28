# Third-party notices

## libmpv

Playback uses **libmpv** (`libmpv-2.dll`), obtained from the
[mpv-winbuild-cmake](https://github.com/shinchiro/mpv-winbuild-cmake) project.

- **License:** GNU General Public License v2.0 or later (GPL-2.0+)
- **Source:** https://github.com/mpv-player/mpv

When you **distribute** IPTV Player with libmpv bundled, GPL obligations apply
(including making corresponding source available for libmpv and documenting
the combined work). For personal/local use only, this is not a concern.

The MIT license on this application's own source code does not apply to libmpv.

Run `scripts/fetch-mpv-windows.ps1` (Windows) or `scripts/fetch-mpv-windows.sh`
(WSL) to download pinned binaries into `third_party/mpv/win/`.

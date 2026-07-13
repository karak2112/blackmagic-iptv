# ffmpeg for Windows recording

Stream recording uses **ffmpeg** with a tee muxer: one authenticated connection to your
provider, bytes written to a `.ts` file and mirrored to a local UDP port for mpv playback.

## Fetch automatically (recommended)

From the repo root (WSL or PowerShell):

```bash
npm run fetch-ffmpeg
```

This downloads [ffmpeg-release-essentials](https://www.gyan.dev/ffmpeg/builds/) into
`third_party/ffmpeg/win/ffmpeg.exe`. Windows installers copy it next to the app as
`ffmpeg.exe` (see `src-tauri/tauri.windows.conf.json`).

## Manual setup

Extract `ffmpeg.exe` from a [gyan.dev essentials build](https://www.gyan.dev/ffmpeg/builds/)
into `third_party/ffmpeg/win/ffmpeg.exe`.

## Verify

```powershell
dir third_party\ffmpeg\win\ffmpeg.exe
```

Recording is unavailable until ffmpeg is found.

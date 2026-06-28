# libmpv for Windows

Binaries are **not committed** (see `.gitignore`). Fetch them once:

```powershell
# Windows (recommended — run from repo root)
.\scripts\fetch-mpv-windows.ps1
```

```bash
# WSL / Linux prep
bash scripts/fetch-mpv-windows.sh
```

After fetch, rebuild:

```bash
npm run tauri dev
```

Tauri's bundler copies `libmpv-2.dll` into the release output next to the executable
(`libmpv-2.dll`, not a subfolder). `build.rs` does the same for dev builds.

## Licensing

libmpv is **GPL-2.0+**. See [../NOTICES.md](../NOTICES.md).

## Pinned version

Release tag: `20260602` (asset `mpv-dev-x86_64-20260602-git-f5d4d9b.7z`)

Source: https://github.com/shinchiro/mpv-winbuild-cmake/releases

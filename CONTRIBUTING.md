# Contributing

## Development setup

1. Install [Rust](https://rustup.rs/) and [Node.js 20+](https://nodejs.org/).
2. Clone the repo and run `npm install`.
3. Run `cargo test -p iptv-core` to verify the core library.
4. Run `npm run tauri dev` for the full app.

## Parser fixtures

When changing M3U or XMLTV parsers, add or update fixtures under:

```
crates/iptv-core/tests/fixtures/
```

Run tests:

```bash
cargo test -p iptv-core
```

## Playback without libmpv

The default build uses a stub playback engine. To test with libmpv:

```bash
cd src-tauri
cargo build --features playback-mpv
```

On Windows, place libmpv DLLs in `third_party/mpv/win/` for bundling.

## Security

- Never fetch playlists or guides from the frontend — all network IO is in Rust.
- Validate URLs against the allowlist before fetch or playback.
- Do not log stream URLs containing credentials in release builds.

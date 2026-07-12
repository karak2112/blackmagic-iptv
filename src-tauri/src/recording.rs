use std::path::{Path, PathBuf};

use chrono::Local;

use crate::error::AppError;
use crate::state::AppState;

/// User-visible folder: `%USERPROFILE%\\Videos\\Black Magic IPTV\\Recordings`
pub fn recordings_dir() -> Result<PathBuf, AppError> {
    let base = dirs::video_dir().ok_or_else(|| {
        AppError::Other("Could not locate the Videos folder on this system.".into())
    })?;
    let dir = base.join("Black Magic IPTV").join("Recordings");
    std::fs::create_dir_all(&dir).map_err(AppError::from)?;
    Ok(dir)
}

pub fn sanitize_filename_component(input: &str) -> String {
    let mut out: String = input
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '-',
            c if c.is_control() => '-',
            c => c,
        })
        .collect();
    out = out.trim().trim_matches('.').to_string();
    if out.is_empty() {
        "Recording".into()
    } else if out.len() > 120 {
        out.chars().take(120).collect()
    } else {
        out
    }
}

pub fn build_recording_filename(channel_name: &str, programme_title: Option<&str>) -> String {
    let date = Local::now().format("%Y-%m-%d");
    let label = programme_title
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(channel_name);
    let safe = sanitize_filename_component(label);
    format!("{date}-{safe}.ts")
}

pub fn unique_recording_path(dir: &Path, filename: &str) -> PathBuf {
    let mut path = dir.join(filename);
    if !path.exists() {
        return path;
    }
    let stem = filename.strip_suffix(".ts").unwrap_or(filename);
    for i in 2..=999 {
        path = dir.join(format!("{stem}-{i}.ts"));
        if !path.exists() {
            return path;
        }
    }
    dir.join(format!("{stem}-{}.ts", uuid::Uuid::new_v4()))
}

pub fn programme_title_for_channel(state: &AppState, channel_id: &str) -> Option<String> {
    let db = state.db.lock();
    let epg_id = db.get_epg_channel_id(channel_id).ok().flatten()?;
    let now_next = db.now_next_for_epg_ids(&[epg_id]).ok()?;
    now_next
        .into_iter()
        .next()
        .and_then(|(_, nn)| nn.now.map(|p| p.title))
        .filter(|t| !t.trim().is_empty())
}

pub fn recording_available() -> bool {
    cfg!(all(target_os = "windows", feature = "playback-mpv"))
}

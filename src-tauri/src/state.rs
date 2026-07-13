use std::sync::Arc;

use iptv_core::PlaybackState;
use parking_lot::Mutex;

use crate::db::Database;
use crate::fetch::HttpFetcher;
use crate::playback::engine::PlaybackEngine;
use crate::recording::FfmpegRecording;
use tauri::AppHandle;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PreviewBounds {
    pub client_x: f64,
    pub client_y: f64,
    pub width: f64,
    pub height: f64,
    pub window_width: f64,
    pub window_height: f64,
}

#[derive(Debug, Clone, Default)]
pub struct RecordingState {
    pub active: bool,
    pub starting: bool,
    pub stopping: bool,
    pub path: Option<std::path::PathBuf>,
    pub source_url: Option<String>,
}

pub struct AppState {
    pub db: Mutex<Database>,
    pub fetcher: HttpFetcher,
    pub playback: Mutex<Box<dyn PlaybackEngine>>,
    pub playback_state: Mutex<PlaybackState>,
    pub preview_bounds: Mutex<Option<PreviewBounds>>,
    pub recording: Mutex<RecordingState>,
    pub ffmpeg_recording: Mutex<Option<FfmpegRecording>>,
}

impl AppState {
    pub fn new(db_path: std::path::PathBuf, app: AppHandle) -> Result<Self, crate::error::AppError> {
        let db = Database::open(&db_path)?;
        Ok(Self {
            db: Mutex::new(db),
            fetcher: HttpFetcher::default(),
            playback: Mutex::new(crate::playback::create_engine(&app)),
            playback_state: Mutex::new(PlaybackState::default()),
            preview_bounds: Mutex::new(None),
            recording: Mutex::new(RecordingState::default()),
            ffmpeg_recording: Mutex::new(None),
        })
    }
}

pub type SharedState = Arc<AppState>;

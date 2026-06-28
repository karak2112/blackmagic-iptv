use std::sync::Arc;

use iptv_core::PlaybackState;
use parking_lot::Mutex;

use crate::db::Database;
use crate::fetch::HttpFetcher;
use crate::playback::engine::PlaybackEngine;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PreviewBounds {
    pub client_x: f64,
    pub client_y: f64,
    pub width: f64,
    pub height: f64,
    pub window_width: f64,
    pub window_height: f64,
}

pub struct AppState {
    pub db: Mutex<Database>,
    pub fetcher: HttpFetcher,
    pub playback: Mutex<Box<dyn PlaybackEngine>>,
    pub playback_state: Mutex<PlaybackState>,
    pub preview_bounds: Mutex<Option<PreviewBounds>>,
}

impl AppState {
    pub fn new(db_path: std::path::PathBuf) -> Result<Self, crate::error::AppError> {
        let db = Database::open(&db_path)?;
        Ok(Self {
            db: Mutex::new(db),
            fetcher: HttpFetcher::default(),
            playback: Mutex::new(crate::playback::create_engine()),
            playback_state: Mutex::new(PlaybackState::default()),
            preview_bounds: Mutex::new(None),
        })
    }
}

pub type SharedState = Arc<AppState>;

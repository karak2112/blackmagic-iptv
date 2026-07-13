use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    M3uLocal,
    M3uRemote,
    XmltvLocal,
    XmltvRemote,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub id: String,
    pub name: String,
    pub source_type: SourceType,
    pub path_or_url: String,
    pub last_loaded: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub source_id: String,
    pub name: String,
    pub group: Option<String>,
    pub logo_url: Option<String>,
    pub stream_url: String,
    pub tvg_id: Option<String>,
    pub tvg_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Programme {
    pub channel_epg_id: String,
    pub start: DateTime<Utc>,
    pub stop: DateTime<Utc>,
    pub title: String,
    pub description: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NowNext {
    pub now: Option<Programme>,
    pub next: Option<Programme>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StreamStats {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fps: Option<f64>,
    pub video_bitrate_kbps: Option<f64>,
    pub audio_bitrate_kbps: Option<f64>,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingStatus {
    pub active: bool,
    pub starting: bool,
    pub stopping: bool,
    pub path: Option<String>,
    pub available: bool,
}

impl Default for RecordingStatus {
    fn default() -> Self {
        Self {
            active: false,
            starting: false,
            stopping: false,
            path: None,
            available: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackState {
    pub channel_id: Option<String>,
    pub playing: bool,
    pub paused: bool,
    pub volume: f64,
    pub muted: bool,
    pub fullscreen: bool,
    pub error: Option<String>,
    pub engine_name: String,
    pub video_available: bool,
    pub preview_mode: bool,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            channel_id: None,
            playing: false,
            paused: false,
            volume: 100.0,
            muted: false,
            fullscreen: false,
            error: None,
            engine_name: "unknown".into(),
            video_available: false,
            preview_mode: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistSummary {
    pub source_id: String,
    pub channel_count: usize,
    pub group_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpgSummary {
    pub channel_count: usize,
    pub programme_count: usize,
    pub matched_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelListPage {
    pub channels: Vec<Channel>,
    pub total: usize,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInfo {
    pub name: String,
    pub channel_count: usize,
}

/// Stable per-stream ID within a source. Always derived from the stream URL so
/// multiple entries that share a `tvg-id` (quality variants, regional feeds)
/// do not collide.
pub fn channel_id(source_id: &str, stream_url: &str, _tvg_id: Option<&str>) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(source_id.as_bytes());
    hasher.update(b":");
    hasher.update(stream_url.as_bytes());
    hex::encode(&hasher.finalize()[..16])
}

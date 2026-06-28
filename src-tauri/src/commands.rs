use iptv_core::{ChannelListPage, EpgSummary, GroupInfo, NowNext, PlaybackState, PlaylistSummary, Source};
use tauri::{Manager, State, WebviewWindow};

use crate::services;
use crate::state::AppState;

fn main_window(window: &WebviewWindow) -> Result<WebviewWindow, crate::error::AppError> {
    if window.label() == "main" {
        return Ok(window.clone());
    }
    window
        .app_handle()
        .get_webview_window("main")
        .ok_or_else(|| crate::error::AppError::Other("main window not found".into()))
}

#[tauri::command]
pub async fn load_playlist(
    state: State<'_, AppState>,
    name: String,
    local_path: Option<String>,
    remote_url: Option<String>,
    source_id: Option<String>,
) -> Result<PlaylistSummary, crate::error::AppError> {
    services::load_playlist(&state, name, local_path, remote_url, source_id).await
}

#[tauri::command]
pub async fn load_epg(
    state: State<'_, AppState>,
    local_path: Option<String>,
    remote_url: Option<String>,
) -> Result<EpgSummary, crate::error::AppError> {
    services::load_epg(&state, local_path, remote_url).await
}

#[tauri::command]
pub fn list_groups(
    state: State<'_, AppState>,
    source_id: Option<String>,
) -> Result<Vec<GroupInfo>, crate::error::AppError> {
    services::list_groups(&state, source_id)
}

#[tauri::command]
pub fn list_channels(
    state: State<'_, AppState>,
    source_id: Option<String>,
    group: Option<String>,
    search: Option<String>,
    favorites_only: Option<bool>,
    offset: Option<usize>,
    limit: Option<usize>,
) -> Result<ChannelListPage, crate::error::AppError> {
    services::list_channels(
        &state,
        source_id,
        group,
        search,
        favorites_only,
        offset.unwrap_or(0),
        limit.unwrap_or(100),
    )
}

#[tauri::command]
pub fn get_now_next(
    state: State<'_, AppState>,
    channel_ids: Vec<String>,
) -> Result<Vec<(String, NowNext)>, crate::error::AppError> {
    services::get_now_next_batch(&state, channel_ids)
}

#[tauri::command]
pub fn play_channel(
    window: WebviewWindow,
    state: State<'_, AppState>,
    channel_id: String,
) -> Result<(), crate::error::AppError> {
    let window = main_window(&window)?;
    services::play_channel(&window, &state, channel_id)
}

#[tauri::command]
pub fn preview_channel(
    window: WebviewWindow,
    state: State<'_, AppState>,
    channel_id: String,
) -> Result<(), crate::error::AppError> {
    let window = main_window(&window)?;
    services::preview_channel(&window, &state, channel_id)
}

#[tauri::command]
pub fn get_channel(
    state: State<'_, AppState>,
    channel_id: String,
) -> Result<Option<iptv_core::Channel>, crate::error::AppError> {
    services::get_channel(&state, channel_id)
}

#[tauri::command]
pub fn set_preview_bounds(
    window: WebviewWindow,
    state: State<'_, AppState>,
    client_x: f64,
    client_y: f64,
    width: f64,
    height: f64,
    window_width: f64,
    window_height: f64,
) -> Result<(), crate::error::AppError> {
    let window = main_window(&window)?;
    services::set_preview_bounds(
        &window,
        &state,
        client_x,
        client_y,
        width,
        height,
        window_width,
        window_height,
    )
}

#[tauri::command]
pub fn hide_preview_surface(
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> Result<(), crate::error::AppError> {
    let window = main_window(&window)?;
    services::hide_preview_surface(&window, &state)
}

#[tauri::command]
pub fn stop_playback(
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> Result<(), crate::error::AppError> {
    let window = main_window(&window)?;
    services::stop_playback(&window, &state)
}

#[tauri::command]
pub fn pause_playback(state: State<'_, AppState>) -> Result<(), crate::error::AppError> {
    services::pause_playback(&state)
}

#[tauri::command]
pub fn resume_playback(state: State<'_, AppState>) -> Result<(), crate::error::AppError> {
    services::resume_playback(&state)
}

#[tauri::command]
pub fn set_volume(
    state: State<'_, AppState>,
    volume: f64,
) -> Result<(), crate::error::AppError> {
    services::set_volume(&state, volume)
}

#[tauri::command]
pub fn set_muted(
    state: State<'_, AppState>,
    muted: bool,
) -> Result<(), crate::error::AppError> {
    services::set_muted(&state, muted)
}

#[tauri::command]
pub fn get_playback_state(state: State<'_, AppState>) -> PlaybackState {
    services::get_playback_state(&state)
}

#[tauri::command]
pub fn toggle_favorite(
    state: State<'_, AppState>,
    channel_id: String,
) -> Result<bool, crate::error::AppError> {
    services::toggle_favorite(&state, channel_id)
}

#[tauri::command]
pub fn list_favorites(state: State<'_, AppState>) -> Result<Vec<String>, crate::error::AppError> {
    services::list_favorites(&state)
}

#[tauri::command]
pub fn get_stream_stats(state: State<'_, AppState>) -> iptv_core::StreamStats {
    services::get_stream_stats(&state)
}

#[tauri::command]
pub fn zap_channel(
    window: WebviewWindow,
    state: State<'_, AppState>,
    delta: i32,
) -> Result<Option<iptv_core::Channel>, crate::error::AppError> {
    let window = main_window(&window)?;
    services::zap_channel(&window, &state, delta)
}

#[tauri::command]
pub fn get_settings(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, crate::error::AppError> {
    services::get_settings(&state)
}

#[tauri::command]
pub fn save_settings(
    state: State<'_, AppState>,
    settings: serde_json::Value,
) -> Result<(), crate::error::AppError> {
    services::save_settings(&state, settings)
}

#[tauri::command]
pub fn list_sources(state: State<'_, AppState>) -> Result<Vec<Source>, crate::error::AppError> {
    services::list_sources(&state)
}

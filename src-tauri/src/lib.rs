pub mod commands;
pub mod db;
pub mod error;
pub mod fetch;
pub mod playback;
pub mod recording;
pub mod services;
pub mod state;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    #[cfg(target_os = "android")]
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_exoplayer::init());

    #[cfg(not(target_os = "android"))]
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init());

    builder
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            std::fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("iptv.db");
            let app_state = AppState::new(db_path, app.handle().clone())?;
            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::load_playlist,
            commands::load_epg,
            commands::list_groups,
            commands::list_channels,
            commands::get_now_next,
            commands::play_channel,
            commands::preview_channel,
            commands::set_preview_bounds,
            commands::hide_preview_surface,
            commands::stop_playback,
            commands::pause_playback,
            commands::resume_playback,
            commands::set_volume,
            commands::set_muted,
            commands::get_playback_state,
            commands::get_stream_stats,
            commands::zap_channel,
            commands::toggle_favorite,
            commands::list_favorites,
            commands::get_settings,
            commands::save_settings,
            commands::get_channel,
            commands::list_sources,
            commands::get_platform,
            commands::get_recording_status,
            commands::toggle_recording,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

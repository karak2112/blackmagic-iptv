use chrono::Utc;
use iptv_core::{
    Channel, ChannelListPage, EpgMatcher, EpgSummary, M3uParser, NowNext, PlaylistSummary,
    RecordingStatus, Source, SourceType, StreamStats, UrlValidator, XmltvParser,
};
use uuid::Uuid;
use tauri::{AppHandle, Manager};

use crate::db::{new_source_id, Database};
use crate::error::AppError;
use crate::fetch::EPG_MAX_BYTES;
use crate::playback::{set_player_chrome, window_id_from_tauri, PlaybackEngine};
use crate::recording::{self, pick_udp_port, udp_playback_url, FfmpegRecording};
use crate::state::{AppState, PreviewBounds};

fn is_mobile_platform() -> bool {
    cfg!(any(target_os = "android", target_os = "ios"))
}

fn enable_player_surface(window: &tauri::WebviewWindow) -> Result<(), AppError> {
    if is_mobile_platform() {
        Ok(())
    } else {
        set_player_chrome(window, true)
    }
}

fn disable_player_surface(window: &tauri::WebviewWindow) -> Result<(), AppError> {
    if is_mobile_platform() {
        Ok(())
    } else {
        set_player_chrome(window, false)
    }
}

fn attach_playback_window(
    window: &tauri::WebviewWindow,
    playback: &mut dyn PlaybackEngine,
) -> Result<(), AppError> {
    if is_mobile_platform() {
        playback.attach_window(0)
    } else {
        let wid = window_id_from_tauri(window)?.0;
        playback.attach_window(wid)
    }
}

pub fn platform_name() -> &'static str {
    #[cfg(target_os = "android")]
    {
        "android"
    }
    #[cfg(target_os = "ios")]
    {
        "ios"
    }
    #[cfg(target_os = "windows")]
    {
        "windows"
    }
    #[cfg(target_os = "linux")]
    {
        "linux"
    }
    #[cfg(target_os = "macos")]
    {
        "macos"
    }
    #[cfg(not(any(
        target_os = "android",
        target_os = "ios",
        target_os = "windows",
        target_os = "linux",
        target_os = "macos"
    )))]
    {
        "unknown"
    }
}

fn apply_preview_geometry(state: &AppState, playback: &mut dyn PlaybackEngine) {
    if !state.playback_state.lock().preview_mode {
        return;
    }
    let Some(bounds) = *state.preview_bounds.lock() else {
        return;
    };
    if let Err(e) = playback.set_pip_geometry(
        bounds.client_x,
        bounds.client_y,
        bounds.width,
        bounds.height,
        bounds.window_width,
        bounds.window_height,
    ) {
        tracing::warn!("preview geometry: {e}");
    }
}

pub async fn load_playlist(
    state: &AppState,
    name: String,
    local_path: Option<String>,
    remote_url: Option<String>,
    source_id: Option<String>,
) -> Result<PlaylistSummary, AppError> {
    let (source_type, path_or_url, content) = if let Some(path) = local_path {
        let content = state.fetcher.read_local(&path)?;
        (SourceType::M3uLocal, path, content)
    } else if let Some(url) = remote_url {
        let content = state.fetcher.fetch_text(&url).await?;
        (SourceType::M3uRemote, url, content)
    } else {
        return Err(AppError::Other(
            "either local_path or remote_url is required".into(),
        ));
    };

    let id = source_id.unwrap_or_else(new_source_id);
    let parser = M3uParser::new(UrlValidator::default());
    let channels = parser.parse_reader(&id, content.as_bytes())?;

    let source = Source {
        id: id.clone(),
        name,
        source_type,
        path_or_url,
        last_loaded: Some(Utc::now()),
    };

    {
        let db = state.db.lock();
        db.upsert_source(&source)?;
        db.replace_channels(&id, &channels)?;
    }

    state.db.lock().playlist_summary(&id)
}

pub async fn load_epg(
    state: &AppState,
    local_path: Option<String>,
    remote_url: Option<String>,
) -> Result<EpgSummary, AppError> {
    let programmes = if let Some(path) = local_path {
        let file = state.fetcher.open_local_stream(&path)?;
        tokio::task::spawn_blocking(move || XmltvParser::parse_reader(file))
            .await
            .map_err(|e| AppError::Other(format!("EPG parse task failed: {e}")))??
    } else if let Some(url) = remote_url {
        let fetcher = state.fetcher.with_epg_limits();
        let path = fetcher
            .fetch_to_path(&url, EPG_MAX_BYTES)
            .await
            .map_err(AppError::from)?;
        tokio::task::spawn_blocking(move || {
            let file = std::fs::File::open(&path).map_err(AppError::from)?;
            XmltvParser::parse_reader(file).map_err(AppError::from)
        })
        .await
        .map_err(|e| AppError::Other(format!("EPG parse task failed: {e}")))??
    } else {
        return Err(AppError::Other(
            "either local_path or remote_url is required".into(),
        ));
    };

    let epg = iptv_core::EpgIndex::from_programmes(programmes.clone());

    let channels = state.db.lock().all_channels()?;
    let matcher = EpgMatcher::default();
    let matches = matcher.match_channels(&channels, &epg);

    {
        let db = state.db.lock();
        db.replace_programmes(&programmes)?;
        db.replace_epg_matches(&matches)?;
    }

    state.db.lock().epg_summary()
}

pub fn list_groups(
    state: &AppState,
    source_id: Option<String>,
) -> Result<Vec<iptv_core::GroupInfo>, AppError> {
    state
        .db
        .lock()
        .list_groups(source_id.as_deref())
}

pub fn list_channels(
    state: &AppState,
    source_id: Option<String>,
    group: Option<String>,
    search: Option<String>,
    favorites_only: Option<bool>,
    offset: usize,
    limit: usize,
) -> Result<ChannelListPage, AppError> {
    let limit = limit.clamp(1, 500);
    let (channels, total) = state.db.lock().list_channels(
        source_id.as_deref(),
        group.as_deref(),
        search.as_deref(),
        favorites_only.unwrap_or(false),
        offset,
        limit,
    )?;
    Ok(ChannelListPage {
        channels,
        total,
        offset,
        limit,
    })
}

pub fn get_now_next_batch(
    state: &AppState,
    channel_ids: Vec<String>,
) -> Result<Vec<(String, NowNext)>, AppError> {
    let db = state.db.lock();
    let mut epg_map = Vec::new();
    for channel_id in &channel_ids {
        if let Some(epg_id) = db.get_epg_channel_id(channel_id)? {
            epg_map.push((channel_id.clone(), epg_id));
        }
    }

    let epg_ids: Vec<String> = epg_map.iter().map(|(_, id)| id.clone()).collect();
    let now_next = db.now_next_for_epg_ids(&epg_ids)?;

    Ok(epg_map
        .into_iter()
        .filter_map(|(channel_id, epg_id)| {
            now_next
                .iter()
                .find(|(id, _)| id == &epg_id)
                .map(|(_, nn)| (channel_id, nn.clone()))
        })
        .collect())
}

pub fn play_channel(
    window: &tauri::WebviewWindow,
    state: &AppState,
    channel_id: String,
) -> Result<(), AppError> {
    play_channel_internal(window, state, channel_id, false)
}

pub fn preview_channel(
    window: &tauri::WebviewWindow,
    state: &AppState,
    channel_id: String,
) -> Result<(), AppError> {
    play_channel_internal(window, state, channel_id, true)
}

pub fn set_preview_bounds(
    window: &tauri::WebviewWindow,
    state: &AppState,
    client_x: f64,
    client_y: f64,
    width: f64,
    height: f64,
    window_width: f64,
    window_height: f64,
) -> Result<(), AppError> {
    enable_player_surface(window)?;

    let bounds = PreviewBounds {
        client_x,
        client_y,
        width,
        height,
        window_width,
        window_height,
    };

    let unchanged = state
        .preview_bounds
        .lock()
        .is_some_and(|prev| prev == bounds);
    *state.preview_bounds.lock() = Some(bounds);

    if unchanged {
        return Ok(());
    }

    if state.playback_state.lock().preview_mode {
        let mut playback = state.playback.lock();
        apply_preview_geometry(state, playback.as_mut());
    }

    Ok(())
}

pub fn hide_preview_surface(
    window: &tauri::WebviewWindow,
    state: &AppState,
) -> Result<(), AppError> {
    let was_preview = state.playback_state.lock().preview_mode;

    if was_preview {
        let mut playback = state.playback.lock();
        playback.stop()?;
        playback.clear_pip_geometry()?;
    }

    *state.preview_bounds.lock() = None;

    let main_player_active = {
        let ps = state.playback_state.lock();
        ps.playing && !ps.preview_mode
    };
    if !main_player_active {
        disable_player_surface(window)?;
    }

    if was_preview {
        let mut ps = state.playback_state.lock();
        ps.playing = false;
        ps.paused = false;
        ps.preview_mode = false;
        ps.channel_id = None;
        ps.error = None;
    }
    Ok(())
}

fn end_guide_preview_if_active(
    window: &tauri::WebviewWindow,
    state: &AppState,
) -> Result<(), AppError> {
    if state.playback_state.lock().preview_mode {
        hide_preview_surface(window, state)
    } else {
        *state.preview_bounds.lock() = None;
        Ok(())
    }
}

pub(crate) struct RecordingStartParams {
    stream_url: String,
    path: std::path::PathBuf,
    playback_url: String,
    udp_port: u16,
    saved_volume: f64,
    muted: bool,
}

fn stop_recording_if_active(
    state: &AppState,
    playback: Option<&mut dyn PlaybackEngine>,
    resume_playback: bool,
) -> Result<Option<String>, AppError> {
    let snapshot = {
        let rec = state.recording.lock();
        if !rec.active {
            return Ok(None);
        }
        (rec.source_url.clone(), rec.path.clone())
    };

    if let Some(session) = state.ffmpeg_recording.lock().take() {
        session.stop()?;
    }

    if resume_playback {
        if let Some(url) = snapshot.0 {
            let saved_volume = state.playback_state.lock().volume;
            let muted = state.playback_state.lock().muted;
            if let Some(pb) = playback {
                pb.load(&url)?;
                pb.play()?;
                pb.set_muted(muted)?;
                pb.set_volume(saved_volume)?;
            } else {
                let mut pb = state.playback.lock();
                pb.load(&url)?;
                pb.play()?;
                pb.set_muted(muted)?;
                pb.set_volume(saved_volume)?;
            }
        }
    }

    let mut rec = state.recording.lock();
    rec.active = false;
    rec.starting = false;
    rec.stopping = false;
    rec.path = None;
    rec.source_url = None;
    Ok(snapshot
        .1
        .map(|p| p.to_string_lossy().into_owned()))
}

pub(crate) struct ToggleRecordingOutcome {
    pub status: RecordingStatus,
    pub start: Option<RecordingStartParams>,
    pub spawn_stop: bool,
}

fn recording_start_worker(state: &AppState, params: RecordingStartParams) {
    let result = recording_start_blocking(state, &params);
    let mut rec = state.recording.lock();
    rec.starting = false;
    match result {
        Ok(()) => {
            rec.active = true;
            tracing::info!(
                "recording started: {}",
                params.path.to_string_lossy()
            );
        }
        Err(e) => {
            tracing::error!("recording start failed: {e}");
            rec.path = None;
            rec.source_url = None;
            state.playback_state.lock().error = Some(e.to_string());
        }
    }
}

fn recording_start_blocking(
    state: &AppState,
    params: &RecordingStartParams,
) -> Result<(), AppError> {
    {
        let mut playback = state.playback.lock();
        playback.stop()?;
    }

    std::thread::sleep(std::time::Duration::from_millis(250));

    let session = match FfmpegRecording::start(
        &params.stream_url,
        &params.path,
        params.udp_port,
    ) {
        Ok(session) => session,
        Err(e) => {
            let _ = std::fs::remove_file(&params.path);
            let mut playback = state.playback.lock();
            playback.load(&params.stream_url)?;
            playback.play()?;
            playback.set_muted(params.muted)?;
            playback.set_volume(params.saved_volume)?;
            return Err(e);
        }
    };

    {
        let mut playback = state.playback.lock();
        if let Err(e) = playback.load(&params.playback_url) {
            let _ = state.ffmpeg_recording.lock().take().map(FfmpegRecording::stop);
            let _ = std::fs::remove_file(&params.path);
            playback.load(&params.stream_url)?;
            playback.play()?;
            playback.set_muted(params.muted)?;
            playback.set_volume(params.saved_volume)?;
            return Err(e);
        }
        playback.play()?;
        playback.set_muted(params.muted)?;
        playback.set_volume(params.saved_volume)?;
    }

    *state.ffmpeg_recording.lock() = Some(session);
    Ok(())
}

fn recording_stop_worker(state: &AppState) {
    let result = stop_recording_if_active(state, None, true);
    let mut rec = state.recording.lock();
    rec.stopping = false;
    if let Err(e) = result {
        tracing::error!("recording stop failed: {e}");
        state.playback_state.lock().error = Some(e.to_string());
    }
}

pub fn get_recording_status(state: &AppState) -> RecordingStatus {
    let rec = state.recording.lock();
    let ps = state.playback_state.lock();
    RecordingStatus {
        active: rec.active,
        starting: rec.starting,
        stopping: rec.stopping,
        path: rec.path.as_ref().map(|p| p.to_string_lossy().into_owned()),
        available: recording::recording_available() && ps.playing && !ps.preview_mode,
    }
}

pub(crate) fn toggle_recording(state: &AppState) -> Result<ToggleRecordingOutcome, AppError> {
    if !recording::recording_available() {
        return Err(AppError::Other(
            "Recording requires ffmpeg on Windows. Install ffmpeg and add it to PATH, or place ffmpeg.exe next to the app.".into(),
        ));
    }

    if state.playback_state.lock().preview_mode {
        return Err(AppError::Other(
            "Recording is not available during guide preview.".into(),
        ));
    }

    {
        let rec = state.recording.lock();
        if rec.starting || rec.stopping {
            return Ok(ToggleRecordingOutcome {
                status: get_recording_status(state),
                start: None,
                spawn_stop: false,
            });
        }
    }

    if state.recording.lock().active {
        state.recording.lock().stopping = true;
        return Ok(ToggleRecordingOutcome {
            status: get_recording_status(state),
            start: None,
            spawn_stop: true,
        });
    }

    let channel_id = state
        .playback_state
        .lock()
        .channel_id
        .clone()
        .ok_or_else(|| AppError::Other("No channel is playing.".into()))?;

    if !state.playback_state.lock().playing {
        return Err(AppError::Other("Start playback before recording.".into()));
    }

    let channel = state
        .db
        .lock()
        .get_channel(&channel_id)?
        .ok_or_else(|| AppError::Other(format!("channel not found: {channel_id}")))?;

    if !recording::recording_supported_url(&channel.stream_url) {
        return Err(AppError::Other(
            "Recording is only supported for direct MPEG-TS streams, not HLS (.m3u8).".into(),
        ));
    }

    let programme_title = recording::programme_title_for_channel(state, &channel_id);
    let dir = recording::recordings_dir()?;
    let filename = recording::build_recording_filename(
        &channel.name,
        programme_title.as_deref(),
    );
    let path = recording::unique_recording_path(&dir, &filename);
    let stream_url = channel.stream_url.clone();
    let saved_volume = state.playback_state.lock().volume;
    let muted = state.playback_state.lock().muted;
    let udp_port = pick_udp_port()?;
    let playback_url = udp_playback_url(udp_port);

    {
        let mut rec = state.recording.lock();
        rec.starting = true;
        rec.path = Some(path.clone());
        rec.source_url = Some(stream_url.clone());
    }

    Ok(ToggleRecordingOutcome {
        status: get_recording_status(state),
        start: Some(RecordingStartParams {
            stream_url,
            path,
            playback_url,
            udp_port,
            saved_volume,
            muted,
        }),
        spawn_stop: false,
    })
}

pub(crate) fn execute_recording_followup(
    app: &AppHandle,
    outcome: ToggleRecordingOutcome,
) -> Result<RecordingStatus, AppError> {
    if outcome.spawn_stop {
        let app = app.clone();
        std::thread::Builder::new()
            .name("recording-stop".into())
            .spawn(move || {
                let state = app.state::<AppState>();
                recording_stop_worker(state.inner());
            })
            .map_err(|e| AppError::Other(format!("recording stop thread: {e}")))?;
    }
    if let Some(params) = outcome.start {
        let app = app.clone();
        std::thread::Builder::new()
            .name("recording-start".into())
            .spawn(move || {
                let state = app.state::<AppState>();
                recording_start_worker(state.inner(), params);
            })
            .map_err(|e| AppError::Other(format!("recording start thread: {e}")))?;
    }
    Ok(outcome.status)
}

fn play_channel_internal(
    window: &tauri::WebviewWindow,
    state: &AppState,
    channel_id: String,
    preview: bool,
) -> Result<(), AppError> {
    if preview {
        // Updating an in-guide preview — keep the surface alive and replace the stream.
    } else {
        end_guide_preview_if_active(window, state)?;
    }

    let channel = state
        .db
        .lock()
        .get_channel(&channel_id)?
        .ok_or_else(|| AppError::Other(format!("channel not found: {channel_id}")))?;

    UrlValidator::default().validate_stream_url(&channel.stream_url)?;

    enable_player_surface(window)?;

    let saved_volume = state.playback_state.lock().volume;
    let (engine_name, video_available) = {
        let mut playback = state.playback.lock();
        let (was_main, _was_preview) = {
            let ps = state.playback_state.lock();
            (
                ps.playing && !ps.preview_mode,
                preview && ps.preview_mode,
            )
        };
        if was_main {
            stop_recording_if_active(state, Some(playback.as_mut()), false)?;
            playback.stop()?;
            playback.clear_pip_geometry().ok();
        } else if !preview {
            playback.clear_pip_geometry().ok();
        }
        attach_playback_window(window, playback.as_mut())?;
        playback.load(&channel.stream_url)?;
        playback.play()?;
        if preview {
            playback.set_muted(true)?;
        } else {
            playback.set_muted(state.playback_state.lock().muted)?;
            playback.set_volume(saved_volume)?;
        }
        (
            playback.engine_name().to_string(),
            playback.is_available(),
        )
    };

    let mut ps = state.playback_state.lock();
    ps.channel_id = Some(channel_id);
    ps.playing = true;
    ps.paused = false;
    ps.preview_mode = preview;
    ps.error = None;
    ps.engine_name = engine_name;
    ps.video_available = video_available;

    if !video_available {
        ps.error = Some(if cfg!(target_os = "android") {
            "Video engine unavailable.".into()
        } else {
            "Video engine unavailable. Install libmpv and rebuild with playback-mpv.".into()
        });
    }
    drop(ps);

    if preview {
        let mut playback = state.playback.lock();
        apply_preview_geometry(state, playback.as_mut());
    }

    Ok(())
}

pub fn stop_playback(window: &tauri::WebviewWindow, state: &AppState) -> Result<(), AppError> {
    stop_recording_if_active(state, None, false)?;
    state.playback.lock().stop()?;
    disable_player_surface(window)?;
    let mut ps = state.playback_state.lock();
    ps.playing = false;
    ps.paused = false;
    ps.channel_id = None;
    ps.preview_mode = false;
    ps.error = None;
    Ok(())
}

pub fn pause_playback(state: &AppState) -> Result<(), AppError> {
    state.playback.lock().pause()?;
    let mut ps = state.playback_state.lock();
    ps.paused = true;
    ps.playing = false;
    Ok(())
}

pub fn resume_playback(state: &AppState) -> Result<(), AppError> {
    state.playback.lock().play()?;
    let mut ps = state.playback_state.lock();
    ps.paused = false;
    ps.playing = true;
    Ok(())
}

pub fn set_volume(state: &AppState, volume: f64) -> Result<(), AppError> {
    state.playback.lock().set_volume(volume)?;
    state.playback_state.lock().volume = volume;
    Ok(())
}

pub fn set_muted(state: &AppState, muted: bool) -> Result<(), AppError> {
    state.playback.lock().set_muted(muted)?;
    state.playback_state.lock().muted = muted;
    Ok(())
}

pub fn get_playback_state(state: &AppState) -> iptv_core::PlaybackState {
    let mut ps = state.playback_state.lock();
    let playback = state.playback.lock();
    ps.engine_name = playback.engine_name().to_string();
    ps.video_available = playback.is_available();
    ps.clone()
}

pub fn list_sources(state: &AppState) -> Result<Vec<Source>, AppError> {
    state.db.lock().list_sources()
}

pub fn toggle_favorite(state: &AppState, channel_id: String) -> Result<bool, AppError> {
    state.db.lock().toggle_favorite(&channel_id)
}

pub fn list_favorites(state: &AppState) -> Result<Vec<String>, AppError> {
    state.db.lock().list_favorite_ids()
}

pub fn get_settings(state: &AppState) -> Result<serde_json::Value, AppError> {
    let db = state.db.lock();
    let last_channel = db.get_setting("last_channel")?;
    let hidden_groups = db
        .get_setting("hidden_groups")?
        .and_then(|v| serde_json::from_str(&v).ok())
        .unwrap_or_else(|| serde_json::json!([]));
    let volume = db
        .get_setting("volume")?
        .and_then(|v| v.parse().ok())
        .unwrap_or(100.0);
    let font_scale = db
        .get_setting("font_scale")?
        .and_then(|v| v.parse().ok())
        .unwrap_or(100.0);
    let last_group = db.get_setting("last_group")?;
    let browse_scroll = db
        .get_setting("browse_scroll")?
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.0);
    let resume_on_startup = db
        .get_setting("resume_on_startup")?
        .map(|v| v != "false")
        .unwrap_or(true);

    Ok(serde_json::json!({
        "last_channel": last_channel,
        "hidden_groups": hidden_groups,
        "volume": volume,
        "font_scale": font_scale,
        "last_group": last_group,
        "browse_scroll": browse_scroll,
        "resume_on_startup": resume_on_startup,
    }))
}

pub fn save_settings(state: &AppState, settings: serde_json::Value) -> Result<(), AppError> {
    let db = state.db.lock();
    if let Some(v) = settings.get("last_channel").and_then(|v| v.as_str()) {
        db.set_setting("last_channel", v)?;
    }
    if let Some(v) = settings.get("hidden_groups") {
        db.set_setting("hidden_groups", &v.to_string())?;
    }
    if let Some(v) = settings.get("volume").and_then(|v| v.as_f64()) {
        db.set_setting("volume", &v.to_string())?;
    }
    if let Some(v) = settings.get("font_scale").and_then(|v| v.as_f64()) {
        db.set_setting("font_scale", &v.to_string())?;
    }
    if let Some(v) = settings.get("last_group").and_then(|v| v.as_str()) {
        db.set_setting("last_group", v)?;
    }
    if let Some(v) = settings.get("browse_scroll").and_then(|v| v.as_f64()) {
        db.set_setting("browse_scroll", &v.to_string())?;
    }
    if let Some(v) = settings.get("resume_on_startup").and_then(|v| v.as_bool()) {
        db.set_setting("resume_on_startup", if v { "true" } else { "false" })?;
    }
    Ok(())
}

pub fn get_channel(
    state: &AppState,
    channel_id: String,
) -> Result<Option<Channel>, AppError> {
    state.db.lock().get_channel(&channel_id)
}

pub fn get_stream_stats(state: &AppState) -> StreamStats {
    state.playback.lock().stream_stats()
}

pub fn zap_channel(
    window: &tauri::WebviewWindow,
    state: &AppState,
    delta: i32,
) -> Result<Option<Channel>, AppError> {
    let current_id = state.playback_state.lock().channel_id.clone();
    let Some(current_id) = current_id else {
        return Ok(None);
    };

    let neighbor = state.db.lock().adjacent_channel(&current_id, delta)?;
    let Some(channel) = neighbor else {
        return Ok(None);
    };

    play_channel(window, state, channel.id.clone())?;
    Ok(Some(channel))
}

// Keep source registration for EPG sources in settings for v1
pub fn register_epg_source(db: &Database, name: &str, path_or_url: &str, remote: bool) -> Result<(), AppError> {
    let source = Source {
        id: Uuid::new_v4().to_string(),
        name: name.to_string(),
        source_type: if remote {
            SourceType::XmltvRemote
        } else {
            SourceType::XmltvLocal
        },
        path_or_url: path_or_url.to_string(),
        last_loaded: Some(Utc::now()),
    };
    db.upsert_source(&source)
}

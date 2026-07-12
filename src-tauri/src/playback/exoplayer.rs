use crate::error::AppError;
use crate::playback::engine::PlaybackEngine;
use iptv_core::StreamStats;
use tauri::AppHandle;
use tauri_plugin_exoplayer::ExoPlayerExt;

pub struct ExoPlayerEngine {
    app: AppHandle,
}

impl ExoPlayerEngine {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    fn plugin(&self) -> Result<&tauri_plugin_exoplayer::ExoPlayer<tauri::Wry>, AppError> {
        self.app
            .try_exoplayer()
            .ok_or_else(|| AppError::Playback("ExoPlayer plugin not initialized".into()))
    }
}

impl PlaybackEngine for ExoPlayerEngine {
    fn attach_window(&mut self, _wid: i64) -> Result<(), AppError> {
        Ok(())
    }

    fn load(&mut self, url: &str) -> Result<(), AppError> {
        self.plugin()?.load(url).map_err(plugin_err)
    }

    fn play(&mut self) -> Result<(), AppError> {
        self.plugin()?.play().map_err(plugin_err)?;
        self.plugin()?.show_player().map_err(plugin_err)
    }

    fn pause(&mut self) -> Result<(), AppError> {
        self.plugin()?.pause().map_err(plugin_err)
    }

    fn stop(&mut self) -> Result<(), AppError> {
        self.plugin()?.stop().map_err(plugin_err)?;
        self.plugin()?.hide_player().map_err(plugin_err)
    }

    fn detach(&mut self) {
        let _ = self.stop();
    }

    fn set_pip_geometry(
        &mut self,
        _pip_x: f64,
        _pip_y: f64,
        _pip_w: f64,
        _pip_h: f64,
        _win_w: f64,
        _win_h: f64,
    ) -> Result<(), AppError> {
        Ok(())
    }

    fn clear_pip_geometry(&mut self) -> Result<(), AppError> {
        Ok(())
    }

    fn set_volume(&mut self, level: f64) -> Result<(), AppError> {
        self.plugin()?.set_volume(level).map_err(plugin_err)
    }

    fn set_muted(&mut self, muted: bool) -> Result<(), AppError> {
        self.plugin()?.set_muted(muted).map_err(plugin_err)
    }

    fn is_available(&self) -> bool {
        self.app.try_exoplayer().is_some()
    }

    fn engine_name(&self) -> &'static str {
        "exoplayer"
    }

    fn stream_stats(&self) -> StreamStats {
        let Ok(stats) = self.plugin().and_then(|p| p.stream_stats().map_err(plugin_err)) else {
            return StreamStats::default();
        };
        StreamStats {
            width: stats.width,
            height: stats.height,
            fps: None,
            video_bitrate_kbps: None,
            audio_bitrate_kbps: None,
            video_codec: stats.video_codec,
            audio_codec: stats.audio_codec,
            error: stats.error,
        }
    }
}

fn plugin_err(e: tauri_plugin_exoplayer::Error) -> AppError {
    AppError::Playback(e.to_string())
}

use crate::error::AppError;
use iptv_core::StreamStats;
use tauri::AppHandle;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PlaybackEvent {
    Idle,
    Playing,
    Paused,
    Buffering,
    Eof,
    Error { message: String },
}

pub trait PlaybackEngine: Send {
    fn attach_window(&mut self, wid: i64) -> Result<(), AppError>;
    fn load(&mut self, url: &str) -> Result<(), AppError>;
    fn play(&mut self) -> Result<(), AppError>;
    fn pause(&mut self) -> Result<(), AppError>;
    fn stop(&mut self) -> Result<(), AppError>;
    fn detach(&mut self);
    fn set_pip_geometry(
        &mut self,
        pip_x: f64,
        pip_y: f64,
        pip_w: f64,
        pip_h: f64,
        win_w: f64,
        win_h: f64,
    ) -> Result<(), AppError>;
    fn clear_pip_geometry(&mut self) -> Result<(), AppError>;
    fn set_volume(&mut self, level: f64) -> Result<(), AppError>;
    fn set_muted(&mut self, muted: bool) -> Result<(), AppError>;
    fn is_available(&self) -> bool;
    fn engine_name(&self) -> &'static str;
    fn stream_stats(&self) -> StreamStats;
}

pub fn create_engine(app: &AppHandle) -> Box<dyn PlaybackEngine> {
    create_engine_with_mode(app, false)
}

fn create_engine_with_mode(app: &AppHandle, preview: bool) -> Box<dyn PlaybackEngine> {
    #[cfg(target_os = "android")]
    {
        let _ = preview;
        return Box::new(crate::playback::exoplayer::ExoPlayerEngine::new(app.clone()));
    }

    #[cfg(all(not(target_os = "android"), feature = "playback-mpv"))]
    {
        let _ = app;
        return Box::new(mpv::MpvEngine::new(preview));
    }

    #[cfg(not(any(target_os = "android", all(not(target_os = "android"), feature = "playback-mpv"))))]
    {
        let _ = (app, preview);
        Box::new(stub::StubEngine)
    }
}

pub mod stub {
    use super::*;

    pub struct StubEngine;

    impl PlaybackEngine for StubEngine {
        fn attach_window(&mut self, _wid: i64) -> Result<(), AppError> {
            Ok(())
        }

        fn load(&mut self, url: &str) -> Result<(), AppError> {
            tracing::info!("stub playback load: {}", crate::fetch::redact_url(url));
            Ok(())
        }

        fn play(&mut self) -> Result<(), AppError> {
            tracing::info!("stub playback play");
            Ok(())
        }

        fn pause(&mut self) -> Result<(), AppError> {
            Ok(())
        }

        fn stop(&mut self) -> Result<(), AppError> {
            Ok(())
        }

        fn detach(&mut self) {}

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
            tracing::debug!("stub volume: {level}");
            Ok(())
        }

        fn set_muted(&mut self, muted: bool) -> Result<(), AppError> {
            tracing::debug!("stub muted: {muted}");
            Ok(())
        }

        fn is_available(&self) -> bool {
            false
        }

        fn engine_name(&self) -> &'static str {
            "stub"
        }

        fn stream_stats(&self) -> StreamStats {
            StreamStats::default()
        }
    }
}

#[cfg(feature = "playback-mpv")]
pub mod mpv {
    use libmpv2::Mpv;

    use super::*;
    use crate::playback::mpv_event_pump::MpvEventPump;

    pub struct MpvEngine {
        mpv: Option<Mpv>,
        attached_wid: Option<i64>,
        preview: bool,
        event_pump: Option<MpvEventPump>,
    }

    fn normalize_wid(wid: i64) -> i64 {
        #[cfg(windows)]
        {
            (wid as u32) as i64
        }
        #[cfg(not(windows))]
        {
            wid
        }
    }

    impl MpvEngine {
        pub fn new(preview: bool) -> Self {
            Self {
                mpv: None,
                attached_wid: None,
                preview,
                event_pump: None,
            }
        }

        fn teardown_mpv(&mut self) {
            if let Some(pump) = self.event_pump.take() {
                pump.stop();
            }
            self.mpv = None;
            self.attached_wid = None;
        }

        fn ensure_mpv(&mut self, wid: i64) -> Result<&mut Mpv, AppError> {
            let wid = normalize_wid(wid);
            if self.mpv.is_some() && self.attached_wid == Some(wid) {
                return Ok(self.mpv.as_mut().expect("mpv checked above"));
            }

            self.teardown_mpv();

            tracing::info!(
                "initializing libmpv for wid={wid} preview={}",
                self.preview
            );

            let preview = self.preview;
            let mpv = Mpv::with_initializer(|init| {
                init.set_option("wid", wid)?;
                init.set_option("vo", "gpu")?;
                init.set_option("gpu-context", "d3d11")?;
                init.set_option("hwdec", "auto-safe")?;
                init.set_option("cache", "yes")?;
                init.set_option("keep-open", "always")?;
                init.set_option("force-window", "yes")?;
                init.set_option("border", "no")?;
                init.set_option("background", "none")?;
                if preview {
                    init.set_option("demuxer-readahead-secs", 5i64)?;
                    init.set_option("video-sync", "audio")?;
                } else {
                    init.set_option("demuxer-readahead-secs", 10i64)?;
                    init.set_option("profile", "low-latency")?;
                    init.set_option("ontop", false)?;
                }
                Ok(())
            })
            .map_err(|e| AppError::Playback(format!("mpv init: {e}")))?;

            self.event_pump = Some(MpvEventPump::spawn(&mpv));
            self.mpv = Some(mpv);
            self.attached_wid = Some(wid);
            Ok(self.mpv.as_mut().expect("mpv just set"))
        }
    }

    impl PlaybackEngine for MpvEngine {
        fn attach_window(&mut self, wid: i64) -> Result<(), AppError> {
            self.ensure_mpv(wid).map(|_| ())
        }

        fn load(&mut self, url: &str) -> Result<(), AppError> {
            let mpv = self
                .mpv
                .as_mut()
                .ok_or_else(|| AppError::Playback("mpv not attached".into()))?;
            if url.starts_with("udp://") {
                let _ = mpv.set_property("cache", "no");
                let _ = mpv.set_property("demuxer-lavf-probesize", 32768i64);
                let _ = mpv.set_property("demuxer-lavf-analyzeduration", 0i64);
                let _ = mpv.set_property("network-timeout", 5i64);
            }
            mpv.command("loadfile", &[url, "replace"])
                .map_err(|e| AppError::Playback(e.to_string()))
        }

        fn play(&mut self) -> Result<(), AppError> {
            let mpv = self
                .mpv
                .as_mut()
                .ok_or_else(|| AppError::Playback("mpv not attached".into()))?;
            mpv.set_property("pause", false)
                .map_err(|e| AppError::Playback(e.to_string()))
        }

        fn pause(&mut self) -> Result<(), AppError> {
            let mpv = self
                .mpv
                .as_mut()
                .ok_or_else(|| AppError::Playback("mpv not attached".into()))?;
            mpv.set_property("pause", true)
                .map_err(|e| AppError::Playback(e.to_string()))
        }

        fn stop(&mut self) -> Result<(), AppError> {
            if let Some(mpv) = self.mpv.as_mut() {
                mpv.command("stop", &[] as &[&str])
                    .map_err(|e| AppError::Playback(e.to_string()))?;
            }
            Ok(())
        }

        fn detach(&mut self) {
            self.teardown_mpv();
        }

        fn set_pip_geometry(
            &mut self,
            pip_x: f64,
            pip_y: f64,
            pip_w: f64,
            pip_h: f64,
            win_w: f64,
            win_h: f64,
        ) -> Result<(), AppError> {
            let mpv = self
                .mpv
                .as_mut()
                .ok_or_else(|| AppError::Playback("mpv not attached".into()))?;
            apply_pip_geometry(mpv, pip_x, pip_y, pip_w, pip_h, win_w, win_h)
        }

        fn clear_pip_geometry(&mut self) -> Result<(), AppError> {
            if let Some(mpv) = self.mpv.as_mut() {
                reset_pip_geometry(mpv);
            }
            Ok(())
        }

        fn set_volume(&mut self, level: f64) -> Result<(), AppError> {
            let mpv = self
                .mpv
                .as_mut()
                .ok_or_else(|| AppError::Playback("mpv not attached".into()))?;
            let clamped = level.clamp(0.0, 100.0);
            mpv.set_property("volume", clamped)
                .map_err(|e| AppError::Playback(e.to_string()))
        }

        fn set_muted(&mut self, muted: bool) -> Result<(), AppError> {
            let mpv = self
                .mpv
                .as_mut()
                .ok_or_else(|| AppError::Playback("mpv not attached".into()))?;
            mpv.set_property("mute", muted)
                .map_err(|e| AppError::Playback(e.to_string()))
        }

        fn is_available(&self) -> bool {
            true
        }

        fn engine_name(&self) -> &'static str {
            "libmpv"
        }

        fn stream_stats(&self) -> StreamStats {
            let Some(mpv) = self.mpv.as_ref() else {
                return StreamStats::default();
            };

            fn prop_i64(mpv: &libmpv2::Mpv, name: &str) -> Option<i64> {
                mpv.get_property(name).ok()
            }
            fn prop_f64(mpv: &libmpv2::Mpv, name: &str) -> Option<f64> {
                mpv.get_property(name).ok()
            }
            fn prop_str(mpv: &libmpv2::Mpv, name: &str) -> Option<String> {
                mpv.get_property(name).ok()
            }

            let width = prop_i64(mpv, "width").filter(|v| *v > 0).map(|v| v as u32);
            let height = prop_i64(mpv, "height").filter(|v| *v > 0).map(|v| v as u32);
            let fps = prop_f64(mpv, "estimated-vf-fps")
                .or_else(|| prop_f64(mpv, "container-fps"))
                .filter(|v| *v > 0.0);

            let video_bitrate_kbps = prop_f64(mpv, "video-bitrate")
                .filter(|v| *v > 0.0)
                .map(|bps| bps / 1000.0);
            let audio_bitrate_kbps = prop_f64(mpv, "audio-bitrate")
                .filter(|v| *v > 0.0)
                .map(|bps| bps / 1000.0);

            StreamStats {
                width,
                height,
                fps,
                video_bitrate_kbps,
                audio_bitrate_kbps,
                video_codec: prop_str(mpv, "video-codec"),
                audio_codec: prop_str(mpv, "audio-codec"),
                error: None,
            }
        }
    }

    fn apply_pip_geometry(
        mpv: &Mpv,
        pip_x: f64,
        pip_y: f64,
        pip_w: f64,
        pip_h: f64,
        win_w: f64,
        win_h: f64,
    ) -> Result<(), AppError> {
        if win_w <= 0.0 || win_h <= 0.0 || pip_w <= 0.0 || pip_h <= 0.0 {
            return Ok(());
        }

        let margin_left = (pip_x / win_w).clamp(0.0, 0.999);
        let margin_top = (pip_y / win_h).clamp(0.0, 0.999);
        let margin_right = ((win_w - pip_x - pip_w) / win_w).clamp(0.0, 0.999);
        let margin_bottom = ((win_h - pip_y - pip_h) / win_h).clamp(0.0, 0.999);

        if margin_left + margin_right >= 0.999 || margin_top + margin_bottom >= 0.999 {
            tracing::warn!("pip geometry margins overflow");
            return Ok(());
        }

        mpv.set_property("video-align-x", 0.0f64)
            .map_err(|e| AppError::Playback(e.to_string()))?;
        mpv.set_property("video-align-y", 0.0f64)
            .map_err(|e| AppError::Playback(e.to_string()))?;
        mpv.set_property("video-zoom", 0.0f64)
            .map_err(|e| AppError::Playback(e.to_string()))?;
        mpv.set_property("video-pan-x", 0.0f64)
            .map_err(|e| AppError::Playback(e.to_string()))?;
        mpv.set_property("video-pan-y", 0.0f64)
            .map_err(|e| AppError::Playback(e.to_string()))?;
        mpv.set_property("video-margin-ratio-left", margin_left)
            .map_err(|e| AppError::Playback(e.to_string()))?;
        mpv.set_property("video-margin-ratio-top", margin_top)
            .map_err(|e| AppError::Playback(e.to_string()))?;
        mpv.set_property("video-margin-ratio-right", margin_right)
            .map_err(|e| AppError::Playback(e.to_string()))?;
        mpv.set_property("video-margin-ratio-bottom", margin_bottom)
            .map_err(|e| AppError::Playback(e.to_string()))?;
        Ok(())
    }

    fn reset_pip_geometry(mpv: &Mpv) {
        let _ = mpv.set_property("video-zoom", 0.0f64);
        let _ = mpv.set_property("video-pan-x", 0.0f64);
        let _ = mpv.set_property("video-pan-y", 0.0f64);
        let _ = mpv.set_property("video-align-x", 0.0f64);
        let _ = mpv.set_property("video-align-y", 0.0f64);
        let _ = mpv.set_property("video-margin-ratio-top", 0.0f64);
        let _ = mpv.set_property("video-margin-ratio-right", 0.0f64);
        let _ = mpv.set_property("video-margin-ratio-left", 0.0f64);
        let _ = mpv.set_property("video-margin-ratio-bottom", 0.0f64);
    }
}

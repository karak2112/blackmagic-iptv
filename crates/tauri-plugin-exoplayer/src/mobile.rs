use tauri::{plugin::PluginHandle, Runtime};

#[cfg(target_os = "android")]
use serde::{de::DeserializeOwned, Serialize};

#[cfg(target_os = "android")]
use tauri::{
    plugin::PluginApi,
    AppHandle,
};

#[cfg(target_os = "android")]
use crate::models::StreamStatsResponse;

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "net.blackmagicsoftware.iptv.exoplayer";

#[derive(Debug, Clone)]
pub struct ExoPlayer<R: Runtime>(PluginHandle<R>);

#[cfg(target_os = "android")]
pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> crate::Result<ExoPlayer<R>> {
    let handle = api
        .register_android_plugin(PLUGIN_IDENTIFIER, "ExoPlayerPlugin")
        .map_err(|e| crate::Error::Tauri(e.to_string()))?;
    Ok(ExoPlayer(handle))
}

impl<R: Runtime> ExoPlayer<R> {
    #[cfg(target_os = "android")]
    pub fn load(&self, url: &str) -> crate::Result<()> {
        self.0
            .run_mobile_plugin::<()>("load", LoadArgs { url: url.to_string() })
            .map_err(|e| crate::Error::Tauri(e.to_string()))
    }

    #[cfg(target_os = "android")]
    pub fn play(&self) -> crate::Result<()> {
        self.0
            .run_mobile_plugin::<()>("play", ())
            .map_err(|e| crate::Error::Tauri(e.to_string()))
    }

    #[cfg(target_os = "android")]
    pub fn pause(&self) -> crate::Result<()> {
        self.0
            .run_mobile_plugin::<()>("pause", ())
            .map_err(|e| crate::Error::Tauri(e.to_string()))
    }

    #[cfg(target_os = "android")]
    pub fn stop(&self) -> crate::Result<()> {
        self.0
            .run_mobile_plugin::<()>("stop", ())
            .map_err(|e| crate::Error::Tauri(e.to_string()))
    }

    #[cfg(target_os = "android")]
    pub fn set_volume(&self, level: f64) -> crate::Result<()> {
        self.0
            .run_mobile_plugin::<()>(
                "setVolume",
                VolumeArgs {
                    level: level.clamp(0.0, 100.0),
                },
            )
            .map_err(|e| crate::Error::Tauri(e.to_string()))
    }

    #[cfg(target_os = "android")]
    pub fn set_muted(&self, muted: bool) -> crate::Result<()> {
        self.0
            .run_mobile_plugin::<()>("setMuted", MuteArgs { muted })
            .map_err(|e| crate::Error::Tauri(e.to_string()))
    }

    #[cfg(target_os = "android")]
    pub fn show_player(&self) -> crate::Result<()> {
        self.0
            .run_mobile_plugin::<()>("showPlayer", ())
            .map_err(|e| crate::Error::Tauri(e.to_string()))
    }

    #[cfg(target_os = "android")]
    pub fn hide_player(&self) -> crate::Result<()> {
        self.0
            .run_mobile_plugin::<()>("hidePlayer", ())
            .map_err(|e| crate::Error::Tauri(e.to_string()))
    }

    #[cfg(target_os = "android")]
    pub fn stream_stats(&self) -> crate::Result<StreamStatsResponse> {
        self.0
            .run_mobile_plugin::<StreamStatsResponse>("getStats", ())
            .map_err(|e| crate::Error::Tauri(e.to_string()))
    }
}

#[cfg(target_os = "android")]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LoadArgs {
    url: String,
}

#[cfg(target_os = "android")]
#[derive(Serialize)]
struct VolumeArgs {
    level: f64,
}

#[cfg(target_os = "android")]
#[derive(Serialize)]
struct MuteArgs {
    muted: bool,
}

use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

mod error;
mod mobile;
mod models;

pub use error::{Error, Result};
pub use mobile::ExoPlayer;
pub use models::StreamStatsResponse;

pub trait ExoPlayerExt<R: Runtime> {
    fn try_exoplayer(&self) -> Option<&ExoPlayer<R>>;
}

impl<R: Runtime, T: Manager<R>> ExoPlayerExt<R> for T {
    fn try_exoplayer(&self) -> Option<&ExoPlayer<R>> {
        self.try_state::<ExoPlayer<R>>().map(|state| state.inner())
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("exoplayer")
        .setup(|app, api| {
            #[cfg(target_os = "android")]
            match mobile::init(app, api) {
                Ok(exoplayer) => {
                    app.manage(exoplayer);
                }
                Err(e) => tracing::error!("ExoPlayer plugin init failed: {e}"),
            }
            #[cfg(not(target_os = "android"))]
            let _ = (app, api);
            Ok(())
        })
        .build()
}

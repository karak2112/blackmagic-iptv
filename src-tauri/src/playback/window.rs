use raw_window_handle::{HasWindowHandle, RawWindowHandle};

use crate::error::AppError;

#[derive(Debug, Clone, Copy)]
pub struct NativeWindowId(pub i64);

pub fn window_id_from_tauri(window: &tauri::WebviewWindow) -> Result<NativeWindowId, AppError> {
    let handle = window
        .window_handle()
        .map_err(|e| AppError::Playback(format!("window handle: {e}")))?;

    let wid = match handle.as_raw() {
        RawWindowHandle::Win32(h) => h.hwnd.get() as i64,
        RawWindowHandle::Xlib(h) => h.window as i64,
        RawWindowHandle::Xcb(h) => h.window.get() as i64,
        RawWindowHandle::AppKit(h) => h.ns_view.as_ptr() as i64,
        other => {
            return Err(AppError::Playback(format!(
                "unsupported window handle: {other:?}"
            )));
        }
    };

    Ok(NativeWindowId(wid))
}

pub fn set_player_chrome(window: &tauri::WebviewWindow, active: bool) -> Result<(), AppError> {
    use tauri::window::Color;
    let color = if active {
        Color(0, 0, 0, 0)
    } else {
        Color(13, 15, 20, 255)
    };
    window
        .set_background_color(Some(color))
        .map_err(|e| AppError::Playback(e.to_string()))
}

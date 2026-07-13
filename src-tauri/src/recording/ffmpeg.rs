use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use crate::error::AppError;
use crate::fetch::redact_url;

const CONNECT_GRACE: Duration = Duration::from_millis(800);

pub struct FfmpegRecording {
    child: Child,
}

impl FfmpegRecording {
    pub fn start(stream_url: &str, output: &Path, udp_port: u16) -> Result<Self, AppError> {
        let ffmpeg = find_ffmpeg().ok_or_else(|| {
            AppError::Other(
                "Recording requires ffmpeg. Install ffmpeg and add it to PATH, or place ffmpeg.exe next to the app.".into(),
            )
        })?;

        let output_path = output.to_string_lossy().replace('\\', "/");
        let tee_spec = format!(
            "[f=mpegts:onfail=ignore]{output_path}|[f=mpegts]udp://127.0.0.1:{udp_port}?pkt_size=1316"
        );

        tracing::info!(
            "starting ffmpeg tee recording for {} -> {}",
            redact_url(stream_url),
            output.display()
        );

        let mut child = Command::new(&ffmpeg)
            .args([
                "-hide_banner",
                "-loglevel",
                "warning",
                "-reconnect",
                "1",
                "-reconnect_at_eof",
                "1",
                "-reconnect_streamed",
                "1",
                "-reconnect_delay_max",
                "5",
                "-user_agent",
                "iptv-player/0.1",
                "-i",
                stream_url,
                "-c",
                "copy",
                "-map",
                "0",
                "-f",
                "tee",
                &tee_spec,
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| AppError::Other(format!("failed to start ffmpeg: {e}")))?;

        std::thread::sleep(CONNECT_GRACE);

        if let Ok(Some(status)) = child.try_wait() {
            return Err(AppError::Other(format!(
                "ffmpeg exited immediately with status {status}"
            )));
        }

        Ok(Self { child })
    }

    pub fn stop(mut self) -> Result<(), AppError> {
        let _ = self.child.kill();
        match self.child.wait() {
            Ok(status) => {
                tracing::info!("ffmpeg recording stopped ({status})");
                Ok(())
            }
            Err(e) => Err(AppError::Other(format!("failed to stop ffmpeg: {e}"))),
        }
    }
}

pub fn find_ffmpeg() -> Option<PathBuf> {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let bundled = dir.join("ffmpeg.exe");
            if bundled.is_file() {
                return Some(bundled);
            }
        }
    }

    let dev = PathBuf::from("third_party/ffmpeg/win/ffmpeg.exe");
    if dev.is_file() {
        return Some(dev);
    }

    ffmpeg_on_path()
}

pub fn ffmpeg_available() -> bool {
    find_ffmpeg().is_some()
}

fn ffmpeg_on_path() -> Option<PathBuf> {
    let output = Command::new("where").arg("ffmpeg").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8(output.stdout).ok()?;
    let line = stdout.lines().next()?.trim();
    if line.is_empty() {
        None
    } else {
        Some(PathBuf::from(line))
    }
}

pub fn pick_udp_port() -> Result<u16, AppError> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|e| AppError::Other(format!("failed to allocate UDP port: {e}")))?;
    Ok(listener
        .local_addr()
        .map_err(|e| AppError::Other(format!("failed to read UDP port: {e}")))?
        .port())
}

pub fn udp_playback_url(port: u16) -> String {
    format!("udp://127.0.0.1:{port}?overrun_nonfatal=1&fifo_size=5000000")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn udp_url_has_buffer_flags() {
        let url = udp_playback_url(41234);
        assert!(url.contains("overrun_nonfatal=1"));
        assert!(url.contains("41234"));
    }
}

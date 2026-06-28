use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use futures_util::StreamExt;
use iptv_core::{IptvError, Result, UrlValidator};

/// Default cap for M3U / small remote fetches.
pub const PLAYLIST_MAX_BYTES: u64 = 50 * 1024 * 1024;
/// XMLTV guides from IPTV providers are often 50–300+ MB.
pub const EPG_MAX_BYTES: u64 = 512 * 1024 * 1024;

pub struct FetchLimits {
    pub max_bytes: u64,
    pub timeout: Duration,
}

impl FetchLimits {
    pub fn playlist() -> Self {
        Self {
            max_bytes: PLAYLIST_MAX_BYTES,
            timeout: Duration::from_secs(120),
        }
    }

    pub fn epg() -> Self {
        Self {
            max_bytes: EPG_MAX_BYTES,
            timeout: Duration::from_secs(600),
        }
    }
}

impl Default for FetchLimits {
    fn default() -> Self {
        Self::playlist()
    }
}

pub struct HttpFetcher {
    client: reqwest::Client,
    limits: FetchLimits,
    validator: UrlValidator,
}

impl HttpFetcher {
    pub fn new(limits: FetchLimits) -> Self {
        let client = reqwest::Client::builder()
            .timeout(limits.timeout)
            .user_agent("iptv-player/0.1")
            .build()
            .expect("failed to build HTTP client");

        Self {
            client,
            limits,
            validator: UrlValidator::default(),
        }
    }

    pub fn with_epg_limits(&self) -> Self {
        Self::new(FetchLimits::epg())
    }

    pub async fn fetch_text(&self, url: &str) -> Result<String> {
        let path = self.fetch_to_path(url, self.limits.max_bytes).await?;
        std::fs::read_to_string(&path).map_err(IptvError::from)
    }

    /// Stream a remote file to disk (avoids holding large EPG/XML in RAM).
    pub async fn fetch_to_path(&self, url: &str, max_bytes: u64) -> Result<PathBuf> {
        let validated = self.validator.validate_fetch_url(url)?;
        let response = self
            .client
            .get(&validated)
            .send()
            .await
            .map_err(|e| IptvError::Parse(format!("HTTP request failed: {e}")))?;

        if !response.status().is_success() {
            return Err(IptvError::Parse(format!(
                "HTTP {} for {}",
                response.status(),
                redact_url(&validated)
            )));
        }

        if let Some(len) = response.content_length() {
            if len > max_bytes {
                return Err(IptvError::Parse(format!(
                    "response too large: {} bytes (max {})",
                    len, max_bytes
                )));
            }
        }

        let mut dest = tempfile::Builder::new()
            .prefix("iptv-fetch-")
            .suffix(".tmp")
            .tempfile()
            .map_err(IptvError::from)?;
        let mut stream = response.bytes_stream();
        let mut total = 0u64;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| IptvError::Parse(format!("HTTP read failed: {e}")))?;
            total += chunk.len() as u64;
            if total > max_bytes {
                return Err(IptvError::Parse(format!(
                    "response exceeded max size of {max_bytes} bytes"
                )));
            }
            dest.write_all(&chunk).map_err(IptvError::from)?;
        }

        dest.flush().map_err(IptvError::from)?;
        let (_file, path) = dest
            .keep()
            .map_err(|e| IptvError::Io(e.error))?;
        Ok(path)
    }

    pub fn read_local(&self, path: &str) -> Result<String> {
        let validated = self.validator.validate_local_path(path)?;
        let metadata = std::fs::metadata(&validated)?;
        if metadata.len() > self.limits.max_bytes {
            return Err(IptvError::Parse(format!(
                "file too large: {} bytes (max {})",
                metadata.len(),
                self.limits.max_bytes
            )));
        }
        std::fs::read_to_string(&validated).map_err(IptvError::from)
    }

    /// Open a user-selected local file for streaming parse (no size cap).
    pub fn open_local_stream(&self, path: &str) -> Result<std::fs::File> {
        let validated = self.validator.validate_local_path(path)?;
        std::fs::File::open(validated).map_err(IptvError::from)
    }

    pub fn read_local_bytes(&self, path: &Path) -> Result<Vec<u8>> {
        let path_str = path.to_string_lossy();
        let validated = self.validator.validate_local_path(&path_str)?;
        let metadata = std::fs::metadata(&validated)?;
        if metadata.len() > self.limits.max_bytes {
            return Err(IptvError::Parse(format!(
                "file too large: {} bytes",
                metadata.len()
            )));
        }
        std::fs::read(&validated).map_err(IptvError::from)
    }
}

impl Default for HttpFetcher {
    fn default() -> Self {
        Self::new(FetchLimits::default())
    }
}

pub fn redact_url(url: &str) -> String {
    if let Ok(mut parsed) = url::Url::parse(url) {
        if parsed.password().is_some() {
            let _ = parsed.set_password(Some("***"));
        }
        parsed.to_string()
    } else {
        "[invalid-url]".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_password() {
        let redacted = redact_url("http://user:secret@example.com/live");
        assert!(!redacted.contains("secret"));
    }

    #[test]
    fn epg_limit_is_larger_than_playlist() {
        assert!(EPG_MAX_BYTES > PLAYLIST_MAX_BYTES);
    }
}

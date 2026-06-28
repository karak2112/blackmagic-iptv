use crate::error::{IptvError, Result};
use url::Url;

const DEFAULT_ALLOWED_SCHEMES: &[&str] = &["http", "https", "rtmp", "rtsp", "udp"];

#[derive(Debug, Clone)]
pub struct UrlValidator {
    allowed_schemes: Vec<String>,
}

impl Default for UrlValidator {
    fn default() -> Self {
        Self {
            allowed_schemes: DEFAULT_ALLOWED_SCHEMES
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
        }
    }
}

impl UrlValidator {
    pub fn new(allowed_schemes: Vec<String>) -> Self {
        Self { allowed_schemes }
    }

    pub fn validate_stream_url(&self, raw: &str) -> Result<String> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(IptvError::InvalidUrl("empty URL".into()));
        }

        let parsed = Url::parse(trimmed)
            .map_err(|e| IptvError::InvalidUrl(format!("{trimmed}: {e}")))?;

        let scheme = parsed.scheme().to_ascii_lowercase();
        if !self.allowed_schemes.iter().any(|s| s == &scheme) {
            return Err(IptvError::DisallowedScheme(scheme));
        }

        if parsed.host().is_none() && scheme != "udp" {
            return Err(IptvError::InvalidUrl(format!("missing host: {trimmed}")));
        }

        Ok(trimmed.to_string())
    }

    pub fn validate_fetch_url(&self, raw: &str) -> Result<String> {
        let trimmed = raw.trim();
        let parsed = Url::parse(trimmed)
            .map_err(|e| IptvError::InvalidUrl(format!("{trimmed}: {e}")))?;

        let scheme = parsed.scheme().to_ascii_lowercase();
        if scheme != "http" && scheme != "https" {
            return Err(IptvError::DisallowedScheme(scheme));
        }

        Ok(trimmed.to_string())
    }

    pub fn validate_local_path(&self, path: &str) -> Result<String> {
        let trimmed = path.trim();
        if trimmed.is_empty() {
            return Err(IptvError::InvalidUrl("empty path".into()));
        }
        if trimmed.contains("..") {
            return Err(IptvError::InvalidUrl("path traversal not allowed".into()));
        }
        Ok(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_http_stream() {
        let v = UrlValidator::default();
        assert!(v.validate_stream_url("http://example.com/live.m3u8").is_ok());
    }

    #[test]
    fn rejects_file_scheme() {
        let v = UrlValidator::default();
        assert!(v.validate_stream_url("file:///etc/passwd").is_err());
    }

    #[test]
    fn rejects_javascript() {
        let v = UrlValidator::default();
        assert!(v.validate_stream_url("javascript:alert(1)").is_err());
    }
}

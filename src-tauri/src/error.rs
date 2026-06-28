use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Iptv(#[from] iptv_core::IptvError),
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("playback error: {0}")]
    Playback(String),
    #[error("{0}")]
    Other(String),
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        AppError::Iptv(iptv_core::IptvError::Io(value))
    }
}

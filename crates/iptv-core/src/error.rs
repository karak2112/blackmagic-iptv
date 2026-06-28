use thiserror::Error;

#[derive(Debug, Error)]
pub enum IptvError {
    #[error("invalid URL: {0}")]
    InvalidUrl(String),

    #[error("URL scheme not allowed: {0}")]
    DisallowedScheme(String),

    #[error("parse error: {0}")]
    Parse(String),

    #[error("XML error: {0}")]
    Xml(#[from] quick_xml::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, IptvError>;

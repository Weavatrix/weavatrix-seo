//! Crawler errors.

use std::fmt::{Display, Formatter, Result as FmtResult};
use weavatrix_seo_model::SeoError;

/// Recoverable crawl failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrawlError {
    /// URL contract error.
    Url(SeoError),
    /// Transport or protocol failure.
    Transport(String),
    /// TLS requested but the `tls` feature is off.
    TlsDisabled,
    /// Body or header budget exceeded.
    Budget(String),
}

impl Display for CrawlError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Url(error) => write!(formatter, "{error}"),
            Self::Transport(message) | Self::Budget(message) => write!(formatter, "{message}"),
            Self::TlsDisabled => {
                write!(formatter, "https URLs require the `tls` feature")
            }
        }
    }
}

impl std::error::Error for CrawlError {}

impl From<SeoError> for CrawlError {
    fn from(error: SeoError) -> Self {
        Self::Url(error)
    }
}

impl From<weavatrix_seo_http::HttpError> for CrawlError {
    fn from(error: weavatrix_seo_http::HttpError) -> Self {
        match error {
            weavatrix_seo_http::HttpError::Transport(message) => Self::Transport(message),
            weavatrix_seo_http::HttpError::TlsDisabled => Self::TlsDisabled,
            weavatrix_seo_http::HttpError::Budget(message) => Self::Budget(message),
        }
    }
}

/// Crawler result alias.
pub type Result<T> = std::result::Result<T, CrawlError>;

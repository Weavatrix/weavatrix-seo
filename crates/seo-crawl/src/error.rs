//! Crawler errors.

use std::fmt::{Display, Formatter, Result as FmtResult};
use weavatrix_seo_model::{FetchOutcome, SeoError};

/// Recoverable crawl failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrawlError {
    /// URL contract error.
    Url(SeoError),
    /// Transport or protocol failure.
    Transport(String),
    /// DNS lookup failed.
    Dns(String),
    /// Connect or read timeout.
    Timeout(String),
    /// TLS handshake failed.
    Tls(String),
    /// TLS requested but the `tls` feature is off.
    TlsDisabled,
    /// Body or header budget exceeded.
    Budget(String),
    /// Network policy blocked the destination.
    Blocked(String),
}

impl CrawlError {
    /// Maps onto a retained fetch outcome.
    #[must_use]
    pub const fn outcome(&self) -> FetchOutcome {
        match self {
            Self::Url(_) => FetchOutcome::ParseFailure,
            Self::Dns(_) => FetchOutcome::Dns,
            Self::Timeout(_) => FetchOutcome::Timeout,
            Self::Tls(_) | Self::TlsDisabled => FetchOutcome::Tls,
            Self::Budget(_) => FetchOutcome::BodyLimit,
            Self::Blocked(_) => FetchOutcome::Blocked,
            Self::Transport(_) => FetchOutcome::Transport,
        }
    }
}

impl Display for CrawlError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Url(error) => write!(formatter, "{error}"),
            Self::Transport(message)
            | Self::Dns(message)
            | Self::Timeout(message)
            | Self::Tls(message)
            | Self::Budget(message)
            | Self::Blocked(message) => write!(formatter, "{message}"),
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
            weavatrix_seo_http::HttpError::Dns(message) => Self::Dns(message),
            weavatrix_seo_http::HttpError::Timeout(message) => Self::Timeout(message),
            weavatrix_seo_http::HttpError::Tls(message) => Self::Tls(message),
            weavatrix_seo_http::HttpError::TlsDisabled => Self::TlsDisabled,
            weavatrix_seo_http::HttpError::Budget(message) => Self::Budget(message),
            weavatrix_seo_http::HttpError::Blocked(message) => Self::Blocked(message),
        }
    }
}

/// Crawler result alias.
pub type Result<T> = std::result::Result<T, CrawlError>;

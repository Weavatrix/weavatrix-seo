//! Transport errors.

use std::fmt::{Display, Formatter, Result as FmtResult};

/// Recoverable HTTP failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpError {
    /// Socket, TLS handshake, or protocol failure.
    Transport(String),
    /// HTTPS requested without the `tls` feature.
    TlsDisabled,
    /// Header or body exceeded the configured cap.
    Budget(String),
}

impl Display for HttpError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Transport(message) | Self::Budget(message) => write!(formatter, "{message}"),
            Self::TlsDisabled => write!(formatter, "https URLs require the `tls` feature"),
        }
    }
}

impl std::error::Error for HttpError {}

impl From<weavatrix_seo_model::SeoError> for HttpError {
    fn from(error: weavatrix_seo_model::SeoError) -> Self {
        Self::Transport(error.to_string())
    }
}

/// HTTP result alias.
pub type Result<T> = std::result::Result<T, HttpError>;

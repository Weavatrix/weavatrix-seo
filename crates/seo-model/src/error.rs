//! Model-level errors.

use std::fmt::{Display, Formatter, Result as FmtResult};

/// Recoverable contract error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeoError {
    /// Empty or whitespace-only identifier.
    EmptyIdentity(&'static str),
    /// URL that is not an absolute `http`/`https` locator.
    InvalidUrl(String),
    /// Relative resolution produced a non-http locator.
    UnresolvableUrl(String),
}

impl Display for SeoError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::EmptyIdentity(field) => write!(formatter, "{field} must not be empty"),
            Self::InvalidUrl(value) => write!(formatter, "invalid absolute URL: {value}"),
            Self::UnresolvableUrl(value) => write!(formatter, "cannot resolve URL: {value}"),
        }
    }
}

impl std::error::Error for SeoError {}

/// Result alias for model construction.
pub type Result<T> = std::result::Result<T, SeoError>;

//! Exact locator for a finding or fact.

use crate::AbsoluteUrl;
use serde::{Deserialize, Serialize};

/// Where a fact was observed.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Locator {
    /// Whole URL.
    Url(String),
    /// HTTP header name.
    Header { url: String, name: String },
    /// HTML/DOM selector or token span.
    Dom { url: String, path: String },
    /// JSON-LD object path.
    JsonLd { url: String, path: String },
    /// Sitemap document URL plus listed loc.
    Sitemap { sitemap: String, loc: String },
    /// Repository path and optional span.
    Source {
        path: String,
        start_line: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        end_line: Option<u32>,
    },
}

impl Locator {
    /// URL locator.
    #[must_use]
    pub fn url(url: &AbsoluteUrl) -> Self {
        Self::Url(url.to_string())
    }

    /// Header locator.
    #[must_use]
    pub fn header(url: &AbsoluteUrl, name: impl Into<String>) -> Self {
        Self::Header {
            url: url.to_string(),
            name: name.into(),
        }
    }

    /// Source span locator.
    #[must_use]
    pub fn source_span(path: impl Into<String>, start_line: Option<u32>, end_line: Option<u32>) -> Self {
        Self::Source {
            path: path.into(),
            start_line,
            end_line,
        }
    }

    /// DOM locator.
    #[must_use]
    pub fn dom(url: &AbsoluteUrl, path: impl Into<String>) -> Self {
        Self::Dom {
            url: url.to_string(),
            path: path.into(),
        }
    }

    /// Primary URL or source path used by the CI baseline.
    #[must_use]
    pub fn subject_url(&self) -> &str {
        match self {
            Self::Url(url)
            | Self::Header { url, .. }
            | Self::Dom { url, .. }
            | Self::JsonLd { url, .. } => url,
            Self::Sitemap { loc, .. } => loc,
            Self::Source { path, .. } => path,
        }
    }
}

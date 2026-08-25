//! Response media classification. HTML-only rules must not fire on PDFs.

use serde::{Deserialize, Serialize};

/// Kind of a fetched body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaKind {
    /// HTML or XHTML document.
    Html,
    /// JSON or JSON-LD API body.
    Json,
    /// PDF.
    Pdf,
    /// Image.
    Image,
    /// XML that is not HTML (sitemaps are handled separately).
    Xml,
    /// Anything else, including empty or unknown.
    Other,
}

impl Default for MediaKind {
    fn default() -> Self {
        Self::Html
    }
}

impl MediaKind {
    /// True when HTML-oriented extractors and findings may run.
    #[must_use]
    pub const fn is_html(self) -> bool {
        matches!(self, Self::Html)
    }

    /// Classifies a response from Content-Type, then sniffs the body.
    #[must_use]
    pub fn classify(content_type: Option<&str>, body: &str) -> Self {
        let mime = content_type
            .unwrap_or("")
            .split(';')
            .next()
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();
        match mime.as_str() {
            "text/html" | "application/xhtml+xml" => Self::Html,
            "application/json" | "application/ld+json" | "text/json" => Self::Json,
            "application/pdf" => Self::Pdf,
            "application/xml" | "text/xml" => Self::Xml,
            other if other.starts_with("image/") => Self::Image,
            "" => sniff(body),
            _ => Self::Other,
        }
    }
}

fn sniff(body: &str) -> MediaKind {
    let trimmed = body.trim_start();
    if trimmed.is_empty() {
        return MediaKind::Other;
    }
    let lower = trimmed.get(..64).unwrap_or(trimmed).to_ascii_lowercase();
    if lower.starts_with("<!doctype html") || lower.starts_with("<html") {
        MediaKind::Html
    } else if lower.starts_with('{') || lower.starts_with('[') {
        MediaKind::Json
    } else if lower.starts_with("%pdf") {
        MediaKind::Pdf
    } else {
        MediaKind::Other
    }
}

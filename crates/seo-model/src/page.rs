//! Extracted HTTP page used as search-surface evidence.

use crate::{AbsoluteUrl, ContentHash, Evidence};
use serde::{Deserialize, Serialize};

/// Indexability conclusion from response signals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Indexability {
    /// No blocking robots/canonical-away signal.
    Indexable,
    /// `noindex` in robots meta or `X-Robots-Tag`.
    Noindex,
    /// Canonical points at a different URL.
    Canonicalized,
    /// Redirected away.
    Redirected,
    /// Non-success HTTP status.
    Error,
}

/// One redirect hop.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedirectHop {
    /// Requested URL.
    pub from: String,
    /// Location target.
    pub to: String,
    /// HTTP status.
    pub status: u16,
}

/// `hreflang` alternate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Alternate {
    /// Language or `x-default`.
    pub hreflang: String,
    /// Target href.
    pub href: String,
}

/// Heading text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Heading {
    /// Heading level, 1–6.
    pub level: u8,
    /// Visible text.
    pub text: String,
}

/// Image reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageRef {
    /// Image URL.
    pub src: String,
    /// Alt text when present.
    pub alt: Option<String>,
}

/// Parsed JSON-LD document or a syntax failure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonLd {
    /// Raw script body.
    pub raw: String,
    /// Recognized `@type` values when JSON parsed.
    pub types: Vec<String>,
    /// True when the script was valid JSON.
    pub valid_json: bool,
}

/// One crawled URL after extraction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractedPage {
    /// Final URL after redirects.
    pub url: AbsoluteUrl,
    /// Requested URL before redirects.
    pub requested: AbsoluteUrl,
    /// HTTP status of the final response.
    pub status: u16,
    /// Redirect chain, empty when none.
    pub redirects: Vec<RedirectHop>,
    /// Response `Content-Type`.
    pub content_type: Option<String>,
    /// Canonical href when present.
    pub canonical: Option<String>,
    /// Robots directives from meta and headers.
    pub robots: Vec<String>,
    /// Title text.
    pub title: Option<String>,
    /// Meta description.
    pub description: Option<String>,
    /// HTML `lang`.
    pub html_lang: Option<String>,
    /// Hreflang alternates.
    pub alternates: Vec<Alternate>,
    /// Headings in document order.
    pub headings: Vec<Heading>,
    /// Internal and external `a[href]` targets.
    pub links: Vec<String>,
    /// Images.
    pub images: Vec<ImageRef>,
    /// JSON-LD blocks.
    pub json_ld: Vec<JsonLd>,
    /// Extracted main-content text.
    pub text: String,
    /// Script/RSC payload used for market and claim integrity.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub payload: String,
    /// Hash of normalized main-content text.
    pub content_hash: ContentHash,
    /// Indexability from this response.
    pub indexability: Indexability,
    /// Whether this URL was listed in a sitemap.
    pub in_sitemap: bool,
    /// Whether a crawled page linked here.
    pub linked_from_page: bool,
    /// Evidence for the extraction.
    pub evidence: Evidence,
}

impl ExtractedPage {
    /// Builds a page and derives content hash plus indexability.
    #[must_use]
    pub fn finalize(mut self) -> Self {
        self.content_hash = ContentHash::of_str(&normalize_text(&self.text));
        self.indexability = classify(&self);
        self
    }
}

fn classify(page: &ExtractedPage) -> Indexability {
    if !page.redirects.is_empty() {
        return Indexability::Redirected;
    }
    if page.status >= 400 {
        return Indexability::Error;
    }
    if page.robots.iter().any(|value| {
        value
            .split(',')
            .any(|token| token.trim().eq_ignore_ascii_case("noindex"))
    }) {
        return Indexability::Noindex;
    }
    if let Some(canonical) = &page.canonical {
        if let (Ok(canon), url) = (AbsoluteUrl::parse(canonical), &page.url)
            && &canon != url
        {
            return Indexability::Canonicalized;
        }
    }
    Indexability::Indexable
}

fn normalize_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

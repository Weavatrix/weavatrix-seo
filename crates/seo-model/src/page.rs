//! Extracted HTTP page used as search-surface evidence.

use crate::{AbsoluteUrl, ContentHash, Evidence, LinkRef, MediaKind};
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
    /// Alt text. `None` means the attribute is absent; `Some("")` is decorative.
    pub alt: Option<String>,
    /// `aria-hidden` or `role=presentation|none`.
    #[serde(default)]
    pub hidden: bool,
}

/// Parsed JSON-LD document or a syntax failure.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct JsonLd {
    /// Raw script body.
    pub raw: String,
    /// Recognized `@type` values when JSON parsed.
    pub types: Vec<String>,
    /// True when the script was valid JSON.
    pub valid_json: bool,
    /// `@id` values on Organization/WebSite nodes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ids: Vec<String>,
    /// `sameAs` values on Organization/WebSite nodes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub same_as: Vec<String>,
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
    /// Classified body kind.
    #[serde(default)]
    pub media: MediaKind,
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
    /// Hyperlinks with anchor/rel/location.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub link_refs: Vec<LinkRef>,
    /// Images.
    pub images: Vec<ImageRef>,
    /// JSON-LD blocks.
    pub json_ld: Vec<JsonLd>,
    /// Visible body text (not headings, not chrome).
    pub text: String,
    /// Concatenated heading text. Hashed with `text` as `visible_text`.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub heading_text: String,
    /// Text collected inside `main`.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub main_text: String,
    /// Recognized RSC / Next.js app data. May participate in claim/market logic.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub payload: String,
    /// Arbitrary inline script. Never serialized; never used as public copy.
    #[serde(default, skip_serializing)]
    pub arbitrary_script: String,
    /// Open Graph title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub og_title: Option<String>,
    /// Open Graph description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub og_description: Option<String>,
    /// Open Graph image URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub og_image: Option<String>,
    /// Selected response headers (name, value), lowercased names.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub headers: Vec<(String, String)>,
    /// Response body size in bytes (lossy UTF-8 length).
    #[serde(default)]
    pub body_bytes: usize,
    /// Fetch duration in milliseconds.
    #[serde(default)]
    pub fetch_ms: u32,
    /// Document exposes a `main` landmark.
    #[serde(default)]
    pub has_main: bool,
    /// Interactive controls without an accessible name.
    #[serde(default)]
    pub unlabeled_controls: usize,
    /// Hash of normalized `visible_text` (`heading_text` + body `text`).
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
        self.content_hash = ContentHash::of_str(&self.visible_text());
        self.indexability = classify(&self);
        self
    }

    /// Documented hash surface: headings then body, whitespace-normalized.
    #[must_use]
    pub fn visible_text(&self) -> String {
        normalize_text(&format!("{} {}", self.heading_text, self.text))
    }

    /// Lowercased response header value.
    #[must_use]
    pub fn header(&self, name: &str) -> Option<&str> {
        let name = name.to_ascii_lowercase();
        self.headers
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| value.as_str())
    }
}

fn classify(page: &ExtractedPage) -> Indexability {
    if (300..400).contains(&page.status) {
        return Indexability::Redirected;
    }
    if page.status >= 400 || page.status == 0 {
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
        let resolved = AbsoluteUrl::parse(canonical).or_else(|_| page.url.join(canonical));
        if let Ok(canon) = resolved
            && canon != page.url
        {
            return Indexability::Canonicalized;
        }
    }
    Indexability::Indexable
}

fn normalize_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

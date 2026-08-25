//! HTML extraction entry.

use super::walk::Walker;
use weavatrix_parse::Language;
use weavatrix_parse::token::{Mode, Tokenizer};
use weavatrix_seo_model::{Alternate, Heading, ImageRef};

/// Extraction before URL/status are attached.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExtractedPageDraft {
    /// `html[lang]`
    pub html_lang: Option<String>,
    /// Document title.
    pub title: Option<String>,
    /// Meta description.
    pub description: Option<String>,
    /// Canonical href.
    pub canonical: Option<String>,
    /// Robots meta content.
    pub robots: Vec<String>,
    /// Hreflang alternates.
    pub alternates: Vec<Alternate>,
    /// Headings.
    pub headings: Vec<Heading>,
    /// `a[href]` values.
    pub links: Vec<String>,
    /// Hyperlinks with semantics.
    pub link_refs: Vec<weavatrix_seo_model::LinkRef>,
    /// Images.
    pub images: Vec<ImageRef>,
    /// JSON-LD blocks.
    pub json_ld: Vec<weavatrix_seo_model::JsonLd>,
    /// Visible body text (not headings).
    pub text: String,
    /// Concatenated heading text.
    pub heading_text: String,
    /// Text collected inside `main`.
    pub main_text: String,
    /// Recognized RSC / Next data. May participate in claim/market logic.
    pub payload: String,
    /// Arbitrary inline JavaScript. Never used as public copy.
    pub arbitrary_script: String,
    /// Open Graph title.
    pub og_title: Option<String>,
    /// Open Graph description.
    pub og_description: Option<String>,
    /// Open Graph image.
    pub og_image: Option<String>,
    /// `main` landmark present.
    pub has_main: bool,
    /// Controls without an accessible name.
    pub unlabeled_controls: usize,
    /// `meta http-equiv=Content-Security-Policy`.
    pub csp_meta: Option<String>,
}

/// Extracts SEO-visible fields from HTML.
#[must_use]
pub fn extract_html(html: &str) -> ExtractedPageDraft {
    let tokens: Vec<_> = Tokenizer::new(html, Language::Html)
        .mode(Mode::Lite)
        .collect();
    let mut walker = Walker::new(html, &tokens);
    walker.run();
    walker.finish()
}

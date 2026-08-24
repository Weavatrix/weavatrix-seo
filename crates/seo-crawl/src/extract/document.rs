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
    /// Images.
    pub images: Vec<ImageRef>,
    /// JSON-LD blocks.
    pub json_ld: Vec<weavatrix_seo_model::JsonLd>,
    /// Visible main text.
    pub text: String,
    /// Script/RSC payload text used for market and claim integrity.
    pub payload: String,
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

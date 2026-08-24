//! HTML extraction from a raw HTTP body.

mod document;
mod jsonld;

pub use document::{ExtractedPageDraft, extract_html};

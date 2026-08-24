//! HTML extraction from a raw HTTP body.

mod controls;
mod document;
mod jsonld;
mod meta;
mod tag;
mod walk;

pub use document::{ExtractedPageDraft, extract_html};

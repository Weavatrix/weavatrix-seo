//! Next.js App Router adapter. Returns unmeasured until source wiring lands.

#![forbid(unsafe_code)]

use weavatrix_seo_source::{SourceSurface, unmeasured};

/// Predicts the Next.js search surface for `repo`.
#[must_use]
pub fn predict(repo: &str) -> SourceSurface {
    unmeasured(repo)
}

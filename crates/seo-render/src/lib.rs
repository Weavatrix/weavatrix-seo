//! Browser render adapter. This crate does not own a browser engine.
//!
//! WVQ/Playwright write `weavatrix-seo-render/v1` JSON. SEO only reconciles
//! those observations against the HTTP inventory.

#![forbid(unsafe_code)]

mod observation;
mod reconcile;

use weavatrix_seo_model::{Evidence, EvidenceSource, LayerState};

pub use observation::{RenderSnapshot, RenderedPage, from_json, load};
pub use reconcile::reconcile;

/// How rendering was attempted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    /// Raw HTTP only.
    RawOnly,
    /// Representative templates rendered.
    Sampled,
    /// Explicit URLs rendered.
    Requested,
}

/// Result of a render pass. Absence is `unmeasured`, never a pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderReport {
    /// Mode used.
    pub mode: RenderMode,
    /// Layer classification for canonical/title/schema when measured.
    pub states: Vec<(String, LayerState)>,
    /// Evidence for the pass.
    pub evidence: Evidence,
}

/// Default: rendering was not measured.
#[must_use]
pub fn unmeasured() -> RenderReport {
    RenderReport {
        mode: RenderMode::RawOnly,
        states: Vec::new(),
        evidence: Evidence::unmeasured(EvidenceSource::RenderedDom),
    }
}

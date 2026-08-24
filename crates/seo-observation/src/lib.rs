//! Provider observation contracts. No global index is built here.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use weavatrix_seo_model::{Evidence, EvidenceSource};

/// One query-URL observation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observation {
    /// Query text when known.
    pub query: Option<String>,
    /// URL.
    pub url: String,
    /// Provider name.
    pub provider: String,
    /// Evidence. Never promoted to deterministic.
    pub evidence: Evidence,
}

/// Snapshot of imported observations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservationSnapshot {
    /// Rows.
    pub rows: Vec<Observation>,
    /// Whether any provider was connected.
    pub connected: bool,
}

/// No provider is connected.
#[must_use]
pub fn unmeasured() -> ObservationSnapshot {
    ObservationSnapshot {
        rows: Vec::new(),
        connected: false,
    }
}

/// Evidence for an unconnected provider.
#[must_use]
pub fn unmeasured_evidence() -> Evidence {
    Evidence::unmeasured(EvidenceSource::Provider)
}

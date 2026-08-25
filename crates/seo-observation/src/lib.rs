//! Provider observation contracts. GSC is the first import.

#![forbid(unsafe_code)]

mod gsc;
mod provider;

use serde::{Deserialize, Serialize};
use weavatrix_seo_model::{Evidence, EvidenceSource};

pub use gsc::{disconnected, from_json, load};
pub use provider::{from_any, load_any};

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
    /// Clicks when the provider supplied them.
    #[serde(default)]
    pub clicks: u32,
    /// Impressions when the provider supplied them.
    #[serde(default)]
    pub impressions: u32,
    /// Average position when known (whole ranks).
    #[serde(default)]
    pub position: u32,
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

/// Demand/visibility for one URL from a snapshot.
#[must_use]
pub fn axes_for(snapshot: &ObservationSnapshot, url: &str) -> (Option<u16>, Option<u16>) {
    if !snapshot.connected {
        return (None, None);
    }
    let mut impressions = 0_u32;
    let mut best_position = 0_u32;
    for row in snapshot.rows.iter().filter(|row| urls_match(&row.url, url)) {
        impressions = impressions.saturating_add(row.impressions);
        if row.position > 0 && (best_position == 0 || row.position < best_position) {
            best_position = row.position;
        }
    }
    if impressions == 0 && best_position == 0 {
        return (None, None);
    }
    let demand = u16::try_from((impressions / 10).min(100)).unwrap_or(100);
    let gap = if best_position > 10 {
        u16::try_from(((best_position - 10) * 5).min(100)).unwrap_or(100)
    } else {
        0
    };
    (Some(demand), Some(gap))
}

fn urls_match(left: &str, right: &str) -> bool {
    left.trim_end_matches('/') == right.trim_end_matches('/')
}

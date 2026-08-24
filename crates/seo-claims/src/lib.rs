//! Public claims versus domain facts.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use weavatrix_seo_model::{Evidence, EvidenceSource};

/// A statement published on a URL.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claim {
    /// Claim identity.
    pub id: String,
    /// URL that publishes it.
    pub url: String,
    /// Visible or structured text.
    pub text: String,
}

/// Authoritative fact that may support or contradict a claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fact {
    /// Fact identity.
    pub id: String,
    /// Domain field or policy key.
    pub field: String,
    /// Recorded value.
    pub value: String,
}

/// Claim review outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimReview {
    /// Claim.
    pub claim: Claim,
    /// Supporting facts.
    pub supported_by: Vec<Fact>,
    /// Contradicting facts.
    pub contradicted_by: Vec<Fact>,
    /// Evidence grade.
    pub evidence: Evidence,
}

/// No domain pack is wired in 0.0.1.
#[must_use]
pub fn unmeasured() -> Evidence {
    Evidence::unmeasured(EvidenceSource::Repo)
}

//! Public competitor graph comparison. Copies no competitor prose.

#![forbid(unsafe_code)]

use weavatrix_seo_model::{Evidence, EvidenceSource, Opportunity};

/// Comparison request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompareRequest {
    /// Owned site.
    pub site: String,
    /// Public competitor origins.
    pub competitors: Vec<String>,
}

/// Compare is unmeasured until the bounded public crawl is wired.
#[must_use]
pub fn compare(request: &CompareRequest) -> Vec<Opportunity> {
    let _ = request;
    vec![Opportunity::unmeasured_demand(
        "cluster_gap",
        "compare",
        "Competitor comparison is unmeasured in this version",
        "Public competitor graphs are not crawled until compare mode is wired.",
        "Re-run compare after the public-site adapter is connected.",
    )]
}

/// Evidence for an unrun comparison.
#[must_use]
pub fn unmeasured_evidence() -> Evidence {
    Evidence::unmeasured(EvidenceSource::Http)
}

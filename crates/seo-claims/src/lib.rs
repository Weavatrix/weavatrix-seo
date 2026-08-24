//! Public claims, domain facts, and market-entity integrity.

#![forbid(unsafe_code)]

mod license;
mod market;
mod repo;

use weavatrix_seo_model::{Evidence, EvidenceSource, Finding, Inventory};

pub use license::{audit_claims, false_facts, page_claims};
pub use market::{Market, audit_pages, foreign_entities, infer_market};
pub use repo::{RepoSignals, scan as scan_repo};

/// Combined live + repo integrity pass.
#[must_use]
pub fn audit(inventory: &Inventory, repo: Option<&str>) -> Vec<Finding> {
    let mut findings = audit_pages(inventory);
    let signals = repo.map(scan_repo);
    if let Some(signals) = &signals {
        findings.extend(signals.findings.clone());
        findings.extend(audit_claims(
            inventory,
            signals.license_false,
            signals.license_field,
        ));
    } else {
        findings.extend(audit_claims(inventory, false, false));
    }
    findings
}

/// Legacy helper kept for callers that only need an unmeasured marker.
#[must_use]
pub fn unmeasured() -> Evidence {
    Evidence::unmeasured(EvidenceSource::Repo)
}

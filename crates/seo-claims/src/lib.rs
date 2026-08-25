//! Public claims, domain facts, and market-entity integrity.

#![forbid(unsafe_code)]

mod entity;
mod license;
mod market;
mod pack;
mod repo;

use weavatrix_seo_model::{Evidence, EvidenceSource, Finding, Inventory};

pub use entity::audit as audit_entities;
pub use license::{audit_claims, false_facts, page_claims};
pub use market::{audit_pages, foreign_entities, infer_market};
pub use pack::{Market, PolicyPack, US_WA, all as packs};
pub use repo::{RepoSignals, scan as scan_repo};

/// Combined live + repo integrity pass.
#[must_use]
pub fn audit(inventory: &Inventory, repo: Option<&str>) -> Vec<Finding> {
    let mut findings = audit_pages(inventory);
    let signals = repo.map(scan_repo);
    if let Some(signals) = &signals {
        findings.extend(signals.findings.clone());
        findings.extend(audit_claims(inventory, &signals.pack_false()));
    } else {
        findings.extend(audit_claims(inventory, &[]));
    }
    findings.extend(audit_entities(inventory));
    findings
}

/// Legacy helper kept for callers that only need an unmeasured marker.
#[must_use]
pub fn unmeasured() -> Evidence {
    Evidence::unmeasured(EvidenceSource::Repo)
}

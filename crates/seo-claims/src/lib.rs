//! Public claims, domain facts, and market-entity integrity.

#![forbid(unsafe_code)]

mod cite;
mod entity;
mod graph;
mod license;
mod local;
mod market;
mod pack;
mod repo;

use weavatrix_seo_model::{Evidence, EvidenceSource, Finding, Inventory};

pub use cite::audit as audit_cite;
pub use entity::audit as audit_entities;
pub use graph::{DomainGraph, claim_id, domain_graph, entity_id, field_id, market_id, policy_id};
pub use license::{audit_claims, false_facts, page_claims};
pub use local::audit as audit_local;
pub use market::{audit_pages, foreign_entities, infer_market};
pub use pack::{Market, PolicyPack, US_WA, all as packs};
pub use repo::{RepoSignals, scan as scan_repo};

/// Combined live + repo integrity pass.
#[must_use]
pub fn audit(inventory: &Inventory, repo: Option<&str>) -> Vec<Finding> {
    audit_with_graph(inventory, repo).0
}

/// Integrity findings plus the domain layer of the evidence graph.
///
/// Both come from one repository scan, so a finding and the graph it should be
/// explained through cannot be built from different reads of the source.
#[must_use]
pub fn audit_with_graph(inventory: &Inventory, repo: Option<&str>) -> (Vec<Finding>, DomainGraph) {
    let mut findings = audit_pages(inventory);
    let signals = repo.map(scan_repo);
    if let Some(signals) = &signals {
        findings.extend(signals.findings.clone());
        findings.extend(audit_claims(inventory, &signals.pack_false()));
    } else {
        findings.extend(audit_claims(inventory, &[]));
    }
    findings.extend(audit_entities(inventory));
    findings.extend(audit_local(inventory));
    findings.extend(audit_cite(inventory));
    let graph = domain_graph(inventory, signals.as_ref());
    (findings, graph)
}

/// Legacy helper kept for callers that only need an unmeasured marker.
#[must_use]
pub fn unmeasured() -> Evidence {
    Evidence::unmeasured(EvidenceSource::Repo)
}

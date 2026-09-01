//! First-party artifacts a URL-list crawler does not emit.
//!
//! Commercial crawler names stay out of this crate. The bench tree is the
//! only place that may mention them, and only as optional external baselines.

use weavatrix_seo_model::{AuditReport, SearchNodeKind};

/// One first-party artifact and whether this report produced it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artifact {
    /// Stable id used in tests and benches.
    pub id: &'static str,
    /// Whether the report contains the artifact.
    pub present: bool,
    /// Why a URL-list crawler cannot substitute this.
    pub note: &'static str,
}

/// Scores an audit against first-party artifacts generic crawlers omit.
#[must_use]
pub fn score(report: &AuditReport) -> Vec<Artifact> {
    let intel = report.intelligence.as_ref();
    let nodes = &report.inventory.nodes;
    vec![
        Artifact {
            id: "evidence_semantics",
            present: report.inventory.semantics.is_some(),
            note: "Snapshot comparability identity. URL lists have no rule digest.",
        },
        Artifact {
            id: "rule_authority",
            present: !report.findings.is_empty(),
            note: "Every finding names why the rule is legitimate, not only severity.",
        },
        Artifact {
            id: "content_profiles",
            present: intel.is_some_and(|item| !item.profiles.is_empty()),
            note: "Per-page MATTR/entropy/fact density. Not a crawler column.",
        },
        Artifact {
            id: "chunks",
            present: intel.is_some_and(|item| !item.chunks.is_empty())
                || nodes.iter().any(|node| node.kind == SearchNodeKind::Chunk),
            note: "Heading-bounded retrieval chunks bound onto the evidence graph.",
        },
        Artifact {
            id: "search_graph",
            present: !nodes.is_empty(),
            note: "Heterogeneous Search Evidence Graph, not a URL table.",
        },
        Artifact {
            id: "unknown_stays_unknown",
            present: report
                .opportunities
                .iter()
                .all(|item| item.demand != "UNMEASURED" || item.axes.confidence != Some(100)),
            note: "Missing confidence is not scored as 100. Generic crawlers zero-fill.",
        },
        Artifact {
            id: "source_producers",
            present: !report.inventory.producers.is_empty(),
            note: "Route/metadata symbols. Requires a repository, which URL crawlers never see.",
        },
        Artifact {
            id: "domain_claims",
            present: nodes.iter().any(|node| node.kind == SearchNodeKind::Claim)
                || nodes.iter().any(|node| node.kind == SearchNodeKind::Entity),
            note: "Claim/entity/policy nodes. Not extracted from competitor prose.",
        },
        Artifact {
            id: "structural_compare",
            present: report.opportunities.iter().any(|item| {
                matches!(
                    item.kind.as_str(),
                    "cluster_gap" | "schema_gap" | "market_gap" | "link_gap" | "content_gap"
                )
            }) || report.inventory.mode != weavatrix_seo_model::AnalysisMode::Compare,
            note: "Compare emits structural gaps only. No competitor copy is stored.",
        },
    ]
}

/// Count of present artifacts versus the catalogue size.
#[must_use]
pub fn tally(report: &AuditReport) -> (usize, usize) {
    let items = score(report);
    (
        items.iter().filter(|item| item.present).count(),
        items.len(),
    )
}

/// Site-only reports still owe the crawl-backed first-party set.
#[must_use]
pub fn site_backed_ids() -> &'static [&'static str] {
    &[
        "evidence_semantics",
        "rule_authority",
        "content_profiles",
        "chunks",
        "search_graph",
        "unknown_stays_unknown",
    ]
}

//! Heterogeneous Search Evidence Graph.

use weavatrix_seo::{AnalysisMode, AuditRequest, run_audit};
use weavatrix_seo_model::{Relation, SearchNodeKind};

#[test]
fn hybrid_binds_url_to_route_and_metadata_symbol() {
    let root = format!(
        "{}/../seo-nextjs/tests/fixtures",
        env!("CARGO_MANIFEST_DIR").replace('\\', "/")
    );
    let report = run_audit(&AuditRequest {
        mode: AnalysisMode::Repo,
        repo: Some(root),
        ..AuditRequest::default()
    })
    .expect("repo audit");
    assert!(
        report
            .inventory
            .nodes
            .iter()
            .any(|node| node.kind == SearchNodeKind::RouteFamily),
        "{:?}",
        report.inventory.nodes
    );
    assert!(
        report.inventory.facts.iter().any(|fact| {
            fact.relation == Relation::MetadataFrom
                && fact.target_kind == SearchNodeKind::SourceSymbol
        }),
        "{:?}",
        report.inventory.facts
    );
}

//! Heterogeneous Search Evidence Graph.

use weavatrix_seo::{AnalysisMode, AuditRequest, explain_chain, run_audit};
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
    assert!(
        report
            .inventory
            .producers
            .iter()
            .any(|item| item.path.contains("citySeo")),
        "{:?}",
        report.inventory.producers
    );
}

#[test]
fn every_declared_producer_reaches_the_graph() {
    // A dedicated repository: the shared fixture deliberately has no JSON-LD
    // producer, because another test proves that case emits ENTITY-002.
    let root = std::env::temp_dir().join(format!(
        "wvx-seo-producers-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_nanos())
    ));
    let page_dir = root.join("src").join("app").join("[city]");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&page_dir).expect("app dir");
    std::fs::write(
        page_dir.join("page.tsx"),
        "export async function generateMetadata() {\n  return { title: \"x\" };\n}\n\nexport function generateStaticParams() {\n  return [{ city: \"vancouver\" }];\n}\n\nexport function jsonLd(city: string) {\n  return { \"@type\": \"Service\", areaServed: city };\n}\n\nexport default function Page() {\n  return null;\n}\n",
    )
    .expect("page");
    let report = run_audit(&AuditRequest {
        mode: AnalysisMode::Repo,
        repo: Some(root.to_string_lossy().into_owned()),
        ..AuditRequest::default()
    })
    .expect("repo audit");
    let bound: Vec<&str> = report
        .inventory
        .facts
        .iter()
        .filter(|fact| fact.target_kind == SearchNodeKind::SourceSymbol)
        .map(|fact| fact.target.as_str())
        .collect();
    for producer in ["generateMetadata", "generateStaticParams", "jsonLd"] {
        assert!(
            bound.iter().any(|target| target.ends_with(producer)),
            "{producer} is extracted but never bound: {bound:?}"
        );
    }
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn explain_chain_reaches_metadata_symbol() {
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
    let finding = report
        .findings
        .iter()
        .find(|item| {
            item.code.starts_with("WVX-SEO-RENDER") || item.code.starts_with("WVX-SEO-PROG")
        })
        .expect("finding");
    let explanation = explain_chain(&report, &finding.fingerprint).expect("chain");
    assert!(
        explanation
            .chain
            .iter()
            .any(|hop| hop.kind == "route" || hop.kind == "symbol"),
        "{:?}",
        explanation.chain
    );
}

//! Imported WVQ render observations versus HTTP.

use std::collections::BTreeMap;
use weavatrix_seo::{AuditRequest, run_audit};
use weavatrix_seo_model::{Relation, SearchNodeKind};

mod common;

use common::{html, page, spawn};

#[test]
fn render_title_drift_is_measured() {
    let mut pages = BTreeMap::new();
    pages.insert(
        "/".into(),
        page(
            200,
            html(
                "HTTP Home",
                "<link rel=\"canonical\" href=\"/\">",
                "<h1>Home</h1><p>Hi.</p>",
            ),
        ),
    );
    let site = spawn(pages);
    let origin = format!("{}/", site.base);
    let path = std::env::temp_dir().join(format!("wvx-seo-render-{}.json", std::process::id()));
    std::fs::write(
        &path,
        format!(
            r#"{{"schema":"weavatrix-seo-render/v1","source":"wvq","pages":[{{"url":"{origin}","title":"Rendered Home","canonical":"/","h1":"Home"}}]}}"#
        ),
    )
    .expect("write");
    let report = run_audit(&AuditRequest {
        site: Some(origin),
        max_pages: Some(4),
        workers: Some(1),
        render: Some(path.to_string_lossy().into_owned()),
        ..AuditRequest::default()
    })
    .expect("audit");
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "WVX-SEO-RENDER-003"),
        "{:?}",
        report
            .findings
            .iter()
            .map(|finding| finding.code.as_str())
            .collect::<Vec<_>>()
    );
    assert!(
        report
            .axes
            .iter()
            .any(|axis| axis.axis == "render_reconciliation" && !axis.unmeasured)
    );
    assert!(
        report.inventory.facts.iter().any(|fact| {
            fact.relation == Relation::ObservedAs
                && fact.target_kind == SearchNodeKind::SearchObservation
        }),
        "{:?}",
        report.inventory.facts
    );
    let _ = std::fs::remove_file(path);
}

#[test]
fn hybrid_without_render_file_stays_unmeasured() {
    let root = format!(
        "{}/../seo-nextjs/tests/fixtures",
        env!("CARGO_MANIFEST_DIR").replace('\\', "/")
    );
    let report = run_audit(&AuditRequest {
        mode: weavatrix_seo::AnalysisMode::Repo,
        repo: Some(root),
        ..AuditRequest::default()
    })
    .expect("repo audit");
    assert!(
        report
            .axes
            .iter()
            .any(|axis| axis.axis == "render_reconciliation" && axis.unmeasured)
    );
}

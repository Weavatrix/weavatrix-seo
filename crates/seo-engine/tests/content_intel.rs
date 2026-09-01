//! Content intelligence, query DSL, and evidence semantics.

use std::collections::BTreeMap;
use weavatrix_seo::{AuditRequest, retrieve, run_audit, run_on_report};
use weavatrix_seo_model::{RuleAuthority, SearchNodeKind};

mod common;

use common::{html, page, spawn};

fn fixture() -> BTreeMap<String, common::Page> {
    let mut pages = BTreeMap::new();
    let city = |name: &str, extra: &str| {
        page(
            200,
            html(
                &format!("Electrician in {name}"),
                &format!(
                    "<link rel=\"canonical\" href=\"/category/electrician/{}\">",
                    name.to_ascii_lowercase().replace(' ', "-")
                ),
                &format!(
                    "<h1>Electrician in {name}</h1><p>Licensed electrician serving {name}. {extra}</p><p>Same-day service. Permit required.</p>"
                ),
            ),
        )
    };
    pages.insert(
        "/".into(),
        page(
            200,
            html(
                "Home",
                "<link rel=\"canonical\" href=\"/\">",
                "<h1>Home</h1><p>Hire a licensed electrician.</p><a href=\"/category/electrician/vancouver-wa\">Vancouver</a><a href=\"/category/electrician/camas-wa\">Camas</a>",
            ),
        ),
    );
    pages.insert(
        "/category/electrician/vancouver-wa".into(),
        city("Vancouver WA", "Clark County permit 98682."),
    );
    pages.insert(
        "/category/electrician/camas-wa".into(),
        city("Camas WA", "Camas municipal license 98607."),
    );
    pages
}

#[test]
fn content_intelligence_and_query_are_additive() {
    let site = spawn(fixture());
    let report = run_audit(&AuditRequest {
        site: Some(format!("{}/", site.base)),
        max_pages: Some(8),
        ..AuditRequest::default()
    })
    .expect("audit");
    let intelligence = report.intelligence.as_ref().expect("intelligence");
    assert_eq!(
        intelligence.semantics.engine_version,
        env!("CARGO_PKG_VERSION")
    );
    assert!(!intelligence.profiles.is_empty());
    assert!(
        intelligence
            .chunks
            .iter()
            .any(|chunk| chunk.url.contains("electrician"))
    );
    assert!(report.findings.iter().any(|finding| finding.authority
        == RuleAuthority::SearchEngineDocumented
        || finding.authority == RuleAuthority::IndustryBestPractice
        || finding.authority == RuleAuthority::ProtocolRequirement));
    assert!(
        report
            .inventory
            .nodes
            .iter()
            .any(|node| node.kind == SearchNodeKind::Chunk),
        "chunks must bind onto the graph"
    );
    let result = run_on_report(
        "FROM urls WHERE indexable = true RETURN url, inbound_links LIMIT 10",
        &report,
    )
    .expect("query");
    assert_eq!(result.collection, "urls");
    assert!(!result.rows.is_empty());
    let hits = retrieve(&report, "licensed electrician vancouver", 5);
    assert!(
        hits.iter().any(|hit| hit.url.contains("electrician")),
        "{hits:?}"
    );
}

#[test]
fn unique_samples_alone_are_not_safe_to_generate() {
    let site = spawn(fixture());
    let report = run_audit(&AuditRequest {
        site: Some(format!("{}/", site.base)),
        max_pages: Some(8),
        ..AuditRequest::default()
    })
    .expect("audit");
    let intelligence = report.intelligence.as_ref().expect("intelligence");
    for matrix in &intelligence.matrices {
        if matrix.family.contains("electrician") {
            assert_ne!(
                matrix.verdict, "SAFE_TO_GENERATE",
                "two unique samples are not enough: {matrix:?}"
            );
            assert!(
                !matrix.unmet_requirements.is_empty()
                    || matrix.verdict == "SAFE_IF_REQUIREMENTS_MET"
                    || matrix.verdict == "REVIEW",
                "{matrix:?}"
            );
        }
    }
}

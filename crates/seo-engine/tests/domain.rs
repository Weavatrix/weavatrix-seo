//! Domain layer: claim, required field, and the source that defines it.

use std::collections::BTreeMap;
use weavatrix_seo::{AnalysisMode, AuditRequest, explain_chain, run_audit};
use weavatrix_seo_model::SearchNodeKind;

mod common;

use common::{html, page, spawn};

const CLAIM_COPY: &str = "<h1>Licensed electrician in Vancouver WA</h1><p>Licensed electrician serving Clark County and Southwest Washington.</p>";

fn contaminated_repo() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("wvx-seo-domain-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let data = dir.join("src").join("data");
    std::fs::create_dir_all(&data).expect("data dir");
    std::fs::write(
        data.join("washington.ts"),
        "export const specialist = {\n  city: \"Vancouver WA\",\n  license_verified: false,\n};\n",
    )
    .expect("data module");
    dir
}

fn hybrid_report(site: &str, repo: &std::path::Path) -> weavatrix_seo::AuditReport {
    run_audit(&AuditRequest {
        mode: AnalysisMode::Hybrid,
        site: Some(site.to_owned()),
        repo: Some(repo.to_string_lossy().into_owned()),
        max_pages: Some(6),
        ..AuditRequest::default()
    })
    .expect("hybrid audit")
}

fn claim_site() -> BTreeMap<String, common::Page> {
    let mut pages = BTreeMap::new();
    pages.insert(
        "/".into(),
        page(
            200,
            html(
                "Home",
                "",
                "<h1>Home</h1><a href=\"/category/electrician/vancouver-wa\">city</a>",
            ),
        ),
    );
    pages.insert(
        "/category/electrician/vancouver-wa".into(),
        page(200, html("Licensed electrician", "", CLAIM_COPY)),
    );
    pages
}

#[test]
fn a_public_claim_reaches_the_field_that_backs_it() {
    let site = spawn(claim_site());
    let repo = contaminated_repo();
    let report = hybrid_report(&format!("{}/", site.base), &repo);

    let kinds: Vec<SearchNodeKind> = report
        .inventory
        .nodes
        .iter()
        .map(|node| node.kind)
        .collect();
    for kind in [
        SearchNodeKind::Claim,
        SearchNodeKind::DataField,
        SearchNodeKind::Policy,
        SearchNodeKind::Market,
        SearchNodeKind::Entity,
    ] {
        assert!(
            kinds.contains(&kind),
            "{kind:?} never reaches the graph: {:?}",
            report
                .inventory
                .nodes
                .iter()
                .map(|node| (node.kind, node.id.clone()))
                .collect::<Vec<_>>()
        );
    }
    let _ = std::fs::remove_dir_all(&repo);
}

#[test]
fn explaining_a_claim_contradiction_names_the_defining_source() {
    let site = spawn(claim_site());
    let repo = contaminated_repo();
    let report = hybrid_report(&format!("{}/", site.base), &repo);

    let finding = report
        .findings
        .iter()
        .find(|finding| finding.code == "WVX-SEO-CLAIM-001")
        .unwrap_or_else(|| {
            panic!(
                "no claim contradiction: {:?}",
                report
                    .findings
                    .iter()
                    .map(|item| item.code.as_str())
                    .collect::<Vec<_>>()
            )
        });
    let explanation = explain_chain(&report, &finding.fingerprint).expect("chain");
    let hops: Vec<(&str, &str)> = explanation
        .chain
        .iter()
        .map(|hop| (hop.kind.as_str(), hop.relation.as_deref().unwrap_or("-")))
        .collect();
    for expected in [
        ("claim", "CLAIMS"),
        ("field", "REQUIRES"),
        ("symbol", "DEFINED_AT"),
        ("policy", "GOVERNED_BY"),
    ] {
        assert!(
            hops.contains(&expected),
            "{expected:?} missing from the chain: {hops:?}"
        );
    }
    let definition = explanation
        .chain
        .iter()
        .find(|hop| hop.relation.as_deref() == Some("DEFINED_AT"))
        .expect("definition hop");
    assert!(
        definition
            .locator
            .as_deref()
            .is_some_and(|path| path.contains("washington.ts")),
        "the chain must reach the file that sets the field: {definition:?}"
    );
    let _ = std::fs::remove_dir_all(&repo);
}

//! Evidence provenance: what one run is allowed to claim.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use weavatrix_seo::{AnalysisMode, AuditRequest, plan_from, run_audit};
use weavatrix_seo_model::{EvidenceSource, Relation, SearchNodeKind};

mod common;

use common::{html, page, spawn};

const SHA: &str = "0123456789abcdef0123456789abcdef01234567";

fn temp_repo(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("wvx-seo-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create repo dir");
    dir
}

fn fake_git(dir: &Path, sha: &str) {
    std::fs::create_dir_all(dir.join(".git")).expect("git dir");
    std::fs::write(dir.join(".git").join("HEAD"), format!("{sha}\n")).expect("HEAD");
}

fn nextjs_fixture() -> String {
    format!(
        "{}/../seo-nextjs/tests/fixtures",
        env!("CARGO_MANIFEST_DIR").replace('\\', "/")
    )
}

fn home() -> BTreeMap<String, common::Page> {
    let mut pages = BTreeMap::new();
    pages.insert(
        "/".into(),
        page(
            200,
            html(
                "Home",
                "<link rel=\"canonical\" href=\"/\">",
                "<h1>Home</h1><p>Welcome.</p>",
            ),
        ),
    );
    pages
}

#[test]
fn live_pages_never_carry_the_worktree_revision() {
    let site = spawn(home());
    let repo = temp_repo("provenance");
    fake_git(&repo, SHA);
    let report = run_audit(&AuditRequest {
        mode: AnalysisMode::Hybrid,
        site: Some(format!("{}/", site.base)),
        repo: Some(repo.to_string_lossy().into_owned()),
        max_pages: Some(4),
        ..AuditRequest::default()
    })
    .expect("hybrid audit");
    assert_eq!(report.inventory.repo_revision.as_deref(), Some(SHA));
    assert!(
        report
            .inventory
            .pages
            .iter()
            .all(|page| page.evidence.revision.is_none()),
        "a crawled response must not claim a source revision: {:?}",
        report
            .inventory
            .pages
            .iter()
            .map(|page| page.evidence.clone())
            .collect::<Vec<_>>()
    );
    assert!(
        report
            .inventory
            .facts
            .iter()
            .all(|fact| fact.relation != Relation::ChangedBy),
        "nothing proves production was built from this worktree"
    );
    assert!(
        report.inventory.facts.iter().any(|fact| {
            fact.relation == Relation::ComparedAgainst
                && fact.target_kind == SearchNodeKind::Revision
        }),
        "{:?}",
        report.inventory.facts
    );
    let _ = std::fs::remove_dir_all(&repo);
}

#[test]
fn source_symbol_facts_carry_repository_provenance() {
    let report = run_audit(&AuditRequest {
        mode: AnalysisMode::Repo,
        repo: Some(nextjs_fixture()),
        ..AuditRequest::default()
    })
    .expect("repo audit");
    let symbols: Vec<_> = report
        .inventory
        .facts
        .iter()
        .filter(|fact| fact.target_kind == SearchNodeKind::SourceSymbol)
        .collect();
    assert!(!symbols.is_empty(), "{:?}", report.inventory.facts);
    assert!(
        symbols
            .iter()
            .all(|fact| fact.evidence.source == EvidenceSource::Repo),
        "a parser established these, not an HTTP response: {symbols:?}"
    );
}

#[test]
fn findings_are_bound_to_the_measured_snapshot() {
    let mut pages = home();
    pages.insert(
        "/thin".into(),
        page(
            200,
            "<html><head></head><body><p>no heading</p></body></html>",
        ),
    );
    let site = spawn(pages);
    let report = run_audit(&AuditRequest {
        site: Some(format!("{}/", site.base)),
        max_pages: Some(6),
        ..AuditRequest::default()
    })
    .expect("site audit");
    assert!(!report.findings.is_empty());
    assert!(
        report
            .findings
            .iter()
            .all(|finding| finding.evidence.snapshot_id.is_some()),
        "{:?}",
        report
            .findings
            .iter()
            .filter(|finding| finding.evidence.snapshot_id.is_none())
            .map(|finding| finding.code.clone())
            .collect::<Vec<_>>()
    );
    assert!(
        report
            .findings
            .iter()
            .all(|finding| finding.evidence.policy_version.is_some())
    );
    assert!(
        report
            .findings
            .iter()
            .all(|finding| finding.evidence.revision.is_none()),
        "a site-only run has no source revision to bind"
    );
}

#[test]
fn unreadable_search_contract_is_reported_not_ignored() {
    let repo = temp_repo("broken-policy");
    std::fs::create_dir_all(repo.join(".weavatrix")).expect("contract dir");
    std::fs::write(
        repo.join(".weavatrix").join("seo.json"),
        "{ \"indexability\": { \"include\": [ }",
    )
    .expect("write contract");
    let report = run_audit(&AuditRequest {
        mode: AnalysisMode::Repo,
        repo: Some(repo.to_string_lossy().into_owned()),
        ..AuditRequest::default()
    })
    .expect("repo audit");
    assert!(report.inventory.policy.is_none());
    assert!(report.inventory.policy_error.is_some());
    let finding = report
        .findings
        .iter()
        .find(|finding| finding.code == "WVX-SEO-IDX-001")
        .expect("contract finding");
    assert_eq!(finding.evidence.source, EvidenceSource::Repo);
    let _ = std::fs::remove_dir_all(&repo);
}

#[test]
fn plan_carries_the_compiled_matrix_verdict() {
    let body = "<h1>Electrician</h1><p>Licensed electrician with same-day calls.</p>";
    let mut pages = BTreeMap::new();
    pages.insert(
        "/".into(),
        page(
            200,
            html(
                "Home",
                "",
                "<h1>Home</h1><a href=\"/category/electrician/vancouver-wa\">a</a><a href=\"/category/electrician/camas-wa\">b</a>",
            ),
        ),
    );
    pages.insert(
        "/category/electrician/vancouver-wa".into(),
        page(200, html("Electrician", "", body)),
    );
    pages.insert(
        "/category/electrician/camas-wa".into(),
        page(200, html("Electrician", "", body)),
    );
    let site = spawn(pages);
    let report = run_audit(&AuditRequest {
        site: Some(format!("{}/", site.base)),
        max_pages: Some(8),
        ..AuditRequest::default()
    })
    .expect("site audit");
    let plan = plan_from(&report);
    let action = plan
        .actions
        .iter()
        .find(|action| action.subject == "category/electrician")
        .expect("matrix action");
    assert_eq!(
        action.programmatic_verdict.as_deref(),
        Some("CONSOLIDATE"),
        "{:?}",
        plan.actions
    );
    assert!(
        plan.actions
            .iter()
            .filter(|action| !action.subject.contains("category/"))
            .all(|action| action.programmatic_verdict.is_none()),
        "a URL that is not a matrix family has no verdict"
    );
}

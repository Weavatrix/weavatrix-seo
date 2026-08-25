//! v0.0.7 truth and safety acceptance.

use std::collections::BTreeMap;
use weavatrix_seo::{AuditRequest, baseline_from_report, evaluate_gate, run_audit};
use weavatrix_seo_model::{FetchOutcome, Indexability, MediaKind, Relation};

mod common;

use common::{Page, html, page, spawn};

#[test]
fn redirect_target_stays_indexable() {
    let mut pages = BTreeMap::new();
    pages.insert(
        "/".into(),
        page(
            200,
            html(
                "Home",
                "<link rel=\"canonical\" href=\"/\">",
                "<h1>Home</h1><p>Hi</p><a href=\"/old\">Old</a>",
            ),
        ),
    );
    pages.insert(
        "/old".into(),
        Page {
            status: 301,
            headers: vec![("Location".into(), "/new".into())],
            body: String::new(),
        },
    );
    pages.insert(
        "/new".into(),
        page(
            200,
            html(
                "New",
                "<link rel=\"canonical\" href=\"/new\">",
                "<h1>New</h1><p>Landed.</p>",
            ),
        ),
    );
    let site = spawn(pages);
    let report = run_audit(&AuditRequest {
        site: Some(format!("{}/", site.base)),
        max_pages: Some(10),
        workers: Some(2),
        ..AuditRequest::default()
    })
    .expect("audit");
    let old = report
        .inventory
        .pages
        .iter()
        .find(|page| page.url.path() == "/old")
        .expect("old");
    let new = report
        .inventory
        .pages
        .iter()
        .find(|page| page.url.path() == "/new")
        .expect("new");
    assert_eq!(old.indexability, Indexability::Redirected);
    assert_eq!(new.indexability, Indexability::Indexable);
    assert!(
        report
            .inventory
            .edges
            .iter()
            .any(|edge| edge.relation == Relation::RedirectsTo
                && edge.source.path() == "/old"
                && edge.target.path() == "/new")
    );
}

#[test]
fn pdf_does_not_get_html_findings() {
    let mut pages = BTreeMap::new();
    pages.insert(
        "/".into(),
        page(
            200,
            html(
                "Home",
                "<link rel=\"canonical\" href=\"/\">",
                "<h1>Home</h1><p>Doc</p><a href=\"/file.pdf\">PDF</a>",
            ),
        ),
    );
    pages.insert(
        "/file.pdf".into(),
        Page {
            status: 200,
            headers: vec![("Content-Type".into(), "application/pdf".into())],
            body: "%PDF-1.4 fake".into(),
        },
    );
    let site = spawn(pages);
    let report = run_audit(&AuditRequest {
        site: Some(format!("{}/", site.base)),
        max_pages: Some(8),
        ..AuditRequest::default()
    })
    .expect("audit");
    let pdf = report
        .inventory
        .pages
        .iter()
        .find(|page| page.url.path() == "/file.pdf")
        .expect("pdf");
    assert_eq!(pdf.media, MediaKind::Pdf);
    assert!(!report.findings.iter().any(|finding| {
        finding.code == "WVX-SEO-CONTENT-001" && finding.locator.subject_url().contains("file.pdf")
    }));
}

#[test]
fn arbitrary_script_does_not_trigger_market() {
    let mut pages = BTreeMap::new();
    pages.insert(
        "/".into(),
        page(
            200,
            html(
                "Home",
                "<link rel=\"canonical\" href=\"/\">",
                "<h1>Southwest Washington</h1><p>Clark County electrician.</p><script>const unitPiece='IEC'; const x='Hevrat HaHashmal';</script>",
            ),
        ),
    );
    let site = spawn(pages);
    let report = run_audit(&AuditRequest {
        site: Some(format!("{}/", site.base)),
        max_pages: Some(4),
        ..AuditRequest::default()
    })
    .expect("audit");
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.code.starts_with("WVX-SEO-MARKET")),
        "{:?}",
        report.findings
    );
}

#[test]
fn public_only_blocks_loopback() {
    let mut pages = BTreeMap::new();
    pages.insert("/".into(), page(200, html("Home", "", "<h1>Home</h1>")));
    let site = spawn(pages);
    let report = run_audit(&AuditRequest {
        site: Some(format!("{}/", site.base)),
        max_pages: Some(4),
        allow_private: false,
        ..AuditRequest::default()
    })
    .expect("audit");
    assert!(
        report
            .inventory
            .observations
            .iter()
            .any(|item| item.outcome == FetchOutcome::Blocked)
            || report.inventory.pages.is_empty(),
        "{:?}",
        report.inventory.observations
    );
}

#[test]
fn smaller_crawl_is_not_resolved() {
    let mut pages = BTreeMap::new();
    pages.insert(
        "/".into(),
        page(
            200,
            html(
                "Home",
                "<link rel=\"canonical\" href=\"/\">",
                "<h1>Home</h1><p>Hi</p><a href=\"/bare\">Bare</a>",
            ),
        ),
    );
    pages.insert(
        "/bare".into(),
        Page {
            status: 200,
            headers: vec![("Content-Type".into(), "text/html".into())],
            body: "<html><head><title></title><link rel=\"canonical\" href=\"/bare\"></head><body><p>No heading.</p></body></html>".into(),
        },
    );
    let site = spawn(pages);
    let full = run_audit(&AuditRequest {
        site: Some(format!("{}/", site.base)),
        max_pages: Some(10),
        ..AuditRequest::default()
    })
    .expect("full");
    let baseline = baseline_from_report(&full, "full".into());
    let small = run_audit(&AuditRequest {
        site: Some(format!("{}/", site.base)),
        max_pages: Some(1),
        ..AuditRequest::default()
    })
    .expect("small");
    let verdict = evaluate_gate(&small, Some(&baseline));
    assert!(
        verdict.resolved.is_empty() || !verdict.coverage_regressions.is_empty(),
        "{verdict:?}"
    );
}

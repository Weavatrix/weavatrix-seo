//! Typed observations: what a provider row is allowed to influence.

use std::collections::BTreeMap;
use weavatrix_seo::{AuditRequest, run_audit};

mod common;

use common::{html, page, spawn};

fn write_import(name: &str, body: &str) -> String {
    let path = std::env::temp_dir().join(format!("wvx-seo-{name}-{}.json", std::process::id()));
    std::fs::write(&path, body).expect("write import");
    path.to_string_lossy().into_owned()
}

fn one_page_site() -> BTreeMap<String, common::Page> {
    let mut pages = BTreeMap::new();
    pages.insert(
        "/".into(),
        page(
            200,
            html(
                "Home",
                "<link rel=\"canonical\" href=\"/\">",
                "<h1>Home</h1><p>Welcome.</p><a href=\"/a\">a</a>",
            ),
        ),
    );
    pages.insert(
        "/a".into(),
        page(
            200,
            "<html><head><title>A</title></head><body><p>no heading here</p></body></html>",
        ),
    );
    pages
}

fn axis(report: &weavatrix_seo::AuditReport, name: &str) -> weavatrix_seo_model::AxisScore {
    report
        .axes
        .iter()
        .find(|axis| axis.axis == name)
        .unwrap_or_else(|| panic!("axis {name} missing from {:?}", report.axes))
        .clone()
}

#[test]
fn ai_visibility_is_unmeasured_without_a_citation() {
    let site = spawn(one_page_site());
    let report = run_audit(&AuditRequest {
        site: Some(format!("{}/", site.base)),
        max_pages: Some(4),
        ..AuditRequest::default()
    })
    .expect("site audit");
    assert!(
        !axis(&report, "ai_retrieval_readiness").unmeasured,
        "readiness is inferable from the crawled document"
    );
    assert!(
        axis(&report, "ai_visibility").unmeasured,
        "nothing observed a generative answer, so visibility is unmeasured"
    );
}

#[test]
fn an_imported_citation_measures_visibility() {
    let site = spawn(one_page_site());
    let origin = format!("{}/", site.base);
    let import = write_import(
        "ai",
        &format!(
            r#"{{"provider":"perplexity","rows":[{{"url":"{origin}","query":"best electrician"}}]}}"#
        ),
    );
    let report = run_audit(&AuditRequest {
        site: Some(origin),
        max_pages: Some(4),
        observations: Some(import.clone()),
        ..AuditRequest::default()
    })
    .expect("site audit");
    assert!(!axis(&report, "ai_visibility").unmeasured);
    let _ = std::fs::remove_file(import);
}

#[test]
fn bot_hits_never_become_opportunity_demand() {
    let site = spawn(one_page_site());
    let origin = format!("{}/", site.base);
    let import = write_import(
        "logs",
        &format!(r#"{{"provider":"logs","rows":[{{"url":"{origin}a","hits":5000}}]}}"#),
    );
    let report = run_audit(&AuditRequest {
        site: Some(origin.clone()),
        max_pages: Some(4),
        observations: Some(import.clone()),
        ..AuditRequest::default()
    })
    .expect("site audit");
    assert!(
        report
            .opportunities
            .iter()
            .all(|item| item.axes.demand.is_none()),
        "crawler traffic is not search demand: {:?}",
        report
            .opportunities
            .iter()
            .map(|item| (item.subject.clone(), item.axes.demand))
            .collect::<Vec<_>>()
    );
    assert!(
        report
            .findings
            .iter()
            .all(|finding| finding.code != "WVX-SEO-OBS-001"),
        "a bot hit on a crawled URL is not a search-coverage gap"
    );
    let _ = std::fs::remove_file(import);
}

#[test]
fn the_same_counts_from_search_console_do_rank() {
    let site = spawn(one_page_site());
    let origin = format!("{}/", site.base);
    let import = write_import(
        "gsc-rank",
        &format!(
            r#"{{"provider":"gsc","rows":[{{"url":"{origin}a","impressions":5000,"position":18.6}}]}}"#
        ),
    );
    let report = run_audit(&AuditRequest {
        site: Some(origin),
        max_pages: Some(4),
        observations: Some(import.clone()),
        ..AuditRequest::default()
    })
    .expect("site audit");
    assert!(
        report
            .opportunities
            .iter()
            .any(|item| item.axes.demand.is_some()),
        "{:?}",
        report
            .opportunities
            .iter()
            .map(|item| (item.subject.clone(), item.axes.demand))
            .collect::<Vec<_>>()
    );
    let _ = std::fs::remove_file(import);
}

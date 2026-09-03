//! Typed observations: what a provider row is allowed to influence.

use std::collections::BTreeMap;
use weavatrix_seo::{AuditRequest, run_audit};

mod common;

use common::{html, page, spawn};

fn write_import(name: &str, body: &str) -> String {
    let path = common::unique_temp(&format!("wvx-seo-{name}")).with_extension("json");
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
fn gsc_url_is_seeded_ahead_of_the_link_graph() {
    let mut pages = BTreeMap::new();
    pages.insert(
        "/".into(),
        page(
            200,
            html(
                "Home",
                "<link rel=\"canonical\" href=\"/\">",
                "<h1>Home</h1><p>Hi.</p>",
            ),
        ),
    );
    pages.insert(
        "/ranking".into(),
        page(
            200,
            html(
                "Ranking",
                "<link rel=\"canonical\" href=\"/ranking\">",
                "<h1>Ranking</h1><p>Still in Search Console.</p>",
            ),
        ),
    );
    let site = spawn(pages);
    let origin = format!("{}/", site.base);
    let ranking = format!("{origin}ranking");
    let import = write_import(
        "gsc-seed",
        &format!(
            r#"{{"provider":"gsc","rows":[{{"url":"{ranking}","impressions":800,"position":4}}]}}"#
        ),
    );
    let report = run_audit(&AuditRequest {
        site: Some(origin.clone()),
        max_pages: Some(2),
        gsc: Some(import),
        ..AuditRequest::default()
    })
    .expect("audit");
    assert!(
        report
            .inventory
            .pages
            .iter()
            .any(|page| page.url.path() == "/ranking"),
        "GSC-known URL must be measured even when it is not linked: {:?}",
        report
            .inventory
            .pages
            .iter()
            .map(|page| page.url.path().to_owned())
            .collect::<Vec<_>>()
    );
    let discovered = report
        .inventory
        .discovery
        .iter()
        .find(|(url, _)| url.contains("/ranking"))
        .map(|(_, source)| *source);
    assert_eq!(discovered, Some(weavatrix_seo_model::DiscoverySource::Gsc));
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

#[test]
fn invalid_gsc_is_not_absence() {
    let site = spawn(one_page_site());
    let import = write_import("gsc-bad", "{not json");
    let report = run_audit(&AuditRequest {
        site: Some(format!("{}/", site.base)),
        max_pages: Some(4),
        gsc: Some(import.clone()),
        ..AuditRequest::default()
    })
    .expect("audit");
    let finding = report
        .findings
        .iter()
        .find(|item| item.code == "WVX-SEO-OBS-003")
        .expect("invalid input finding");
    assert!(
        finding.summary.contains("GSC_INVALID"),
        "{}",
        finding.summary
    );
    assert_eq!(finding.severity, weavatrix_seo_model::Severity::Error);
    let _ = std::fs::remove_file(import);
}

#[test]
fn gsc_windows_emit_decay_ctr_and_striking_distance() {
    let site = spawn(one_page_site());
    let origin = format!("{}/", site.base);
    let page_a = format!("{origin}a");
    let import = write_import(
        "gsc-intel",
        &format!(
            r#"{{"provider":"gsc","rows":[
                {{"query":"home service","url":"{page_a}","clicks":40,"impressions":800,"position":3,"period":"previous"}},
                {{"query":"home service","url":"{page_a}","clicks":8,"impressions":700,"position":8.0,"period":"current"}}
            ]}}"#
        ),
    );
    let report = run_audit(&AuditRequest {
        site: Some(origin),
        max_pages: Some(4),
        observations: Some(import.clone()),
        ..AuditRequest::default()
    })
    .expect("audit");
    let codes: Vec<&str> = report
        .findings
        .iter()
        .map(|item| item.code.as_str())
        .collect();
    assert!(codes.contains(&"WVX-SEO-OBS-006"), "{codes:?}");
    assert!(codes.contains(&"WVX-SEO-OBS-004"), "{codes:?}");
    assert!(codes.contains(&"WVX-SEO-OBS-005"), "{codes:?}");
    assert!(
        report
            .opportunities
            .iter()
            .any(|item| item.kind == "ctr_gap" && item.axes.expected_ctr.is_some()),
        "{:?}",
        report
            .opportunities
            .iter()
            .map(|item| item.kind.as_str())
            .collect::<Vec<_>>()
    );
    let _ = std::fs::remove_file(import);
}

#[test]
fn gsc_query_without_an_answering_chunk_is_a_passage_gap() {
    let site = spawn(one_page_site());
    let origin = format!("{}/", site.base);
    let import = write_import(
        "gsc-passage",
        &format!(
            r#"{{"provider":"gsc","rows":[{{"query":"licensed electrician permit warranty","url":"{origin}","impressions":90,"clicks":1,"position":11}}]}}"#
        ),
    );
    let report = run_audit(&AuditRequest {
        site: Some(origin),
        max_pages: Some(4),
        observations: Some(import.clone()),
        ..AuditRequest::default()
    })
    .expect("audit");
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "WVX-SEO-CONTENT-004"),
        "{:?}",
        report
            .findings
            .iter()
            .map(|finding| finding.code.as_str())
            .collect::<Vec<_>>()
    );
    let _ = std::fs::remove_file(import);
}

#[test]
fn nginx_googlebot_404_is_logged() {
    let site = spawn(one_page_site());
    let origin = format!("{}/", site.base);
    let import = write_import(
        "nginx-404",
        &format!(
            r#"{{"provider":"nginx","origin":"{origin}","format":"combined","lines":["66.249.66.1 - - [03/Sep/2026:10:00:00 +0000] \"GET /missing HTTP/1.1\" 404 12 \"-\" \"Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)\""]}}"#
        ),
    );
    let report = run_audit(&AuditRequest {
        site: Some(origin),
        max_pages: Some(4),
        observations: Some(import.clone()),
        ..AuditRequest::default()
    })
    .expect("audit");
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "WVX-SEO-OBS-007"),
        "{:?}",
        report
            .findings
            .iter()
            .map(|finding| finding.code.as_str())
            .collect::<Vec<_>>()
    );
    let _ = std::fs::remove_file(import);
}

#[test]
fn ai_discovery_without_citation_fills_the_funnel() {
    let site = spawn(one_page_site());
    let origin = format!("{}/", site.base);
    let import = write_import(
        "ai-funnel",
        &format!(
            r#"{{"provider":"nginx","rows":[{{"url":"{origin}","hits":12,"user_agent":"OAI-SearchBot/1.0"}}]}}"#
        ),
    );
    let report = run_audit(&AuditRequest {
        site: Some(origin),
        max_pages: Some(4),
        observations: Some(import.clone()),
        ..AuditRequest::default()
    })
    .expect("audit");
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "WVX-SEO-OBS-011"),
        "{:?}",
        report
            .findings
            .iter()
            .map(|finding| finding.code.as_str())
            .collect::<Vec<_>>()
    );
    let funnels = &report
        .intelligence
        .as_ref()
        .expect("intelligence")
        .ai_funnels;
    assert!(
        funnels.iter().any(|row| row.discovery_hits == Some(12)),
        "{funnels:?}"
    );
    let _ = std::fs::remove_file(import);
}

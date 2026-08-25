//! Semantic inference, GSC demand, and plan verbs.

use std::collections::BTreeMap;
use weavatrix_seo::{AuditRequest, PlanKind, plan_from, run_audit};

mod common;

use common::{html, page, spawn};

#[test]
fn similar_intent_pages_are_cannibalization_candidates() {
    let mut pages = BTreeMap::new();
    pages.insert(
        "/".into(),
        page(
            200,
            html(
                "Home",
                "<link rel=\"canonical\" href=\"/\">",
                "<h1>Home</h1><p>Hub.</p><a href=\"/a\">A</a><a href=\"/b\">B</a>",
            ),
        ),
    );
    pages.insert(
        "/a".into(),
        page(
            200,
            html(
                "Electrician Vancouver",
                "<link rel=\"canonical\" href=\"/a\">",
                "<h1>Electrician in Vancouver</h1><p>Licensed electrician serving Vancouver with same-day calls and panel upgrades.</p>",
            ),
        ),
    );
    pages.insert(
        "/b".into(),
        page(
            200,
            html(
                "Electrician Vancouver WA",
                "<link rel=\"canonical\" href=\"/b\">",
                "<h1>Electrician in Vancouver</h1><p>Licensed electrician serving Vancouver with same-day calls and panel service.</p>",
            ),
        ),
    );
    let site = spawn(pages);
    let report = run_audit(&AuditRequest {
        site: Some(format!("{}/", site.base)),
        max_pages: Some(8),
        ..AuditRequest::default()
    })
    .expect("audit");
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "WVX-SEO-CANN-001" || finding.code == "WVX-SEO-LINK-004"),
        "{:?}",
        report
            .findings
            .iter()
            .map(|finding| finding.code.as_str())
            .collect::<Vec<_>>()
    );
    let plan = plan_from(&report);
    assert!(
        plan.actions.iter().any(|action| matches!(
            action.kind,
            PlanKind::Link | PlanKind::Improve | PlanKind::Consolidate
        )),
        "{:?}",
        plan.actions
    );
}

#[test]
fn gsc_export_marks_demand_and_unmeasured_urls() {
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
    let site = spawn(pages);
    let origin = format!("{}/", site.base);
    let path = std::env::temp_dir().join(format!("wvx-seo-gsc-{}.json", std::process::id()));
    std::fs::write(
        &path,
        format!(
            r#"{{"rows":[{{"query":"home","url":"{origin}","clicks":2,"impressions":500,"position":4}},{{"query":"missing","url":"{origin}absent","clicks":1,"impressions":80,"position":22}}]}}"#
        ),
    )
    .expect("write");
    let report = run_audit(&AuditRequest {
        site: Some(origin),
        max_pages: Some(4),
        gsc: Some(path.to_string_lossy().into_owned()),
        ..AuditRequest::default()
    })
    .expect("audit");
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "WVX-SEO-OBS-001"),
        "{:?}",
        report.findings
    );
    assert!(
        report
            .axes
            .iter()
            .any(|axis| axis.axis == "observed_search" && !axis.unmeasured)
    );
    let _ = std::fs::remove_file(path);
}

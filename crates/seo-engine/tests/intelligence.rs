//! Semantic inference, GSC demand, and plan verbs.

use std::collections::BTreeMap;
use weavatrix_seo::{AuditRequest, PlanKind, link_inputs, plan_from, run_audit};

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
fn different_services_same_city_are_not_cannibal() {
    let mut pages = BTreeMap::new();
    pages.insert(
        "/".into(),
        page(
            200,
            html(
                "Home",
                "<link rel=\"canonical\" href=\"/\">",
                "<h1>Home</h1><a href=\"/category/electrician/camas-wa\">e</a><a href=\"/category/plumbing/camas-wa\">p</a>",
            ),
        ),
    );
    pages.insert(
        "/category/electrician/camas-wa".into(),
        page(
            200,
            html(
                "Electrician Camas WA",
                "<link rel=\"canonical\" href=\"/category/electrician/camas-wa\">",
                "<h1>Electrician in Camas WA</h1><p>Licensed electrician serving Camas with panel upgrades and same-day calls.</p>",
            ),
        ),
    );
    pages.insert(
        "/category/plumbing/camas-wa".into(),
        page(
            200,
            html(
                "Plumber Camas WA",
                "<link rel=\"canonical\" href=\"/category/plumbing/camas-wa\">",
                "<h1>Plumber in Camas WA</h1><p>Licensed plumber serving Camas with leak repair and same-day calls.</p>",
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
            .all(|finding| finding.code != "WVX-SEO-CANN-001"),
        "{:?}",
        report
            .findings
            .iter()
            .map(|finding| finding.code.as_str())
            .collect::<Vec<_>>()
    );
}

#[test]
fn link_inputs_are_self_contained() {
    let mut pages = BTreeMap::new();
    pages.insert(
        "/".into(),
        page(
            200,
            html(
                "Home",
                "<link rel=\"canonical\" href=\"/\">",
                "<h1>Home</h1><p>Hub of the site.</p><a href=\"/a\">A</a>",
            ),
        ),
    );
    pages.insert(
        "/a".into(),
        page(
            200,
            html(
                "Electrician Camas",
                "<link rel=\"canonical\" href=\"/a\">",
                "<h1>Electrician in Camas</h1><p>Licensed electrician serving Camas with panel upgrades.</p>",
            ),
        ),
    );
    pages.insert(
        "/b".into(),
        page(
            200,
            html(
                "Plumber Camas",
                "<link rel=\"canonical\" href=\"/b\">",
                "<h1>Plumber in Camas</h1><p>Licensed plumber serving Camas with leak repair.</p>",
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
    let inputs = link_inputs(&report);
    assert_eq!(inputs.model, "wvx-seo-lexhash-v1");
    assert_eq!(inputs.dimension, 64);
    assert!(inputs.vectors.len() >= 2, "{:?}", inputs.vectors.len());
    assert_eq!(inputs.vectors.len(), inputs.pages.len());
    assert!(
        inputs
            .vectors
            .iter()
            .all(|row| row.values.len() == inputs.dimension && row.node.starts_with("page:"))
    );
    assert!(
        inputs
            .pages
            .iter()
            .all(|row| !row.site.is_empty() && !row.canonical.is_empty())
    );
    let root = inputs
        .pages
        .iter()
        .find(|row| row.node.ends_with('/'))
        .expect("root profile");
    assert!(root.cornerstone, "{root:?}");
    assert!(
        inputs
            .pages
            .iter()
            .any(|row| !row.existing_targets.is_empty()),
        "{:?}",
        inputs.pages
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
    let path = common::unique_temp("wvx-seo-gsc").with_extension("json");
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
            .inventory
            .pages
            .iter()
            .any(|page| page.url.path() == "/absent")
            || report
                .findings
                .iter()
                .any(|finding| finding.code == "WVX-SEO-CRAWL-001"),
        "a GSC URL is now a crawl seed, so it is measured: {:?}",
        report
            .inventory
            .pages
            .iter()
            .map(|page| page.url.path().to_owned())
            .collect::<Vec<_>>()
    );
    assert!(
        report
            .axes
            .iter()
            .any(|axis| axis.axis == "observed_search" && !axis.unmeasured)
    );
    let _ = std::fs::remove_file(path);
}

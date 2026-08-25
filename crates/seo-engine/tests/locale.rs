//! Locale clusters and query-city cannibalization.

use std::collections::BTreeMap;
use weavatrix_seo::{AuditRequest, plan_from, run_audit};

mod common;

use common::{Page, html, page, spawn};

#[test]
fn locale_twins_without_hreflang_are_i18n_warn() {
    let mut pages = BTreeMap::new();
    pages.insert(
        "/".into(),
        page(
            200,
            html(
                "Home",
                "<link rel=\"canonical\" href=\"/\">",
                "<h1>Home</h1><p>EN.</p><a href=\"/ru\">RU</a>",
            ),
        ),
    );
    pages.insert(
        "/ru".into(),
        page(
            200,
            html(
                "Дом",
                "<link rel=\"canonical\" href=\"/ru\">",
                "<h1>Дом</h1><p>RU.</p><a href=\"/\">EN</a>",
            ),
        ),
    );
    let site = spawn(pages);
    let report = run_audit(&AuditRequest {
        site: Some(format!("{}/", site.base)),
        max_pages: Some(8),
        workers: Some(1),
        ..AuditRequest::default()
    })
    .expect("audit");
    assert!(
        report
            .findings
            .iter()
            .any(|item| item.code == "WVX-SEO-I18N-002"),
        "{:?}",
        report
            .findings
            .iter()
            .map(|item| item.code.as_str())
            .collect::<Vec<_>>()
    );
}

#[test]
fn query_city_cannibalizes_path_landing() {
    let mut pages = BTreeMap::new();
    pages.insert(
        "/".into(),
        page(
            200,
            html(
                "Home",
                "<link rel=\"canonical\" href=\"/\">",
                "<h1>Home</h1><a href=\"/specialists?city=vancouver-wa\">q</a><a href=\"/category/electrician/vancouver-wa\">p</a>",
            ),
        ),
    );
    pages.insert(
        "/specialists".into(),
        page(
            200,
            html(
                "Specialists",
                "<link rel=\"canonical\" href=\"/specialists\">",
                "<h1>Specialists</h1><p>Filter.</p>",
            ),
        ),
    );
    pages.insert(
        "/category/electrician/vancouver-wa".into(),
        page(
            200,
            html(
                "Electrician Vancouver WA",
                "<link rel=\"canonical\" href=\"/category/electrician/vancouver-wa\">",
                "<h1>Electrician in Vancouver WA</h1><p>Licensed electrician.</p>",
            ),
        ),
    );
    let site = spawn(pages);
    let report = run_audit(&AuditRequest {
        site: Some(format!("{}/", site.base)),
        max_pages: Some(8),
        workers: Some(1),
        ..AuditRequest::default()
    })
    .expect("audit");
    let urls: Vec<_> = report
        .inventory
        .pages
        .iter()
        .map(|page| page.url.to_string())
        .collect();
    assert!(
        report
            .findings
            .iter()
            .any(|item| item.code == "WVX-SEO-CANN-002"),
        "urls={urls:?} codes={:?}",
        report
            .findings
            .iter()
            .map(|item| item.code.as_str())
            .collect::<Vec<_>>()
    );
}

#[test]
fn city_path_redirect_to_query_is_cannibal() {
    let mut pages = BTreeMap::new();
    pages.insert(
        "/".into(),
        page(
            200,
            html(
                "Home",
                "<link rel=\"canonical\" href=\"/\">",
                "<h1>Home</h1><a href=\"/ru/cities/yavne\">Yavne</a>",
            ),
        ),
    );
    pages.insert(
        "/ru/cities/yavne".into(),
        Page {
            status: 301,
            headers: vec![("Location".into(), "/ru/specialists?city=yavne".into())],
            body: String::new(),
        },
    );
    pages.insert(
        "/ru/specialists".into(),
        page(
            200,
            html(
                "Specialists",
                "<link rel=\"canonical\" href=\"/ru/specialists\">",
                "<h1>Specialists</h1><p>Yavne list.</p>",
            ),
        ),
    );
    let site = spawn(pages);
    let report = run_audit(&AuditRequest {
        site: Some(format!("{}/", site.base)),
        max_pages: Some(8),
        workers: Some(1),
        ..AuditRequest::default()
    })
    .expect("audit");
    assert!(
        report
            .findings
            .iter()
            .any(|item| item.code == "WVX-SEO-CANN-003"),
        "{:?}",
        report
            .findings
            .iter()
            .map(|item| item.code.as_str())
            .collect::<Vec<_>>()
    );
}

#[test]
fn plan_skips_private_create_families() {
    let root = format!(
        "{}/../seo-nextjs/tests/fixtures",
        env!("CARGO_MANIFEST_DIR").replace('\\', "/")
    );
    let report = run_audit(&weavatrix_seo::AuditRequest {
        mode: weavatrix_seo::AnalysisMode::Repo,
        repo: Some(root),
        ..weavatrix_seo::AuditRequest::default()
    })
    .expect("repo");
    let plan = plan_from(&report);
    assert!(
        plan.actions
            .iter()
            .all(|item| !item.subject.contains("/auth") && !item.subject.contains("/admin")),
        "{:?}",
        plan.actions
            .iter()
            .map(|item| item.subject.as_str())
            .collect::<Vec<_>>()
    );
}

#[test]
fn policy_include_limits_create_families() {
    let root = format!(
        "{}/../seo-nextjs/tests/fixtures",
        env!("CARGO_MANIFEST_DIR").replace('\\', "/")
    );
    let report = run_audit(&weavatrix_seo::AuditRequest {
        mode: weavatrix_seo::AnalysisMode::Repo,
        repo: Some(root),
        ..weavatrix_seo::AuditRequest::default()
    })
    .expect("repo");
    assert!(report.inventory.policy.is_some());
    let plan = plan_from(&report);
    assert!(
        plan.actions
            .iter()
            .filter(|item| item.kind == weavatrix_seo::PlanKind::Create)
            .all(|item| item.subject.contains("/category") || item.subject == "/:locale"),
        "{:?}",
        plan.actions
    );
}

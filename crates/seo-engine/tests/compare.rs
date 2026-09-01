//! Loopback competitor compare: structural gaps, never prose.

use std::collections::BTreeMap;
use weavatrix_seo::{AnalysisMode, AuditRequest, plan_from, retrieve, run_audit, run_on_report};
use weavatrix_seo_competitor::{compare_inventories, score_artifacts, site_backed_ids};

mod common;

use common::{Page, html, page, spawn};

fn robots() -> Page {
    Page {
        status: 200,
        headers: vec![("Content-Type".into(), "text/plain".into())],
        body: "User-agent: *\nAllow: /\nSitemap: /sitemap.xml\n".into(),
    }
}

fn sitemap(locs: impl IntoIterator<Item = impl AsRef<str>>) -> Page {
    let mut body = String::from("<?xml version=\"1.0\"?><urlset>");
    for loc in locs {
        body.push_str("<url><loc>");
        body.push_str(loc.as_ref());
        body.push_str("</loc></url>");
    }
    body.push_str("</urlset>");
    Page {
        status: 200,
        headers: vec![("Content-Type".into(), "application/xml".into())],
        body,
    }
}

fn owned_pages() -> BTreeMap<String, Page> {
    let mut pages = BTreeMap::new();
    pages.insert("/robots.txt".into(), robots());
    pages.insert(
        "/sitemap.xml".into(),
        sitemap(["/", "/category/electrician", "/service/one"]),
    );
    pages.insert(
        "/".into(),
        page(
            200,
            html(
                "Home",
                "<link rel=\"canonical\" href=\"/\">",
                "<h1>Home</h1><p>Southwest Washington electrician.</p><a href=\"/category/electrician\">Electrician</a><a href=\"/service/one\">One</a>",
            ),
        ),
    );
    pages.insert(
        "/category/electrician".into(),
        page(
            200,
            html(
                "Electrician",
                "<link rel=\"canonical\" href=\"/category/electrician\">",
                "<p>Hire a licensed electrician. No heading on this landing.</p>",
            ),
        ),
    );
    pages.insert(
        "/service/one".into(),
        page(
            200,
            html(
                "Panel upgrade",
                "<link rel=\"canonical\" href=\"/service/one\">",
                "<p>One service page without an H1.</p>",
            ),
        ),
    );
    pages
}

fn competitor_pages() -> BTreeMap<String, Page> {
    let mut pages = BTreeMap::new();
    let mut locs = vec![
        "/".to_owned(),
        "/faq".into(),
        "/guides/how-to-permit".into(),
    ];
    for index in 0..6 {
        locs.push(format!("/service/{index}"));
    }
    pages.insert("/robots.txt".into(), robots());
    pages.insert("/sitemap.xml".into(), sitemap(locs));
    let faq = r#"<script type="application/ld+json">{"@type":"FAQPage","mainEntity":[]}</script>"#;
    pages.insert(
        "/".into(),
        Page {
            status: 200,
            headers: vec![("Content-Type".into(), "text/html".into())],
            body: format!(
                "<!doctype html><html lang=\"he-IL\"><head><title>Home</title>\
                 <link rel=\"canonical\" href=\"/\">\
                 <link rel=\"alternate\" hreflang=\"he-IL\" href=\"/\">\
                 {faq}</head><body><h1>Home</h1>\
                 <p>SECRET COMPETITOR COPY UNIQUE PHRASE</p>\
                 <a href=\"/faq\">FAQ</a>\
                 <a href=\"/guides/how-to-permit\">Guide</a>\
                 <a href=\"/service/0\">0</a></body></html>"
            ),
        },
    );
    pages.insert(
        "/faq".into(),
        page(
            200,
            html(
                "FAQ",
                "<link rel=\"canonical\" href=\"/faq\">",
                "<h1>FAQ</h1><p>Frequently asked questions.</p>",
            ),
        ),
    );
    pages.insert(
        "/guides/how-to-permit".into(),
        page(
            200,
            html(
                "Permit guide",
                "<link rel=\"canonical\" href=\"/guides/how-to-permit\">",
                "<h1>Permit guide</h1><p>How to pull a permit.</p>",
            ),
        ),
    );
    for index in 0..6 {
        let path = format!("/service/{index}");
        pages.insert(
            path.clone(),
            page(
                200,
                html(
                    &format!("Service {index}"),
                    &format!("<link rel=\"canonical\" href=\"{path}\">"),
                    &format!("<h1>Service {index}</h1><p>First-party service facts {index}.</p>"),
                ),
            ),
        );
    }
    pages
}

#[test]
fn loopback_compare_flags_schema_locale_faq_cardinality() {
    let owned_site = spawn(owned_pages());
    let competitor_site = spawn(competitor_pages());
    let owned = run_audit(&AuditRequest {
        site: Some(format!("{}/", owned_site.base)),
        max_pages: Some(16),
        workers: Some(4),
        ..AuditRequest::default()
    })
    .expect("owned");
    let other = run_audit(&AuditRequest {
        site: Some(format!("{}/", competitor_site.base)),
        max_pages: Some(16),
        workers: Some(4),
        ..AuditRequest::default()
    })
    .expect("competitor");
    let items = compare_inventories(
        &owned.inventory,
        &[(competitor_site.base.clone(), other.inventory.clone())],
    );
    let kinds: Vec<&str> = items.iter().map(|item| item.kind.as_str()).collect();
    assert!(
        items
            .iter()
            .any(|item| item.kind == "schema_gap" && item.summary.contains("FAQPage")),
        "schema_gap missing: {items:?}"
    );
    assert!(
        items
            .iter()
            .any(|item| item.kind == "market_gap" && item.summary.contains("he-il")),
        "locale gap missing: {items:?}"
    );
    assert!(
        items.iter().any(|item| item.summary.contains("faq")),
        "faq archetype missing: {items:?}"
    );
    assert!(
        items.iter().any(|item| item.kind == "cluster_gap"
            && (item.subject.contains("cardinality") || item.summary.contains("inventory is"))),
        "cardinality missing: {items:?}"
    );
    assert!(
        items.iter().any(|item| item.summary.contains("/guides/")),
        "guide prefix missing: {items:?}"
    );
    assert!(kinds.contains(&"content_gap"), "H1 gap missing: {items:?}");
    assert!(
        items
            .iter()
            .all(|item| !item.summary.contains("SECRET COMPETITOR")
                && !item.why.contains("SECRET COMPETITOR")
                && !item.action.contains("SECRET COMPETITOR")),
        "compare copied competitor prose: {items:?}"
    );
}

#[test]
fn site_audit_emits_first_party_artifacts_url_crawlers_omit() {
    let site = spawn(owned_pages());
    let report = run_audit(&AuditRequest {
        site: Some(format!("{}/", site.base)),
        max_pages: Some(8),
        ..AuditRequest::default()
    })
    .expect("audit");
    let scored = score_artifacts(&report);
    for id in site_backed_ids() {
        assert!(
            scored.iter().any(|item| item.id == *id && item.present),
            "missing first-party artifact {id}: {scored:?}"
        );
    }
    let query = run_on_report(
        "FROM urls WHERE indexable = true RETURN url LIMIT 10",
        &report,
    )
    .expect("query");
    assert!(!query.rows.is_empty());
    let hits = retrieve(&report, "electrician southwest washington", 5);
    assert!(
        !hits.is_empty(),
        "retrieve should rank the owned landing: {hits:?}"
    );
    let plan = plan_from(&report);
    assert_eq!(plan.handoff.from, "weavatrix-seo");
    assert_eq!(plan.handoff.to, "weavatrix-refactor");
    assert!(plan.handoff.read_only);
}

#[test]
fn compare_mode_without_public_competitor_stays_unmeasured() {
    let site = spawn(owned_pages());
    let report = run_audit(&AuditRequest {
        mode: AnalysisMode::Compare,
        site: Some(format!("{}/", site.base)),
        competitors: Vec::new(),
        max_pages: Some(4),
        ..AuditRequest::default()
    })
    .expect("compare");
    assert!(
        report
            .opportunities
            .iter()
            .any(|item| item.summary.contains("unmeasured")),
        "{:?}",
        report.opportunities
    );
}

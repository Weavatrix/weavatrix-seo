//! Quality axes, thin programmatic cities, and HTML report.

use std::collections::BTreeMap;
use weavatrix_seo::{AuditRequest, render_html, run_audit};

mod common;

use common::{Page, html, page, spawn};

fn fixture() -> BTreeMap<String, Page> {
    let mut pages = BTreeMap::new();
    pages.insert(
        "/robots.txt".into(),
        Page {
            status: 200,
            headers: vec![("Content-Type".into(), "text/plain".into())],
            body: "User-agent: *\nAllow: /\n".into(),
        },
    );
    let city = |name: &str, path: &str| {
        page(
            200,
            html(
                &format!("Electrician in {name}"),
                &format!("<link rel=\"canonical\" href=\"{path}\">"),
                &format!("<h1>Electrician in {name}</h1><p>Licensed electrician. Same facts for every city.</p>"),
            ),
        )
    };
    pages.insert(
        "/category/electrician/vancouver-wa".into(),
        city("Vancouver WA", "/category/electrician/vancouver-wa"),
    );
    pages.insert(
        "/category/electrician/camas-wa".into(),
        city("Camas WA", "/category/electrician/camas-wa"),
    );
    pages.insert(
        "/bare".into(),
        Page {
            status: 200,
            headers: vec![("Content-Type".into(), "text/html".into())],
            body: "<html><head><title>Bare</title><link rel=\"canonical\" href=\"/bare\"></head><body><p>No heading.</p></body></html>".into(),
        },
    );
    pages.insert(
        "/".into(),
        page(
            200,
            html(
                "Home",
                "<link rel=\"canonical\" href=\"/\">",
                "<h1>Home</h1><p>Cities.</p><a href=\"/category/electrician/vancouver-wa\">Vancouver</a><a href=\"/category/electrician/camas-wa\">Camas</a><a href=\"/bare\">Bare</a><img src=\"/x.png\">",
            ),
        ),
    );
    pages
}

#[test]
fn quality_and_thin_cities_and_html() {
    let site = spawn(fixture());
    let report = run_audit(&AuditRequest {
        site: Some(format!("{}/", site.base)),
        max_pages: Some(10),
        workers: Some(3),
        ..AuditRequest::default()
    })
    .expect("audit");
    let codes: Vec<_> = report
        .findings
        .iter()
        .map(|finding| finding.code.as_str())
        .collect();
    assert!(codes.contains(&"WVX-SEO-A11Y-002"), "{codes:?}");
    assert!(codes.contains(&"WVX-SEO-CONTENT-001"), "{codes:?}");
    assert!(codes.contains(&"WVX-SEO-META-004"), "{codes:?}");
    assert!(codes.contains(&"WVX-SEO-PROG-002"), "{codes:?}");
    assert!(
        report
            .axes
            .iter()
            .any(|axis| axis.axis == "accessibility" && !axis.unmeasured)
    );
    let html_report = render_html(&report);
    assert!(html_report.contains("WVX-SEO-PROG-002"));
    assert!(html_report.contains("Search Evidence Graph"));
    assert!(!html_report.contains("<script>"));
}

//! Site-only vertical: crawl, audit, opportunities.

use std::collections::BTreeMap;
use weavatrix_seo::{AuditRequest, run_audit};

mod common;

use common::{Page, html, page, spawn};

fn fixture() -> BTreeMap<String, Page> {
    let mut pages = BTreeMap::new();
    pages.insert(
        "/robots.txt".into(),
        Page {
            status: 200,
            headers: vec![("Content-Type".into(), "text/plain".into())],
            body: "User-agent: *\nAllow: /\nSitemap: /sitemap.xml\n".into(),
        },
    );
    pages.insert(
        "/sitemap.xml".into(),
        Page {
            status: 200,
            headers: vec![("Content-Type".into(), "application/xml".into())],
            body: r#"<?xml version="1.0"?><urlset>
              <url><loc>/ </loc></url>
              <url><loc>/about</loc></url>
              <url><loc>/orphan</loc></url>
              <url><loc>/dup-a</loc></url>
              <url><loc>/dup-b</loc></url>
              <url><loc>/noindex</loc></url>
            </urlset>"#
                .into(),
        },
    );
    pages.insert(
        "/".into(),
        page(
            200,
            html(
                "Home",
                "<link rel=\"canonical\" href=\"/\">",
                "<h1>Home</h1><p>Welcome.</p><a href=\"/about\">About</a><a href=\"/missing\">Broken</a><a href=\"/old\">Old</a>",
            ),
        ),
    );
    pages.insert(
        "/about".into(),
        page(
            200,
            html(
                "About",
                "<link rel=\"canonical\" href=\"/about\"><meta name=\"description\" content=\"About us.\"><script type=\"application/ld+json\">{\"@type\":\"Organization\",\"name\":\"X\"}</script>",
                "<h1>About</h1><p>About the project.</p>",
            ),
        ),
    );
    pages.insert(
        "/orphan".into(),
        page(
            200,
            html(
                "Orphan",
                "<link rel=\"canonical\" href=\"/orphan\">",
                "<h1>Orphan</h1><p>Only in the sitemap.</p>",
            ),
        ),
    );
    let dup = html(
        "Duplicate",
        "<link rel=\"canonical\" href=\"/dup-a\">",
        "<h1>Same</h1><p>Identical body used twice.</p>",
    );
    pages.insert("/dup-a".into(), page(200, dup.clone()));
    pages.insert(
        "/dup-b".into(),
        page(
            200,
            html(
                "Duplicate",
                "<link rel=\"canonical\" href=\"/dup-b\">",
                "<h1>Same</h1><p>Identical body used twice.</p>",
            ),
        ),
    );
    pages.insert(
        "/noindex".into(),
        page(
            200,
            html(
                "Hidden",
                "<meta name=\"robots\" content=\"noindex\">",
                "<p>Hidden.</p>",
            ),
        ),
    );
    pages.insert(
        "/old".into(),
        Page {
            status: 301,
            headers: vec![("Location".into(), "/about".into())],
            body: String::new(),
        },
    );
    pages.insert("/missing".into(), page(404, "gone"));
    pages
}

#[test]
fn site_audit_finds_broken_orphan_duplicate_and_sitemap_noindex() {
    let site = spawn(fixture());
    let report = run_audit(&AuditRequest {
        site: Some(format!("{}/", site.base)),
        max_pages: Some(30),
        workers: Some(4),
        ..AuditRequest::default()
    })
    .expect("audit");
    assert!(report.inventory.counts.fetched >= 4, "{report:?}");
    let codes: Vec<_> = report
        .findings
        .iter()
        .map(|finding| finding.code.as_str())
        .collect();
    assert!(codes.contains(&"WVX-SEO-CRAWL-001"), "{codes:?}");
    assert!(codes.contains(&"WVX-SEO-LINK-001"), "{codes:?}");
    assert!(codes.contains(&"WVX-SEO-LINK-002"), "{codes:?}");
    assert!(codes.contains(&"WVX-SEO-DUP-001"), "{codes:?}");
    assert!(codes.contains(&"WVX-SEO-SITEMAP-002"), "{codes:?}");
    assert!(
        report
            .opportunities
            .iter()
            .any(|item| item.kind == "link_gap"),
        "{:?}",
        report.opportunities
    );
    assert!(
        report
            .axes
            .iter()
            .any(|axis| axis.axis == "render_reconciliation" && axis.unmeasured)
    );
}

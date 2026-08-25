//! Known Kablay-class gap: Israeli entities on a Washington landing.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use weavatrix_seo::{AuditRequest, run_audit};

mod common;

use common::{Page, html, page, spawn};

fn fixture() -> BTreeMap<String, Page> {
    let mut pages = BTreeMap::new();
    let mut sitemap = String::from("<?xml version=\"1.0\"?><urlset>");
    for index in 0..40 {
        let _ = write!(sitemap, "<url><loc>/blog/post-{index}</loc></url>");
        pages.insert(
            format!("/blog/post-{index}"),
            page(
                200,
                html(
                    &format!("Post {index}"),
                    "<link rel=\"canonical\" href=\"#\">",
                    "<h1>Post</h1><p>Filler.</p>",
                ),
            ),
        );
    }
    sitemap.push_str("<url><loc>/category/electrician</loc></url></urlset>");
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
            body: sitemap,
        },
    );
    pages.insert(
        "/".into(),
        page(
            200,
            html(
                "Home",
                "<link rel=\"canonical\" href=\"/\">",
                "<h1>Home</h1><p>Southwest Washington.</p><a href=\"/category/electrician\">Electrician</a>",
            ),
        ),
    );
    pages.insert(
        "/category/electrician".into(),
        page(
            200,
            html(
                "Electrician in Southwest Washington",
                "<link rel=\"canonical\" href=\"/category/electrician\">",
                "<h1>Electrician</h1><p>Southwest Washington Electric Company (Hevrat HaHashmal). Book on Shabbat. Gush Dan to the north. IEC approval. Licensed electrician — חשמלאי מוסמך. document/license verification badges.</p>",
            ),
        ),
    );
    pages
}

#[test]
fn link_priority_reaches_electrician_and_flags_market() {
    let site = spawn(fixture());
    let report = run_audit(&AuditRequest {
        site: Some(format!("{}/", site.base)),
        max_pages: Some(8),
        workers: Some(4),
        ..AuditRequest::default()
    })
    .expect("audit");
    assert!(
        report
            .inventory
            .pages
            .iter()
            .any(|page| page.url.path().contains("/category/electrician")),
        "electrician landing missed: {:?}",
        report
            .inventory
            .pages
            .iter()
            .map(|page| page.url.path().to_owned())
            .collect::<Vec<_>>()
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code.starts_with("WVX-SEO-MARKET-001")),
        "{:?}",
        report
            .findings
            .iter()
            .map(|finding| finding.fingerprint.clone())
            .collect::<Vec<_>>()
    );
}

//! Schema required fields and AI-crawler surface.

use std::collections::BTreeMap;
use weavatrix_seo::{AuditRequest, run_audit};

mod common;

use common::{Page, html, page, spawn};

#[test]
fn faqpage_without_main_entity_and_gptbot_block() {
    let mut pages = BTreeMap::new();
    pages.insert(
        "/robots.txt".into(),
        Page {
            status: 200,
            headers: vec![("Content-Type".into(), "text/plain".into())],
            body: "User-agent: *\nAllow: /\n\nUser-agent: GPTBot\nDisallow: /\n".into(),
        },
    );
    pages.insert(
        "/".into(),
        page(
            200,
            html(
                "FAQ",
                "<link rel=\"canonical\" href=\"/\"><script type=\"application/ld+json\">{\"@type\":\"FAQPage\"}</script>",
                "<h1>FAQ</h1><p>Questions.</p>",
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
    let codes: Vec<_> = report
        .findings
        .iter()
        .map(|finding| finding.code.as_str())
        .collect();
    assert!(codes.contains(&"WVX-SEO-SCHEMA-002"), "{codes:?}");
    assert!(
        codes.contains(&"WVX-SEO-AI-004"),
        "missing llms.txt should be measured: {codes:?}"
    );
    assert!(
        codes.contains(&"WVX-SEO-AI-005"),
        "GPTBot Disallow: / should be reported: {codes:?}"
    );
}

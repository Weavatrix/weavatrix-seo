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
    assert!(
        codes.contains(&"WVX-SEO-SCHEMA-004"),
        "retired FAQ rich results are historical, not current eligibility: {codes:?}"
    );
    assert!(
        !codes.contains(&"WVX-SEO-SCHEMA-002"),
        "FAQPage must not emit a current Google eligibility Warn: {codes:?}"
    );
    assert!(
        codes.contains(&"WVX-SEO-AI-004"),
        "missing llms.txt should be measured: {codes:?}"
    );
    assert!(
        codes.contains(&"WVX-SEO-AI-005"),
        "GPTBot Disallow: / should be reported: {codes:?}"
    );
    let gptbot = report
        .findings
        .iter()
        .find(|finding| finding.code == "WVX-SEO-AI-005")
        .expect("gptbot finding");
    assert_eq!(gptbot.severity, weavatrix_seo_model::Severity::Info);
    assert_eq!(
        gptbot.severity_override,
        Some(weavatrix_seo_model::Severity::Info)
    );
    let matrix = report
        .inventory
        .ai_surface
        .as_ref()
        .expect("ai surface")
        .agent_matrix
        .iter()
        .find(|row| row.agent == "gptbot")
        .expect("gptbot matrix");
    assert_eq!(matrix.policy_intent, "BLOCK");
    assert_eq!(matrix.role, "training");
}

#[test]
fn product_without_offer_is_a_current_eligibility_failure() {
    let mut pages = BTreeMap::new();
    pages.insert(
        "/".into(),
        page(
            200,
            html(
                "Widget",
                "<link rel=\"canonical\" href=\"/\"><script type=\"application/ld+json\">{\"@type\":\"Product\",\"name\":\"Widget\"}</script>",
                "<h1>Widget</h1><p>A widget.</p>",
            ),
        ),
    );
    let site = spawn(pages);
    let report = run_audit(&AuditRequest {
        site: Some(format!("{}/", site.base)),
        max_pages: Some(4),
        ..AuditRequest::default()
    })
    .expect("audit");
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "WVX-SEO-SCHEMA-002"
                && finding.summary.contains("Product")),
        "{:?}",
        report
            .findings
            .iter()
            .map(|finding| finding.code.as_str())
            .collect::<Vec<_>>()
    );
}

//! Origin-level security headers. One fact per host, not a per-URL crawler dump.

use std::collections::BTreeMap;
use weavatrix_seo_model::{
    Evidence, ExtractedPage, Finding, FindingFamily, Inventory, Locator, Scheme, Severity,
};

pub fn audit_origin(inventory: &Inventory, findings: &mut Vec<Finding>) {
    let mut by_host: BTreeMap<&str, Vec<&ExtractedPage>> = BTreeMap::new();
    for page in inventory
        .pages
        .iter()
        .filter(|page| page.status == 200 && page.media.is_html())
    {
        by_host.entry(page.url.host()).or_default().push(page);
    }
    for pages in by_host.values() {
        audit_host(pages, findings);
    }
}

fn audit_host(pages: &[&ExtractedPage], findings: &mut Vec<Finding>) {
    let Some(sample) = pages.first().copied() else {
        return;
    };
    if sample.url.scheme() == Scheme::Https {
        hsts(pages, sample, findings);
    }
    header_presence(
        sample,
        findings,
        "x-content-type-options",
        2,
        Severity::Info,
        !pages.iter().all(|page| has_nosniff(page)),
        "origin is missing X-Content-Type-Options: nosniff",
        "MIME sniffing is a common XSS footgun.",
        "Send X-Content-Type-Options: nosniff.",
    );
    let csp_split = is_split(pages, has_csp);
    if csp_split {
        split_finding(sample, findings, "content-security-policy");
    } else if !pages.iter().any(|page| has_csp(page)) {
        let report_only = pages
            .iter()
            .any(|page| page.header("content-security-policy-report-only").is_some());
        let summary = if report_only {
            "origin has CSP-Report-Only but no enforcing Content-Security-Policy"
        } else {
            "origin is missing Content-Security-Policy"
        };
        findings.push(origin_finding(
            sample,
            "content-security-policy",
            3,
            Severity::Info,
            summary,
            "CSP is the primary XSS containment header.",
            "Ship an enforcing Content-Security-Policy on the origin.",
        ));
    }
    if !pages.iter().any(|page| csp_frame_ancestors(page)) {
        header_presence(
            sample,
            findings,
            "x-frame-options",
            4,
            Severity::Info,
            pages
                .iter()
                .all(|page| page.header("x-frame-options").is_none()),
            "origin is missing X-Frame-Options",
            "Clickjacking protection is an origin header.",
            "Send X-Frame-Options or CSP frame-ancestors.",
        );
    }
    header_presence(
        sample,
        findings,
        "referrer-policy",
        5,
        Severity::Info,
        pages
            .iter()
            .all(|page| page.header("referrer-policy").is_none()),
        "origin is missing Referrer-Policy",
        "Referrer leakage is an origin privacy and search-surface fact.",
        "Send a Referrer-Policy on the origin.",
    );
}

fn hsts(pages: &[&ExtractedPage], sample: &ExtractedPage, findings: &mut Vec<Finding>) {
    let present = pages
        .iter()
        .filter(|page| page.header("strict-transport-security").is_some())
        .count();
    if present == 0 {
        findings.push(origin_finding(
            sample,
            "strict-transport-security",
            1,
            Severity::Warn,
            "origin is missing Strict-Transport-Security",
            "HTTPS origins should lock browsers onto TLS.",
            "Send Strict-Transport-Security on the origin.",
        ));
        return;
    }
    if present < pages.len() && pages.len() > 1 {
        split_finding(sample, findings, "strict-transport-security");
        return;
    }
    if pages.iter().all(|page| {
        page.header("strict-transport-security")
            .is_none_or(hsts_disabled)
    }) {
        findings.push(origin_finding(
            sample,
            "strict-transport-security",
            6,
            Severity::Warn,
            "origin sends HSTS with max-age=0 or without max-age",
            "HSTS with a zero or missing max-age does not pin the origin to HTTPS.",
            "Set HSTS max-age to a positive number of seconds.",
        ));
    }
}

#[allow(clippy::too_many_arguments)]
fn header_presence(
    sample: &ExtractedPage,
    findings: &mut Vec<Finding>,
    header: &str,
    number: u16,
    severity: Severity,
    missing: bool,
    summary: &str,
    why: &str,
    action: &str,
) {
    if !missing {
        return;
    }
    findings.push(origin_finding(
        sample, header, number, severity, summary, why, action,
    ));
}

fn split_finding(sample: &ExtractedPage, findings: &mut Vec<Finding>, header: &str) {
    let summary = format!("origin sends {header} on some HTML responses but not others");
    findings.push(origin_finding(
        sample,
        header,
        7,
        Severity::Warn,
        &summary,
        "Search and browser agents see a mixed security surface on one host.",
        "Emit the header on every indexable HTML response for this origin.",
    ));
}

/// Active mixed content on an HTTPS document. One finding per URL.
pub fn mixed_content(page: &ExtractedPage, findings: &mut Vec<Finding>) {
    if page.url.scheme() != Scheme::Https {
        return;
    }
    let mut offenders = Vec::new();
    for href in page.images.iter().map(|image| &image.src) {
        if href.starts_with("http://") {
            offenders.push(href.clone());
        }
    }
    if let Some(image) = &page.og_image
        && image.starts_with("http://")
    {
        offenders.push(image.clone());
    }
    if offenders.is_empty() {
        return;
    }
    offenders.sort();
    offenders.dedup();
    findings.push(
        Finding::new(
            FindingFamily::Security,
            8,
            Severity::Warn,
            &page.url.to_string(),
            format!(
                "{} loads {} http:// subresource(s) on HTTPS",
                page.url,
                offenders.len()
            ),
            Locator::url(&page.url),
            page.evidence.clone(),
        )
        .with_affected(offenders)
        .explained(
            "Mixed content applies to loaded subresources (img, og:image), not navigation links.",
            "Serve those URLs over HTTPS or drop the http:// references.",
            "Every loaded subresource on an HTTPS document is HTTPS.",
        ),
    );
}

fn origin_finding(
    sample: &ExtractedPage,
    header: &str,
    number: u16,
    severity: Severity,
    summary: &str,
    why: &str,
    action: &str,
) -> Finding {
    let host = sample.url.host();
    Finding::new(
        FindingFamily::Security,
        number,
        severity,
        &format!("{host}:{header}:{number}"),
        format!("{host} {summary}"),
        Locator::header(&sample.url, header),
        Evidence::http(),
    )
    .explained(
        why,
        action,
        "The header is consistent and effective on the origin.",
    )
}

fn has_csp(page: &ExtractedPage) -> bool {
    page.header("content-security-policy").is_some() || page.csp_meta.is_some()
}

fn has_nosniff(page: &ExtractedPage) -> bool {
    page.header("x-content-type-options").is_some_and(|value| {
        value
            .split(',')
            .any(|token| token.trim().eq_ignore_ascii_case("nosniff"))
    })
}

/// Header-delivered `frame-ancestors` only.
///
/// CSP Level 3 requires user agents to ignore `frame-ancestors` in a policy
/// delivered through `meta http-equiv`, so a meta policy can never stand in for
/// `X-Frame-Options`.
fn csp_frame_ancestors(page: &ExtractedPage) -> bool {
    page.header("content-security-policy").is_some_and(|value| {
        value
            .to_ascii_lowercase()
            .split(';')
            .any(|directive| directive.trim().starts_with("frame-ancestors"))
    })
}

fn hsts_disabled(value: &str) -> bool {
    let mut max_age = None;
    for part in value.split(';') {
        let part = part.trim();
        let lower = part.to_ascii_lowercase();
        if let Some(raw) = lower.strip_prefix("max-age=") {
            max_age = raw.trim().parse::<u64>().ok();
        }
    }
    max_age.is_none_or(|age| age == 0)
}

fn is_split(pages: &[&ExtractedPage], present: fn(&ExtractedPage) -> bool) -> bool {
    if pages.len() < 2 {
        return false;
    }
    let count = pages.iter().filter(|page| present(page)).count();
    count > 0 && count < pages.len()
}

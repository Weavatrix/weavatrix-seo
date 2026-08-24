//! Origin-level security headers. One fact per host, not a per-URL crawler dump.

use weavatrix_seo_model::{
    Evidence, ExtractedPage, Finding, FindingFamily, Inventory, Locator, Scheme, Severity,
};

const CHECKS: &[(&str, u16, Severity, &str, &str, &str)] = &[
    (
        "strict-transport-security",
        1,
        Severity::Warn,
        "origin is missing Strict-Transport-Security",
        "HTTPS origins should lock browsers onto TLS.",
        "Send Strict-Transport-Security on the origin.",
    ),
    (
        "x-content-type-options",
        2,
        Severity::Info,
        "origin is missing X-Content-Type-Options",
        "MIME sniffing is a common XSS footgun.",
        "Send X-Content-Type-Options: nosniff.",
    ),
    (
        "content-security-policy",
        3,
        Severity::Info,
        "origin is missing Content-Security-Policy",
        "CSP is the primary XSS containment header.",
        "Ship a Content-Security-Policy for this origin.",
    ),
    (
        "x-frame-options",
        4,
        Severity::Info,
        "origin is missing X-Frame-Options",
        "Clickjacking protection is an origin header.",
        "Send X-Frame-Options or a frame-ancestors CSP.",
    ),
];

pub fn audit_origin(inventory: &Inventory, findings: &mut Vec<Finding>) {
    let Some(sample) = inventory.pages.iter().find(|page| page.status == 200) else {
        return;
    };
    for (header, number, severity, summary, why, action) in CHECKS {
        if *header == "strict-transport-security" && sample.url.scheme() != Scheme::Https {
            continue;
        }
        if sample.header(header).is_some() {
            continue;
        }
        findings.push(origin_finding(sample, header, *number, *severity, summary, why, action));
    }
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
        host,
        format!("{host} {summary}"),
        Locator::header(&sample.url, header),
        Evidence::http(),
    )
    .explained(why, action, "The header is present on the origin.")
}

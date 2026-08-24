//! Live-document quality: H1, Open Graph, accessibility, security, performance.

use weavatrix_seo_model::{
    ExtractedPage, Finding, FindingFamily, Indexability, Inventory, Locator, Scheme, Severity,
};

pub fn audit(inventory: &Inventory, findings: &mut Vec<Finding>) {
    for page in inventory
        .pages
        .iter()
        .filter(|page| page.status == 200 && page.indexability == Indexability::Indexable)
    {
        headings(page, findings);
        open_graph(page, findings);
        accessibility(page, findings);
        security(page, findings);
        performance(page, findings);
    }
}

fn headings(page: &ExtractedPage, findings: &mut Vec<Finding>) {
    let h1: Vec<_> = page
        .headings
        .iter()
        .filter(|heading| heading.level == 1)
        .collect();
    if h1.is_empty() {
        findings.push(
            Finding::new(
                FindingFamily::Content,
                1,
                Severity::Error,
                &page.url.to_string(),
                format!("{} is missing an H1", page.url),
                Locator::dom(&page.url, "h1"),
                page.evidence.clone(),
            )
            .explained(
                "Indexable pages need one clear H1.",
                "Emit a single H1 from the page template.",
                "Exactly one H1 is present.",
            ),
        );
    } else if h1.len() > 1 {
        findings.push(
            Finding::new(
                FindingFamily::Content,
                2,
                Severity::Warn,
                &page.url.to_string(),
                format!("{} has {} H1 headings", page.url, h1.len()),
                Locator::dom(&page.url, "h1"),
                page.evidence.clone(),
            )
            .explained(
                "Multiple H1s dilute the primary heading.",
                "Keep one H1 and demote the rest.",
                "The document has a single H1.",
            ),
        );
    }
}

fn open_graph(page: &ExtractedPage, findings: &mut Vec<Finding>) {
    if page.og_title.as_ref().is_none_or(|value| value.trim().is_empty()) {
        findings.push(
            Finding::new(
                FindingFamily::Meta,
                4,
                Severity::Info,
                &page.url.to_string(),
                format!("{} has no og:title", page.url),
                Locator::dom(&page.url, "meta[property=og:title]"),
                page.evidence.clone(),
            )
            .explained(
                "Social and some crawlers read Open Graph title separately from <title>.",
                "Add og:title on this template.",
                "og:title is present.",
            ),
        );
    }
    if page.og_image.as_ref().is_none_or(|value| value.trim().is_empty()) {
        findings.push(
            Finding::new(
                FindingFamily::Meta,
                5,
                Severity::Info,
                &page.url.to_string(),
                format!("{} has no og:image", page.url),
                Locator::dom(&page.url, "meta[property=og:image]"),
                page.evidence.clone(),
            )
            .explained(
                "Link unfurls need an image.",
                "Add og:image on this template.",
                "og:image points at an absolute URL.",
            ),
        );
    }
}

fn accessibility(page: &ExtractedPage, findings: &mut Vec<Finding>) {
    if page
        .html_lang
        .as_ref()
        .is_none_or(|value| value.trim().is_empty())
    {
        findings.push(
            Finding::new(
                FindingFamily::A11y,
                1,
                Severity::Warn,
                &page.url.to_string(),
                format!("{} is missing html lang", page.url),
                Locator::dom(&page.url, "html[lang]"),
                page.evidence.clone(),
            )
            .explained(
                "Assistive tech and hreflang consumers use html lang.",
                "Set html lang on the document template.",
                "html[lang] is a BCP 47 tag.",
            ),
        );
    }
    let missing_alt = page
        .images
        .iter()
        .filter(|image| image.alt.as_ref().is_none_or(|alt| alt.trim().is_empty()))
        .count();
    if missing_alt > 0 {
        findings.push(
            Finding::new(
                FindingFamily::A11y,
                2,
                Severity::Warn,
                &page.url.to_string(),
                format!("{} has {missing_alt} images without alt", page.url),
                Locator::dom(&page.url, "img"),
                page.evidence.clone(),
            )
            .explained(
                "Images without alt fail basic accessibility checks.",
                "Add alt text or mark decorative images explicitly.",
                "Every img has an alt attribute.",
            ),
        );
    }
    let mut expected = 0_u8;
    for heading in &page.headings {
        if expected > 0 && heading.level > expected + 1 {
            findings.push(
                Finding::new(
                    FindingFamily::A11y,
                    3,
                    Severity::Info,
                    &page.url.to_string(),
                    format!(
                        "{} skips from h{expected} to h{}",
                        page.url, heading.level
                    ),
                    Locator::dom(&page.url, "h1,h2,h3,h4,h5,h6"),
                    page.evidence.clone(),
                )
                .explained(
                    "Heading levels should increase by one.",
                    "Do not skip heading ranks in the template.",
                    "Heading ranks are sequential.",
                ),
            );
            break;
        }
        expected = heading.level.max(expected);
    }
}

fn security(page: &ExtractedPage, findings: &mut Vec<Finding>) {
    if page.url.scheme() == Scheme::Https
        && page.header("strict-transport-security").is_none()
    {
        findings.push(
            Finding::new(
                FindingFamily::Security,
                1,
                Severity::Warn,
                &page.url.to_string(),
                format!("{} is missing Strict-Transport-Security", page.url),
                Locator::header(&page.url, "strict-transport-security"),
                page.evidence.clone(),
            )
            .explained(
                "HTTPS pages should lock browsers onto TLS.",
                "Send Strict-Transport-Security on the origin.",
                "The HSTS header is present.",
            ),
        );
    }
    if page.header("x-content-type-options").is_none() {
        findings.push(
            Finding::new(
                FindingFamily::Security,
                2,
                Severity::Info,
                &page.url.to_string(),
                format!("{} is missing X-Content-Type-Options", page.url),
                Locator::header(&page.url, "x-content-type-options"),
                page.evidence.clone(),
            )
            .explained(
                "MIME sniffing is a common XSS footgun.",
                "Send X-Content-Type-Options: nosniff.",
                "The header is present.",
            ),
        );
    }
    if page.header("content-security-policy").is_none() {
        findings.push(
            Finding::new(
                FindingFamily::Security,
                3,
                Severity::Info,
                &page.url.to_string(),
                format!("{} is missing Content-Security-Policy", page.url),
                Locator::header(&page.url, "content-security-policy"),
                page.evidence.clone(),
            )
            .explained(
                "CSP is the primary XSS containment header.",
                "Ship a Content-Security-Policy for this origin.",
                "A CSP header is present.",
            ),
        );
    }
}

fn performance(page: &ExtractedPage, findings: &mut Vec<Finding>) {
    if page.body_bytes > 524_288 {
        findings.push(
            Finding::new(
                FindingFamily::Perf,
                1,
                Severity::Warn,
                &page.url.to_string(),
                format!("{} HTML is {} bytes", page.url, page.body_bytes),
                Locator::url(&page.url),
                page.evidence.clone(),
            )
            .explained(
                "Large HTML slows first parse and wastes crawl budget.",
                "Trim the template or defer non-critical payload.",
                "HTML is under 512 KiB.",
            ),
        );
    }
    if page.fetch_ms > 2_500 {
        findings.push(
            Finding::new(
                FindingFamily::Perf,
                2,
                Severity::Warn,
                &page.url.to_string(),
                format!("{} took {} ms to fetch", page.url, page.fetch_ms),
                Locator::url(&page.url),
                page.evidence.clone(),
            )
            .explained(
                "Slow origin responses shrink crawl throughput.",
                "Speed up TTFB for this template.",
                "Fetch time is under 2.5s from this crawler.",
            ),
        );
    }
}

//! Fetch cost evidence.

use weavatrix_seo_model::{ExtractedPage, Finding, FindingFamily, Locator, Severity};

pub fn audit(page: &ExtractedPage, findings: &mut Vec<Finding>) {
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
                "Slow origin responses shrink audit throughput.",
                "Speed up TTFB for this template.",
                "Fetch time is under 2.5s from this transport.",
            ),
        );
    }
}

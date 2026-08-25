//! Open Graph evidence.

use weavatrix_seo_model::{ExtractedPage, Finding, FindingFamily, Locator, Severity};

pub fn audit(page: &ExtractedPage, findings: &mut Vec<Finding>) {
    if page
        .og_title
        .as_ref()
        .is_none_or(|value| value.trim().is_empty())
    {
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
    if page
        .og_image
        .as_ref()
        .is_none_or(|value| value.trim().is_empty())
    {
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

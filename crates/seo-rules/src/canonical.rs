//! Canonical graph.

use weavatrix_seo_model::{
    AbsoluteUrl, Finding, FindingFamily, Indexability, Inventory, Locator, Severity,
};

pub fn audit(inventory: &Inventory, findings: &mut Vec<Finding>) {
    for page in inventory.pages.iter().filter(|page| {
        page.indexability == Indexability::Indexable && page.status == 200 && page.media.is_html()
    }) {
        if page.canonical.is_none() {
            findings.push(
                Finding::new(
                    FindingFamily::Canon,
                    1,
                    Severity::Warn,
                    &page.url.to_string(),
                    format!("{} has no canonical", page.url),
                    Locator::dom(&page.url, "link[rel=canonical]"),
                    page.evidence.clone(),
                )
                .explained(
                    "Indexable pages should declare a self-canonical.",
                    "Emit a self-referencing canonical on this template.",
                    "A canonical href matches the final URL.",
                ),
            );
        }
    }
    for page in inventory
        .pages
        .iter()
        .filter(|page| page.status == 200 && page.media.is_html())
    {
        let Some(href) = page.canonical.as_deref() else {
            continue;
        };
        let Ok(target) = AbsoluteUrl::parse(href).or_else(|_| page.url.join(href)) else {
            continue;
        };
        let Some(dest) = inventory.page(&target) else {
            findings.push(
                Finding::new(
                    FindingFamily::Canon,
                    4,
                    Severity::Info,
                    &page.url.to_string(),
                    format!("{} canonical target is unmeasured in this crawl", page.url),
                    Locator::dom(&page.url, "link[rel=canonical]"),
                    page.evidence.clone(),
                )
                .explained(
                    "A canonical outside the crawl budget is not proof the target is healthy.",
                    "Raise the budget or fetch the canonical URL before treating it as resolved.",
                    "The canonical target is measured, or the gap is recorded as unmeasured.",
                ),
            );
            continue;
        };
        if dest.status >= 400 {
            findings.push(
                Finding::new(
                    FindingFamily::Canon,
                    2,
                    Severity::Error,
                    &page.url.to_string(),
                    format!("{} canonical points at {}", page.url, dest.status),
                    Locator::dom(&page.url, "link[rel=canonical]"),
                    page.evidence.clone(),
                )
                .explained(
                    "A canonical must resolve to a reachable URL.",
                    "Point the canonical at a live indexable URL.",
                    "The canonical target returns 200.",
                ),
            );
            continue;
        }
        let Some(next) = dest.canonical.as_deref() else {
            continue;
        };
        let Ok(next_url) = AbsoluteUrl::parse(next).or_else(|_| dest.url.join(next)) else {
            continue;
        };
        if next_url != dest.url && next_url != page.url {
            findings.push(
                Finding::new(
                    FindingFamily::Canon,
                    3,
                    Severity::Warn,
                    &page.url.to_string(),
                    format!(
                        "{} canonicalizes to {} which canonicalizes away",
                        page.url, dest.url
                    ),
                    Locator::dom(&page.url, "link[rel=canonical]"),
                    page.evidence.clone(),
                )
                .explained(
                    "Canonical chains waste crawl budget the same way redirect chains do.",
                    "Point every URL in the set at the final canonical.",
                    "The canonical target is self-canonical.",
                ),
            );
        }
    }
}

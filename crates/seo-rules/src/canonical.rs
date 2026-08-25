//! Canonical graph.

use weavatrix_seo_model::{
    AbsoluteUrl, Finding, FindingFamily, Indexability, Inventory, Locator, Severity,
};

pub fn audit(inventory: &Inventory, findings: &mut Vec<Finding>) {
    for page in inventory
        .pages
        .iter()
        .filter(|page| {
            page.indexability == Indexability::Indexable
                && page.status == 200
                && page.media.is_html()
        })
    {
        match &page.canonical {
            None => findings.push(
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
            ),
            Some(href) => {
                if let Ok(target) = AbsoluteUrl::parse(href).or_else(|_| page.url.join(href))
                    && let Some(dest) = inventory.page(&target)
                    && dest.status >= 400
                {
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
                }
            }
        }
    }
}

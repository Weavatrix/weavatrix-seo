//! Sitemap membership.

use weavatrix_seo_model::{
    Evidence, Finding, FindingFamily, Indexability, Inventory, Locator, Severity,
};

pub fn audit(inventory: &Inventory, findings: &mut Vec<Finding>) {
    for page in inventory.pages.iter().filter(|page| page.in_sitemap) {
        if page.status >= 400 {
            findings.push(
                Finding::new(
                    FindingFamily::Sitemap,
                    1,
                    Severity::Error,
                    &page.url.to_string(),
                    format!("sitemap lists unreachable {}", page.url),
                    Locator::Sitemap {
                        sitemap: inventory.site.clone().unwrap_or_default(),
                        loc: page.url.to_string(),
                    },
                    Evidence::sitemap(),
                )
                .explained(
                    "Sitemap loc values must exist.",
                    "Remove the loc or restore the URL.",
                    "The listed URL returns 200.",
                ),
            );
        }
        if page.indexability == Indexability::Noindex {
            findings.push(
                Finding::new(
                    FindingFamily::Sitemap,
                    2,
                    Severity::Warn,
                    &page.url.to_string(),
                    format!("sitemap lists noindex URL {}", page.url),
                    Locator::url(&page.url),
                    Evidence::sitemap(),
                )
                .explained(
                    "Sitemaps should list canonical indexable URLs only.",
                    "Drop the loc or drop the noindex signal.",
                    "Sitemap membership matches indexability.",
                ),
            );
        }
    }
}

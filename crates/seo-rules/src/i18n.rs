//! Reciprocal hreflang.

use weavatrix_seo_model::{AbsoluteUrl, Finding, FindingFamily, Inventory, Locator, Severity};

pub fn audit(inventory: &Inventory, findings: &mut Vec<Finding>) {
    for page in &inventory.pages {
        for alternate in &page.alternates {
            let Ok(target) =
                AbsoluteUrl::parse(&alternate.href).or_else(|_| page.url.join(&alternate.href))
            else {
                continue;
            };
            let Some(dest) = inventory.page(&target) else {
                continue;
            };
            let returns = dest.alternates.iter().any(|item| {
                AbsoluteUrl::parse(&item.href)
                    .or_else(|_| dest.url.join(&item.href))
                    .is_ok_and(|href| href == page.url)
            });
            if !returns {
                findings.push(
                    Finding::new(
                        FindingFamily::I18n,
                        1,
                        Severity::Warn,
                        &page.url.to_string(),
                        format!(
                            "{} hreflang {} is not reciprocal",
                            page.url, alternate.hreflang
                        ),
                        Locator::dom(&page.url, "link[rel=alternate]"),
                        page.evidence.clone(),
                    )
                    .explained(
                        "Hreflang annotations must be reciprocal.",
                        "Add the return alternate on the target locale.",
                        "Each locale in the set lists every other locale.",
                    ),
                );
            }
        }
    }
}

//! Deterministic technical SEO rules over a crawl inventory.

#![forbid(unsafe_code)]

mod ai;
mod canonical;
mod i18n;
mod links;
mod metadata;
mod query;
mod schema;
mod sitemap;
mod status;

use weavatrix_seo_model::{Finding, Inventory};

/// Runs the site-only technical rule pack.
#[must_use]
pub fn audit(inventory: &Inventory) -> Vec<Finding> {
    let mut findings = Vec::new();
    status::audit(inventory, &mut findings);
    canonical::audit(inventory, &mut findings);
    sitemap::audit(inventory, &mut findings);
    metadata::audit(inventory, &mut findings);
    i18n::audit(inventory, &mut findings);
    query::audit(inventory, &mut findings);
    schema::audit(inventory, &mut findings);
    links::audit(inventory, &mut findings);
    ai::audit(inventory, &mut findings);
    findings.sort_by(|left, right| {
        right
            .severity
            .cmp(&left.severity)
            .then_with(|| left.code.cmp(&right.code))
            .then_with(|| left.fingerprint.cmp(&right.fingerprint))
    });
    findings
}

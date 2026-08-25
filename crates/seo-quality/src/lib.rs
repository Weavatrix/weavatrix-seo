//! Live-document quality: headings, Open Graph, a11y, origin security, fetch cost.

#![forbid(unsafe_code)]

mod accessibility;
mod heading;
mod open_graph;
mod performance;
mod security;

use weavatrix_seo_model::{Finding, Indexability, Inventory};

/// Runs quality evidence over crawled pages. Missing HTTP is not a pass.
#[must_use]
pub fn audit(inventory: &Inventory) -> Vec<Finding> {
    let mut findings = Vec::new();
    for page in inventory.pages.iter().filter(|page| {
        page.status == 200 && page.indexability == Indexability::Indexable && page.media.is_html()
    }) {
        heading::audit(page, &mut findings);
        open_graph::audit(page, &mut findings);
        accessibility::audit(page, &mut findings);
        performance::audit(page, &mut findings);
    }
    accessibility::audit_controls(inventory, &mut findings);
    security::audit_origin(inventory, &mut findings);
    findings
}

//! Broken internal links.

use weavatrix_seo_model::{Finding, FindingFamily, Inventory, Locator, Relation, Severity};

pub fn audit(inventory: &Inventory, findings: &mut Vec<Finding>) {
    for edge in inventory
        .edges
        .iter()
        .filter(|edge| edge.relation == Relation::LinksTo)
    {
        if let Some(target) = inventory.page(&edge.target)
            && target.status >= 400
        {
            findings.push(
                Finding::new(
                    FindingFamily::Link,
                    1,
                    Severity::Error,
                    &format!("{}->{}", edge.source, edge.target),
                    format!("{} links to {}", edge.source, edge.target),
                    Locator::url(&edge.source),
                    edge.evidence.clone(),
                )
                .with_affected([edge.target.to_string()])
                .explained(
                    "Broken internal links waste crawl budget.",
                    "Update or remove the href in the owning component.",
                    "The target returns 200.",
                ),
            );
        }
    }
}

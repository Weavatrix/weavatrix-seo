//! Template-frequency annotation for repeated internal links.

use std::collections::BTreeMap;
use weavatrix_seo_model::{Inventory, LinkLocation, Relation};

/// Marks `LinksTo` edges that repeat across many templates.
pub fn annotate(inventory: &mut Inventory) {
    let mut counts: BTreeMap<(String, LinkLocation), u32> = BTreeMap::new();
    for edge in inventory
        .edges
        .iter()
        .filter(|edge| edge.relation == Relation::LinksTo)
    {
        let location = edge.location.unwrap_or(LinkLocation::Contextual);
        *counts
            .entry((edge.target.to_string(), location))
            .or_insert(0) += 1;
    }
    let indexable = u32::try_from(inventory.counts.indexable.max(1)).unwrap_or(1);
    for edge in inventory
        .edges
        .iter_mut()
        .filter(|edge| edge.relation == Relation::LinksTo)
    {
        let location = edge.location.unwrap_or(LinkLocation::Contextual);
        let count = counts
            .get(&(edge.target.to_string(), location))
            .copied()
            .unwrap_or(0);
        if count >= 3 && count * 2 >= indexable {
            edge.template_frequency = Some(count);
        }
    }
}

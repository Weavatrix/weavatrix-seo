//! Gaps: what the site should have that it does not.

#![forbid(unsafe_code)]

use weavatrix_seo_architecture::Architecture;
use weavatrix_seo_model::{Indexability, Inventory, Opportunity};

/// Structural opportunities from the current graph. Demand stays `UNMEASURED`.
#[must_use]
pub fn opportunities(inventory: &Inventory, architecture: &Architecture) -> Vec<Opportunity> {
    let mut items = Vec::new();
    for page in architecture.pages.iter().filter(|page| page.orphan) {
        items.push(Opportunity::unmeasured_demand(
            "link_gap",
            page.url.to_string(),
            format!(
                "{} is indexable but has no internal inlinks from the seed",
                page.url
            ),
            "Related pages exist, but the graph does not connect this URL.",
            "Add a contextual or template link from a crawlable parent.",
        ));
    }
    for page in inventory.pages.iter().filter(|page| {
        page.status == 200
            && page.indexability == Indexability::Indexable
            && !page.in_sitemap
            && page.linked_from_page
    }) {
        items.push(Opportunity::unmeasured_demand(
            "sitemap_gap",
            page.url.to_string(),
            format!(
                "{} is linked and indexable but absent from the sitemap",
                page.url
            ),
            "Sitemaps should cover the intended indexable inventory.",
            "Include the URL in the sitemap generator for this family.",
        ));
    }
    for page in inventory.pages.iter().filter(|page| {
        page.status == 200
            && page.indexability == Indexability::Indexable
            && page.headings.iter().all(|heading| heading.level != 1)
    }) {
        items.push(Opportunity::unmeasured_demand(
            "content_gap",
            page.url.to_string(),
            format!("{} has no H1", page.url),
            "The page has no primary heading to state its intent.",
            "Add one H1 that names the page purpose.",
        ));
    }
    items
}

/// Sorts opportunities by [`weavatrix_seo_model::OpportunityAxes::rank_key`].
///
/// Trust gates first, then measured demand, then the declared value axes, with
/// effort as the tie-breaker. Nothing is dropped.
#[must_use]
pub fn rank(mut items: Vec<Opportunity>) -> Vec<Opportunity> {
    items.sort_by(|left, right| right.axes.rank_key().cmp(&left.axes.rank_key()));
    items
}

//! Public competitor graph comparison. Copies no competitor prose.

#![forbid(unsafe_code)]

mod shape;

use std::collections::BTreeSet;
use weavatrix_seo_model::{Evidence, EvidenceSource, Inventory, Opportunity};

pub use shape::Shape;

/// Comparison request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompareRequest {
    /// Owned site.
    pub site: String,
    /// Public competitor origins.
    pub competitors: Vec<String>,
}

/// Compares an owned inventory with crawled public competitor inventories.
#[must_use]
pub fn compare_inventories(owned: &Inventory, others: &[(String, Inventory)]) -> Vec<Opportunity> {
    let ours = shape::of(owned);
    let mut items = Vec::new();
    let mut schema_missing: BTreeSet<String> = BTreeSet::new();
    let mut seen_archetype = BTreeSet::new();
    for (origin, inventory) in others {
        let theirs = shape::of(inventory);
        for archetype in theirs.archetypes.difference(&ours.archetypes) {
            if !seen_archetype.insert(archetype.clone()) {
                continue;
            }
            items.push(Opportunity::unmeasured_demand(
                "cluster_gap",
                format!("{origin}:{archetype}"),
                format!("owned site lacks the `{archetype}` page archetype seen on a public competitor origin"),
                "The gap is structural. Do not copy competitor copy.",
                format!("Decide whether a first-party `{archetype}` family belongs in the target architecture."),
            ));
        }
        for prefix in theirs.prefixes.difference(&ours.prefixes) {
            if prefix.contains("/blog/") || prefix.contains("/category/") {
                continue;
            }
            items.push(Opportunity::unmeasured_demand(
                "cluster_gap",
                format!("{origin}:prefix:{prefix}"),
                format!(
                    "owned site has no `{prefix}` URL family observed on a public competitor origin"
                ),
                "Prefix gaps are structural, not content to clone.",
                "Add the family only when first-party demand and facts exist.",
            ));
        }
        for schema in theirs.schema_types.difference(&ours.schema_types) {
            schema_missing.insert(schema.clone());
        }
        for locale in theirs.locales.difference(&ours.locales) {
            items.push(Opportunity::unmeasured_demand(
                "market_gap",
                format!("{origin}:locale:{locale}"),
                format!("owned site has no observed `{locale}` locale while a public competitor origin does"),
                "Locale coverage is observed from hreflang and html lang.",
                "Add the locale only when the market and content exist.",
            ));
        }
        cardinality_gap(origin, &ours, &theirs, &mut items);
        if theirs.internal_links > ours.internal_links.saturating_mul(2)
            && theirs.internal_links >= 8
        {
            items.push(Opportunity::unmeasured_demand(
                "link_gap",
                origin.clone(),
                format!(
                    "public competitor origin has {} internal links vs owned {}",
                    theirs.internal_links, ours.internal_links
                ),
                "Graph leverage is structural. Do not copy competitor anchors.",
                "Add first-party contextual links toward owned landings and families.",
            ));
        }
        if theirs.indexable > 0
            && ours.indexable > 0
            && theirs.with_h1 * 2 > theirs.indexable
            && ours.with_h1 * 2 < ours.indexable
        {
            items.push(Opportunity::unmeasured_demand(
                "content_gap",
                origin.clone(),
                "owned indexable pages are missing H1 coverage that a public competitor origin has",
                "H1 coverage is a template fact, not a license to copy competitor headings.",
                "Emit one H1 from the owned templates that lack it.",
            ));
        }
    }
    for schema in schema_missing {
        items.push(Opportunity::unmeasured_demand(
            "schema_gap",
            schema.clone(),
            format!(
                "owned site does not emit `{schema}` JSON-LD observed on a public competitor origin"
            ),
            "Schema must stay backed by first-party facts.",
            "Add the type only when domain facts support every required field.",
        ));
    }
    if others.is_empty() {
        items.push(Opportunity::unmeasured_demand(
            "cluster_gap",
            "compare",
            "Competitor comparison is unmeasured",
            "No public competitor origin was crawled.",
            "Pass --competitor URL.",
        ));
    }
    items
}

fn cardinality_gap(origin: &str, ours: &Shape, theirs: &Shape, items: &mut Vec<Opportunity>) {
    for (archetype, count) in &theirs.archetype_counts {
        let owned = ours.archetype_counts.get(archetype).copied().unwrap_or(0);
        if *count >= 4 && *count > owned.saturating_mul(2).saturating_add(2) {
            items.push(Opportunity::unmeasured_demand(
                "cluster_gap",
                format!("{origin}:{archetype}:cardinality"),
                format!(
                    "owned `{archetype}` inventory is {owned} URLs vs {count} on a public competitor origin"
                ),
                "Cardinality is not a reason to generate thin pages.",
                "Expand the family only when each URL has unique facts.",
            ));
        }
    }
}

/// Evidence for an unrun comparison.
#[must_use]
pub fn unmeasured_evidence() -> Evidence {
    Evidence::unmeasured(EvidenceSource::Http)
}

#[cfg(test)]
mod tests {
    use super::compare_inventories;
    use weavatrix_seo_model::{
        AbsoluteUrl, AnalysisMode, ContentHash, Evidence, ExtractedPage, Indexability, Inventory,
        MediaKind, Relation,
    };

    fn page(path: &str, h1: &str) -> ExtractedPage {
        let url = AbsoluteUrl::parse(&format!("https://x.test{path}")).unwrap();
        ExtractedPage {
            url: url.clone(),
            requested: url,
            status: 200,
            redirects: Vec::new(),
            content_type: Some("text/html".into()),
            media: MediaKind::Html,
            canonical: None,
            robots: Vec::new(),
            title: Some(h1.into()),
            description: None,
            html_lang: Some("en".into()),
            alternates: Vec::new(),
            headings: vec![weavatrix_seo_model::Heading {
                level: 1,
                text: h1.into(),
            }],
            links: Vec::new(),
            link_refs: Vec::new(),
            images: Vec::new(),
            json_ld: Vec::new(),
            text: h1.into(),
            heading_text: h1.into(),
            main_text: String::new(),
            payload: String::new(),
            arbitrary_script: String::new(),
            og_title: None,
            og_description: None,
            og_image: None,
            headers: Vec::new(),
            body_bytes: 1,
            fetch_ms: 1,
            has_main: false,
            unlabeled_controls: 0,
            content_hash: ContentHash::of_str(h1),
            indexability: Indexability::Indexable,
            in_sitemap: true,
            linked_from_page: true,
            evidence: Evidence::http(),
        }
        .finalize()
    }

    #[test]
    fn flags_missing_faq_archetype() {
        let mut owned = Inventory::blank(AnalysisMode::Site);
        owned.pages = vec![page("/", "Home")];
        let mut other = Inventory::blank(AnalysisMode::Site);
        other.pages = vec![page("/faq", "FAQ")];
        let items = compare_inventories(&owned, &[("https://c.test/".into(), other)]);
        assert!(
            items.iter().any(|item| item.summary.contains("faq")),
            "{items:?}"
        );
    }

    #[test]
    fn flags_internal_link_leverage() {
        let mut owned = Inventory::blank(AnalysisMode::Site);
        owned.pages = vec![page("/", "Home")];
        let mut other = Inventory::blank(AnalysisMode::Site);
        other.pages = vec![page("/", "Home"), page("/a", "A")];
        let src = AbsoluteUrl::parse("https://x.test/").unwrap();
        let dst = AbsoluteUrl::parse("https://x.test/a").unwrap();
        other.edges = (0..8)
            .map(|_| {
                weavatrix_seo_model::GraphEdge::new(
                    src.clone(),
                    dst.clone(),
                    Relation::LinksTo,
                    Evidence::http(),
                )
            })
            .collect();
        let items = compare_inventories(&owned, &[("https://c.test/".into(), other)]);
        assert!(
            items.iter().any(|item| item.kind == "link_gap"),
            "{items:?}"
        );
    }
}

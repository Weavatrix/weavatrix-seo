//! Public competitor graph comparison. Copies no competitor prose.

#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use weavatrix_seo_model::{Evidence, EvidenceSource, Inventory, Opportunity};

/// Comparison request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompareRequest {
    /// Owned site.
    pub site: String,
    /// Public competitor origins.
    pub competitors: Vec<String>,
}

/// Structural archetypes derived from URL paths and schema types.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Shape {
    archetypes: BTreeSet<String>,
    schema_types: BTreeSet<String>,
    locales: BTreeSet<String>,
}

/// Compares an owned inventory with crawled public competitor inventories.
#[must_use]
pub fn compare_inventories(owned: &Inventory, others: &[(String, Inventory)]) -> Vec<Opportunity> {
    let ours = shape(owned);
    let mut items = Vec::new();
    let mut missing: BTreeSet<String> = BTreeSet::new();
    let mut schema_missing: BTreeSet<String> = BTreeSet::new();
    for (origin, inventory) in others {
        let theirs = shape(inventory);
        for archetype in theirs.archetypes.difference(&ours.archetypes) {
            missing.insert(archetype.clone());
            items.push(Opportunity::unmeasured_demand(
                "cluster_gap",
                format!("{origin}:{archetype}"),
                format!("owned site lacks the `{archetype}` page archetype seen on a public competitor origin"),
                "The gap is structural. Do not copy competitor copy.",
                format!("Decide whether a first-party `{archetype}` family belongs in the target architecture."),
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
    let _ = missing;
    items
}

/// Evidence for an unrun comparison.
#[must_use]
pub fn unmeasured_evidence() -> Evidence {
    Evidence::unmeasured(EvidenceSource::Http)
}

fn shape(inventory: &Inventory) -> Shape {
    let mut archetypes = BTreeSet::new();
    let mut schema_types = BTreeSet::new();
    let mut locales = BTreeSet::new();
    for page in &inventory.pages {
        let path = page.url.path().to_ascii_lowercase();
        for (token, name) in [
            ("/blog", "blog"),
            ("/category", "category"),
            ("/categories", "category"),
            ("/service", "service"),
            ("/faq", "faq"),
            ("/price", "pricing"),
            ("/review", "reviews"),
            ("/about", "about"),
            ("/specialist", "profile"),
            ("/pro", "pro"),
            ("/city", "city"),
        ] {
            if path.contains(token) {
                archetypes.insert(name.to_owned());
            }
        }
        if page
            .headings
            .iter()
            .any(|heading| heading.text.to_ascii_lowercase().contains("faq"))
        {
            archetypes.insert("faq".into());
        }
        for block in &page.json_ld {
            schema_types.extend(block.types.iter().cloned());
        }
        if let Some(lang) = &page.html_lang {
            locales.insert(lang.to_ascii_lowercase());
        }
        for alternate in &page.alternates {
            locales.insert(alternate.hreflang.to_ascii_lowercase());
        }
    }
    Shape {
        archetypes,
        schema_types,
        locales,
    }
}

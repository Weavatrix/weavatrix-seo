//! Structural shape of a crawled public origin. No prose.

use std::collections::{BTreeMap, BTreeSet};
use weavatrix_seo_model::{Indexability, Inventory, Relation};

/// Structural archetypes derived from URL paths and schema types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shape {
    /// Page-family tokens.
    pub archetypes: BTreeSet<String>,
    /// Counts per archetype.
    pub archetype_counts: BTreeMap<String, usize>,
    /// JSON-LD types.
    pub schema_types: BTreeSet<String>,
    /// html lang and hreflang.
    pub locales: BTreeSet<String>,
    /// First two path segments.
    pub prefixes: BTreeSet<String>,
    /// Indexable 200s.
    pub indexable: usize,
    /// Indexable pages with an H1.
    pub with_h1: usize,
    /// Internal `LinksTo` edges.
    pub internal_links: usize,
}

/// Builds a shape from one inventory.
#[must_use]
pub fn of(inventory: &Inventory) -> Shape {
    let mut archetypes = BTreeSet::new();
    let mut archetype_counts = BTreeMap::new();
    let mut schema_types = BTreeSet::new();
    let mut locales = BTreeSet::new();
    let mut prefixes = BTreeSet::new();
    let mut indexable = 0;
    let mut with_h1 = 0;
    for page in &inventory.pages {
        let path = page.url.path().to_ascii_lowercase();
        for name in archetypes_in(&path) {
            archetypes.insert(name.clone());
            *archetype_counts.entry(name).or_insert(0) += 1;
        }
        if page
            .headings
            .iter()
            .any(|heading| heading.text.to_ascii_lowercase().contains("faq"))
        {
            archetypes.insert("faq".into());
        }
        if let Some(prefix) = prefix_of(&path) {
            prefixes.insert(prefix);
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
        if page.status == 200 && page.indexability == Indexability::Indexable {
            indexable += 1;
            if page.headings.iter().any(|heading| heading.level == 1) {
                with_h1 += 1;
            }
        }
    }
    let internal_links = inventory
        .edges
        .iter()
        .filter(|edge| edge.relation == Relation::LinksTo)
        .count();
    Shape {
        archetypes,
        archetype_counts,
        schema_types,
        locales,
        prefixes,
        indexable,
        with_h1,
        internal_links,
    }
}

fn archetypes_in(path: &str) -> Vec<String> {
    let mut out = Vec::new();
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
        ("/guide", "guide"),
        ("/docs", "docs"),
        ("/compare", "compare"),
    ] {
        if path.contains(token) {
            out.push(name.to_owned());
        }
    }
    out
}

fn prefix_of(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('/').filter(|part| !part.is_empty()).collect();
    match parts.as_slice() {
        [] => None,
        [one] => Some(format!("/{one}")),
        [one, two, ..] => Some(format!("/{one}/{two}")),
    }
}

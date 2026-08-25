//! Compile route families into programmatic safety verdicts.

use crate::{PageMatrix, SafetyVerdict};
use std::collections::BTreeMap;
use weavatrix_seo_model::{ContentHash, Indexability, Inventory};

/// Compiles measured URLs and predicted families into a page matrix.
#[must_use]
pub fn compile(inventory: &Inventory, predicted: &[String]) -> Vec<PageMatrix> {
    let mut families: BTreeMap<String, Vec<&weavatrix_seo_model::ExtractedPage>> = BTreeMap::new();
    for page in inventory.pages.iter().filter(|page| page.status == 200) {
        if let Some(family) = family_of(page.url.path()) {
            families.entry(family).or_default().push(page);
        }
    }
    for pattern in predicted {
        if is_programmatic(pattern) {
            families.entry(pattern.clone()).or_default();
        }
    }
    let mut matrices = Vec::new();
    for (family, pages) in families {
        let indexable: Vec<_> = pages
            .iter()
            .filter(|page| page.indexability == Indexability::Indexable)
            .copied()
            .collect();
        let hashes: Vec<_> = indexable
            .iter()
            .map(|page| ContentHash::of_str(&page.visible_text()))
            .collect();
        let unique = {
            let mut sorted = hashes.clone();
            sorted.sort();
            sorted.dedup();
            sorted.len()
        };
        let thin = indexable.len() >= 2 && unique == 1;
        let sitemap_only = !pages.is_empty()
            && pages
                .iter()
                .all(|page| page.in_sitemap && !page.linked_from_page);
        let all_noindex = !pages.is_empty()
            && pages
                .iter()
                .all(|page| page.indexability != Indexability::Indexable);
        let verdict = if pages.is_empty() {
            SafetyVerdict::Unmeasured
        } else if all_noindex {
            SafetyVerdict::NoindexByDefault
        } else if thin {
            SafetyVerdict::Consolidate
        } else if unique >= 2 {
            SafetyVerdict::SafeToGenerate
        } else if sitemap_only {
            SafetyVerdict::Review
        } else if unique == 1 {
            SafetyVerdict::SafeIfRequirementsMet
        } else {
            SafetyVerdict::Unmeasured
        };
        matrices.push(PageMatrix {
            family,
            cardinality: Some(u64::try_from(pages.len()).unwrap_or(0)),
            verdict,
        });
    }
    matrices.sort_by(|left, right| left.family.cmp(&right.family));
    matrices
}

fn is_programmatic(pattern: &str) -> bool {
    pattern.contains(":city")
        || pattern.contains(":slug")
        || pattern.contains(':')
        || pattern.contains('*')
}

fn family_of(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('/').filter(|part| !part.is_empty()).collect();
    let rest = if parts
        .first()
        .is_some_and(|part| matches!(*part, "en" | "ru" | "he" | "es" | "fr" | "de"))
    {
        parts.get(1..)?
    } else {
        &parts
    };
    match rest {
        ["category" | "services", slug, city] if city.contains('-') || city.len() >= 4 => {
            Some(format!("category/{slug}"))
        }
        ["category" | "services", slug] => Some(format!("category/{slug}")),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::is_programmatic;

    #[test]
    fn city_pattern_is_programmatic() {
        assert!(is_programmatic("/:locale/category/:city"));
        assert!(!is_programmatic("/about"));
    }
}

//! Family-level template vs unique-fact decomposition.

use crate::tokens::{is_fact_token, tokens};
use std::collections::{BTreeMap, BTreeSet};
use weavatrix_seo_model::{
    ExtractedPage, FamilyContent, Finding, FindingFamily, Indexability, Inventory, Locator,
    ProducerFact, Severity,
};

/// Decomposes measured programmatic families into shared / substituted / unique facts.
#[must_use]
pub fn decompose(inventory: &Inventory) -> (Vec<FamilyContent>, Vec<Finding>) {
    let mut families: BTreeMap<String, Vec<&ExtractedPage>> = BTreeMap::new();
    for page in inventory.pages.iter().filter(|page| {
        page.status == 200
            && page.indexability == Indexability::Indexable
            && !page.visible_text().trim().is_empty()
    }) {
        if let Some((family, _)) = city_family(page.url.path()) {
            families.entry(family).or_default().push(page);
        }
    }
    let mut rows = Vec::new();
    let mut findings = Vec::new();
    for (family, pages) in families {
        let row = decompose_family(&family, &pages, &inventory.producers);
        if row.measured_urls >= 2
            && row.template_shared_ratio.unwrap_or(0) >= 70
            && row.unique_fact_ratio.unwrap_or(100) < 15
        {
            findings.push(
                Finding::new(
                    FindingFamily::Content,
                    3,
                    Severity::Warn,
                    &family,
                    format!(
                        "{family} is {}% shared template with {}% unique facts across {} URLs",
                        row.template_shared_ratio.unwrap_or(0),
                        row.unique_fact_ratio.unwrap_or(0),
                        row.measured_urls
                    ),
                    Locator::Url(pages[0].url.to_string()),
                    weavatrix_seo_model::Evidence::http(),
                )
                .explained(
                    "Programmatic families need unique local facts, not only a swapped city token.",
                    "Add city-specific facts to the producer, or consolidate the matrix.",
                    "Unique factual content is a meaningful share of each URL body.",
                ),
            );
        }
        rows.push(row);
    }
    rows.sort_by(|left, right| left.family.cmp(&right.family));
    (rows, findings)
}

fn decompose_family(
    family: &str,
    pages: &[&ExtractedPage],
    producers: &[ProducerFact],
) -> FamilyContent {
    let token_sets: Vec<(BTreeSet<String>, String)> = pages
        .iter()
        .map(|page| {
            let (_, city) = city_family(page.url.path()).unwrap_or_default();
            (tokens(&page.visible_text()).into_iter().collect(), city)
        })
        .collect();
    let shared = intersection(token_sets.iter().map(|(set, _)| set));
    let mut param_tokens = 0_usize;
    let mut unique_fact = 0_usize;
    let mut unique_other = 0_usize;
    let mut total = 0_usize;
    let mut local_facts = 0_usize;
    let mut with_schema = 0_usize;
    for (index, (set, city)) in token_sets.iter().enumerate() {
        total += set.len();
        let city_tokens: BTreeSet<String> = tokens(&city.replace('-', " ")).into_iter().collect();
        for token in set {
            if shared.contains(token) {
                continue;
            }
            if city_tokens.contains(token) {
                param_tokens += 1;
            } else if is_fact_token(token) {
                unique_fact += 1;
            } else {
                unique_other += 1;
            }
        }
        if set
            .iter()
            .any(|token| is_fact_token(token) && !city_tokens.contains(token))
        {
            local_facts += 1;
        }
        if !pages[index].json_ld.is_empty() {
            with_schema += 1;
        }
    }
    let producer = producers.iter().find_map(|item| {
        if item.path.contains("city") || item.name.contains("city") || item.path.contains(family) {
            Some(item.key())
        } else {
            None
        }
    });
    FamilyContent {
        family: family.to_owned(),
        measured_urls: u64::try_from(pages.len()).unwrap_or(0),
        template_shared_ratio: share(shared.len().saturating_mul(pages.len()), total),
        parameter_substitution_ratio: share(param_tokens, total),
        unique_fact_ratio: share(unique_fact, total),
        unique_semantic_ratio: share(unique_other, total),
        local_fact_coverage: share(local_facts, pages.len()),
        schema_fact_coverage: share(with_schema, pages.len()),
        primary_producer: producer,
    }
}

fn share(part: usize, whole: usize) -> Option<u16> {
    crate::tokens::ratio(part, whole)
}

fn intersection<'a>(sets: impl Iterator<Item = &'a BTreeSet<String>>) -> BTreeSet<String> {
    let mut iter = sets.peekable();
    let Some(first) = iter.next() else {
        return BTreeSet::new();
    };
    let mut out = first.clone();
    for set in iter {
        out.retain(|token| set.contains(token));
    }
    out
}

pub(crate) fn city_family(path: &str) -> Option<(String, String)> {
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
            Some((format!("category/{slug}"), (*city).to_owned()))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::city_family;

    #[test]
    fn city_path_is_a_family() {
        assert_eq!(
            city_family("/category/electrician/vancouver-wa"),
            Some(("category/electrician".into(), "vancouver-wa".into()))
        );
    }
}

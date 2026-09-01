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
        let dimensions = dimensions_of(&family);
        let mut unmet = Vec::new();
        if unique < 2 {
            unmet.push("sufficient sample diversity".into());
        }
        if sitemap_only {
            unmet.push("discovery support beyond sitemap".into());
        }
        let (verdict, unmet) = if pages.is_empty() {
            (SafetyVerdict::Unmeasured, vec!["no measured URLs".into()])
        } else if all_noindex {
            (SafetyVerdict::NoindexByDefault, Vec::new())
        } else if thin {
            (
                SafetyVerdict::Consolidate,
                vec!["unique facts per URL".into()],
            )
        } else if unique >= 2 && unmet.is_empty() {
            // Unique samples are necessary, not sufficient. Without fact and
            // discovery evidence the family stays SAFE_IF_REQUIREMENTS_MET.
            unmet.push("fact coverage".into());
            unmet.push("semantic distinctness".into());
            unmet.push("canonical strategy".into());
            (SafetyVerdict::SafeIfRequirementsMet, unmet)
        } else if sitemap_only {
            (SafetyVerdict::Review, unmet)
        } else if unique == 1 {
            unmet.push("unique facts before expansion".into());
            (SafetyVerdict::SafeIfRequirementsMet, unmet)
        } else {
            (SafetyVerdict::Unmeasured, unmet)
        };
        matrices.push(PageMatrix {
            family,
            measured_urls: u64::try_from(pages.len()).unwrap_or(0),
            verdict,
            dimensions,
            estimated_cardinality: None,
            fact_coverage: None,
            unique_fact_ratio: None,
            template_boilerplate_ratio: None,
            semantic_distinctness: if unique >= 2 {
                Some(u16::try_from((unique * 100) / pages.len().max(1)).unwrap_or(100))
            } else {
                None
            },
            unmet_requirements: unmet,
        });
    }
    matrices.sort_by(|left, right| left.family.cmp(&right.family));
    matrices
}

fn dimensions_of(family: &str) -> Vec<String> {
    let mut dims = Vec::new();
    for part in family.split('/') {
        let token = part.strip_prefix(':').unwrap_or(part);
        if part.starts_with(':') && !dims.contains(&token.to_owned()) {
            dims.push(token.to_owned());
        } else if part == "category" && !dims.iter().any(|item| item == "service") {
            dims.push("service".into());
        }
    }
    dims
}

/// Fills fact-coverage fields from family content intelligence.
#[must_use]
pub fn enrich(
    mut matrices: Vec<PageMatrix>,
    families: &[weavatrix_seo_model::FamilyContent],
) -> Vec<PageMatrix> {
    for matrix in &mut matrices {
        let Some(row) = families
            .iter()
            .find(|item| item.family == matrix.family || matrix.family.contains(&item.family))
        else {
            continue;
        };
        matrix.fact_coverage = row.local_fact_coverage;
        matrix.unique_fact_ratio = row.unique_fact_ratio;
        matrix.template_boilerplate_ratio = row.template_shared_ratio;
        if let Some(distinct) = row.unique_semantic_ratio {
            matrix.semantic_distinctness = Some(
                distinct
                    .saturating_add(row.unique_fact_ratio.unwrap_or(0))
                    .min(100),
            );
        }
        matrix
            .unmet_requirements
            .retain(|item| item.as_str() != "fact coverage");
        if row.local_fact_coverage.unwrap_or(0) < 40 {
            if !matrix
                .unmet_requirements
                .iter()
                .any(|item| item.contains("fact"))
            {
                matrix.unmet_requirements.push("fact coverage".into());
            }
        } else {
            matrix
                .unmet_requirements
                .retain(|item| item.as_str() != "fact coverage");
        }
        if row.unique_fact_ratio.unwrap_or(0) >= 15
            && row.local_fact_coverage.unwrap_or(0) >= 40
            && matrix.measured_urls >= 2
            && matrix.verdict == SafetyVerdict::SafeIfRequirementsMet
            && matrix.unmet_requirements.is_empty()
        {
            matrix.verdict = SafetyVerdict::SafeToGenerate;
        }
    }
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

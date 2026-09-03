//! Compile route families into programmatic safety verdicts.

use crate::{PageMatrix, RequirementKind, RequirementState, SafetyVerdict, unmeasured_gates};
use std::collections::BTreeMap;
use weavatrix_seo_model::{ContentHash, Indexability, Inventory, required_gates_passed};

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
        matrices.push(compile_family(family, &pages));
    }
    matrices.sort_by(|left, right| left.family.cmp(&right.family));
    matrices
}

#[allow(clippy::too_many_lines)]
fn compile_family(family: String, pages: &[&weavatrix_seo_model::ExtractedPage]) -> PageMatrix {
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
    let mut matrix = PageMatrix {
        family,
        measured_urls: u64::try_from(pages.len()).unwrap_or(0),
        verdict: SafetyVerdict::Unmeasured,
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
        unmet_requirements: Vec::new(),
        requirements: unmeasured_gates(),
    };
    if pages.is_empty() {
        matrix.verdict = SafetyVerdict::Unmeasured;
        matrix.unmet_requirements = vec!["no measured URLs".into()];
        return matrix;
    }
    let diversity_state = if unique >= 2 {
        RequirementState::Passed
    } else {
        RequirementState::Failed
    };
    matrix.set_requirement(
        RequirementKind::SampleDiversity,
        diversity_state,
        u16::try_from(unique).ok(),
        Some(format!("{unique} unique samples")),
    );
    let discovery_state = if sitemap_only {
        RequirementState::Failed
    } else {
        RequirementState::Passed
    };
    matrix.set_requirement(
        RequirementKind::DiscoverySupport,
        discovery_state,
        None,
        Some(if sitemap_only {
            "sitemap-only URLs".into()
        } else {
            "linked from a page".into()
        }),
    );
    let with_canonical = indexable
        .iter()
        .filter(|page| page.canonical.as_ref().is_some_and(|item| !item.is_empty()))
        .count();
    if indexable.is_empty() {
        matrix.set_requirement(
            RequirementKind::CanonicalStrategy,
            RequirementState::Unmeasured,
            None,
            None,
        );
    } else {
        let percent = u16::try_from((with_canonical * 100) / indexable.len().max(1)).unwrap_or(0);
        let state = if with_canonical == indexable.len() {
            RequirementState::Passed
        } else {
            RequirementState::Failed
        };
        matrix.set_requirement(
            RequirementKind::CanonicalStrategy,
            state,
            Some(percent),
            Some(format!(
                "{with_canonical}/{} pages declare a canonical",
                indexable.len()
            )),
        );
    }
    // Unique hashes are not fact coverage or semantic distinctness.
    matrix.set_requirement(
        RequirementKind::FactCoverage,
        RequirementState::Unmeasured,
        None,
        Some("awaiting content intelligence".into()),
    );
    matrix.set_requirement(
        RequirementKind::SemanticDistinctness,
        RequirementState::Unmeasured,
        None,
        Some("awaiting content intelligence".into()),
    );
    matrix.verdict = if all_noindex {
        SafetyVerdict::NoindexByDefault
    } else if thin {
        SafetyVerdict::Consolidate
    } else if unique >= 2 && !sitemap_only {
        SafetyVerdict::SafeIfRequirementsMet
    } else if sitemap_only {
        SafetyVerdict::Review
    } else if unique == 1 {
        SafetyVerdict::SafeIfRequirementsMet
    } else {
        SafetyVerdict::Unmeasured
    };
    if thin {
        matrix.unmet_requirements = vec!["unique facts per URL".into()];
        matrix
            .unmet_requirements
            .extend(weavatrix_seo_model::unmet_labels(&matrix.requirements));
        matrix.unmet_requirements.sort();
        matrix.unmet_requirements.dedup();
    } else if unique == 1
        && !all_noindex
        && !matrix
            .unmet_requirements
            .iter()
            .any(|item| item.contains("unique facts"))
    {
        matrix
            .unmet_requirements
            .insert(0, "unique facts before expansion".into());
    }
    matrix
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
        let fact = row.local_fact_coverage;
        matrix.fact_coverage = fact;
        let fact_state = match fact {
            Some(value) if value >= 40 => RequirementState::Passed,
            Some(_) => RequirementState::Failed,
            None => RequirementState::Unmeasured,
        };
        matrix.set_requirement(
            RequirementKind::FactCoverage,
            fact_state,
            fact,
            fact.map(|value| format!("local fact coverage {value}")),
        );
        let unique_facts = row.unique_fact_ratio;
        let semantic_state = match (unique_facts, matrix.semantic_distinctness) {
            (Some(ratio), Some(_)) if ratio >= 15 => RequirementState::Passed,
            (Some(ratio), _) if ratio < 15 => RequirementState::Failed,
            _ => RequirementState::Unmeasured,
        };
        matrix.set_requirement(
            RequirementKind::SemanticDistinctness,
            semantic_state,
            matrix.semantic_distinctness,
            unique_facts.map(|value| format!("unique fact ratio {value}")),
        );
        if matrix.verdict == SafetyVerdict::SafeIfRequirementsMet
            && matrix.measured_urls >= 2
            && required_gates_passed(&matrix.requirements)
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

    #[test]
    fn unique_samples_leave_required_gates_unmeasured() {
        use crate::{PageMatrix, RequirementKind, RequirementState, SafetyVerdict};
        use weavatrix_seo_model::required_gates_passed;

        let mut matrix = PageMatrix {
            family: "category/electrician".into(),
            measured_urls: 2,
            verdict: SafetyVerdict::SafeIfRequirementsMet,
            dimensions: vec!["service".into()],
            estimated_cardinality: None,
            fact_coverage: None,
            unique_fact_ratio: None,
            template_boilerplate_ratio: None,
            semantic_distinctness: Some(100),
            unmet_requirements: Vec::new(),
            requirements: crate::unmeasured_gates(),
        };
        matrix.set_requirement(
            RequirementKind::SampleDiversity,
            RequirementState::Passed,
            Some(2),
            None,
        );
        matrix.set_requirement(
            RequirementKind::CanonicalStrategy,
            RequirementState::Passed,
            Some(100),
            None,
        );
        assert!(!required_gates_passed(&matrix.requirements));
        assert_ne!(matrix.verdict, SafetyVerdict::SafeToGenerate);
        assert!(
            matrix
                .unmet_requirements
                .iter()
                .any(|item| item == "fact coverage")
        );
    }

    #[test]
    fn enrich_promotes_only_when_required_gates_pass() {
        use crate::{PageMatrix, RequirementKind, RequirementState, SafetyVerdict, enrich};
        use weavatrix_seo_model::FamilyContent;

        let ready = |unique_facts: u16, coverage: u16| {
            let mut matrix = PageMatrix {
                family: "category/electrician".into(),
                measured_urls: 2,
                verdict: SafetyVerdict::SafeIfRequirementsMet,
                dimensions: vec!["service".into()],
                estimated_cardinality: None,
                fact_coverage: None,
                unique_fact_ratio: None,
                template_boilerplate_ratio: None,
                semantic_distinctness: Some(80),
                unmet_requirements: Vec::new(),
                requirements: crate::unmeasured_gates(),
            };
            matrix.set_requirement(
                RequirementKind::SampleDiversity,
                RequirementState::Passed,
                Some(2),
                None,
            );
            matrix.set_requirement(
                RequirementKind::CanonicalStrategy,
                RequirementState::Passed,
                Some(100),
                None,
            );
            let families = [FamilyContent {
                family: "category/electrician".into(),
                measured_urls: 2,
                template_shared_ratio: Some(40),
                parameter_substitution_ratio: None,
                unique_fact_ratio: Some(unique_facts),
                unique_semantic_ratio: Some(40),
                local_fact_coverage: Some(coverage),
                schema_fact_coverage: None,
                primary_producer: None,
                gsc_clicks: None,
                gsc_impressions: None,
                error_findings: None,
            }];
            enrich(vec![matrix], &families).remove(0)
        };
        let blocked = ready(10, 80);
        assert_eq!(blocked.verdict, SafetyVerdict::SafeIfRequirementsMet);
        assert!(
            blocked
                .requirements
                .iter()
                .any(|item| item.kind == RequirementKind::SemanticDistinctness
                    && item.state == RequirementState::Failed)
        );
        let allowed = ready(20, 80);
        assert_eq!(allowed.verdict, SafetyVerdict::SafeToGenerate);
        assert!(allowed.unmet_requirements.is_empty());
    }
}

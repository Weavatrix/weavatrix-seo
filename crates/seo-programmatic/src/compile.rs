//! Compile route families into programmatic safety verdicts.

use crate::{PageMatrix, RequirementKind, RequirementState, SafetyVerdict, unmeasured_gates};
use std::collections::BTreeMap;
use weavatrix_seo_model::{
    ContentHash, Finding, FindingFamily, Indexability, Inventory, required_gates_passed,
};

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
        matrices.push(compile_family(family, &pages, inventory));
    }
    matrices.sort_by(|left, right| left.family.cmp(&right.family));
    matrices
}

#[allow(clippy::too_many_lines)]
fn compile_family(
    family: String,
    pages: &[&weavatrix_seo_model::ExtractedPage],
    inventory: &Inventory,
) -> PageMatrix {
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
        measured_sample_rate: None,
        canonical_coverage: None,
        internal_discovery: None,
        demand_coverage: None,
        schema_fact_coverage: None,
        claim_integrity: None,
        cannibalization_risk: None,
        doorway_risk: None,
        conversion_readiness: None,
    };
    let estimated = estimate_cardinality(&matrix.family, pages, inventory);
    matrix.estimated_cardinality = estimated;
    if let Some(total) = estimated.filter(|value| *value > 0) {
        matrix.measured_sample_rate =
            u16::try_from((matrix.measured_urls.saturating_mul(100)) / total)
                .ok()
                .map(|value| value.min(100));
    }
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
        matrix.canonical_coverage = Some(percent);
    }
    matrix.internal_discovery = Some(if sitemap_only { 0 } else { 100 });
    matrix.set_requirement(
        RequirementKind::InternalLinkSupport,
        if sitemap_only {
            RequirementState::Failed
        } else {
            RequirementState::Passed
        },
        matrix.internal_discovery,
        None,
    );
    if !pages.is_empty() {
        let risk = u16::try_from(100usize.saturating_sub((unique * 100) / pages.len().max(1)))
            .unwrap_or(0);
        matrix.cannibalization_risk = Some(risk);
        matrix.set_requirement(
            RequirementKind::CannibalizationRisk,
            if thin {
                RequirementState::Failed
            } else if unique >= 2 {
                RequirementState::Passed
            } else {
                RequirementState::Unmeasured
            },
            Some(risk),
            None,
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
        matrix.schema_fact_coverage = row.schema_fact_coverage;
        matrix.conversion_readiness = row.schema_fact_coverage;
        let conversion_state = match row.schema_fact_coverage {
            Some(value) if value >= 40 => RequirementState::Passed,
            Some(_) => RequirementState::Failed,
            None => RequirementState::Unmeasured,
        };
        matrix.set_requirement(
            RequirementKind::ConversionReadiness,
            conversion_state,
            row.schema_fact_coverage,
            None,
        );
        let doorway = match (row.template_shared_ratio, unique_facts) {
            (Some(shared), Some(facts)) if shared >= 70 && facts < 15 => Some(80),
            (Some(shared), _) if shared >= 70 => Some(55),
            (_, Some(facts)) if facts >= 15 => Some(20),
            _ => None,
        };
        matrix.doorway_risk = doorway;
        matrix.set_requirement(
            RequirementKind::DoorwayRisk,
            match doorway {
                Some(value) if value >= 70 => RequirementState::Failed,
                Some(_) => RequirementState::Passed,
                None => RequirementState::Unmeasured,
            },
            doorway,
            None,
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

/// Fills demand and claim-integrity after GSC rollup and claim findings exist.
#[must_use]
pub fn annotate(
    mut matrices: Vec<PageMatrix>,
    findings: &[Finding],
    families: &[weavatrix_seo_model::FamilyContent],
) -> Vec<PageMatrix> {
    let claims_ran = findings
        .iter()
        .any(|item| item.family == FindingFamily::Claim);
    for matrix in &mut matrices {
        if let Some(row) = families
            .iter()
            .find(|item| item.family == matrix.family || matrix.family.contains(&item.family))
        {
            if let Some(impressions) = row.gsc_impressions {
                let coverage = if impressions > 0 { 100 } else { 0 };
                matrix.demand_coverage = Some(coverage);
                matrix.set_requirement(
                    RequirementKind::DemandEvidence,
                    if impressions > 0 {
                        RequirementState::Passed
                    } else {
                        RequirementState::Failed
                    },
                    Some(coverage),
                    Some(format!("gsc impressions {impressions}")),
                );
            }
        }
        if !claims_ran {
            continue;
        }
        let hits = findings
            .iter()
            .filter(|item| item.family == FindingFamily::Claim)
            .filter(|item| {
                item.locator.subject_url().contains(&matrix.family)
                    || item.summary.contains(&matrix.family)
            })
            .count();
        let score = 100_u16.saturating_sub(u16::try_from(hits.saturating_mul(20)).unwrap_or(100));
        matrix.claim_integrity = Some(score);
        matrix.set_requirement(
            RequirementKind::ClaimIntegrity,
            if hits == 0 {
                RequirementState::Passed
            } else {
                RequirementState::Failed
            },
            Some(score),
            Some(format!("{hits} claim findings")),
        );
    }
    matrices
}

fn estimate_cardinality(
    family: &str,
    pages: &[&weavatrix_seo_model::ExtractedPage],
    inventory: &Inventory,
) -> Option<u64> {
    let mut slugs = std::collections::BTreeSet::new();
    for page in pages {
        if let Some(slug) = city_slug(page.url.path(), family) {
            slugs.insert(slug);
        }
    }
    let needle = format!("/{family}/");
    for page in &inventory.pages {
        for link in &page.links {
            let Some(at) = link.find(&needle) else {
                continue;
            };
            let rest = &link[at + needle.len()..];
            let slug = rest.split(['/', '?', '#']).next().unwrap_or("");
            if looks_like_city_slug(slug) {
                slugs.insert(slug.to_owned());
            }
        }
    }
    if slugs.is_empty() {
        None
    } else {
        Some(u64::try_from(slugs.len()).unwrap_or(0))
    }
}

fn city_slug(path: &str, family: &str) -> Option<String> {
    let needle = format!("/{family}/");
    let rest = path.find(&needle).map(|at| &path[at + needle.len()..])?;
    let slug = rest.split('/').find(|part| !part.is_empty())?;
    looks_like_city_slug(slug).then(|| (*slug).to_owned())
}

fn looks_like_city_slug(segment: &str) -> bool {
    if matches!(
        segment,
        "prices" | "reviews" | "about" | "new" | "edit" | "index" | "all"
    ) {
        return false;
    }
    segment.contains('-') || segment.len() >= 4
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
            measured_sample_rate: None,
            canonical_coverage: None,
            internal_discovery: None,
            demand_coverage: None,
            schema_fact_coverage: None,
            claim_integrity: None,
            cannibalization_risk: None,
            doorway_risk: None,
            conversion_readiness: None,
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
                measured_sample_rate: None,
                canonical_coverage: None,
                internal_discovery: None,
                demand_coverage: None,
                schema_fact_coverage: None,
                claim_integrity: None,
                cannibalization_risk: None,
                doorway_risk: None,
                conversion_readiness: None,
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
        assert_eq!(allowed.doorway_risk, Some(20));
    }

    #[test]
    fn thin_shared_templates_are_doorway_risk() {
        use crate::{PageMatrix, RequirementKind, RequirementState, SafetyVerdict, enrich};
        use weavatrix_seo_model::FamilyContent;

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
            measured_sample_rate: None,
            canonical_coverage: None,
            internal_discovery: None,
            demand_coverage: None,
            schema_fact_coverage: None,
            claim_integrity: None,
            cannibalization_risk: None,
            doorway_risk: None,
            conversion_readiness: None,
        };
        matrix.set_requirement(
            RequirementKind::SampleDiversity,
            RequirementState::Passed,
            Some(2),
            None,
        );
        let families = [FamilyContent {
            family: "category/electrician".into(),
            measured_urls: 2,
            template_shared_ratio: Some(85),
            parameter_substitution_ratio: None,
            unique_fact_ratio: Some(1),
            unique_semantic_ratio: Some(10),
            local_fact_coverage: Some(100),
            schema_fact_coverage: Some(0),
            primary_producer: None,
            gsc_clicks: None,
            gsc_impressions: None,
            error_findings: None,
        }];
        let out = enrich(vec![matrix], &families).remove(0);
        assert_eq!(out.doorway_risk, Some(80));
        assert!(
            out.requirements
                .iter()
                .any(|item| item.kind == RequirementKind::DoorwayRisk
                    && item.state == RequirementState::Failed)
        );
        assert_eq!(out.schema_fact_coverage, Some(0));
    }
}

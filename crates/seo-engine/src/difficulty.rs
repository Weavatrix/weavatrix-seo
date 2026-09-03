//! Personalized difficulty: rank in the SERP vs build from owned truth.
//!
//! These axes stay separate. [`weavatrix_seo_model::OpportunityAxes::rank_key`]
//! never consults them.

use weavatrix_seo_architecture::Architecture;
use weavatrix_seo_model::{FamilyContent, Inventory, Opportunity};
use weavatrix_seo_observation::{ObservationKind, ObservationSnapshot};

/// Fills `difficulty_to_rank` and `difficulty_to_build` from owned evidence.
pub fn apply(
    item: &mut Opportunity,
    snapshot: &ObservationSnapshot,
    inventory: &Inventory,
    architecture: &Architecture,
    families: &[FamilyContent],
) {
    if item.axes.implementation_cost.is_none() {
        item.axes.implementation_cost = Some(kind_cost(&item.kind));
    }
    if item.axes.graph_leverage.is_none() {
        if let Some(page) = arch_page(architecture, &item.subject) {
            item.axes.graph_leverage = Some(leverage_from_inbound(page.inbound));
        } else if item.kind == "link_gap" || item.kind == "link_rec" {
            item.axes.graph_leverage = Some(80);
        }
    }
    let family = family_for(&item.subject, families);
    item.axes.difficulty_to_rank = difficulty_to_rank(item, snapshot, architecture);
    item.axes.difficulty_to_build = Some(difficulty_to_build(item, inventory, family));
}

/// External ranking difficulty. Unmeasured without keyword/SERP evidence.
fn difficulty_to_rank(
    item: &Opportunity,
    snapshot: &ObservationSnapshot,
    architecture: &Architecture,
) -> Option<u16> {
    let mut external: Option<i32> = None;
    let mut serp_boost = 0_i32;
    for row in snapshot.rows.iter().filter(|row| {
        row.kind == ObservationKind::KeywordVolume
            || row.kind == ObservationKind::SerpPosition
            || row.kind == ObservationKind::SerpFeature
    }) {
        if !row_matches(row, item) {
            continue;
        }
        if let Some(difficulty) = row.difficulty {
            external = Some(external.map_or(i32::from(difficulty), |current| {
                current.max(i32::from(difficulty))
            }));
        }
        if row.kind == ObservationKind::SerpPosition {
            serp_boost = serp_boost.saturating_add(8);
            if row.position.is_some_and(|position| position <= 3.0) {
                serp_boost = serp_boost.saturating_add(10);
            }
        }
        if row
            .serp_features
            .iter()
            .any(|feature| feature.contains("ai_overview") || feature == "paa")
        {
            serp_boost = serp_boost.saturating_add(6);
        }
    }
    let Some(base) = external else {
        if serp_boost == 0 {
            return None;
        }
        return Some(clamp(
            40 + serp_boost.min(40) - authority_bonus(item, architecture),
        ));
    };
    Some(clamp(
        base + serp_boost.min(25) - authority_bonus(item, architecture),
    ))
}

/// Difficulty of building the page truthfully from owned facts and source.
fn difficulty_to_build(
    item: &Opportunity,
    inventory: &Inventory,
    family: Option<&FamilyContent>,
) -> u16 {
    let mut score = i32::from(
        item.axes
            .implementation_cost
            .unwrap_or_else(|| kind_cost(&item.kind)),
    );
    if let Some(risk) = item.axes.risk {
        score += i32::from(risk) / 5;
    }
    if let Some(family) = family {
        if let Some(facts) = family.unique_fact_ratio {
            score += i32::from(100_u16.saturating_sub(facts)) / 4;
        }
        if let Some(coverage) = family.local_fact_coverage {
            score -= i32::from(coverage) / 5;
        }
        if family.primary_producer.is_some() {
            score -= 12;
        }
    }
    if inventory.producers.iter().any(|producer| {
        producer
            .families
            .iter()
            .any(|name| item.subject.contains(name))
    }) {
        score -= 8;
    }
    if let Some(leverage) = item.axes.graph_leverage {
        score -= i32::from(leverage) / 5;
    }
    clamp(score)
}

fn kind_cost(kind: &str) -> u16 {
    match kind {
        "create_family" => 55,
        "cannibal" => 50,
        "noindex" => 25,
        "content_gap" => 28,
        "sitemap_gap" => 12,
        "link_gap" | "link_rec" => 18,
        _ => 35,
    }
}

fn leverage_from_inbound(inbound: usize) -> u16 {
    if inbound == 0 {
        80
    } else {
        20_u16
            .saturating_add(u16::try_from(inbound.saturating_mul(8)).unwrap_or(80))
            .min(80)
    }
}

fn family_for<'a>(subject: &str, families: &'a [FamilyContent]) -> Option<&'a FamilyContent> {
    families.iter().find(|family| {
        subject == family.family
            || subject.contains(&family.family)
            || family.family.contains(subject)
    })
}

fn arch_page<'a>(
    architecture: &'a Architecture,
    subject: &str,
) -> Option<&'a weavatrix_seo_architecture::PageArchitecture> {
    let needle = subject.trim_end_matches('/');
    architecture.pages.iter().find(|page| {
        let url = page.url.to_string();
        url.trim_end_matches('/') == needle || url.contains(needle)
    })
}

fn authority_bonus(item: &Opportunity, architecture: &Architecture) -> i32 {
    let Some(page) = arch_page(architecture, &item.subject) else {
        return 0;
    };
    let max = architecture
        .pages
        .iter()
        .map(|item| item.authority)
        .fold(0.0_f64, f64::max);
    if max <= 0.0 {
        return 0;
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let scaled = ((page.authority / max) * 30.0).round() as i32;
    scaled.clamp(0, 30)
}

fn row_matches(row: &weavatrix_seo_observation::Observation, item: &Opportunity) -> bool {
    let url_hit = row.url.trim_end_matches('/') == item.subject.trim_end_matches('/');
    let query_hit = row.query.as_ref().is_some_and(|query| {
        item.summary
            .to_ascii_lowercase()
            .contains(&query.to_ascii_lowercase())
            || item
                .subject
                .to_ascii_lowercase()
                .contains(&query.to_ascii_lowercase())
    });
    url_hit || query_hit
}

fn clamp(value: i32) -> u16 {
    u16::try_from(value.clamp(0, 100)).unwrap_or(100)
}

#[cfg(test)]
mod tests {
    use super::{difficulty_to_build, kind_cost};
    use weavatrix_seo_model::{
        AnalysisMode, FamilyContent, Inventory, Opportunity, OpportunityAxes,
    };

    fn family(unique_facts: u16, coverage: u16, producer: bool) -> FamilyContent {
        FamilyContent {
            family: "category/electrician".into(),
            measured_urls: 2,
            template_shared_ratio: Some(85),
            parameter_substitution_ratio: None,
            unique_fact_ratio: Some(unique_facts),
            unique_semantic_ratio: None,
            local_fact_coverage: Some(coverage),
            schema_fact_coverage: None,
            primary_producer: producer.then(|| "cities.ts".into()),
            gsc_clicks: None,
            gsc_impressions: None,
            error_findings: None,
        }
    }

    fn item(kind: &str, subject: &str) -> Opportunity {
        let mut out = Opportunity::unmeasured_demand(kind, subject, "gap", "why", "action");
        out.axes.implementation_cost = Some(kind_cost(kind));
        out
    }

    #[test]
    fn thin_family_is_harder_to_build_than_an_h1_gap() {
        let inventory = Inventory::blank(AnalysisMode::Repo);
        let create = item("create_family", "category/electrician");
        let h1 = item("content_gap", "https://kablay.co.il/he/about");
        let thin = difficulty_to_build(&create, &inventory, Some(&family(1, 100, false)));
        let heading = difficulty_to_build(&h1, &inventory, None);
        assert!(
            thin > heading,
            "thin programmatic build {thin} should exceed H1 gap {heading}"
        );
    }

    #[test]
    fn source_reuse_and_facts_lower_build_difficulty() {
        let inventory = Inventory::blank(AnalysisMode::Repo);
        let create = item("create_family", "category/electrician");
        let thin = difficulty_to_build(&create, &inventory, Some(&family(1, 40, false)));
        let ready = difficulty_to_build(&create, &inventory, Some(&family(40, 80, true)));
        assert!(ready < thin, "ready {ready} vs thin {thin}");
    }

    #[test]
    fn rank_stays_unmeasured_without_market_data() {
        let item = item("create_family", "category/electrician");
        let snapshot = weavatrix_seo_observation::unmeasured();
        let architecture = weavatrix_seo_architecture::Architecture { pages: Vec::new() };
        assert!(super::difficulty_to_rank(&item, &snapshot, &architecture).is_none());
    }

    #[test]
    fn keyword_difficulty_becomes_rank_not_demand() {
        let item = item("create_family", "https://x.test/a");
        let snapshot = weavatrix_seo_observation::from_any(
            r#"{"provider":"semrush","keywords":[{"url":"https://x.test/a","query":"electrician","volume":2400,"difficulty":47}]}"#,
        )
        .expect("semrush");
        let architecture = weavatrix_seo_architecture::Architecture { pages: Vec::new() };
        assert_eq!(
            super::difficulty_to_rank(&item, &snapshot, &architecture),
            Some(47)
        );
        assert!(item.axes.demand.is_none());
    }

    #[test]
    fn difficulty_axes_do_not_change_rank_key() {
        let low = OpportunityAxes {
            demand: Some(40),
            difficulty_to_rank: Some(10),
            difficulty_to_build: Some(10),
            ..OpportunityAxes::default()
        };
        let high = OpportunityAxes {
            demand: Some(40),
            difficulty_to_rank: Some(90),
            difficulty_to_build: Some(90),
            ..OpportunityAxes::default()
        };
        assert_eq!(low.rank_key(), high.rank_key());
    }
}

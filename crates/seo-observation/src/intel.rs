//! Deterministic GSC views: decay, CTR gap, striking distance, cannibalization.
//!
//! Expected CTR is [`weavatrix_seo_model::EvidenceKind::Inferred`]. Period
//! tags (`current` / `previous`) are required for decay; a single untagged
//! export still supports CTR and striking-distance views.

use crate::{Observation, ObservationKind, ObservationSnapshot};
use std::collections::BTreeMap;
use weavatrix_seo_model::{
    Evidence, EvidenceKind, EvidenceSource, Finding, FindingFamily, Inventory, Locator, Opportunity,
};

/// Inferred desktop web CTR by average position. Not a ranking guarantee.
#[must_use]
pub fn expected_ctr(position: f32) -> f32 {
    if position <= 1.0 {
        0.28
    } else if position <= 2.0 {
        0.15
    } else if position <= 3.0 {
        0.11
    } else if position <= 4.0 {
        0.08
    } else if position <= 5.0 {
        0.06
    } else if position <= 7.0 {
        0.04
    } else if position <= 10.0 {
        0.025
    } else if position <= 20.0 {
        0.012
    } else {
        0.005
    }
}

/// Findings and construction opportunities from a GSC snapshot.
pub struct ObservationIntel {
    /// Catalogue findings.
    pub findings: Vec<Finding>,
    /// Additive opportunities.
    pub opportunities: Vec<Opportunity>,
}

/// Runs the GSC intelligence pass. Invalid or empty snapshots yield nothing.
#[must_use]
pub fn analyze(snapshot: &ObservationSnapshot, inventory: &Inventory) -> ObservationIntel {
    if !snapshot.connected {
        return ObservationIntel {
            findings: Vec::new(),
            opportunities: Vec::new(),
        };
    }
    let current: Vec<&Observation> = snapshot
        .rows
        .iter()
        .filter(|row| row.kind == ObservationKind::SearchPerformance)
        .filter(|row| period_bucket(row.period.as_deref()) == "current")
        .collect();
    let previous: Vec<&Observation> = snapshot
        .rows
        .iter()
        .filter(|row| row.kind == ObservationKind::SearchPerformance)
        .filter(|row| period_bucket(row.period.as_deref()) == "previous")
        .collect();
    let mut findings = Vec::new();
    let mut opportunities = Vec::new();
    findings.extend(decay_findings(&current, &previous));
    let (ctr_findings, ctr_opps) = ctr_gaps(&current);
    findings.extend(ctr_findings);
    opportunities.extend(ctr_opps);
    let (strike_findings, strike_opps) = striking_distance(&current, inventory);
    findings.extend(strike_findings);
    opportunities.extend(strike_opps);
    let (cann_findings, cann_opps) = cannibalization(&current);
    findings.extend(cann_findings);
    opportunities.extend(cann_opps);
    ObservationIntel {
        findings,
        opportunities,
    }
}

fn period_bucket(period: Option<&str>) -> &'static str {
    let Some(period) = period else {
        return "current";
    };
    let lower = period.to_ascii_lowercase();
    if lower.contains("prev") || lower.contains("prior") || lower.contains("before") {
        "previous"
    } else if lower.contains("yoy") || lower.contains("year") {
        "yoy"
    } else {
        "current"
    }
}

fn decay_findings(current: &[&Observation], previous: &[&Observation]) -> Vec<Finding> {
    if previous.is_empty() {
        return Vec::new();
    }
    let prev_by_url = rollup(previous);
    let cur_by_url = rollup(current);
    let mut findings = Vec::new();
    for (url, before) in prev_by_url {
        let after = cur_by_url
            .get(&url)
            .copied()
            .unwrap_or(UrlRollup::default());
        if before.clicks < 10 {
            continue;
        }
        let drop = before.clicks.saturating_sub(after.clicks);
        if drop * 100 / before.clicks < 30 {
            continue;
        }
        findings.push(
            Finding::from_rule(
                FindingFamily::Obs,
                6,
                &url,
                format!(
                    "clicks on {url} fell from {} to {} versus the previous window",
                    before.clicks, after.clicks
                ),
                Locator::Url(url.clone()),
                inferred_gsc(),
            )
            .explained(
                "Search Console clicks dropped at least 30% against the tagged previous window.",
                "Diff the title/producer revision and sibling URLs that share the helper.",
                "A later window recovers clicks, or the page is intentionally deprioritised.",
            ),
        );
    }
    findings
}

fn ctr_gaps(current: &[&Observation]) -> (Vec<Finding>, Vec<Opportunity>) {
    let mut findings = Vec::new();
    let mut opportunities = Vec::new();
    for row in current
        .iter()
        .filter(|row| row.impressions >= 50)
        .filter(|row| row.position.is_some())
    {
        let position = row.position.unwrap_or(0.0);
        let expected = expected_ctr(position);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let expected_clicks = (f64::from(row.impressions) * f64::from(expected)).round() as u32;
        let recoverable = expected_clicks.saturating_sub(row.clicks);
        if recoverable < 5 {
            continue;
        }
        let subject = row.query.clone().unwrap_or_else(|| row.url.clone());
        findings.push(
            Finding::from_rule(
                FindingFamily::Obs,
                4,
                &subject,
                format!(
                    "{} gets {} clicks vs ~{expected_clicks} expected at position {position:.1}",
                    row.url, row.clicks
                ),
                Locator::Url(row.url.clone()),
                inferred_gsc(),
            )
            .explained(
                "Expected CTR is an inferred curve, not a guarantee.",
                "Refresh title and snippet, or add the missing section the query asks for.",
                "Clicks approach the inferred expectation, or the query is no longer targeted.",
            ),
        );
        let mut item = Opportunity::unmeasured_demand(
            "ctr_gap",
            row.url.clone(),
            format!(
                "recover ~{recoverable} clicks on {} at position {position:.1}",
                row.url
            ),
            "Impressions exist; the snippet under-converts relative to an inferred CTR curve.",
            "Rewrite title/description from first-party facts; do not invent claims.",
        );
        item.demand = format!("impressions:{}", row.impressions);
        item.axes.raw_impressions = Some(row.impressions);
        item.axes.raw_clicks = Some(row.clicks);
        item.axes.recoverable_clicks = Some(recoverable);
        item.axes.expected_ctr = Some(ctr_pct(expected));
        item.axes.confidence = Some(55);
        opportunities.push(item);
    }
    (findings, opportunities)
}

fn striking_distance(
    current: &[&Observation],
    inventory: &Inventory,
) -> (Vec<Finding>, Vec<Opportunity>) {
    let measured = inventory.measured_urls();
    let mut findings = Vec::new();
    let mut opportunities = Vec::new();
    for row in current.iter().filter(|row| {
        row.impressions >= 50
            && row
                .position
                .is_some_and(|position| (4.0..20.0).contains(&position))
    }) {
        let known = measured.iter().any(|url| urls_match(url, &row.url));
        if !known {
            continue;
        }
        let query = row.query.as_deref().unwrap_or("(unspecified query)");
        let position = row.position.unwrap_or(0.0);
        findings.push(
            Finding::from_rule(
                FindingFamily::Obs,
                5,
                query,
                format!(
                    "{query} ranks #{position:.1} on {} with {} impressions",
                    row.url, row.impressions
                ),
                Locator::Url(row.url.clone()),
                gsc_evidence(),
            )
            .explained(
                "The page already ranks on page two or the lower first page.",
                "Add the missing section, internal links from donors, or a tighter title.",
                "Position enters the top three, or impressions fall because the query is abandoned.",
            ),
        );
        let mut item = Opportunity::unmeasured_demand(
            "striking_distance",
            row.url.clone(),
            format!("push {query} from #{position:.0} on {}", row.url),
            "The URL is already relevant enough to rank; it is not a new-page problem.",
            "Refresh the answering chunk and add an internal link from a higher-authority sibling.",
        );
        item.demand = format!("impressions:{}", row.impressions);
        item.axes.raw_impressions = Some(row.impressions);
        item.axes.raw_clicks = Some(row.clicks);
        item.axes.confidence = Some(60);
        opportunities.push(item);
    }
    (findings, opportunities)
}

fn cannibalization(current: &[&Observation]) -> (Vec<Finding>, Vec<Opportunity>) {
    let mut by_query: BTreeMap<String, Vec<&Observation>> = BTreeMap::new();
    for row in current
        .iter()
        .filter(|row| row.impressions >= 50)
        .filter(|row| row.query.as_ref().is_some_and(|query| query.len() > 3))
    {
        by_query
            .entry(row.query.clone().unwrap_or_default())
            .or_default()
            .push(*row);
    }
    let mut findings = Vec::new();
    let mut opportunities = Vec::new();
    for (query, rows) in by_query {
        let mut urls: Vec<&Observation> = Vec::new();
        for row in rows {
            if urls.iter().any(|seen| urls_match(&seen.url, &row.url)) {
                continue;
            }
            urls.push(row);
        }
        if urls.len() < 2 {
            continue;
        }
        let total_clicks: u32 = urls.iter().map(|row| row.clicks).sum();
        if total_clicks < 10 {
            continue;
        }
        let families: Vec<String> = urls.iter().map(|row| url_family(&row.url)).collect();
        let same_family = families.windows(2).all(|pair| pair[0] == pair[1]);
        let action = if same_family {
            "CONSOLIDATE"
        } else {
            "DIFFERENTIATE"
        };
        let listed = urls
            .iter()
            .map(|row| row.url.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        findings.push(
            Finding::from_rule(
                FindingFamily::Cann,
                1,
                &query,
                format!("{query} ranks on {} URLs ({action}): {listed}", urls.len()),
                Locator::Url(urls[0].url.clone()),
                gsc_evidence(),
            )
            .with_affected(urls.iter().map(|row| row.url.clone()))
            .explained(
                "Repeated impressions on several URLs split click share.",
                if same_family {
                    "Canonicalize or merge the family; they share a producer path."
                } else {
                    "Differentiate intent so each URL owns a distinct query."
                },
                "One URL holds the query, or each remaining URL ranks for a distinct intent.",
            ),
        );
        opportunities.push(Opportunity::unmeasured_demand(
            "cannibalization",
            query,
            format!("{action} overlapping URLs for this query"),
            "Two or more measured URLs collect the same query's clicks.",
            if same_family {
                "Pick one canonical URL in the family and point the others at it."
            } else {
                "Split the intents and titles so each URL answers a different question."
            },
        ));
    }
    (findings, opportunities)
}

#[derive(Clone, Copy, Default)]
struct UrlRollup {
    clicks: u32,
    impressions: u32,
}

fn rollup(rows: &[&Observation]) -> BTreeMap<String, UrlRollup> {
    let mut out = BTreeMap::new();
    for row in rows {
        let entry = out.entry(row.url.clone()).or_insert(UrlRollup::default());
        entry.clicks = entry.clicks.saturating_add(row.clicks);
        entry.impressions = entry.impressions.saturating_add(row.impressions);
    }
    out
}

fn url_family(url: &str) -> String {
    let path = url
        .split_once("://")
        .and_then(|(_, rest)| rest.split_once('/'))
        .map_or(url, |(_, path)| path);
    let trimmed = path.trim_end_matches('/');
    let mut parts: Vec<&str> = trimmed.split('/').filter(|part| !part.is_empty()).collect();
    if parts.len() >= 2 {
        parts.pop();
    }
    parts.join("/")
}

fn urls_match(left: &str, right: &str) -> bool {
    left.trim_end_matches('/') == right.trim_end_matches('/')
}

fn inferred_gsc() -> Evidence {
    Evidence {
        kind: EvidenceKind::Inferred,
        source: EvidenceSource::Gsc,
        confidence: weavatrix_seo_model::Confidence::Medium,
        snapshot_id: None,
        revision: None,
        policy_version: None,
    }
}

fn gsc_evidence() -> Evidence {
    Evidence {
        kind: EvidenceKind::Observed,
        source: EvidenceSource::Gsc,
        confidence: weavatrix_seo_model::Confidence::High,
        snapshot_id: None,
        revision: None,
        policy_version: None,
    }
}

fn ctr_pct(rate: f32) -> u16 {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let value = (rate * 100.0).round() as u16;
    value.min(100)
}

#[cfg(test)]
mod tests {
    use super::{analyze, expected_ctr, url_family};
    use crate::{Observation, ObservationKind, ObservationSnapshot};
    use weavatrix_seo_model::{
        AnalysisMode, Evidence, EvidenceKind, EvidenceSource, FetchObservation, FetchOutcome,
        Inventory,
    };

    fn row(
        url: &str,
        query: &str,
        impressions: u32,
        clicks: u32,
        position: f32,
        period: Option<&str>,
    ) -> Observation {
        Observation {
            kind: ObservationKind::SearchPerformance,
            query: Some(query.into()),
            url: url.into(),
            provider: "gsc".into(),
            evidence: Evidence {
                kind: EvidenceKind::Observed,
                source: EvidenceSource::Gsc,
                confidence: weavatrix_seo_model::Confidence::High,
                snapshot_id: None,
                revision: None,
                policy_version: None,
            },
            clicks,
            impressions,
            hits: 0,
            position: Some(position),
            period: period.map(str::to_owned),
            user_agent: None,
            status: None,
            bot_role: None,
            verified_bot: None,
            referer: None,
        }
    }

    fn snap(rows: Vec<Observation>) -> ObservationSnapshot {
        ObservationSnapshot {
            rows,
            connected: true,
            input: weavatrix_seo_model::InputState::connected("GSC"),
        }
    }

    #[test]
    fn top_positions_convert_higher_than_page_two() {
        assert!(expected_ctr(1.0) > expected_ctr(12.0));
        assert!(expected_ctr(4.0) > expected_ctr(18.0));
    }

    #[test]
    fn family_strips_the_last_slug() {
        assert_eq!(
            url_family("https://x.test/category/electrician/haifa"),
            "category/electrician"
        );
    }

    #[test]
    fn decay_requires_a_previous_window() {
        let untagged = snap(vec![row("https://x.test/a", "home", 400, 5, 8.0, None)]);
        let inventory = Inventory::blank(AnalysisMode::Site);
        assert!(
            analyze(&untagged, &inventory)
                .findings
                .iter()
                .all(|finding| finding.code != "WVX-SEO-OBS-006")
        );
        let tagged = snap(vec![
            row("https://x.test/a", "home", 500, 40, 3.0, Some("previous")),
            row("https://x.test/a", "home", 400, 10, 8.0, Some("current")),
        ]);
        assert!(
            analyze(&tagged, &inventory)
                .findings
                .iter()
                .any(|finding| finding.code == "WVX-SEO-OBS-006")
        );
    }

    #[test]
    fn ctr_gap_is_inferred_not_guaranteed() {
        let snapshot = snap(vec![row("https://x.test/a", "home", 1000, 1, 1.0, None)]);
        let intel = analyze(&snapshot, &Inventory::blank(AnalysisMode::Site));
        let finding = intel
            .findings
            .iter()
            .find(|item| item.code == "WVX-SEO-OBS-004")
            .expect("ctr gap");
        assert_eq!(
            finding.evidence.kind,
            weavatrix_seo_model::EvidenceKind::Inferred
        );
        assert!(
            intel
                .opportunities
                .iter()
                .any(|item| item.kind == "ctr_gap" && item.axes.recoverable_clicks.is_some())
        );
    }

    #[test]
    fn striking_distance_needs_a_measured_url() {
        let snapshot = snap(vec![row(
            "https://x.test/a",
            "home service",
            200,
            4,
            12.0,
            None,
        )]);
        let missing = analyze(&snapshot, &Inventory::blank(AnalysisMode::Site));
        assert!(
            missing
                .findings
                .iter()
                .all(|finding| finding.code != "WVX-SEO-OBS-005")
        );
        let mut inventory = Inventory::blank(AnalysisMode::Site);
        inventory.observations.push(FetchObservation::new(
            "https://x.test/a",
            FetchOutcome::Response,
            "measured",
        ));
        let present = analyze(&snapshot, &inventory);
        assert!(
            present
                .findings
                .iter()
                .any(|finding| finding.code == "WVX-SEO-OBS-005")
        );
        assert!(
            present
                .opportunities
                .iter()
                .any(|item| item.kind == "striking_distance")
        );
    }

    #[test]
    fn same_family_cannibalization_says_consolidate() {
        let snapshot = snap(vec![
            row(
                "https://x.test/category/electrician/haifa",
                "electrician haifa",
                200,
                8,
                7.0,
                None,
            ),
            row(
                "https://x.test/category/electrician/tel-aviv",
                "electrician haifa",
                180,
                6,
                9.0,
                None,
            ),
        ]);
        let intel = analyze(&snapshot, &Inventory::blank(AnalysisMode::Site));
        let finding = intel
            .findings
            .iter()
            .find(|item| item.code == "WVX-SEO-CANN-001")
            .expect("cannibalization");
        assert!(
            finding.summary.contains("CONSOLIDATE"),
            "{}",
            finding.summary
        );
        assert!(
            intel
                .opportunities
                .iter()
                .any(|item| item.kind == "cannibalization")
        );
    }

    #[test]
    fn distinct_families_say_differentiate() {
        let snapshot = snap(vec![
            row(
                "https://x.test/category/electrician/haifa",
                "hire electrician",
                200,
                8,
                7.0,
                None,
            ),
            row(
                "https://x.test/guides/hire-electrician",
                "hire electrician",
                180,
                6,
                9.0,
                None,
            ),
        ]);
        let finding = analyze(&snapshot, &Inventory::blank(AnalysisMode::Site))
            .findings
            .into_iter()
            .find(|item| item.code == "WVX-SEO-CANN-001")
            .expect("cannibalization");
        assert!(
            finding.summary.contains("DIFFERENTIATE"),
            "{}",
            finding.summary
        );
    }
}

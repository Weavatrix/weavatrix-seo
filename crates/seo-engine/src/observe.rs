//! Attach GSC demand to opportunities and emit unmeasured URL observations.

use weavatrix_seo_model::{Finding, FindingFamily, Inventory, Locator, Opportunity, Severity};
use weavatrix_seo_observation::{ObservationKind, ObservationSnapshot, axes_for};

pub fn decorate(
    snapshot: &ObservationSnapshot,
    inventory: &Inventory,
    items: &mut [Opportunity],
) -> Vec<Finding> {
    for item in items.iter_mut() {
        let (demand, gap) = axes_for(snapshot, &item.subject);
        if demand.is_some() || gap.is_some() {
            item.axes.demand = demand;
            item.axes.visibility_gap = gap;
            if let Some(demand) = demand {
                item.demand = format!("impressions:{demand}");
            }
        }
        let mut impressions = 0_u32;
        let mut clicks = 0_u32;
        for row in snapshot
            .rows
            .iter()
            .filter(|row| row.kind == ObservationKind::SearchPerformance)
            .filter(|row| row.url.trim_end_matches('/') == item.subject.trim_end_matches('/'))
        {
            impressions = impressions.saturating_add(row.impressions);
            clicks = clicks.saturating_add(row.clicks);
        }
        if impressions > 0 {
            item.axes.raw_impressions = Some(impressions);
            item.axes.raw_clicks = Some(clicks);
            if let Some(gap) = item.axes.visibility_gap {
                item.axes.recoverable_clicks =
                    Some(u32::from(gap).saturating_mul(impressions.max(1) / 100));
            }
        }
        if item.kind == "create_family" {
            item.axes.difficulty_to_build = Some(70);
        }
        if item.kind == "content_gap" {
            item.axes.difficulty_to_build = Some(20);
        }
        if item.kind == "link_gap" {
            item.axes.graph_leverage = Some(80);
        }
    }
    if !snapshot.connected {
        return Vec::new();
    }
    let measured: Vec<String> = inventory.measured_urls();
    let mut findings = Vec::new();
    for row in &snapshot.rows {
        // Only measured search demand says a missing URL matters. Bot hits on an
        // uncrawled URL are a crawl-budget fact, not a search-coverage gap.
        if row.kind != ObservationKind::SearchPerformance || row.impressions < 50 {
            continue;
        }
        let known = measured
            .iter()
            .any(|url| url.trim_end_matches('/') == row.url.trim_end_matches('/'));
        if known {
            continue;
        }
        findings.push(
            Finding::new(
                FindingFamily::Obs,
                1,
                Severity::Info,
                &row.url,
                format!(
                    "GSC observes {} impressions for {} which was not in this crawl",
                    row.impressions, row.url
                ),
                Locator::Url(row.url.clone()),
                row.evidence.clone(),
            )
            .explained(
                "Search Console demand exists for a URL this snapshot did not measure.",
                "Raise the crawl budget or add the URL to the seed/sitemap.",
                "The URL is present in a later inventory or is intentionally excluded.",
            ),
        );
    }
    findings.extend(crawl_budget_waste(snapshot, inventory));
    findings
}

/// Ranking URLs that search still measures but bots barely hit, and the reverse.
fn crawl_budget_waste(snapshot: &ObservationSnapshot, inventory: &Inventory) -> Vec<Finding> {
    if !snapshot.has(ObservationKind::BotCrawl) || !snapshot.has(ObservationKind::SearchPerformance)
    {
        return Vec::new();
    }
    let mut findings = Vec::new();
    for row in snapshot
        .rows
        .iter()
        .filter(|row| row.kind == ObservationKind::SearchPerformance)
        .filter(|row| row.impressions >= 50)
    {
        let hits: u32 = snapshot
            .rows
            .iter()
            .filter(|item| item.kind == ObservationKind::BotCrawl)
            .filter(|item| item.url.trim_end_matches('/') == row.url.trim_end_matches('/'))
            .map(|item| item.hits)
            .sum();
        if hits > 0 {
            continue;
        }
        let known = inventory
            .measured_urls()
            .iter()
            .any(|url| url.trim_end_matches('/') == row.url.trim_end_matches('/'));
        if !known {
            continue;
        }
        findings.push(
            Finding::new(
                FindingFamily::Obs,
                2,
                Severity::Info,
                &row.url,
                format!(
                    "search demand exists for {} but no bot hits were imported",
                    row.url
                ),
                Locator::Url(row.url.clone()),
                row.evidence.clone(),
            )
            .explained(
                "Google still reports impressions, yet server logs show no crawler activity.",
                "Check robots, canonical, and internal links for this URL.",
                "A later log import shows crawler hits, or the URL is intentionally noindexed.",
            ),
        );
    }
    findings
}

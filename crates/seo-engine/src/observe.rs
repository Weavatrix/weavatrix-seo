//! Attach GSC demand to opportunities and emit unmeasured URL observations.

use weavatrix_seo_architecture::Architecture;
use weavatrix_seo_model::{
    Chunk, FamilyContent, Finding, FindingFamily, InputStateKind, Inventory, Locator, Opportunity,
    Severity,
};
use weavatrix_seo_observation::{
    ObservationKind, ObservationSnapshot, analyze_gsc, analyze_logs, axes_for, citation_drops,
    expected_ctr,
};

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub fn decorate(
    snapshot: &ObservationSnapshot,
    inventory: &Inventory,
    architecture: &Architecture,
    chunks: &[Chunk],
    families: &mut [FamilyContent],
    items: &mut Vec<Opportunity>,
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
            let position = snapshot
                .rows
                .iter()
                .filter(|row| row.kind == ObservationKind::SearchPerformance)
                .filter(|row| row.url.trim_end_matches('/') == item.subject.trim_end_matches('/'))
                .filter_map(|row| row.position)
                .min_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
            if let Some(position) = position {
                let rate = expected_ctr(position);
                item.axes.expected_ctr = Some(ctr_percent(rate));
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let expected_clicks = (f64::from(impressions) * f64::from(rate)).round() as u32;
                item.axes.recoverable_clicks = Some(expected_clicks.saturating_sub(clicks));
            } else if let Some(gap) = item.axes.visibility_gap {
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
    if snapshot.input.kind == InputStateKind::Invalid {
        return vec![
            Finding::from_rule(
                FindingFamily::Obs,
                3,
                &snapshot.input.label,
                format!(
                    "observation file is {}: {}",
                    snapshot.input.label,
                    snapshot.input.error.as_deref().unwrap_or("invalid")
                ),
                Locator::Url(snapshot.input.label.clone()),
                weavatrix_seo_model::Evidence::unmeasured(weavatrix_seo_model::EvidenceSource::Gsc),
            )
            .explained(
                "An unreadable evidence file is not the same as no file being supplied.",
                "Fix the JSON or pass a valid GSC/observations export.",
                "A later run reports GSC_CONNECTED or GSC_EMPTY, not GSC_INVALID.",
            ),
        ];
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
    findings.extend(analyze_logs(snapshot, inventory));
    findings.extend(citation_drops(snapshot));
    let intel = analyze_gsc(snapshot, inventory);
    findings.extend(intel.findings);
    items.extend(intel.opportunities);
    findings.extend(passage_gaps(snapshot, chunks));
    findings.extend(high_authority_without_demand(snapshot, architecture));
    rollup_families(snapshot, inventory, families);
    findings
}

fn ctr_percent(rate: f32) -> u16 {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let value = (rate * 100.0).round() as u16;
    value.min(100)
}

fn passage_gaps(snapshot: &ObservationSnapshot, chunks: &[Chunk]) -> Vec<Finding> {
    let mut findings = Vec::new();
    for row in snapshot
        .rows
        .iter()
        .filter(|row| row.kind == ObservationKind::SearchPerformance)
        .filter(|row| row.impressions >= 30)
        .filter(|row| row.query.as_ref().is_some_and(|query| query.len() > 3))
    {
        let query = row.query.as_deref().unwrap_or("");
        let tokens: Vec<String> = query
            .split(|ch: char| !ch.is_alphanumeric())
            .filter(|part| part.len() >= 3)
            .map(str::to_ascii_lowercase)
            .collect();
        if tokens.is_empty() {
            continue;
        }
        let mut best = 0_u16;
        let mut best_id = String::new();
        for chunk in chunks
            .iter()
            .filter(|chunk| chunk.url.trim_end_matches('/') == row.url.trim_end_matches('/'))
        {
            let hay = format!("{} {}", chunk.heading, chunk.text).to_ascii_lowercase();
            let hit = tokens
                .iter()
                .filter(|token| hay.contains(token.as_str()))
                .count();
            let score = u16::try_from(hit.saturating_mul(100) / tokens.len()).unwrap_or(0);
            if score > best {
                best = score;
                best_id.clone_from(&chunk.id);
            }
        }
        if best >= 40 {
            continue;
        }
        findings.push(
            Finding::from_rule(
                FindingFamily::Content,
                4,
                query,
                format!(
                    "{query} ranks on {} but no self-contained chunk answers it (best {} {best}%)",
                    row.url,
                    if best_id.is_empty() {
                        "none"
                    } else {
                        best_id.as_str()
                    }
                ),
                Locator::Url(row.url.clone()),
                row.evidence.clone(),
            )
            .explained(
                "The query has impressions, yet the page has no chunk that covers its tokens.",
                "Add a heading-bounded section that answers the query from first-party facts.",
                "A later retrieve returns a chunk with relevance at least 40.",
            ),
        );
    }
    findings
}

fn high_authority_without_demand(
    snapshot: &ObservationSnapshot,
    architecture: &Architecture,
) -> Vec<Finding> {
    if !snapshot.has(ObservationKind::SearchPerformance) {
        return Vec::new();
    }
    let mut scores: Vec<f64> = architecture
        .pages
        .iter()
        .map(|page| page.authority)
        .collect();
    scores.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    let median = scores.get(scores.len() / 2).copied().unwrap_or(0.0);
    let mut findings = Vec::new();
    for page in architecture
        .pages
        .iter()
        .filter(|page| page.authority > median && page.authority > 0.0)
    {
        let page_url = page.url.to_string();
        let impressions: u32 = snapshot
            .rows
            .iter()
            .filter(|row| row.kind == ObservationKind::SearchPerformance)
            .filter(|row| row.url.trim_end_matches('/') == page_url.trim_end_matches('/'))
            .map(|row| row.impressions)
            .sum();
        if impressions > 0 {
            continue;
        }
        findings.push(
            Finding::from_rule(
                FindingFamily::Link,
                4,
                &page.url.to_string(),
                format!(
                    "{} has high internal PageRank but no imported search impressions",
                    page.url
                ),
                Locator::url(&page.url),
                weavatrix_seo_model::Evidence::unmeasured(weavatrix_seo_model::EvidenceSource::Gsc),
            )
            .explained(
                "Internal authority is not the same as search demand.",
                "Target a query this page can uniquely answer, or use it as a donor of links.",
                "GSC impressions appear, or the page is intentionally a hub.",
            ),
        );
    }
    findings
}

fn rollup_families(
    snapshot: &ObservationSnapshot,
    inventory: &Inventory,
    families: &mut [FamilyContent],
) {
    for family in families.iter_mut() {
        let mut clicks = 0_u32;
        let mut impressions = 0_u32;
        for page in inventory
            .pages
            .iter()
            .filter(|page| page.url.path().contains(&family.family))
        {
            for row in snapshot
                .rows
                .iter()
                .filter(|row| row.kind == ObservationKind::SearchPerformance)
                .filter(|row| {
                    row.url.trim_end_matches('/') == page.url.to_string().trim_end_matches('/')
                })
            {
                clicks = clicks.saturating_add(row.clicks);
                impressions = impressions.saturating_add(row.impressions);
            }
        }
        if clicks > 0 {
            family.gsc_clicks = Some(clicks);
        }
        if impressions > 0 {
            family.gsc_impressions = Some(impressions);
        }
    }
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

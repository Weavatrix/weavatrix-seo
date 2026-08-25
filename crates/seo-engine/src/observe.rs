//! Attach GSC demand to opportunities and emit unmeasured URL observations.

use weavatrix_seo_model::{Finding, FindingFamily, Inventory, Locator, Opportunity, Severity};
use weavatrix_seo_observation::{ObservationSnapshot, axes_for};

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
        if row.impressions < 50 {
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
    findings
}

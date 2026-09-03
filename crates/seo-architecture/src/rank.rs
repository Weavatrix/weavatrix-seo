//! Weighted internal `PageRank`. Chrome links do not count as body links.

use std::collections::BTreeMap;
use weavatrix_seo_model::{AbsoluteUrl, Inventory, LinkLocation, Relation};

const DAMPING: f64 = 0.85;
const ITERATIONS: usize = 20;

/// Body 1.0, nav 0.3, footer 0.15. Repeated chrome is down-weighted further.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn weighted(inventory: &Inventory) -> BTreeMap<AbsoluteUrl, f64> {
    let mut ids: Vec<String> = inventory
        .pages
        .iter()
        .map(|page| page.url.to_string())
        .collect();
    for edge in inventory
        .edges
        .iter()
        .filter(|edge| edge.relation == Relation::LinksTo)
    {
        ids.push(edge.source.to_string());
        ids.push(edge.target.to_string());
    }
    ids.sort();
    ids.dedup();
    if ids.is_empty() {
        return BTreeMap::new();
    }
    let index: BTreeMap<&str, usize> = ids
        .iter()
        .enumerate()
        .map(|(idx, id)| (id.as_str(), idx))
        .collect();
    let n = ids.len();
    let mut outgoing = vec![0.0_f64; n];
    let mut inbound: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
    for edge in inventory
        .edges
        .iter()
        .filter(|edge| edge.relation == Relation::LinksTo)
    {
        let source = edge.source.to_string();
        let target = edge.target.to_string();
        let Some(&from) = index.get(source.as_str()) else {
            continue;
        };
        let Some(&to) = index.get(target.as_str()) else {
            continue;
        };
        let weight = link_weight(edge.location, edge.template_frequency);
        outgoing[from] += weight;
        inbound[to].push((from, weight));
    }
    let mut rank = vec![1.0 / n as f64; n];
    for _ in 0..ITERATIONS {
        let mut next = vec![(1.0 - DAMPING) / n as f64; n];
        let mut dangling = 0.0;
        for (idx, mass) in rank.iter().enumerate() {
            if outgoing[idx] <= f64::EPSILON {
                dangling += *mass;
            }
        }
        let share = DAMPING * dangling / n as f64;
        for item in &mut next {
            *item += share;
        }
        for (target, sources) in inbound.iter().enumerate() {
            let mut incoming = 0.0;
            for (source, weight) in sources {
                if outgoing[*source] > f64::EPSILON {
                    incoming += rank[*source] * (*weight / outgoing[*source]);
                }
            }
            next[target] += DAMPING * incoming;
        }
        rank = next;
    }
    let mut out = BTreeMap::new();
    for (idx, id) in ids.iter().enumerate() {
        if let Ok(url) = AbsoluteUrl::parse(id) {
            out.insert(url, rank[idx]);
        }
    }
    out
}

fn link_weight(location: Option<LinkLocation>, template_frequency: Option<u32>) -> f64 {
    let base = match location.unwrap_or(LinkLocation::Contextual) {
        LinkLocation::Contextual | LinkLocation::Breadcrumb => 1.0,
        LinkLocation::Header => 0.4,
        LinkLocation::Nav => 0.3,
        LinkLocation::Footer => 0.15,
    };
    if template_frequency.unwrap_or(0) >= 3 {
        base * 0.25
    } else {
        base
    }
}

#[cfg(test)]
mod tests {
    use super::link_weight;
    use weavatrix_seo_model::LinkLocation;

    #[test]
    fn body_links_outweigh_footer_chrome() {
        assert!(
            link_weight(Some(LinkLocation::Contextual), None)
                > link_weight(Some(LinkLocation::Footer), None)
        );
        assert!(
            link_weight(Some(LinkLocation::Nav), None)
                > link_weight(Some(LinkLocation::Footer), None)
        );
        assert!(
            link_weight(Some(LinkLocation::Contextual), None)
                > link_weight(Some(LinkLocation::Nav), Some(12))
        );
    }
}

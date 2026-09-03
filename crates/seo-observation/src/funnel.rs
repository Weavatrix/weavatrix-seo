//! AI search funnel: discovery → citation fetch → referral.

use crate::{ObservationKind, ObservationSnapshot};
use std::collections::BTreeMap;
use weavatrix_seo_model::{AiFunnel, Finding, FindingFamily, Locator};

/// Rolls discovery, citation, and referral observations into per-URL funnels.
#[must_use]
pub fn analyze(snapshot: &ObservationSnapshot) -> (Vec<AiFunnel>, Vec<Finding>) {
    if !snapshot.connected {
        return (Vec::new(), Vec::new());
    }
    let mut by_url: BTreeMap<String, Acc> = BTreeMap::new();
    for row in &snapshot.rows {
        let entry = by_url.entry(row.url.clone()).or_default();
        match row.kind {
            ObservationKind::BotCrawl if row.bot_role.as_deref() == Some("search_discovery") => {
                entry.discovery = entry.discovery.saturating_add(row.hits.max(1));
            }
            ObservationKind::BotCrawl if row.bot_role.as_deref() == Some("citation_fetch") => {
                entry.citations = entry.citations.saturating_add(row.hits.max(1));
            }
            ObservationKind::AiCitation => {
                entry.citations = entry.citations.saturating_add(1);
            }
            ObservationKind::AiReferral => {
                entry.referrals = entry
                    .referrals
                    .saturating_add(row.hits.max(row.clicks).max(1));
            }
            _ => {}
        }
    }
    let mut funnels = Vec::new();
    let mut findings = Vec::new();
    for (url, acc) in by_url {
        if acc.discovery == 0 && acc.citations == 0 && acc.referrals == 0 {
            continue;
        }
        let citation_rate = rate(acc.citations, acc.discovery);
        let click_through = rate(acc.referrals, acc.citations);
        if acc.discovery >= 5 && acc.citations == 0 {
            if let Some(row) = snapshot.rows.iter().find(|row| row.url == url) {
                findings.push(
                    Finding::from_rule(
                        FindingFamily::Obs,
                        11,
                        &url,
                        format!(
                            "AI search bots discovered {url} {} times with no citation fetch",
                            acc.discovery
                        ),
                        Locator::Url(url.clone()),
                        row.evidence.clone(),
                    )
                    .explained(
                        "Discovery without citation means the page is seen but not used as an answer.",
                        "Add a self-contained chunk that answers the query with first-party facts.",
                        "A later import shows citation hits, or discovery stops.",
                    ),
                );
            }
        }
        funnels.push(AiFunnel {
            url,
            discovery_hits: nonzero(acc.discovery),
            citation_hits: nonzero(acc.citations),
            referrals: nonzero(acc.referrals),
            citation_rate,
            click_through,
            family: None,
            producer: None,
        });
    }
    (funnels, findings)
}

#[derive(Default)]
struct Acc {
    discovery: u32,
    citations: u32,
    referrals: u32,
}

fn nonzero(value: u32) -> Option<u32> {
    (value > 0).then_some(value)
}

fn rate(part: u32, whole: u32) -> Option<u16> {
    if whole == 0 {
        return None;
    }
    let scaled = u32::from(100_u16).saturating_mul(part) / whole;
    Some(u16::try_from(scaled).unwrap_or(100).min(100))
}

#[cfg(test)]
mod tests {
    use super::analyze;
    use crate::{Observation, ObservationKind, ObservationSnapshot};
    use weavatrix_seo_model::{Evidence, EvidenceKind, EvidenceSource, InputState};

    fn bot(url: &str, role: &str, hits: u32) -> Observation {
        Observation {
            kind: ObservationKind::BotCrawl,
            query: None,
            url: url.into(),
            provider: "nginx".into(),
            evidence: Evidence {
                kind: EvidenceKind::Observed,
                source: EvidenceSource::Logs,
                confidence: weavatrix_seo_model::Confidence::High,
                snapshot_id: None,
                revision: None,
                policy_version: None,
            },
            clicks: 0,
            impressions: 0,
            hits,
            position: None,
            period: None,
            user_agent: Some("OAI-SearchBot/1.0".into()),
            status: Some(200),
            bot_role: Some(role.into()),
            verified_bot: Some(true),
            referer: None,
        }
    }

    #[test]
    fn discovery_without_citation_is_a_gap() {
        let snapshot = ObservationSnapshot {
            rows: vec![bot("https://x.test/a", "search_discovery", 8)],
            connected: true,
            input: InputState::connected("logs"),
        };
        let (funnels, findings) = analyze(&snapshot);
        assert_eq!(funnels[0].discovery_hits, Some(8));
        assert!(funnels[0].citation_hits.is_none());
        assert!(findings.iter().any(|item| item.code == "WVX-SEO-OBS-011"));
    }
}

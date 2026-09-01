//! Provider observation contracts. Imports only; no vendor crawlers.

#![forbid(unsafe_code)]

mod gsc;
mod kind;
mod outcome;
mod provider;

use serde::{Deserialize, Serialize};
use weavatrix_seo_model::{Evidence, EvidenceSource};

pub use gsc::{disconnected, from_json, load};
pub use kind::ObservationKind;
pub use outcome::metrics as outcome_metrics;
pub use provider::{from_any, load_any};

/// One provider observation.
///
/// The kind decides what the counters mean. Nothing here is interchangeable:
/// crawler activity is not search demand, and a generative-search citation is
/// not a SERP position.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Observation {
    /// What this row measured.
    pub kind: ObservationKind,
    /// Query text when known.
    pub query: Option<String>,
    /// URL.
    pub url: String,
    /// Provider name.
    pub provider: String,
    /// Evidence. Never promoted to deterministic.
    pub evidence: Evidence,
    /// Clicks when the provider supplied them.
    #[serde(default)]
    pub clicks: u32,
    /// Search impressions. Only a `search_performance` row can have these.
    #[serde(default)]
    pub impressions: u32,
    /// Bot, crawler, or analytics requests. Never search impressions.
    #[serde(default)]
    pub hits: u32,
    /// Average position when known. Search Console reports fractions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<f32>,
}

/// Snapshot of imported observations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservationSnapshot {
    /// Rows.
    pub rows: Vec<Observation>,
    /// Whether any provider was connected.
    pub connected: bool,
}

impl ObservationSnapshot {
    /// Whether any row of this kind was imported.
    #[must_use]
    pub fn has(&self, kind: ObservationKind) -> bool {
        self.connected && self.rows.iter().any(|row| row.kind == kind)
    }
}

/// No provider is connected.
#[must_use]
pub fn unmeasured() -> ObservationSnapshot {
    ObservationSnapshot {
        rows: Vec::new(),
        connected: false,
    }
}

/// Evidence for an unconnected provider.
#[must_use]
pub fn unmeasured_evidence() -> Evidence {
    Evidence::unmeasured(EvidenceSource::Provider)
}

/// Demand and visibility gap for one URL.
///
/// Only `search_performance` rows count. Bot hits, analytics sessions, and AI
/// citations are real observations, but treating them as search demand would
/// let crawler traffic promote a page up the opportunity list.
#[must_use]
pub fn axes_for(snapshot: &ObservationSnapshot, url: &str) -> (Option<u16>, Option<u16>) {
    if !snapshot.connected {
        return (None, None);
    }
    let mut impressions = 0_u32;
    let mut best: Option<f32> = None;
    for row in snapshot
        .rows
        .iter()
        .filter(|row| row.kind == ObservationKind::SearchPerformance)
        .filter(|row| urls_match(&row.url, url))
    {
        impressions = impressions.saturating_add(row.impressions);
        if let Some(position) = row.position.filter(|value| *value > 0.0) {
            best = Some(best.map_or(position, |current: f32| current.min(position)));
        }
    }
    if impressions == 0 && best.is_none() {
        return (None, None);
    }
    let demand = u16::try_from((impressions / 10).min(100)).unwrap_or(100);
    let gap = best.map_or(0, |position| {
        if position > 10.0 {
            // Round rather than truncate: `12.4_f32` is 12.39999..., and
            // truncating would report a smaller gap than the provider measured.
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let scaled = ((position - 10.0) * 5.0).min(100.0).round() as u16;
            scaled
        } else {
            0
        }
    });
    (Some(demand), Some(gap))
}

fn urls_match(left: &str, right: &str) -> bool {
    left.trim_end_matches('/') == right.trim_end_matches('/')
}

#[cfg(test)]
mod tests {
    use super::{ObservationKind, ObservationSnapshot, axes_for, from_any};

    fn snapshot(raw: &str) -> ObservationSnapshot {
        from_any(raw).expect("parse")
    }

    #[test]
    fn bot_hits_are_not_search_demand() {
        let logs =
            snapshot(r#"{"provider":"logs","rows":[{"url":"https://x.test/a","hits":500}]}"#);
        assert_eq!(logs.rows[0].kind, ObservationKind::BotCrawl);
        assert_eq!(logs.rows[0].hits, 500);
        assert_eq!(logs.rows[0].impressions, 0);
        assert_eq!(axes_for(&logs, "https://x.test/a"), (None, None));
    }

    #[test]
    fn search_performance_still_ranks() {
        let gsc = snapshot(
            r#"{"provider":"gsc","rows":[{"url":"https://x.test/a","impressions":500,"position":12.4}]}"#,
        );
        let (demand, gap) = axes_for(&gsc, "https://x.test/a");
        assert_eq!(demand, Some(50));
        assert_eq!(gap, Some(12));
    }

    #[test]
    fn fractional_position_is_kept() {
        let gsc = snapshot(
            r#"{"provider":"gsc","rows":[{"url":"https://x.test/a","impressions":1,"position":12.4}]}"#,
        );
        assert_eq!(gsc.rows[0].position, Some(12.4));
    }

    #[test]
    fn ai_citations_are_their_own_kind() {
        let ai = snapshot(
            r#"{"provider":"perplexity","rows":[{"url":"https://x.test/a","query":"best electrician"}]}"#,
        );
        assert_eq!(ai.rows[0].kind, ObservationKind::AiCitation);
        assert!(ai.has(ObservationKind::AiCitation));
        assert!(!ai.has(ObservationKind::SearchPerformance));
        assert_eq!(axes_for(&ai, "https://x.test/a"), (None, None));
    }

    #[test]
    fn an_unknown_provider_does_not_become_search_demand() {
        let other = snapshot(
            r#"{"provider":"mystery","rows":[{"url":"https://x.test/a","impressions":900}]}"#,
        );
        assert_eq!(other.rows[0].kind, ObservationKind::Analytics);
        assert_eq!(axes_for(&other, "https://x.test/a"), (None, None));
    }

    #[test]
    fn an_explicit_kind_wins_over_the_provider_name() {
        let declared = snapshot(
            r#"{"provider":"mystery","rows":[{"url":"https://x.test/a","kind":"search_performance","impressions":900}]}"#,
        );
        assert_eq!(declared.rows[0].kind, ObservationKind::SearchPerformance);
        assert_eq!(axes_for(&declared, "https://x.test/a").0, Some(90));
    }
}

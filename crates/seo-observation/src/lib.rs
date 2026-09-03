//! Provider observation contracts. Imports only; no vendor crawlers.

#![forbid(unsafe_code)]

mod funnel;
mod gsc;
mod intel;
mod kind;
mod logs;
mod outcome;
mod prompts;
mod provider;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use weavatrix_seo_model::{Evidence, EvidenceSource, InputState, UrlMetric};

pub use funnel::analyze as analyze_funnel;
pub use gsc::{disconnected, from_json, load};
pub use intel::{ObservationIntel, analyze as analyze_gsc, expected_ctr};
pub use kind::ObservationKind;
pub use logs::{analyze as analyze_logs, classify_agent, from_combined};
pub use outcome::metrics as outcome_metrics;
pub use prompts::citation_drops;
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
    /// Window tag: `current`, `previous`, or a provider label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub period: Option<String>,
    /// User-Agent when a log row supplied it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    /// HTTP status from a log row.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
    /// Classified bot role: `search_discovery`, `citation_fetch`, `training`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bot_role: Option<String>,
    /// True when the UA matched a documented crawler token.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verified_bot: Option<bool>,
    /// Referer when a log row supplied it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub referer: Option<String>,
}

/// Snapshot of imported observations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservationSnapshot {
    /// Rows.
    pub rows: Vec<Observation>,
    /// Whether any provider was connected.
    pub connected: bool,
    /// How the file was resolved. Invalid is never treated as absence.
    #[serde(default)]
    pub input: InputState,
    /// Imported AI-visibility prompts. Additive; older files omit it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub prompts: Vec<weavatrix_seo_model::PromptObservation>,
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
        input: InputState::absent("GSC"),
        prompts: Vec::new(),
    }
}

/// Loads GSC or a generic observation file without turning parse errors into absence.
#[must_use]
pub fn load_state(path: Option<&str>, prefix: &str) -> ObservationSnapshot {
    let Some(path) = path else {
        return ObservationSnapshot {
            rows: Vec::new(),
            connected: false,
            input: InputState::absent(prefix),
            prompts: Vec::new(),
        };
    };
    match load_any(path) {
        Ok(mut snapshot) => {
            snapshot.input = if snapshot.rows.is_empty() && snapshot.prompts.is_empty() {
                InputState::empty(prefix)
            } else {
                InputState::connected(prefix)
            };
            snapshot.connected = true;
            snapshot
        }
        Err(error) => ObservationSnapshot {
            rows: Vec::new(),
            connected: false,
            input: InputState::invalid(prefix, format!("{path}: {error}")),
            prompts: Vec::new(),
        },
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

/// Rolls imported GSC and AI-citation rows up per URL.
#[must_use]
pub fn url_metrics(snapshot: &ObservationSnapshot) -> Vec<UrlMetric> {
    if !snapshot.connected {
        return Vec::new();
    }
    let mut by_url: BTreeMap<String, UrlMetric> = BTreeMap::new();
    for row in &snapshot.rows {
        let entry = by_url.entry(row.url.clone()).or_insert_with(|| UrlMetric {
            url: row.url.clone(),
            gsc_clicks: None,
            gsc_impressions: None,
            citations: None,
        });
        match row.kind {
            ObservationKind::SearchPerformance => {
                entry.gsc_clicks = Some(entry.gsc_clicks.unwrap_or(0).saturating_add(row.clicks));
                entry.gsc_impressions = Some(
                    entry
                        .gsc_impressions
                        .unwrap_or(0)
                        .saturating_add(row.impressions),
                );
            }
            ObservationKind::AiCitation => {
                entry.citations = Some(entry.citations.unwrap_or(0).saturating_add(1));
            }
            _ => {}
        }
    }
    by_url.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::{ObservationKind, ObservationSnapshot, axes_for, from_any, url_metrics};

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

    #[test]
    fn url_metrics_roll_up_gsc_and_citations() {
        let snap = snapshot(
            r#"{"rows":[
                {"url":"https://x.test/a","kind":"search_performance","clicks":4,"impressions":80},
                {"url":"https://x.test/a","kind":"ai_citation","query":"best electrician"}
            ]}"#,
        );
        let metrics = url_metrics(&snap);
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].gsc_clicks, Some(4));
        assert_eq!(metrics[0].gsc_impressions, Some(80));
        assert_eq!(metrics[0].citations, Some(1));
    }
}

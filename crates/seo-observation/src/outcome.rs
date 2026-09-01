//! Observed AI and search outcome metrics. Parallel to findings; never error/warn/info.

use crate::{ObservationKind, ObservationSnapshot};
use weavatrix_seo_model::OutcomeMetric;

/// Outcome metrics derived from an observation snapshot.
///
/// Missing providers stay [`OutcomeMetric::unmeasured`]. Zero is never used as
/// a stand-in for "we did not look".
#[must_use]
pub fn metrics(snapshot: &ObservationSnapshot) -> Vec<OutcomeMetric> {
    let mut out = vec![
        rate(
            snapshot,
            "citation_rate",
            ObservationKind::AiCitation,
            "ai-search",
        ),
        rate(
            snapshot,
            "mention_rate",
            ObservationKind::AiCitation,
            "ai-search",
        ),
    ];
    let citations: Vec<_> = snapshot
        .rows
        .iter()
        .filter(|row| row.kind == ObservationKind::AiCitation)
        .collect();
    if citations.is_empty() {
        out.push(OutcomeMetric::unmeasured("share_of_voice", "ai-search"));
        out.push(OutcomeMetric::unmeasured("prompts_observed", "ai-search"));
        out.push(OutcomeMetric::unmeasured("cited_urls", "ai-search"));
    } else {
        let mut urls: Vec<&str> = citations.iter().map(|row| row.url.as_str()).collect();
        urls.sort_unstable();
        urls.dedup();
        let mut prompts: Vec<&str> = citations
            .iter()
            .filter_map(|row| row.query.as_deref())
            .collect();
        prompts.sort_unstable();
        prompts.dedup();
        out.push(OutcomeMetric {
            name: "prompts_observed".into(),
            value: Some(prompts.len().to_string()),
            numerator: Some(u64::try_from(prompts.len()).unwrap_or(0)),
            denominator: None,
            window: None,
            source: "ai-search".into(),
            confidence: "high".into(),
        });
        out.push(OutcomeMetric {
            name: "cited_urls".into(),
            value: Some(urls.len().to_string()),
            numerator: Some(u64::try_from(urls.len()).unwrap_or(0)),
            denominator: None,
            window: None,
            source: "ai-search".into(),
            confidence: "high".into(),
        });
        out.push(OutcomeMetric {
            name: "share_of_voice".into(),
            value: None,
            numerator: Some(u64::try_from(citations.len()).unwrap_or(0)),
            denominator: None,
            window: None,
            source: "ai-search".into(),
            confidence: "low".into(),
        });
    }
    if snapshot.has(ObservationKind::SearchPerformance) {
        let clicks: u64 = snapshot
            .rows
            .iter()
            .filter(|row| row.kind == ObservationKind::SearchPerformance)
            .map(|row| u64::from(row.clicks))
            .sum();
        let impressions: u64 = snapshot
            .rows
            .iter()
            .filter(|row| row.kind == ObservationKind::SearchPerformance)
            .map(|row| u64::from(row.impressions))
            .sum();
        out.push(OutcomeMetric {
            name: "search_clicks".into(),
            value: Some(clicks.to_string()),
            numerator: Some(clicks),
            denominator: Some(impressions),
            window: None,
            source: "gsc".into(),
            confidence: "high".into(),
        });
    } else {
        out.push(OutcomeMetric::unmeasured("search_clicks", "gsc"));
    }
    out
}

fn rate(
    snapshot: &ObservationSnapshot,
    name: &str,
    kind: ObservationKind,
    source: &str,
) -> OutcomeMetric {
    if !snapshot.connected || !snapshot.has(kind) {
        return OutcomeMetric::unmeasured(name, source);
    }
    let n = snapshot.rows.iter().filter(|row| row.kind == kind).count();
    OutcomeMetric {
        name: name.into(),
        value: Some(n.to_string()),
        numerator: Some(u64::try_from(n).unwrap_or(0)),
        denominator: None,
        window: None,
        source: source.into(),
        confidence: "high".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::metrics;
    use crate::{ObservationKind, from_any};

    #[test]
    fn missing_provider_stays_unmeasured() {
        let snapshot = crate::unmeasured();
        let citation = metrics(&snapshot)
            .into_iter()
            .find(|item| item.name == "citation_rate")
            .expect("metric");
        assert!(citation.value.is_none());
        assert_eq!(citation.confidence, "unmeasured");
    }

    #[test]
    fn ai_citations_count_prompts() {
        let snapshot = from_any(
            r#"{"provider":"perplexity","rows":[{"url":"https://x.test/a","query":"best electrician"}]}"#,
        )
        .expect("parse");
        assert_eq!(snapshot.rows[0].kind, ObservationKind::AiCitation);
        let prompts = metrics(&snapshot)
            .into_iter()
            .find(|item| item.name == "prompts_observed")
            .expect("prompts");
        assert_eq!(prompts.value.as_deref(), Some("1"));
    }
}

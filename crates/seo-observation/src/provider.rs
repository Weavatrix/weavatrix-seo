//! Observation providers. File imports only; no vendor crawlers.

use crate::{Observation, ObservationKind, ObservationSnapshot};
use serde::Deserialize;
use weavatrix_seo_model::{Evidence, EvidenceKind};

#[derive(Debug, Deserialize)]
struct File {
    #[serde(default)]
    provider: Option<String>,
    /// Default kind for every row in the file.
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    rows: Vec<Row>,
}

#[derive(Debug, Deserialize)]
struct Row {
    #[serde(default)]
    query: Option<String>,
    url: String,
    #[serde(default)]
    clicks: u32,
    #[serde(default)]
    impressions: u32,
    #[serde(default)]
    hits: u32,
    #[serde(default)]
    position: Option<f32>,
    #[serde(default)]
    provider: Option<String>,
    /// Row-level kind. Wins over the file and the provider name.
    #[serde(default)]
    kind: Option<String>,
}

/// Loads GSC, Bing, bot-log, analytics, or AI-citation JSON.
///
/// An unknown provider stays labelled and is never faked as search demand.
///
/// # Errors
///
/// Returns IO or JSON errors.
pub fn load_any(path: &str) -> Result<ObservationSnapshot, String> {
    let raw = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    from_any(&raw)
}

/// Parses a provider export in memory.
///
/// # Errors
///
/// Returns JSON errors.
pub fn from_any(raw: &str) -> Result<ObservationSnapshot, String> {
    let file: File = blazingly_json::from_str(raw).map_err(|error| error.to_string())?;
    let default_provider = file
        .provider
        .as_deref()
        .unwrap_or("gsc")
        .to_ascii_lowercase();
    let file_kind = file.kind.as_deref().and_then(ObservationKind::parse);
    let rows = file
        .rows
        .into_iter()
        .map(|row| {
            let provider = row
                .provider
                .clone()
                .unwrap_or_else(|| default_provider.clone())
                .to_ascii_lowercase();
            let kind = row
                .kind
                .as_deref()
                .and_then(ObservationKind::parse)
                .or(file_kind)
                .unwrap_or_else(|| ObservationKind::from_provider(&provider));
            let source = kind.source(&provider);
            Observation {
                kind,
                query: row.query,
                url: row.url,
                provider,
                evidence: Evidence {
                    kind: EvidenceKind::Observed,
                    source,
                    confidence: weavatrix_seo_model::Confidence::High,
                    snapshot_id: None,
                    revision: None,
                    policy_version: None,
                },
                clicks: row.clicks,
                impressions: if kind.is_search_demand() {
                    row.impressions
                } else {
                    0
                },
                hits: if kind.is_search_demand() {
                    0
                } else {
                    row.hits.max(row.impressions)
                },
                position: row.position,
            }
        })
        .collect();
    Ok(ObservationSnapshot {
        rows,
        connected: true,
    })
}

#[cfg(test)]
mod tests {
    use super::from_any;
    use crate::ObservationKind;
    use weavatrix_seo_model::EvidenceSource;

    #[test]
    fn bing_is_search_performance_and_keeps_its_provider() {
        let bing =
            from_any(r#"{"provider":"bing","rows":[{"url":"https://x.test/","impressions":9}]}"#)
                .expect("bing");
        assert_eq!(bing.rows[0].provider, "bing");
        assert_eq!(bing.rows[0].kind, ObservationKind::SearchPerformance);
        assert_eq!(bing.rows[0].impressions, 9);
        assert_eq!(bing.rows[0].evidence.source, EvidenceSource::Provider);
    }

    #[test]
    fn log_hits_stay_hits() {
        let logs = from_any(r#"{"provider":"logs","rows":[{"url":"https://x.test/","hits":40}]}"#)
            .expect("logs");
        assert_eq!(logs.rows[0].kind, ObservationKind::BotCrawl);
        assert_eq!(logs.rows[0].hits, 40);
        assert_eq!(
            logs.rows[0].impressions, 0,
            "crawler activity is not a search impression"
        );
        assert_eq!(logs.rows[0].evidence.source, EvidenceSource::Logs);
    }

    #[test]
    fn a_log_row_labelled_as_impressions_is_still_a_hit() {
        let logs =
            from_any(r#"{"provider":"logs","rows":[{"url":"https://x.test/","impressions":40}]}"#)
                .expect("logs");
        assert_eq!(logs.rows[0].hits, 40);
        assert_eq!(logs.rows[0].impressions, 0);
    }
}

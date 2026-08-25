//! Observation providers. File imports only; no vendor crawlers.

use crate::{Observation, ObservationSnapshot};
use serde::Deserialize;
use weavatrix_seo_model::{Evidence, EvidenceKind, EvidenceSource};

#[derive(Debug, Deserialize)]
struct File {
    #[serde(default)]
    provider: Option<String>,
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
    position: u32,
    #[serde(default)]
    provider: Option<String>,
}

/// Loads GSC, Bing, or bot-log JSON. Unknown provider stays labelled, never faked as GSC.
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
    let rows = file
        .rows
        .into_iter()
        .map(|row| {
            let provider = row
                .provider
                .clone()
                .unwrap_or_else(|| default_provider.clone());
            let source = source_of(&provider);
            let impressions = if row.impressions > 0 {
                row.impressions
            } else {
                row.hits
            };
            Observation {
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
                impressions,
                position: row.position,
            }
        })
        .collect();
    Ok(ObservationSnapshot {
        rows,
        connected: true,
    })
}

fn source_of(provider: &str) -> EvidenceSource {
    match provider {
        "gsc" | "search-console" => EvidenceSource::Gsc,
        "logs" | "cdn" | "bot-logs" => EvidenceSource::Logs,
        _ => EvidenceSource::Provider,
    }
}

#[cfg(test)]
mod tests {
    use super::from_any;
    use weavatrix_seo_model::EvidenceSource;

    #[test]
    fn bing_and_logs_keep_their_provider() {
        let bing =
            from_any(r#"{"provider":"bing","rows":[{"url":"https://x.test/","impressions":9}]}"#)
                .expect("bing");
        assert_eq!(bing.rows[0].provider, "bing");
        assert_eq!(bing.rows[0].evidence.source, EvidenceSource::Provider);
        let logs = from_any(r#"{"provider":"logs","rows":[{"url":"https://x.test/","hits":40}]}"#)
            .expect("logs");
        assert_eq!(logs.rows[0].impressions, 40);
        assert_eq!(logs.rows[0].evidence.source, EvidenceSource::Logs);
    }
}

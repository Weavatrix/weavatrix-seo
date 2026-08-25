//! First-party GSC import. No Google API client.

use crate::{Observation, ObservationSnapshot};
use serde::Deserialize;
use weavatrix_seo_model::{Evidence, EvidenceKind, EvidenceSource};

#[derive(Debug, Deserialize)]
struct GscFile {
    #[serde(default)]
    rows: Vec<GscRow>,
}

#[derive(Debug, Deserialize)]
struct GscRow {
    #[serde(default)]
    query: Option<String>,
    url: String,
    #[serde(default)]
    clicks: u32,
    #[serde(default)]
    impressions: u32,
    #[serde(default)]
    position: u32,
}

/// Loads a compact GSC export. Absence of a file is unmeasured, not a pass.
///
/// # Errors
///
/// Returns IO or JSON errors.
pub fn load(path: &str) -> Result<ObservationSnapshot, String> {
    let raw = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    from_json(&raw)
}

/// Parses GSC JSON already in memory.
///
/// # Errors
///
/// Returns JSON errors.
pub fn from_json(raw: &str) -> Result<ObservationSnapshot, String> {
    let file: GscFile = blazingly_json::from_str(raw).map_err(|error| error.to_string())?;
    let evidence = Evidence {
        kind: EvidenceKind::Observed,
        source: EvidenceSource::Gsc,
        confidence: weavatrix_seo_model::Confidence::High,
        snapshot_id: None,
        revision: None,
        policy_version: None,
    };
    let rows = file
        .rows
        .into_iter()
        .map(|row| Observation {
            query: row.query,
            url: row.url,
            provider: "gsc".into(),
            evidence: evidence.clone(),
            clicks: row.clicks,
            impressions: row.impressions,
            position: row.position,
        })
        .collect();
    Ok(ObservationSnapshot {
        rows,
        connected: true,
    })
}

/// Empty snapshot used when no export was supplied.
#[must_use]
pub fn disconnected() -> ObservationSnapshot {
    ObservationSnapshot {
        rows: Vec::new(),
        connected: false,
    }
}

#[cfg(test)]
mod tests {
    use super::from_json;

    #[test]
    fn parses_gsc_rows() {
        let snapshot = from_json(
            r#"{"rows":[{"query":"electrician vancouver","url":"https://x.test/","clicks":4,"impressions":200,"position":12}]}"#,
        )
        .expect("json");
        assert!(snapshot.connected);
        assert_eq!(snapshot.rows[0].impressions, 200);
        assert_eq!(snapshot.rows[0].provider, "gsc");
    }
}

//! First-party GSC import. No Google API client.

use crate::{Observation, ObservationKind, ObservationSnapshot};
use serde::Deserialize;
use weavatrix_seo_model::{Evidence, EvidenceKind, EvidenceSource, InputState};

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
    /// Search Console reports a fractional average position.
    #[serde(default)]
    position: Option<f32>,
    #[serde(default)]
    period: Option<String>,
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
    let file: GscFile = blazingly_json::from_str(weavatrix_seo_model::strip_bom(raw))
        .map_err(|error| error.to_string())?;
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
            kind: ObservationKind::SearchPerformance,
            query: row.query,
            url: row.url,
            provider: "gsc".into(),
            evidence: evidence.clone(),
            clicks: row.clicks,
            impressions: row.impressions,
            hits: 0,
            position: row.position,
            period: row.period,
            user_agent: None,
            status: None,
            bot_role: None,
            verified_bot: None,
            referer: None,
            volume: 0,
            difficulty: None,
            serp_features: Vec::new(),
            referring_domains: None,
        })
        .collect::<Vec<_>>();
    let input = if rows.is_empty() {
        InputState::empty("GSC")
    } else {
        InputState::connected("GSC")
    };
    Ok(ObservationSnapshot {
        rows,
        connected: true,
        input,
        prompts: Vec::new(),
    })
}

/// Empty snapshot used when no export was supplied.
#[must_use]
pub fn disconnected() -> ObservationSnapshot {
    ObservationSnapshot {
        rows: Vec::new(),
        connected: false,
        input: InputState::absent("GSC"),
        prompts: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::from_json;

    #[test]
    fn a_byte_order_mark_does_not_break_an_import() {
        let snapshot =
            from_json("\u{feff}{\"rows\":[{\"url\":\"https://x.test/\",\"impressions\":5}]}")
                .expect("a mark is not a syntax error");
        assert_eq!(snapshot.rows[0].impressions, 5);
    }

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

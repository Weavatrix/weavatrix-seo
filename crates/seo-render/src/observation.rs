//! Rendered-DOM facts imported from WVQ / Playwright. No browser lives here.

use serde::{Deserialize, Serialize};
use weavatrix_seo_model::{Evidence, EvidenceKind, EvidenceSource};

/// One URL as observed after render/hydration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderedPage {
    /// Final URL.
    pub url: String,
    /// `document.title` after render.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Canonical href after render.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical: Option<String>,
    /// First visible H1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub h1: Option<String>,
    /// Meta description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JSON-LD `@type` values after render.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub json_ld_types: Vec<String>,
    /// `html[lang]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_lang: Option<String>,
}

/// Snapshot of rendered pages. Produced by WVQ/Playwright, consumed here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderSnapshot {
    /// Artifact schema.
    #[serde(default)]
    pub schema: String,
    /// Producer name (`wvq`, `playwright`, …).
    #[serde(default)]
    pub source: String,
    /// Rendered pages.
    #[serde(default)]
    pub pages: Vec<RenderedPage>,
}

impl RenderSnapshot {
    /// True when at least one URL was rendered.
    #[must_use]
    pub fn connected(&self) -> bool {
        !self.pages.is_empty()
    }

    /// Observed evidence for this snapshot.
    #[must_use]
    pub fn evidence(&self) -> Evidence {
        Evidence {
            kind: EvidenceKind::Observed,
            source: EvidenceSource::RenderedDom,
            confidence: weavatrix_seo_model::Confidence::High,
            snapshot_id: None,
            revision: None,
            policy_version: None,
        }
    }
}

/// Loads a render observation JSON.
///
/// # Errors
///
/// Returns IO or JSON errors.
pub fn load(path: &str) -> Result<RenderSnapshot, String> {
    let raw = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    from_json(&raw)
}

/// Parses render observations already in memory.
///
/// # Errors
///
/// Returns JSON errors.
pub fn from_json(raw: &str) -> Result<RenderSnapshot, String> {
    let mut snapshot: RenderSnapshot =
        blazingly_json::from_str(raw).map_err(|error| error.to_string())?;
    if snapshot.schema.is_empty() {
        snapshot.schema = "weavatrix-seo-render/v1".into();
    }
    if snapshot.source.is_empty() {
        snapshot.source = "wvq".into();
    }
    Ok(snapshot)
}

#[cfg(test)]
mod tests {
    use super::from_json;

    #[test]
    fn parses_rendered_title() {
        let snap = from_json(r#"{"pages":[{"url":"https://x.test/","title":"Home","h1":"Home"}]}"#)
            .expect("json");
        assert!(snap.connected());
        assert_eq!(snap.pages[0].title.as_deref(), Some("Home"));
    }
}

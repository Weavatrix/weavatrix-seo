//! Evidence CI: fail on error findings, regress against a comparable baseline.

#![forbid(unsafe_code)]

mod baseline;
mod verdict;

use serde::{Deserialize, Serialize};
use weavatrix_seo_model::AnalysisMode;

/// Compact baseline artifact. Prefer this over a full audit JSON.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Baseline {
    /// Artifact schema id.
    pub schema: String,
    /// Seed origin.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
    /// Repository path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    /// Analysis mode.
    pub mode: AnalysisMode,
    /// Policy identifier.
    #[serde(default)]
    pub policy_version: String,
    /// Crawl/config digest.
    #[serde(default)]
    pub config_digest: String,
    /// Git revision when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_revision: Option<String>,
    /// URLs actually measured in the baseline run.
    #[serde(default)]
    pub measured_urls: Vec<String>,
    /// Error fingerprint plus subject URL.
    #[serde(default)]
    pub issues: Vec<(String, String)>,
    /// Incomplete fetch count.
    #[serde(default)]
    pub incomplete: usize,
}

pub use baseline::{from_report, load_baseline, load_fingerprints};
pub use verdict::{GateVerdict, evaluate};

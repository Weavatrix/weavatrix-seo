//! Load fingerprints from a previous audit JSON.

use std::collections::BTreeSet;
use std::fs;
use weavatrix_seo_model::AuditReport;

/// Reads error fingerprints from a saved audit report.
///
/// # Errors
///
/// Returns IO or JSON errors.
pub fn load_fingerprints(path: &str) -> Result<BTreeSet<String>, String> {
    let raw = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let report: AuditReport =
        blazingly_json::from_str(&raw).map_err(|error| error.to_string())?;
    Ok(report
        .findings
        .into_iter()
        .filter(|finding| finding.severity == weavatrix_seo_model::Severity::Error)
        .map(|finding| finding.fingerprint)
        .collect())
}

//! Compact CI baseline. A full audit JSON is accepted as a fallback.

use crate::Baseline;
use std::collections::BTreeSet;
use std::fs;
use weavatrix_seo_model::{AuditReport, EvidenceScope, POLICY_VERSION, Severity};

/// Reads a dedicated baseline artifact, or error fingerprints from a saved audit.
///
/// # Errors
///
/// Returns IO or JSON errors.
pub fn load_baseline(path: &str) -> Result<Baseline, String> {
    let raw = fs::read_to_string(path).map_err(|error| error.to_string())?;
    if let Ok(baseline) = blazingly_json::from_str::<Baseline>(&raw)
        && baseline.schema.starts_with("weavatrix-seo-baseline")
    {
        return Ok(baseline);
    }
    let report: AuditReport = blazingly_json::from_str(&raw).map_err(|error| error.to_string())?;
    Ok(from_report(&report, String::new()))
}

/// Previous helper kept for callers that only need fingerprints.
///
/// # Errors
///
/// Returns IO or JSON errors.
pub fn load_fingerprints(path: &str) -> Result<BTreeSet<String>, String> {
    Ok(load_baseline(path)?.fingerprints())
}

/// Builds a comparable baseline from a report.
#[must_use]
pub fn from_report(report: &AuditReport, config_digest: String) -> Baseline {
    let issues: Vec<(String, String)> = report
        .findings
        .iter()
        .filter(|finding| finding.severity == Severity::Error)
        .map(|finding| {
            (
                finding.fingerprint.clone(),
                finding.locator.subject_url().to_owned(),
            )
        })
        .collect();
    Baseline {
        schema: "weavatrix-seo-baseline/v1".into(),
        origin: report.inventory.site.clone(),
        repo: report.inventory.repo.clone(),
        mode: report.inventory.mode,
        policy_version: if report.inventory.policy_version.is_empty() {
            POLICY_VERSION.to_owned()
        } else {
            report.inventory.policy_version.clone()
        },
        config_digest,
        repo_revision: report.inventory.repo_revision.clone(),
        measured_urls: report.inventory.measured_urls(),
        issues,
        incomplete: report.inventory.counts.incomplete,
    }
}

impl Baseline {
    /// Error fingerprints only.
    #[must_use]
    pub fn fingerprints(&self) -> BTreeSet<String> {
        self.issues.iter().map(|(fp, _)| fp.clone()).collect()
    }

    /// Comparison identity of this baseline.
    #[must_use]
    pub fn scope(&self) -> EvidenceScope {
        EvidenceScope::new(
            self.origin.clone(),
            self.mode,
            self.policy_version.clone(),
            self.config_digest.clone(),
        )
    }

    /// True when origin, mode, and policy match.
    #[must_use]
    pub fn comparable(&self, report: &AuditReport) -> bool {
        self.scope().comparable_with(&report.inventory.scope())
    }
}

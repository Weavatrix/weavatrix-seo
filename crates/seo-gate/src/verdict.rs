//! Gate verdict over the current evidence graph.

use std::collections::BTreeSet;
use weavatrix_seo_model::{AuditReport, Severity};

/// CI result. `10` is a fingerprint regression against baseline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateVerdict {
    /// Process exit code.
    pub code: i32,
    /// Error fingerprints not in the baseline.
    pub new_errors: Vec<String>,
    /// Baseline errors that disappeared.
    pub resolved: Vec<String>,
}

/// Evaluates the report. Baseline is optional.
#[must_use]
pub fn evaluate(report: &AuditReport, baseline: Option<&BTreeSet<String>>) -> GateVerdict {
    let errors: BTreeSet<String> = report
        .findings
        .iter()
        .filter(|finding| finding.severity == Severity::Error)
        .map(|finding| finding.fingerprint.clone())
        .collect();
    let Some(baseline) = baseline else {
        return GateVerdict {
            code: i32::from(!errors.is_empty()),
            new_errors: errors.into_iter().collect(),
            resolved: Vec::new(),
        };
    };
    let new_errors: Vec<String> = errors.difference(baseline).cloned().collect();
    let resolved: Vec<String> = baseline.difference(&errors).cloned().collect();
    let code = if new_errors.is_empty() { 0 } else { 10 };
    GateVerdict {
        code,
        new_errors,
        resolved,
    }
}

#[cfg(test)]
mod tests {
    use super::evaluate;
    use std::collections::BTreeSet;
    use weavatrix_seo_model::{
        AnalysisMode, AuditReport, Evidence, Finding, FindingFamily, Inventory, InventoryCounts,
        Locator, Severity,
    };

    #[test]
    fn new_error_is_regression() {
        let finding = Finding::new(
            FindingFamily::Claim,
            1,
            Severity::Error,
            "x",
            "claim",
            Locator::Url("https://x.test/".into()),
            Evidence::http(),
        );
        let report = AuditReport {
            inventory: Inventory {
                mode: AnalysisMode::Site,
                snapshot_id: "x".into(),
                site: None,
                repo: None,
                hosts: Vec::new(),
                pages: Vec::new(),
                edges: Vec::new(),
                predicted_routes: Vec::new(),
                sitemap_discovered: 0,
                counts: InventoryCounts {
                    crawled: 0,
                    fetched: 0,
                    redirected: 0,
                    errors: 0,
                    sitemap_urls: 0,
                    indexable: 0,
                },
            },
            findings: vec![finding.clone()],
            axes: Vec::new(),
            opportunities: Vec::new(),
        };
        let empty = BTreeSet::new();
        let verdict = evaluate(&report, Some(&empty));
        assert_eq!(verdict.code, 10);
        assert_eq!(verdict.new_errors, vec![finding.fingerprint]);
    }
}

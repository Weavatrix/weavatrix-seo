//! Gate verdict over the current evidence graph.

use crate::Baseline;
use weavatrix_seo_model::{AuditReport, Severity};

/// CI result. `10` is a fingerprint or coverage regression. `11` is incomparable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateVerdict {
    /// Process exit code.
    pub code: i32,
    /// Error fingerprints not in the baseline.
    pub new_errors: Vec<String>,
    /// Baseline errors that disappeared on a still-measured URL.
    pub resolved: Vec<String>,
    /// Baseline errors whose URLs were not measured this run.
    pub coverage_regressions: Vec<String>,
    /// Whether origin/mode/policy matched.
    pub comparable: bool,
}

/// Evaluates the report. Baseline is optional.
#[must_use]
pub fn evaluate(report: &AuditReport, baseline: Option<&Baseline>) -> GateVerdict {
    let errors: Vec<(String, String)> = report
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
    let Some(baseline) = baseline else {
        return GateVerdict {
            code: i32::from(!errors.is_empty()),
            new_errors: errors.into_iter().map(|(fp, _)| fp).collect(),
            resolved: Vec::new(),
            coverage_regressions: Vec::new(),
            comparable: true,
        };
    };
    if !baseline.comparable(report) {
        return GateVerdict {
            code: 11,
            new_errors: errors.into_iter().map(|(fp, _)| fp).collect(),
            resolved: Vec::new(),
            coverage_regressions: Vec::new(),
            comparable: false,
        };
    }
    let measured: Vec<String> = report.inventory.measured_urls();
    let current: Vec<String> = errors.iter().map(|(fp, _)| fp.clone()).collect();
    let mut new_errors = Vec::new();
    for (fp, _) in &errors {
        if !baseline.issues.iter().any(|(known, _)| known == fp) {
            new_errors.push(fp.clone());
        }
    }
    let mut resolved = Vec::new();
    let mut coverage_regressions = Vec::new();
    for (fp, url) in &baseline.issues {
        if current.iter().any(|item| item == fp) {
            continue;
        }
        let still_measured = measured.iter().any(|item| item == url)
            || url.is_empty()
                && !config_shrank(baseline, report);
        if still_measured {
            resolved.push(fp.clone());
        } else {
            coverage_regressions.push(fp.clone());
        }
    }
    let code = if new_errors.is_empty() && coverage_regressions.is_empty() {
        0
    } else {
        10
    };
    GateVerdict {
        code,
        new_errors,
        resolved,
        coverage_regressions,
        comparable: true,
    }
}

fn config_shrank(baseline: &Baseline, report: &AuditReport) -> bool {
    !baseline.config_digest.is_empty()
        && !report.inventory.config_digest.is_empty()
        && baseline.config_digest != report.inventory.config_digest
}

#[cfg(test)]
mod tests {
    use super::evaluate;
    use crate::{Baseline, from_report};
    use weavatrix_seo_model::{
        AnalysisMode, AuditReport, Evidence, Finding, FindingFamily, Inventory, Locator, Severity,
    };

    fn report_with(url: &str, pages: usize) -> AuditReport {
        let finding = Finding::new(
            FindingFamily::Claim,
            1,
            Severity::Error,
            url,
            "claim",
            Locator::Url(url.into()),
            Evidence::http(),
        );
        let mut inventory = Inventory::blank(AnalysisMode::Site);
        inventory.site = Some("https://x.test/".into());
        inventory.counts.crawled = pages;
        inventory.counts.incomplete = 0;
        AuditReport {
            inventory,
            findings: vec![finding],
            axes: Vec::new(),
            opportunities: Vec::new(),
        }
    }

    #[test]
    fn new_error_is_regression() {
        let report = report_with("https://x.test/", 1);
        let baseline = Baseline {
            schema: "weavatrix-seo-baseline/v1".into(),
            origin: Some("https://x.test/".into()),
            repo: None,
            mode: AnalysisMode::Site,
            policy_version: String::new(),
            config_digest: String::new(),
            repo_revision: None,
            measured_urls: vec!["https://x.test/".into()],
            issues: Vec::new(),
            incomplete: 0,
        };
        let verdict = evaluate(&report, Some(&baseline));
        assert_eq!(verdict.code, 10);
        assert!(!verdict.new_errors.is_empty());
    }

    #[test]
    fn smaller_crawl_is_coverage_not_resolved() {
        let full = report_with("https://x.test/old", 200);
        let mut baseline = from_report(&full, "full".into());
        baseline.origin = Some("https://x.test/".into());
        baseline.measured_urls = vec!["https://x.test/old".into()];
        let mut small = Inventory::blank(AnalysisMode::Site);
        small.site = Some("https://x.test/".into());
        small.config_digest = "small".into();
        let report = AuditReport {
            inventory: small,
            findings: Vec::new(),
            axes: Vec::new(),
            opportunities: Vec::new(),
        };
        let verdict = evaluate(&report, Some(&baseline));
        assert!(verdict.resolved.is_empty(), "{verdict:?}");
        assert!(!verdict.coverage_regressions.is_empty());
        assert_eq!(verdict.code, 10);
    }

    #[test]
    fn incomparable_origin_does_not_resolve() {
        let report = report_with("https://x.test/", 1);
        let baseline = Baseline {
            schema: "weavatrix-seo-baseline/v1".into(),
            origin: Some("https://other.test/".into()),
            repo: None,
            mode: AnalysisMode::Site,
            policy_version: String::new(),
            config_digest: String::new(),
            repo_revision: None,
            measured_urls: vec!["https://other.test/".into()],
            issues: vec![("WVX-SEO-CLAIM-001:deadbeef".into(), "https://other.test/".into())],
            incomplete: 0,
        };
        let verdict = evaluate(&report, Some(&baseline));
        assert!(!verdict.comparable);
        assert!(verdict.resolved.is_empty());
        assert_eq!(verdict.code, 11);
    }
}

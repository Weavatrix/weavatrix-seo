//! Named axes. Missing evidence is unmeasured, never green.

use weavatrix_seo_model::{AxisScore, Finding, FindingFamily, Severity};

/// Which evidence surfaces were actually connected for this run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct Coverage {
    /// Repository / route model.
    pub source: bool,
    /// HTTP pages.
    pub http: bool,
    /// GSC / provider import.
    pub obs: bool,
    /// WVQ / Playwright render snapshot.
    pub render: bool,
}

pub fn axes(findings: &[Finding], coverage: Coverage) -> Vec<AxisScore> {
    let named = [
        ("technical_discoverability", FindingFamily::Crawl),
        ("indexability", FindingFamily::Idx),
        ("canonical_integrity", FindingFamily::Canon),
        ("architecture", FindingFamily::Link),
        ("content_coverage", FindingFamily::Content),
        ("claim_integrity", FindingFamily::Claim),
        ("market_integrity", FindingFamily::Market),
        ("entity_integrity", FindingFamily::Entity),
        ("local_seo", FindingFamily::Local),
        ("international", FindingFamily::I18n),
        ("accessibility", FindingFamily::A11y),
        ("security", FindingFamily::Security),
        ("performance", FindingFamily::Perf),
        ("programmatic_safety", FindingFamily::Prog),
        ("observed_search", FindingFamily::Obs),
        ("ai_search", FindingFamily::Ai),
    ];
    named
        .into_iter()
        .map(|(axis, family)| score(findings, axis, family, coverage))
        .chain([render_axis(findings, coverage.render)])
        .collect()
}

fn score(findings: &[Finding], axis: &str, family: FindingFamily, coverage: Coverage) -> AxisScore {
    let subset: Vec<_> = findings
        .iter()
        .filter(|item| item.family == family)
        .collect();
    let unmeasured = (family == FindingFamily::Obs && !coverage.obs && subset.is_empty())
        || (family == FindingFamily::Ai && subset.is_empty() && !coverage.http && !coverage.source)
        || (family == FindingFamily::Prog && !coverage.source && subset.is_empty())
        || (matches!(
            family,
            FindingFamily::A11y | FindingFamily::Security | FindingFamily::Perf
        ) && !coverage.http
            && subset.is_empty());
    AxisScore {
        axis: axis.into(),
        errors: subset
            .iter()
            .filter(|item| item.severity == Severity::Error)
            .count(),
        warnings: subset
            .iter()
            .filter(|item| item.severity == Severity::Warn)
            .count(),
        infos: subset
            .iter()
            .filter(|item| item.severity == Severity::Info)
            .count(),
        unmeasured,
    }
}

fn render_axis(findings: &[Finding], has_render: bool) -> AxisScore {
    AxisScore {
        axis: "render_reconciliation".into(),
        errors: count(findings, FindingFamily::Render, Severity::Error),
        warnings: count(findings, FindingFamily::Render, Severity::Warn),
        infos: count(findings, FindingFamily::Render, Severity::Info),
        unmeasured: !has_render,
    }
}

fn count(findings: &[Finding], family: FindingFamily, severity: Severity) -> usize {
    findings
        .iter()
        .filter(|item| item.family == family && item.severity == severity)
        .count()
}

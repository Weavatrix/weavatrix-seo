//! Named axes. Missing evidence is unmeasured, never green.

use weavatrix_seo_model::{AxisScore, Finding, FindingFamily, Severity};

pub fn axes(findings: &[Finding], has_source: bool, has_http: bool) -> Vec<AxisScore> {
    let named = [
        ("technical_discoverability", FindingFamily::Crawl),
        ("indexability", FindingFamily::Idx),
        ("canonical_integrity", FindingFamily::Canon),
        ("architecture", FindingFamily::Link),
        ("content_coverage", FindingFamily::Content),
        ("claim_integrity", FindingFamily::Claim),
        ("market_integrity", FindingFamily::Market),
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
        .map(|(axis, family)| score(findings, axis, family, has_source, has_http))
        .chain([render_axis(findings, has_source)])
        .collect()
}

fn score(
    findings: &[Finding],
    axis: &str,
    family: FindingFamily,
    has_source: bool,
    has_http: bool,
) -> AxisScore {
    let subset: Vec<_> = findings
        .iter()
        .filter(|item| item.family == family)
        .collect();
    let unmeasured = (matches!(family, FindingFamily::Obs | FindingFamily::Ai)
        && subset.is_empty())
        || (family == FindingFamily::Prog && !has_source && subset.is_empty())
        || (matches!(
            family,
            FindingFamily::A11y | FindingFamily::Security | FindingFamily::Perf
        ) && !has_http
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

fn render_axis(findings: &[Finding], has_source: bool) -> AxisScore {
    AxisScore {
        axis: "render_reconciliation".into(),
        errors: count(findings, FindingFamily::Render, Severity::Error),
        warnings: count(findings, FindingFamily::Render, Severity::Warn),
        infos: count(findings, FindingFamily::Render, Severity::Info),
        unmeasured: !has_source,
    }
}

fn count(findings: &[Finding], family: FindingFamily, severity: Severity) -> usize {
    findings
        .iter()
        .filter(|item| item.family == family && item.severity == severity)
        .count()
}

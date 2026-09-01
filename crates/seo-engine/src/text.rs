//! Human-readable audit text. Not a single opaque score.

use std::fmt::Write as _;
use weavatrix_seo_model::{AuditReport, Severity};

/// Renders a compact text report.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn render_text(report: &AuditReport) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "Weavatrix SEO");
    let _ = writeln!(out, "mode: {:?}", report.inventory.mode);
    if let Some(site) = &report.inventory.site {
        let _ = writeln!(out, "site: {site}");
    }
    let counts = &report.inventory.counts;
    let _ = writeln!(
        out,
        "inventory: crawled={} fetched={} redirected={} errors={} sitemap={} indexable={} routes={}",
        counts.crawled,
        counts.fetched,
        counts.redirected,
        counts.errors,
        counts.sitemap_urls,
        counts.indexable,
        report.inventory.predicted_routes.len()
    );
    let _ = writeln!(out, "\naxes");
    for axis in &report.axes {
        if axis.unmeasured {
            let _ = writeln!(out, "  {}: UNMEASURED", axis.axis);
        } else {
            let _ = writeln!(
                out,
                "  {}: errors={} warnings={} infos={}",
                axis.axis, axis.errors, axis.warnings, axis.infos
            );
        }
    }
    let _ = writeln!(out, "\nfindings");
    if report.findings.is_empty() {
        let _ = writeln!(out, "  none");
    }
    for finding in &report.findings {
        let mark = match finding.severity {
            Severity::Error => "error",
            Severity::Warn => "warn",
            Severity::Info => "info",
        };
        let _ = writeln!(
            out,
            "  [{mark}] {} {}",
            finding.fingerprint, finding.summary
        );
    }
    let plan = crate::plan_from(report);
    if !plan.handoff.targets.is_empty() {
        let _ = writeln!(
            out,
            "\nhandoff: weavatrix-seo → weavatrix-refactor (read-only, {} targets)",
            plan.handoff.targets.len()
        );
        for target in plan.handoff.targets.iter().take(8) {
            let span = match (target.start_line, target.end_line) {
                (Some(start), Some(end)) => format!(":{start}-{end}"),
                (Some(start), None) => format!(":{start}"),
                _ => String::new(),
            };
            let _ = writeln!(
                out,
                "  {} {}{}  [{}] {}",
                target.intent,
                target.path,
                span,
                target.symbol.as_deref().unwrap_or("-"),
                target.subject
            );
        }
    }
    if let Some(intelligence) = &report.intelligence {
        let _ = writeln!(out, "\nintelligence");
        let _ = writeln!(
            out,
            "  semantics: engine={} artifact={} digest={}",
            intelligence.semantics.engine_version,
            intelligence.semantics.artifact_schema_version,
            &intelligence.semantics.rule_semantics_digest
                [..8.min(intelligence.semantics.rule_semantics_digest.len())]
        );
        let _ = writeln!(
            out,
            "  profiles={} families={} chunks={} matrices={} outcomes={}",
            intelligence.profiles.len(),
            intelligence.families.len(),
            intelligence.chunks.len(),
            intelligence.matrices.len(),
            intelligence.outcomes.len()
        );
        for family in intelligence.families.iter().take(8) {
            let _ = writeln!(
                out,
                "  family {} urls={} shared={:?} unique_facts={:?}",
                family.family,
                family.measured_urls,
                family.template_shared_ratio,
                family.unique_fact_ratio
            );
        }
    }
    let _ = writeln!(out, "\nopportunities");
    if report.opportunities.is_empty() {
        let _ = writeln!(out, "  none");
    }
    for item in &report.opportunities {
        let _ = writeln!(
            out,
            "  {} {} (demand {})",
            item.id, item.summary, item.demand
        );
    }
    out
}

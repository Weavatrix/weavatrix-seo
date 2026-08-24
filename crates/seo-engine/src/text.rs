//! Human-readable audit text. Not a single opaque score.

use std::fmt::Write as _;
use weavatrix_seo_model::{AuditReport, Severity};

/// Renders a compact text report.
#[must_use]
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
        "inventory: crawled={} fetched={} redirected={} errors={} sitemap={} indexable={}",
        counts.crawled,
        counts.fetched,
        counts.redirected,
        counts.errors,
        counts.sitemap_urls,
        counts.indexable
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

//! Self-contained HTML audit report.

use std::fmt::Write as _;
use weavatrix_seo_model::{AuditReport, Severity};

/// Renders a standalone HTML report.
#[must_use]
pub fn render_html(report: &AuditReport) -> String {
    let mut body = String::new();
    write_header(&mut body, report);
    write_axes(&mut body, report);
    write_findings(&mut body, report);
    write_intelligence(&mut body, report);
    write_opportunities(&mut body, report);
    wrap(&body)
}

fn write_header(body: &mut String, report: &AuditReport) {
    let counts = &report.inventory.counts;
    let _ = writeln!(body, "<header>");
    let _ = writeln!(body, "<p class=\"kicker\">Weavatrix SEO</p>");
    let _ = writeln!(body, "<h1>Search Evidence Graph</h1>");
    let _ = writeln!(
        body,
        "<p class=\"meta\">mode {:?} · no LLM</p>",
        report.inventory.mode
    );
    if let Some(site) = &report.inventory.site {
        let _ = writeln!(body, "<p class=\"meta\">site {}</p>", escape(site));
    }
    if let Some(repo) = &report.inventory.repo {
        let _ = writeln!(body, "<p class=\"meta\">repo {}</p>", escape(repo));
    }
    let _ = writeln!(
        body,
        "<p class=\"meta\">crawled {} · fetched {} · indexable {} · sitemap {} · routes {}</p>",
        counts.crawled,
        counts.fetched,
        counts.indexable,
        counts.sitemap_urls,
        report.inventory.predicted_routes.len()
    );
    let _ = writeln!(body, "</header>");
}

fn write_axes(body: &mut String, report: &AuditReport) {
    let _ = writeln!(body, "<section><h2>Axes</h2><div class=\"axes\">");
    for axis in &report.axes {
        let class = if axis.unmeasured {
            "unmeasured"
        } else if axis.errors > 0 {
            "error"
        } else if axis.warnings > 0 {
            "warn"
        } else {
            "ok"
        };
        let value = if axis.unmeasured {
            "UNMEASURED".into()
        } else {
            format!("e{} w{} i{}", axis.errors, axis.warnings, axis.infos)
        };
        let _ = writeln!(
            body,
            "<div class=\"axis {class}\"><strong>{}</strong><span>{value}</span></div>",
            escape(&axis.axis)
        );
    }
    let _ = writeln!(body, "</div></section>");
}

fn write_intelligence(body: &mut String, report: &AuditReport) {
    let Some(intelligence) = &report.intelligence else {
        return;
    };
    let _ = writeln!(body, "<section><h2>Intelligence</h2>");
    let _ = writeln!(
        body,
        "<p class=\"meta\">engine {} · artifact {} · digest {}</p>",
        escape(&intelligence.semantics.engine_version),
        escape(&intelligence.semantics.artifact_schema_version),
        escape(&intelligence.semantics.rule_semantics_digest)
    );
    if !intelligence.families.is_empty() {
        let _ = writeln!(
            body,
            "<table><thead><tr><th>family</th><th>urls</th><th>shared</th><th>unique facts</th></tr></thead><tbody>"
        );
        for family in &intelligence.families {
            let _ = writeln!(
                body,
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                escape(&family.family),
                family.measured_urls,
                family.template_shared_ratio.unwrap_or(0),
                family.unique_fact_ratio.unwrap_or(0)
            );
        }
        let _ = writeln!(body, "</tbody></table>");
    }
    let _ = writeln!(body, "</section>");
}

fn write_findings(body: &mut String, report: &AuditReport) {
    let _ = writeln!(
        body,
        "<section><h2>Findings ({})</h2><table><thead><tr><th>sev</th><th>id</th><th>summary</th></tr></thead><tbody>",
        report.findings.len()
    );
    if report.findings.is_empty() {
        let _ = writeln!(body, "<tr><td colspan=\"3\">none</td></tr>");
    }
    for finding in &report.findings {
        let sev = match finding.severity {
            Severity::Error => "error",
            Severity::Warn => "warn",
            Severity::Info => "info",
        };
        let _ = writeln!(
            body,
            "<tr class=\"{sev}\"><td>{sev}</td><td><code>{}</code></td><td>{}</td></tr>",
            escape(&finding.fingerprint),
            escape(&finding.summary)
        );
    }
    let _ = writeln!(body, "</tbody></table></section>");
}

fn write_opportunities(body: &mut String, report: &AuditReport) {
    let _ = writeln!(
        body,
        "<section><h2>Opportunities ({})</h2><ul>",
        report.opportunities.len()
    );
    if report.opportunities.is_empty() {
        let _ = writeln!(body, "<li>none</li>");
    }
    for item in &report.opportunities {
        let _ = writeln!(
            body,
            "<li><code>{}</code> {} <em>demand {}</em></li>",
            escape(&item.id),
            escape(&item.summary),
            escape(&item.demand)
        );
    }
    let _ = writeln!(body, "</ul></section>");
}

fn wrap(body: &str) -> String {
    format!(
        "<!doctype html>
<html lang=\"en\">
<head>
<meta charset=\"utf-8\">
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">
<title>Weavatrix SEO report</title>
<style>
body{{font:14px/1.45 ui-sans-serif,system-ui,sans-serif;margin:0;background:#0f1419;color:#e8edf2}}
header,section{{max-width:1100px;margin:0 auto;padding:24px}}
h1{{margin:0 0 8px;font-size:28px}}
h2{{margin:0 0 12px;font-size:18px}}
.kicker{{letter-spacing:.12em;text-transform:uppercase;color:#8fb5ff;margin:0}}
.meta{{color:#9aa7b4;margin:4px 0}}
.axes{{display:grid;grid-template-columns:repeat(auto-fill,minmax(220px,1fr));gap:8px}}
.axis{{border:1px solid #243041;border-radius:8px;padding:10px 12px;display:flex;justify-content:space-between;gap:12px}}
.axis.error{{border-color:#c44}}
.axis.warn{{border-color:#c90}}
.axis.ok{{border-color:#2a6}}
.axis.unmeasured{{opacity:.65}}
table{{width:100%;border-collapse:collapse}}
th,td{{text-align:left;padding:8px;border-bottom:1px solid #243041;vertical-align:top}}
tr.error td:first-child{{color:#ff8a8a}}
tr.warn td:first-child{{color:#ffd27a}}
tr.info td:first-child{{color:#9ecbff}}
code{{color:#cde3ff}}
em{{color:#9aa7b4;font-style:normal}}
</style>
</head>
<body>
{body}
</body>
</html>
"
    )
}

fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::escape;

    #[test]
    fn escapes_markup() {
        assert_eq!(escape("<a&\"b\">"), "&lt;a&amp;&quot;b&quot;&gt;");
    }
}

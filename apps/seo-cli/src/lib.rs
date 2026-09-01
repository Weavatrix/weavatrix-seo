//! `weavatrix-seo` CLI. Parses argv and calls the engine.

#![forbid(unsafe_code)]

mod args;

use std::collections::BTreeMap;
use std::fmt::Write as _;
use weavatrix_seo::{
    diff_paths, evaluate_gate, explain_chain, load_baseline, plan_from, render_html, render_text,
    retrieve, run_audit, run_on_report,
};

/// CLI stdout/stderr + process code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliOutput {
    /// Process exit code.
    pub code: i32,
    /// Report body.
    pub stdout: String,
    /// Error text.
    pub stderr: String,
}

/// Usage text.
#[must_use]
pub fn usage() -> String {
    "weavatrix-seo — Weavatrix SEO

Usage:
  weavatrix-seo audit --site URL [--repo PATH] [--max-pages N] [--workers N] [--html PATH] [--ci] [--baseline PATH] [--gsc PATH] [--observations PATH] [--render PATH] [--history DIR] [--public-only] [--json]
  weavatrix-seo inventory --site URL [--repo PATH] [--max-pages N] [--workers N] [--json]
  weavatrix-seo opportunities --site URL [--max-pages N] [--json]
  weavatrix-seo plan --site URL [--max-pages N] [--json]
  weavatrix-seo compare --site URL --competitor URL [--max-pages N] [--json]
  weavatrix-seo diff --base PATH --head PATH [--json]
  weavatrix-seo explain ID --site URL [--json]
  weavatrix-seo query --site URL --q 'FROM urls WHERE indexable = true LIMIT 20' [--json]
  weavatrix-seo retrieve --site URL --q QUERY [--json]
  weavatrix-seo mcp
  weavatrix-seo --version
"
    .to_owned()
}

/// Parse argv (without the binary name) and run.
#[must_use]
pub fn run(args: &[String]) -> CliOutput {
    if args.iter().any(|item| item == "--help" || item == "-h") {
        return ok(usage());
    }
    if args.iter().any(|item| item == "--version" || item == "-V") {
        return ok(format!("weavatrix-seo {}\n", env!("CARGO_PKG_VERSION")));
    }
    if args.first().is_some_and(|item| item == "mcp") {
        return CliOutput {
            code: 2,
            stdout: String::new(),
            stderr: "mcp is hosted by the weavatrix-seo binary entrypoint\n".into(),
        };
    }
    match dispatch(args) {
        Ok(output) => output,
        Err(message) => CliOutput {
            code: 2,
            stdout: String::new(),
            stderr: format!("{message}\n"),
        },
    }
}

#[allow(clippy::too_many_lines)]
fn dispatch(args: &[String]) -> Result<CliOutput, String> {
    let (command, flags, positionals) = args::split(args)?;
    let json = flags.contains_key("json") || args.iter().any(|item| item == "--json");
    if command == "diff" {
        return diff_command(&flags, json);
    }
    let request = args::request(&command, &flags, &positionals)?;
    if command == "compare" && request.competitors.is_empty() {
        return Err("compare requires --competitor URL".into());
    }
    let report = run_audit(&request).map_err(|error| error.to_string())?;
    if let Some(path) = flags.get("html") {
        std::fs::write(path, render_html(&report)).map_err(|error| error.to_string())?;
    }
    let body = match command.as_str() {
        "inventory" if json => encode(&report.inventory)?,
        "audit" if json => encode(&report)?,
        "opportunities" if json => encode(&report.opportunities)?,
        "plan" if json => encode(&plan_from(&report))?,
        "query" if json => {
            let q = flags
                .get("q")
                .or_else(|| flags.get("query"))
                .ok_or_else(|| "query requires --q DSL".to_owned())?;
            encode(&run_on_report(q, &report)?)?
        }
        "retrieve" if json => {
            let q = flags
                .get("q")
                .or_else(|| flags.get("query"))
                .ok_or_else(|| "retrieve requires --q QUERY".to_owned())?;
            encode(&retrieve(&report, q, 10))?
        }
        "compare" if json => encode(&report.opportunities)?,
        "explain" => {
            let id = positionals
                .first()
                .ok_or_else(|| "explain requires a finding id".to_owned())?;
            let Some(explanation) = explain_chain(&report, id) else {
                return Err(format!("unknown finding {id}"));
            };
            if json {
                encode(&explanation)?
            } else {
                let mut text = format!(
                    "{}\n{}\nwhy: {}\naction: {}\nverify: {}\n",
                    explanation.finding.fingerprint,
                    explanation.finding.summary,
                    explanation.finding.why,
                    explanation.finding.action,
                    explanation.finding.verification
                );
                for hop in &explanation.chain {
                    let _ = writeln!(
                        text,
                        "chain: {} {} {}",
                        hop.kind,
                        hop.relation.as_deref().unwrap_or("-"),
                        hop.label
                    );
                }
                text
            }
        }
        "opportunities" => {
            let mut text = String::new();
            for item in &report.opportunities {
                let _ = writeln!(text, "{} {}", item.id, item.summary);
            }
            text
        }
        "plan" => {
            let mut text = String::new();
            for item in plan_from(&report).actions {
                let _ = writeln!(text, "{} {}", item.kind, item.subject);
            }
            text
        }
        "query" => {
            let q = flags
                .get("q")
                .or_else(|| flags.get("query"))
                .ok_or_else(|| "query requires --q DSL".to_owned())?;
            let result = run_on_report(q, &report)?;
            if json {
                encode(&result)?
            } else {
                let mut text = format!(
                    "collection {} rows {}\n",
                    result.collection,
                    result.rows.len()
                );
                for row in result.rows {
                    let _ = writeln!(
                        text,
                        "  {}",
                        row.into_iter()
                            .map(|(k, v)| format!("{k}={v}"))
                            .collect::<Vec<_>>()
                            .join(" ")
                    );
                }
                text
            }
        }
        "retrieve" => {
            let q = flags
                .get("q")
                .or_else(|| flags.get("query"))
                .ok_or_else(|| "retrieve requires --q QUERY".to_owned())?;
            let hits = retrieve(&report, q, 10);
            if json {
                encode(&hits)?
            } else {
                let mut text = String::new();
                for hit in hits {
                    let _ = writeln!(text, "{} lexical={}", hit.url, hit.lexical);
                }
                text
            }
        }
        _ => render_text(&report),
    };
    let mut code = i32::from(
        report
            .findings
            .iter()
            .any(|item| matches!(item.severity, weavatrix_seo::Severity::Error)),
    );
    if request.ci || request.baseline.is_some() {
        let baseline = request.baseline.as_deref().map(load_baseline).transpose()?;
        code = evaluate_gate(&report, baseline.as_ref()).code;
    }
    Ok(CliOutput {
        code,
        stdout: body,
        stderr: String::new(),
    })
}

fn diff_command(flags: &BTreeMap<String, String>, json: bool) -> Result<CliOutput, String> {
    let base = flags
        .get("base")
        .ok_or_else(|| "diff requires --base PATH".to_owned())?;
    let head = flags
        .get("head")
        .ok_or_else(|| "diff requires --head PATH".to_owned())?;
    let delta = diff_paths(base, head)?;
    let stdout = if json {
        encode(&delta)?
    } else {
        format!(
            "comparable={} added={} removed={} changed={} new_errors={} resolved={} impacted_families={}\n",
            delta.comparable,
            delta.urls_added.len(),
            delta.urls_removed.len(),
            delta.urls_changed.len(),
            delta.findings_added.len(),
            delta.findings_resolved.len(),
            delta.families_impacted.len()
        )
    };
    Ok(CliOutput {
        code: i32::from(!delta.comparable || !delta.findings_added.is_empty()),
        stdout,
        stderr: String::new(),
    })
}

fn encode<T: serde::Serialize>(value: &T) -> Result<String, String> {
    blazingly_json::to_string(value).map_err(|error| error.to_string())
}

fn ok(stdout: String) -> CliOutput {
    CliOutput {
        code: 0,
        stdout,
        stderr: String::new(),
    }
}

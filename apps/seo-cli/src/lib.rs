//! `weavatrix-seo` CLI. Parses argv and calls the engine.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fmt::Write as _;
use weavatrix_seo::{
    AnalysisMode, AuditRequest, diff_paths, evaluate_gate, explain, load_baseline, plan_from,
    render_html, render_text, run_audit,
};

type ParsedArgs = (String, BTreeMap<String, String>, Vec<String>);

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
  weavatrix-seo audit --site URL [--repo PATH] [--max-pages N] [--workers N] [--html PATH] [--ci] [--baseline PATH] [--gsc PATH] [--observations PATH] [--history DIR] [--public-only] [--json]
  weavatrix-seo inventory --site URL [--repo PATH] [--max-pages N] [--workers N] [--json]
  weavatrix-seo opportunities --site URL [--max-pages N] [--json]
  weavatrix-seo plan --site URL [--max-pages N] [--json]
  weavatrix-seo compare --site URL --competitor URL [--max-pages N] [--json]
  weavatrix-seo diff --base PATH --head PATH [--json]
  weavatrix-seo explain ID --site URL [--json]
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

fn dispatch(args: &[String]) -> Result<CliOutput, String> {
    let (command, flags, positionals) = split(args)?;
    let json = flags.contains_key("json") || args.iter().any(|item| item == "--json");
    if command == "diff" {
        return diff_command(&flags, json);
    }
    let request = request(&command, &flags, &positionals)?;
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
        "compare" if json => encode(&report.opportunities)?,
        "explain" => {
            let id = positionals
                .first()
                .ok_or_else(|| "explain requires a finding id".to_owned())?;
            let Some(finding) = explain(&report, id) else {
                return Err(format!("unknown finding {id}"));
            };
            if json {
                encode(finding)?
            } else {
                format!(
                    "{}\n{}\nwhy: {}\naction: {}\nverify: {}\n",
                    finding.fingerprint,
                    finding.summary,
                    finding.why,
                    finding.action,
                    finding.verification
                )
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
        _ => render_text(&report),
    };
    let mut code = i32::from(
        report
            .findings
            .iter()
            .any(|item| matches!(item.severity, weavatrix_seo::Severity::Error)),
    );
    if request.ci || request.baseline.is_some() {
        let baseline = request
            .baseline
            .as_deref()
            .map(load_baseline)
            .transpose()?;
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
            "comparable={} added={} removed={} changed={} new_errors={} resolved={}\n",
            delta.comparable,
            delta.urls_added.len(),
            delta.urls_removed.len(),
            delta.urls_changed.len(),
            delta.findings_added.len(),
            delta.findings_resolved.len()
        )
    };
    Ok(CliOutput {
        code: i32::from(!delta.comparable || !delta.findings_added.is_empty()),
        stdout,
        stderr: String::new(),
    })
}

fn request(
    command: &str,
    flags: &BTreeMap<String, String>,
    _positionals: &[String],
) -> Result<AuditRequest, String> {
    let site = flags.get("site").cloned();
    let repo = flags.get("repo").cloned();
    let max_pages = flags
        .get("max-pages")
        .map(|value| {
            value
                .parse::<usize>()
                .map_err(|_| "invalid --max-pages".to_owned())
        })
        .transpose()?;
    let workers = flags
        .get("workers")
        .map(|value| {
            value
                .parse::<usize>()
                .map_err(|_| "invalid --workers".to_owned())
        })
        .transpose()?;
    let ci = flags.contains_key("ci");
    let baseline = flags.get("baseline").cloned();
    let allow_private = !flags.contains_key("public-only");
    let gsc = flags.get("gsc").cloned();
    let observations = flags.get("observations").cloned();
    let history = flags.get("history").cloned();
    let competitors = flags
        .get("competitor")
        .map(|value| {
            value
                .lines()
                .filter(|item| !item.is_empty())
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    let mode = if command == "compare" {
        AnalysisMode::Compare
    } else if site.is_some() && repo.is_some() {
        AnalysisMode::Hybrid
    } else if repo.is_some() {
        AnalysisMode::Repo
    } else {
        AnalysisMode::Site
    };
    if site.is_none() && repo.is_none() {
        return Err("provide --site URL and/or --repo PATH".into());
    }
    Ok(AuditRequest {
        mode,
        site,
        repo,
        competitors,
        max_pages,
        workers,
        ci,
        baseline,
        allow_private,
        gsc,
        observations,
        history,
    })
}

fn split(args: &[String]) -> Result<ParsedArgs, String> {
    if args.is_empty() {
        return Err(usage());
    }
    let command = args[0].clone();
    if !matches!(
        command.as_str(),
        "audit" | "inventory" | "opportunities" | "plan" | "compare" | "explain" | "diff"
    ) {
        return Err(format!("unknown command `{command}`\n{}", usage()));
    }
    let mut flags = BTreeMap::new();
    let mut positionals = Vec::new();
    let mut index = 1;
    while index < args.len() {
        let item = &args[index];
        if item == "--json" || item == "--ci" || item == "--public-only" {
            flags.insert(item.trim_start_matches('-').to_owned(), "true".into());
            index += 1;
            continue;
        }
        if let Some(name) = item.strip_prefix("--") {
            index += 1;
            let value = args
                .get(index)
                .ok_or_else(|| format!("flag --{name} requires a value"))?;
            if name == "competitor" {
                let mut current: String = flags.remove("competitor").unwrap_or_default();
                if !current.is_empty() {
                    current.push('\n');
                }
                current.push_str(value);
                flags.insert("competitor".into(), current);
            } else if flags.insert(name.to_owned(), value.clone()).is_some() {
                return Err(format!("flag --{name} was supplied more than once"));
            }
            index += 1;
            continue;
        }
        positionals.push(item.clone());
        index += 1;
    }
    Ok((command, flags, positionals))
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

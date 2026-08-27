//! Agent MCP surface. Eleven tools. No shell.

#![forbid(unsafe_code)]

mod schema;

use mcport::{ConcurrentMcpServer, RuntimeConfig, ToolReply, json};
use serde::Deserialize;
use std::time::Duration;
use weavatrix_seo::{
    AnalysisMode, AuditRequest, diff_paths, evaluate_gate, explain_chain, link_inputs,
    load_baseline, plan_from, run_audit,
};
use weavatrix_seo_observation::{
    load as load_gsc, load_any, unmeasured as observations_unmeasured,
};

/// Host options. Startup only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostOptions {
    /// Page cap applied to every crawl.
    pub max_pages: usize,
}

/// Parse stdio host arguments.
///
/// # Errors
///
/// Unknown or incomplete options are rejected.
pub fn parse_host_args(args: &[String]) -> Result<HostOptions, String> {
    let mut max_pages = 200_usize;
    let mut index = 0;
    while index < args.len() {
        let name = args[index]
            .strip_prefix("--")
            .ok_or_else(|| format!("unexpected argument `{}`", args[index]))?;
        let value = args
            .get(index + 1)
            .ok_or_else(|| format!("option --{name} requires a value"))?;
        match name {
            "max-pages" => {
                max_pages = value
                    .parse()
                    .map_err(|_| format!("invalid --max-pages {value}"))?;
            }
            other => return Err(format!("unknown option --{other}")),
        }
        index += 2;
    }
    Ok(HostOptions { max_pages })
}

/// Controlled concurrency for crawl-backed tools.
#[must_use]
pub fn runtime_config() -> RuntimeConfig {
    RuntimeConfig {
        max_in_flight: 2,
        queue_depth: 16,
        output_queue_depth: 16,
        handler_deadline: Some(Duration::from_secs(5 * 60)),
        ..RuntimeConfig::default()
    }
}

#[derive(Debug, Clone, Deserialize)]
struct SiteInput {
    #[serde(default)]
    mode: Option<String>,
    #[serde(default)]
    site: Option<String>,
    #[serde(default)]
    repo: Option<String>,
    #[serde(default)]
    competitor: Option<String>,
    #[serde(default)]
    competitors: Vec<String>,
    #[serde(default)]
    max_pages: Option<usize>,
    #[serde(default)]
    workers: Option<usize>,
    #[serde(default)]
    render: Option<String>,
    #[serde(default)]
    gsc: Option<String>,
    #[serde(default)]
    observations: Option<String>,
    #[serde(default)]
    history: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ExplainInput {
    id: String,
    #[serde(default)]
    site: Option<String>,
    #[serde(default)]
    repo: Option<String>,
    #[serde(default)]
    max_pages: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
struct DiffInput {
    #[serde(default)]
    repo: Option<String>,
    #[serde(default)]
    base: Option<String>,
    #[serde(default)]
    head: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct GateInput {
    #[serde(default)]
    mode: Option<String>,
    #[serde(default)]
    site: Option<String>,
    #[serde(default)]
    repo: Option<String>,
    #[serde(default)]
    max_pages: Option<usize>,
    #[serde(default)]
    workers: Option<usize>,
    #[serde(default)]
    render: Option<String>,
    #[serde(default)]
    gsc: Option<String>,
    #[serde(default)]
    observations: Option<String>,
    #[serde(default)]
    baseline: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ObservationsInput {
    #[serde(default)]
    observations: Option<String>,
    #[serde(default)]
    gsc: Option<String>,
    #[serde(default)]
    provider: Option<String>,
    #[serde(default)]
    limit: Option<usize>,
}

/// Eleven-tool SEO server.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn seo_server(max_pages: usize) -> ConcurrentMcpServer {
    ConcurrentMcpServer::new("weavatrix-seo", env!("CARGO_PKG_VERSION"))
        .instructions(
            "Weavatrix SEO. Eleven bounded tools. No shell. Missing evidence is unmeasured.",
        )
        .strict_schemas()
        .typed_tool(
            "seo_inventory",
            "Build the search surface inventory for a site, repo, or hybrid run.",
            schema::site(),
            move |_ctx, input: SiteInput| tool_audit(max_pages, &input, "inventory"),
        )
        .typed_tool(
            "seo_audit",
            "Return bounded findings by axis, severity, and evidence.",
            schema::site(),
            move |_ctx, input: SiteInput| tool_audit(max_pages, &input, "audit"),
        )
        .typed_tool(
            "seo_opportunities",
            "Return gaps and construction opportunities, not current errors.",
            schema::site(),
            move |_ctx, input: SiteInput| tool_audit(max_pages, &input, "opportunities"),
        )
        .typed_tool(
            "seo_plan",
            "Produce a target search-architecture plan with acceptance conditions.",
            schema::site(),
            move |_ctx, input: SiteInput| tool_audit(max_pages, &input, "plan"),
        )
        .typed_tool(
            "seo_compare",
            "Compare an owned site against public competitor origins.",
            schema::site(),
            move |_ctx, input: SiteInput| tool_audit(max_pages, &input, "compare"),
        )
        .typed_tool(
            "seo_links",
            "Directed internal-link recommendations from first-party page vectors. Inferred, never a ranking proof.",
            schema::site(),
            move |_ctx, input: SiteInput| tool_audit(max_pages, &input, "links"),
        )
        .typed_tool(
            "seo_vectors",
            "Deterministic page vectors and SEO link profiles. Lexical model, no embedding service.",
            schema::site(),
            move |_ctx, input: SiteInput| tool_audit(max_pages, &input, "vectors"),
        )
        .typed_tool(
            "seo_diff",
            "Compare two revision-bound snapshots or audit JSON files.",
            schema::diff(),
            move |_ctx, input: DiffInput| match (input.base.as_deref(), input.head.as_deref()) {
                (Some(base), Some(head)) => match diff_paths(base, head) {
                    Ok(delta) => ToolReply::structured(delta),
                    Err(error) => ToolReply::error(error),
                },
                _ => ToolReply::structured(json!({
                    "unmeasured": true,
                    "repo": input.repo,
                    "reason": "seo_diff requires base and head snapshot paths. Git SHAs without snapshots stay unmeasured."
                })),
            },
        )
        .typed_tool(
            "seo_gate",
            "Evidence CI: compare the current run against a baseline and return the gate verdict.",
            schema::gate(),
            move |_ctx, input: GateInput| tool_gate(max_pages, &input),
        )
        .typed_tool(
            "seo_explain",
            "Explain one finding or opportunity with its evidence chain.",
            schema::explain(),
            move |_ctx, input: ExplainInput| {
                if input.site.is_none() && input.repo.is_none() {
                    return ToolReply::error("seo_explain requires site or repo");
                }
                let mode = if input.repo.is_some() && input.site.is_some() {
                    AnalysisMode::Hybrid
                } else if input.repo.is_some() {
                    AnalysisMode::Repo
                } else {
                    AnalysisMode::Site
                };
                let request = AuditRequest {
                    mode,
                    site: input.site.clone(),
                    repo: input.repo.clone(),
                    competitors: Vec::new(),
                    max_pages: input.max_pages.or(Some(max_pages)),
                    workers: None,
                    ci: false,
                    baseline: None,
                    allow_private: false,
                    gsc: None,
                    observations: None,
                    history: None,
                    render: None,
                };
                match run_audit(&request) {
                    Ok(report) => match explain_chain(&report, &input.id) {
                        Some(explanation) => ToolReply::structured(explanation),
                        None => ToolReply::error(format!("unknown finding {}", input.id)),
                    },
                    Err(error) => ToolReply::error(error.to_string()),
                }
            },
        )
        .typed_tool(
            "seo_observations",
            "Query imported GSC, log, analytics, or AI-search evidence.",
            schema::observations(),
            move |_ctx, input: ObservationsInput| tool_observations(&input),
        )
}

/// Serves stdio MCP.
///
/// # Errors
///
/// Returns an IO error from the runtime.
pub fn serve(options: &HostOptions) -> Result<(), String> {
    seo_server(options.max_pages)
        .serve(runtime_config())
        .map_err(|error| error.to_string())
}

fn tool_audit(default_pages: usize, input: &SiteInput, view: &str) -> ToolReply {
    let mut competitors = input.competitors.clone();
    if let Some(one) = &input.competitor {
        competitors.push(one.clone());
    }
    let mode = match input.mode.as_deref() {
        Some("repo") => AnalysisMode::Repo,
        Some("hybrid") => AnalysisMode::Hybrid,
        Some("compare") => AnalysisMode::Compare,
        _ if !competitors.is_empty() => AnalysisMode::Compare,
        _ if input.repo.is_some() && input.site.is_some() => AnalysisMode::Hybrid,
        _ if input.repo.is_some() => AnalysisMode::Repo,
        _ => AnalysisMode::Site,
    };
    let request = AuditRequest {
        mode,
        site: input.site.clone(),
        repo: input.repo.clone(),
        competitors,
        max_pages: input.max_pages.or(Some(default_pages)),
        workers: input.workers,
        ci: false,
        baseline: None,
        allow_private: false,
        gsc: input.gsc.clone(),
        observations: input.observations.clone(),
        history: input.history.clone(),
        render: input.render.clone(),
    };
    match run_audit(&request) {
        Ok(report) => match view {
            "inventory" => ToolReply::structured(&report.inventory),
            "opportunities" | "compare" => ToolReply::structured(&report.opportunities),
            "plan" => ToolReply::structured(plan_from(&report)),
            "vectors" => ToolReply::structured(link_inputs(&report)),
            "links" => {
                let inputs = link_inputs(&report);
                let links: Vec<_> = report
                    .findings
                    .iter()
                    .filter(|finding| finding.code == "WVX-SEO-LINK-004")
                    .collect();
                let placements: Vec<_> = report
                    .opportunities
                    .iter()
                    .filter(|item| item.kind == "link_rec")
                    .collect();
                ToolReply::structured(json!({
                    "model": inputs.model,
                    "dimension": inputs.dimension,
                    "vectors": inputs.vectors.len(),
                    "evidence": "INFERRED",
                    "links": links,
                    "opportunities": placements
                }))
            }
            _ => ToolReply::structured(&report),
        },
        Err(error) => ToolReply::error(error.to_string()),
    }
}

fn tool_gate(default_pages: usize, input: &GateInput) -> ToolReply {
    if input.site.is_none() && input.repo.is_none() {
        return ToolReply::error("seo_gate requires site or repo");
    }
    let mode = match input.mode.as_deref() {
        Some("repo") => AnalysisMode::Repo,
        Some("hybrid") => AnalysisMode::Hybrid,
        _ if input.repo.is_some() && input.site.is_some() => AnalysisMode::Hybrid,
        _ if input.repo.is_some() => AnalysisMode::Repo,
        _ => AnalysisMode::Site,
    };
    let request = AuditRequest {
        mode,
        site: input.site.clone(),
        repo: input.repo.clone(),
        competitors: Vec::new(),
        max_pages: input.max_pages.or(Some(default_pages)),
        workers: input.workers,
        ci: true,
        baseline: input.baseline.clone(),
        allow_private: false,
        gsc: input.gsc.clone(),
        observations: input.observations.clone(),
        history: None,
        render: input.render.clone(),
    };
    let report = match run_audit(&request) {
        Ok(report) => report,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let baseline = match input.baseline.as_deref().map(load_baseline).transpose() {
        Ok(baseline) => baseline,
        Err(error) => return ToolReply::error(error),
    };
    let verdict = evaluate_gate(&report, baseline.as_ref());
    ToolReply::structured(json!({
        "code": verdict.code,
        "comparable": verdict.comparable,
        "new_errors": verdict.new_errors,
        "resolved": verdict.resolved,
        "coverage_regressions": verdict.coverage_regressions,
        "baseline": input.baseline,
        "measured_urls": report.inventory.measured_urls().len()
    }))
}

fn tool_observations(input: &ObservationsInput) -> ToolReply {
    let loaded = match (input.observations.as_deref(), input.gsc.as_deref()) {
        (Some(path), _) => Some(load_any(path)),
        (None, Some(path)) => Some(load_gsc(path)),
        (None, None) => None,
    };
    let snapshot = match loaded {
        Some(Ok(snapshot)) => snapshot,
        Some(Err(error)) => return ToolReply::error(error.to_string()),
        None => observations_unmeasured(),
    };
    let rows: Vec<_> = snapshot
        .rows
        .iter()
        .filter(|row| {
            input
                .provider
                .as_deref()
                .is_none_or(|provider| row.provider == provider)
        })
        .collect();
    let mut providers: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for row in &rows {
        *providers.entry(row.provider.as_str()).or_default() += 1;
    }
    let limit = input.limit.unwrap_or(200);
    ToolReply::structured(json!({
        "connected": snapshot.connected,
        "unmeasured": !snapshot.connected,
        "total": rows.len(),
        "providers": providers,
        "returned": rows.len().min(limit),
        "rows": rows.iter().take(limit).collect::<Vec<_>>(),
        "reason": if snapshot.connected {
            None
        } else {
            Some("No provider import was supplied. Pass observations or gsc.")
        }
    }))
}

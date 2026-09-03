//! Agent MCP surface. Existing tools plus query/retrieve/similar/chunks. No shell.

#![forbid(unsafe_code)]

mod roots;
mod schema;

pub use roots::Roots;

use mcport::{ConcurrentMcpServer, RuntimeConfig, ToolReply, json};
use serde::Deserialize;
use std::time::Duration;
use weavatrix_seo::{
    AnalysisMode, AuditRequest, chunks_for, diff_paths, evaluate_gate, explain_chain, link_inputs,
    load_baseline, plan_from, retrieve, run_audit, run_on_history, run_on_report, similar,
};
use weavatrix_seo_observation::{
    load as load_gsc, load_any, unmeasured as observations_unmeasured,
};

/// Host options. Startup only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostOptions {
    /// Page cap applied to every crawl.
    pub max_pages: usize,
    /// Directories a caller may reference. Empty means the working directory.
    pub roots: Vec<String>,
}

/// Parse stdio host arguments.
///
/// # Errors
///
/// Unknown or incomplete options are rejected.
pub fn parse_host_args(args: &[String]) -> Result<HostOptions, String> {
    let mut max_pages = 200_usize;
    let mut roots = Vec::new();
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
            "allow-root" => roots.push(value.clone()),
            other => return Err(format!("unknown option --{other}")),
        }
        index += 2;
    }
    Ok(HostOptions { max_pages, roots })
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
struct QueryInput {
    query: String,
    #[serde(default)]
    mode: Option<String>,
    #[serde(default)]
    site: Option<String>,
    #[serde(default)]
    repo: Option<String>,
    #[serde(default)]
    max_pages: Option<usize>,
    #[serde(default)]
    gsc: Option<String>,
    #[serde(default)]
    observations: Option<String>,
    #[serde(default)]
    render: Option<String>,
    #[serde(default)]
    history: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct RetrieveInput {
    #[serde(default)]
    query: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    limit: Option<usize>,
    #[serde(default)]
    mode: Option<String>,
    #[serde(default)]
    site: Option<String>,
    #[serde(default)]
    repo: Option<String>,
    #[serde(default)]
    max_pages: Option<usize>,
    #[serde(default)]
    gsc: Option<String>,
    #[serde(default)]
    observations: Option<String>,
    #[serde(default)]
    render: Option<String>,
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

/// Bounded SEO server. Existing tools remain; query/retrieve are additive.
///
/// `roots` bounds every caller-supplied path. Without it a connected agent
/// could read any file the host process can reach.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn seo_server(max_pages: usize, roots: &Roots) -> ConcurrentMcpServer {
    ConcurrentMcpServer::new("weavatrix-seo", env!("CARGO_PKG_VERSION"))
        .instructions(
            "Weavatrix SEO. Bounded tools. No shell. Paths are confined to the allowed roots. Missing evidence is unmeasured. Prefer seo_query and seo_retrieve over raw vectors.",
        )
        .strict_schemas()
        .typed_tool(
            "seo_inventory",
            "Build the search surface inventory for a site, repo, or hybrid run.",
            schema::site(),
            {
                let roots = roots.clone();
                move |_ctx, input: SiteInput| tool_audit(max_pages, &roots, &input, "inventory")
            },
        )
        .typed_tool(
            "seo_audit",
            "Return bounded findings by axis, severity, and evidence.",
            schema::site(),
            {
                let roots = roots.clone();
                move |_ctx, input: SiteInput| tool_audit(max_pages, &roots, &input, "audit")
            },
        )
        .typed_tool(
            "seo_opportunities",
            "Return gaps and construction opportunities, not current errors.",
            schema::site(),
            {
                let roots = roots.clone();
                move |_ctx, input: SiteInput| tool_audit(max_pages, &roots, &input, "opportunities")
            },
        )
        .typed_tool(
            "seo_plan",
            "Produce a target search-architecture plan with acceptance conditions.",
            schema::site(),
            {
                let roots = roots.clone();
                move |_ctx, input: SiteInput| tool_audit(max_pages, &roots, &input, "plan")
            },
        )
        .typed_tool(
            "seo_compare",
            "Compare an owned site against public competitor origins.",
            schema::site(),
            {
                let roots = roots.clone();
                move |_ctx, input: SiteInput| tool_audit(max_pages, &roots, &input, "compare")
            },
        )
        .typed_tool(
            "seo_links",
            "Directed internal-link recommendations from first-party page vectors. Inferred, never a ranking proof.",
            schema::site(),
            {
                let roots = roots.clone();
                move |_ctx, input: SiteInput| tool_audit(max_pages, &roots, &input, "links")
            },
        )
        .typed_tool(
            "seo_vectors",
            "Deterministic page vectors and SEO link profiles. Lexical model, no embedding service.",
            schema::site(),
            {
                let roots = roots.clone();
                move |_ctx, input: SiteInput| tool_audit(max_pages, &roots, &input, "vectors")
            },
        )
        .typed_tool(
            "seo_diff",
            "Compare two revision-bound snapshots or audit JSON files.",
            schema::diff(),
            {
                let roots = roots.clone();
                move |_ctx, input: DiffInput| tool_diff(&roots, &input)
            },
        )
        .typed_tool(
            "seo_gate",
            "Evidence CI: compare the current run against a baseline and return the gate verdict.",
            schema::gate(),
            {
                let roots = roots.clone();
                move |_ctx, input: GateInput| tool_gate(max_pages, &roots, &input)
            },
        )
        .typed_tool(
            "seo_explain",
            "Explain one finding or opportunity with its evidence chain.",
            schema::explain(),
            {
                let roots = roots.clone();
                move |_ctx, input: ExplainInput| tool_explain(max_pages, &roots, &input)
            },
        )
        .typed_tool(
            "seo_observations",
            "Query imported GSC, log, analytics, or AI-search evidence.",
            schema::observations(),
            {
                let roots = roots.clone();
                move |_ctx, input: ObservationsInput| tool_observations(&roots, &input)
            },
        )
        .typed_tool(
            "seo_query",
            "Bounded read-only query over the last audit or --history SQLite: FROM urls|findings|claims|route_families|chunks|opportunities|runs WHERE ... RETURN ... LIMIT n.",
            schema::query(),
            {
                let roots = roots.clone();
                move |_ctx, input: QueryInput| tool_query(max_pages, &roots, &input)
            },
        )
        .typed_tool(
            "seo_retrieve",
            "Rank candidate pages for a query. Rust computes similarity; do not re-do vector math.",
            schema::retrieve(),
            {
                let roots = roots.clone();
                move |_ctx, input: RetrieveInput| tool_retrieve(max_pages, &roots, &input, "retrieve")
            },
        )
        .typed_tool(
            "seo_similar",
            "Pages most similar to a URL in the same audit. Lexical, inferred.",
            schema::retrieve(),
            {
                let roots = roots.clone();
                move |_ctx, input: RetrieveInput| tool_retrieve(max_pages, &roots, &input, "similar")
            },
        )
        .typed_tool(
            "seo_chunks",
            "Chunks that best answer a query, with citation-suitability signals.",
            schema::retrieve(),
            {
                let roots = roots.clone();
                move |_ctx, input: RetrieveInput| tool_retrieve(max_pages, &roots, &input, "chunks")
            },
        )
}

/// Serves stdio MCP.
///
/// # Errors
///
/// Returns an IO error from the runtime.
pub fn serve(options: &HostOptions) -> Result<(), String> {
    seo_server(options.max_pages, &Roots::new(&options.roots))
        .serve(runtime_config())
        .map_err(|error| error.to_string())
}

fn audit_request(
    default_pages: usize,
    roots: &Roots,
    input: &SiteInput,
) -> Result<AuditRequest, String> {
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
    Ok(AuditRequest {
        mode,
        site: input.site.clone(),
        repo: roots.resolve_optional("repo", input.repo.as_ref())?,
        competitors,
        max_pages: input.max_pages.or(Some(default_pages)),
        workers: input.workers,
        ci: false,
        baseline: None,
        allow_private: false,
        gsc: roots.resolve_optional("gsc", input.gsc.as_ref())?,
        observations: roots.resolve_optional("observations", input.observations.as_ref())?,
        history: roots.resolve_optional("history", input.history.as_ref())?,
        render: roots.resolve_optional("render", input.render.as_ref())?,
    })
}

fn tool_audit(default_pages: usize, roots: &Roots, input: &SiteInput, view: &str) -> ToolReply {
    let request = match audit_request(default_pages, roots, input) {
        Ok(request) => request,
        Err(error) => return ToolReply::error(error),
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

fn diff_sides(
    roots: &Roots,
    input: &DiffInput,
) -> Result<(Option<String>, Option<String>), String> {
    Ok((
        roots.resolve_optional("base", input.base.as_ref())?,
        roots.resolve_optional("head", input.head.as_ref())?,
    ))
}

fn tool_diff(roots: &Roots, input: &DiffInput) -> ToolReply {
    let (base, head) = match diff_sides(roots, input) {
        Ok(sides) => sides,
        Err(error) => return ToolReply::error(error),
    };
    match (base, head) {
        (Some(base), Some(head)) => match diff_paths(&base, &head) {
            Ok(delta) => ToolReply::structured(delta),
            Err(error) => ToolReply::error(error),
        },
        _ => ToolReply::structured(json!({
            "unmeasured": true,
            "repo": input.repo,
            "reason": "seo_diff requires base and head snapshot paths. Git SHAs without snapshots stay unmeasured."
        })),
    }
}

fn tool_explain(default_pages: usize, roots: &Roots, input: &ExplainInput) -> ToolReply {
    if input.site.is_none() && input.repo.is_none() {
        return ToolReply::error("seo_explain requires site or repo");
    }
    let repo = match roots.resolve_optional("repo", input.repo.as_ref()) {
        Ok(repo) => repo,
        Err(error) => return ToolReply::error(error),
    };
    let mode = if repo.is_some() && input.site.is_some() {
        AnalysisMode::Hybrid
    } else if repo.is_some() {
        AnalysisMode::Repo
    } else {
        AnalysisMode::Site
    };
    let request = AuditRequest {
        mode,
        site: input.site.clone(),
        repo,
        competitors: Vec::new(),
        max_pages: input.max_pages.or(Some(default_pages)),
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
}

fn gate_request(
    default_pages: usize,
    roots: &Roots,
    input: &GateInput,
) -> Result<AuditRequest, String> {
    let repo = roots.resolve_optional("repo", input.repo.as_ref())?;
    let mode = match input.mode.as_deref() {
        Some("repo") => AnalysisMode::Repo,
        Some("hybrid") => AnalysisMode::Hybrid,
        _ if repo.is_some() && input.site.is_some() => AnalysisMode::Hybrid,
        _ if repo.is_some() => AnalysisMode::Repo,
        _ => AnalysisMode::Site,
    };
    Ok(AuditRequest {
        mode,
        site: input.site.clone(),
        repo,
        competitors: Vec::new(),
        max_pages: input.max_pages.or(Some(default_pages)),
        workers: input.workers,
        ci: true,
        baseline: roots.resolve_optional("baseline", input.baseline.as_ref())?,
        allow_private: false,
        gsc: roots.resolve_optional("gsc", input.gsc.as_ref())?,
        observations: roots.resolve_optional("observations", input.observations.as_ref())?,
        history: None,
        render: roots.resolve_optional("render", input.render.as_ref())?,
    })
}

fn tool_gate(default_pages: usize, roots: &Roots, input: &GateInput) -> ToolReply {
    if input.site.is_none() && input.repo.is_none() {
        return ToolReply::error("seo_gate requires site or repo");
    }
    let request = match gate_request(default_pages, roots, input) {
        Ok(request) => request,
        Err(error) => return ToolReply::error(error),
    };
    let baseline_path = request.baseline.clone();
    let report = match run_audit(&request) {
        Ok(report) => report,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let baseline = match baseline_path.as_deref().map(load_baseline).transpose() {
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
        "baseline": baseline_path,
        "measured_urls": report.inventory.measured_urls().len()
    }))
}

fn observation_paths(
    roots: &Roots,
    input: &ObservationsInput,
) -> Result<(Option<String>, Option<String>), String> {
    Ok((
        roots.resolve_optional("observations", input.observations.as_ref())?,
        roots.resolve_optional("gsc", input.gsc.as_ref())?,
    ))
}

fn tool_query(default_pages: usize, roots: &Roots, input: &QueryInput) -> ToolReply {
    if input.site.is_none() && input.repo.is_none() {
        let Some(history) = input.history.as_ref() else {
            return ToolReply::error("seo_query requires site, repo, or history");
        };
        let dir = match roots.resolve("history", history) {
            Ok(dir) => dir,
            Err(error) => return ToolReply::error(error),
        };
        return match run_on_history(&input.query, &dir) {
            Ok(result) => ToolReply::structured(result),
            Err(error) => ToolReply::error(error),
        };
    }
    let site = SiteInput {
        mode: input.mode.clone(),
        site: input.site.clone(),
        repo: input.repo.clone(),
        competitor: None,
        competitors: Vec::new(),
        max_pages: input.max_pages,
        workers: None,
        render: input.render.clone(),
        gsc: input.gsc.clone(),
        observations: input.observations.clone(),
        history: input.history.clone(),
    };
    let request = match audit_request(default_pages, roots, &site) {
        Ok(request) => request,
        Err(error) => return ToolReply::error(error),
    };
    match run_audit(&request) {
        Ok(report) => match run_on_report(&input.query, &report) {
            Ok(result) => ToolReply::structured(result),
            Err(error) => ToolReply::error(error),
        },
        Err(error) => ToolReply::error(error.to_string()),
    }
}

fn tool_retrieve(
    default_pages: usize,
    roots: &Roots,
    input: &RetrieveInput,
    view: &str,
) -> ToolReply {
    let site = SiteInput {
        mode: input.mode.clone(),
        site: input.site.clone(),
        repo: input.repo.clone(),
        competitor: None,
        competitors: Vec::new(),
        max_pages: input.max_pages,
        workers: None,
        render: input.render.clone(),
        gsc: input.gsc.clone(),
        observations: input.observations.clone(),
        history: None,
    };
    let request = match audit_request(default_pages, roots, &site) {
        Ok(request) => request,
        Err(error) => return ToolReply::error(error),
    };
    let report = match run_audit(&request) {
        Ok(report) => report,
        Err(error) => return ToolReply::error(error.to_string()),
    };
    let limit = input.limit.unwrap_or(10);
    match view {
        "similar" => {
            let Some(url) = input.url.as_deref() else {
                return ToolReply::error("seo_similar requires url");
            };
            ToolReply::structured(similar(&report, url, limit))
        }
        "chunks" => {
            let query = input
                .query
                .as_deref()
                .or(input.url.as_deref())
                .unwrap_or("");
            ToolReply::structured(chunks_for(&report, query, limit))
        }
        _ => {
            let Some(query) = input.query.as_deref() else {
                return ToolReply::error("seo_retrieve requires query");
            };
            ToolReply::structured(retrieve(&report, query, limit))
        }
    }
}

fn tool_observations(roots: &Roots, input: &ObservationsInput) -> ToolReply {
    let (observations, gsc) = match observation_paths(roots, input) {
        Ok(paths) => paths,
        Err(error) => return ToolReply::error(error),
    };
    let loaded = match (observations.as_deref(), gsc.as_deref()) {
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

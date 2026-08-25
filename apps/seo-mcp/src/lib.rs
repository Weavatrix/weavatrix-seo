//! Agent MCP surface. Eight tools. No shell.

#![forbid(unsafe_code)]

use mcport::{ConcurrentMcpServer, RuntimeConfig, ToolReply, json};
use serde::Deserialize;
use std::time::Duration;
use weavatrix_seo::{AnalysisMode, AuditRequest, explain, plan_from, run_audit};
use weavatrix_seo_observation::unmeasured as observations_unmeasured;

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
    /// Reserved for URL-family scoping.
    #[serde(default)]
    #[allow(dead_code)]
    scope: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ExplainInput {
    id: String,
    #[serde(default)]
    site: Option<String>,
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

/// Eight-tool SEO server.
#[must_use]
pub fn seo_server(max_pages: usize) -> ConcurrentMcpServer {
    ConcurrentMcpServer::new("weavatrix-seo", env!("CARGO_PKG_VERSION"))
        .instructions(
            "Weavatrix SEO. Eight bounded tools. No shell. Missing evidence is unmeasured.",
        )
        .strict_schemas()
        .typed_tool(
            "seo_inventory",
            "Build the search surface inventory for a site, repo, or hybrid run.",
            schema_site(),
            move |_ctx, input: SiteInput| tool_audit(max_pages, &input, "inventory"),
        )
        .typed_tool(
            "seo_audit",
            "Return bounded findings by axis, severity, and evidence.",
            schema_site(),
            move |_ctx, input: SiteInput| tool_audit(max_pages, &input, "audit"),
        )
        .typed_tool(
            "seo_opportunities",
            "Return gaps and construction opportunities, not current errors.",
            schema_site(),
            move |_ctx, input: SiteInput| tool_audit(max_pages, &input, "opportunities"),
        )
        .typed_tool(
            "seo_plan",
            "Produce a target search-architecture plan with acceptance conditions.",
            schema_site(),
            move |_ctx, input: SiteInput| tool_audit(max_pages, &input, "plan"),
        )
        .typed_tool(
            "seo_compare",
            "Compare an owned site against public competitor origins.",
            schema_site(),
            move |_ctx, input: SiteInput| tool_audit(max_pages, &input, "compare"),
        )
        .typed_tool(
            "seo_diff",
            "Compare search surface between two revisions. Unmeasured until repo mode is wired.",
            schema_diff(),
            move |_ctx, input: DiffInput| {
                ToolReply::structured(json!({
                    "unmeasured": true,
                    "repo": input.repo,
                    "base": input.base,
                    "head": input.head,
                    "reason": "SEO diff requires the repository adapter."
                }))
            },
        )
        .typed_tool(
            "seo_explain",
            "Explain one finding or opportunity with its evidence chain.",
            schema_explain(),
            move |_ctx, input: ExplainInput| {
                let Some(site) = input.site else {
                    return ToolReply::error("seo_explain requires site");
                };
                let request = AuditRequest {
                    mode: AnalysisMode::Site,
                    site: Some(site),
                    repo: None,
                    competitors: Vec::new(),
                    max_pages: input.max_pages.or(Some(max_pages)),
                    workers: None,
                    ci: false,
                    baseline: None,
                    allow_private: false,
                };
                match run_audit(&request) {
                    Ok(report) => match explain(&report, &input.id) {
                        Some(finding) => ToolReply::structured(finding),
                        None => ToolReply::error(format!("unknown finding {}", input.id)),
                    },
                    Err(error) => ToolReply::error(error.to_string()),
                }
            },
        )
        .typed_tool(
            "seo_observations",
            "Query imported GSC, log, analytics, or AI-search evidence.",
            json!({
                "type": "object",
                "properties": {
                    "provider": { "type": "string", "description": "Optional provider name." }
                },
                "additionalProperties": false
            }),
            move |_ctx, _input: serde::de::IgnoredAny| {
                let snapshot = observations_unmeasured();
                ToolReply::structured(json!({
                    "connected": snapshot.connected,
                    "rows": snapshot.rows.len(),
                    "unmeasured": true
                }))
            },
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
        workers: None,
        ci: false,
        baseline: None,
        allow_private: false,
    };
    match run_audit(&request) {
        Ok(report) => match view {
            "inventory" => ToolReply::structured(&report.inventory),
            "opportunities" | "compare" => ToolReply::structured(&report.opportunities),
            "plan" => ToolReply::structured(plan_from(&report.opportunities)),
            _ => ToolReply::structured(&report),
        },
        Err(error) => ToolReply::error(error.to_string()),
    }
}

fn schema_site() -> mcport::Value {
    json!({
        "type": "object",
        "properties": {
            "mode": { "type": "string", "description": "site, repo, hybrid, or compare." },
            "site": { "type": "string", "description": "Absolute http(s) URL." },
            "repo": { "type": "string", "description": "Repository path." },
            "competitor": { "type": "string", "description": "Public competitor origin." },
            "competitors": {
                "type": "array",
                "items": { "type": "string" },
                "description": "Public competitor origins."
            },
            "max_pages": { "type": "integer", "minimum": 1, "description": "Crawl page cap." },
            "scope": { "type": "string", "description": "Optional URL glob." }
        },
        "additionalProperties": false
    })
}

fn schema_explain() -> mcport::Value {
    json!({
        "type": "object",
        "properties": {
            "id": { "type": "string", "description": "Finding fingerprint or code." },
            "site": { "type": "string", "description": "Site used to rebuild the audit." },
            "max_pages": { "type": "integer", "minimum": 1 }
        },
        "required": ["id"],
        "additionalProperties": false
    })
}

fn schema_diff() -> mcport::Value {
    json!({
        "type": "object",
        "properties": {
            "repo": { "type": "string" },
            "base": { "type": "string" },
            "head": { "type": "string" }
        },
        "additionalProperties": false
    })
}

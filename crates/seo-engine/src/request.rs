//! Audit invocation and crawl budget.

use serde::{Deserialize, Serialize};
use weavatrix_seo_crawl::{Crawl, CrawlBudget, CrawlConfig, CrawlError};
use weavatrix_seo_model::{
    AbsoluteUrl, AnalysisMode, Inventory, POLICY_VERSION, SeoError, config_digest, new_run_id,
};

/// Invocation for one engine run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditRequest {
    /// Analysis mode.
    pub mode: AnalysisMode,
    /// Site origin or seed URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site: Option<String>,
    /// Repository path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    /// Public competitor origins.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub competitors: Vec<String>,
    /// Page cap.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_pages: Option<usize>,
    /// Parallel fetch workers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workers: Option<usize>,
    /// Fail the process on error findings.
    #[serde(default)]
    pub ci: bool,
    /// Previous audit JSON or baseline artifact.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline: Option<String>,
    /// Local/staging opt-in. MCP defaults to false.
    #[serde(default = "default_allow_private")]
    pub allow_private: bool,
    /// Optional GSC export JSON.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gsc: Option<String>,
    /// Optional provider export (GSC, Bing, bot logs).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observations: Option<String>,
    /// Directory that receives a compact snapshot after the run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub history: Option<String>,
    /// Optional WVQ/Playwright render snapshot JSON (`weavatrix-seo-render/v1`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub render: Option<String>,
}

fn default_allow_private() -> bool {
    true
}

impl Default for AuditRequest {
    fn default() -> Self {
        Self {
            mode: AnalysisMode::Site,
            site: None,
            repo: None,
            competitors: Vec::new(),
            max_pages: None,
            workers: None,
            ci: false,
            baseline: None,
            allow_private: true,
            gsc: None,
            observations: None,
            history: None,
            render: None,
        }
    }
}

impl AuditRequest {
    /// Site-only request.
    #[must_use]
    pub fn site(url: impl Into<String>) -> Self {
        Self {
            site: Some(url.into()),
            ..Self::default()
        }
    }
}

/// Engine-level error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineError {
    /// Missing required input.
    Usage(String),
    /// URL parse error.
    Url(SeoError),
    /// Crawl error.
    Crawl(CrawlError),
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Usage(message) => write!(formatter, "{message}"),
            Self::Url(error) => write!(formatter, "{error}"),
            Self::Crawl(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for EngineError {}

pub fn budget(request: &AuditRequest) -> CrawlBudget {
    let mut budget = CrawlBudget::default();
    if let Some(max_pages) = request.max_pages {
        budget = budget.with_max_pages(max_pages);
    }
    if let Some(workers) = request.workers {
        budget = budget.with_workers(workers);
    }
    budget.allow_private = request.allow_private;
    budget
}

pub fn crawl_site(site: &str, budget: &CrawlBudget) -> Result<Inventory, EngineError> {
    let seed = AbsoluteUrl::parse(site).map_err(EngineError::Url)?;
    Crawl::new(CrawlConfig {
        seed,
        budget: budget.clone(),
    })
    .inventory()
    .map_err(EngineError::Crawl)
}

pub fn empty_repo_inventory(request: &AuditRequest) -> Inventory {
    let seed = request.repo.as_deref().unwrap_or("repo");
    Inventory::blank(AnalysisMode::Repo).bind_run(&new_run_id(seed), seed)
}

pub fn request_config_digest(request: &AuditRequest) -> String {
    config_digest(&[
        POLICY_VERSION,
        &format!("{:?}", request.mode),
        &request
            .max_pages
            .map_or_else(|| "default".into(), |n| n.to_string()),
        &request
            .workers
            .map_or_else(|| "default".into(), |n| n.to_string()),
        if request.allow_private {
            "private"
        } else {
            "public"
        },
        request.gsc.as_deref().unwrap_or("no-gsc"),
        request.observations.as_deref().unwrap_or("no-obs"),
        request.render.as_deref().unwrap_or("no-render"),
    ])
}

pub fn read_revision(repo: &str) -> Option<String> {
    let git = std::path::Path::new(repo).join(".git");
    let head = std::fs::read_to_string(git.join("HEAD")).ok()?;
    let trimmed = head.trim();
    if let Some(refer) = trimmed.strip_prefix("ref: ") {
        std::fs::read_to_string(git.join(refer.trim()))
            .ok()
            .map(|value| value.trim().to_owned())
    } else if trimmed.len() >= 7 {
        Some(trimmed.to_owned())
    } else {
        None
    }
}

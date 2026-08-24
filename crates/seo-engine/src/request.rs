//! Audit invocation and crawl budget.

use serde::{Deserialize, Serialize};
use weavatrix_seo_crawl::{Crawl, CrawlBudget, CrawlConfig, CrawlError};
use weavatrix_seo_model::{
    AbsoluteUrl, AnalysisMode, ContentHash, Inventory, InventoryCounts, SeoError,
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
    /// Previous audit JSON whose error fingerprints are the baseline.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline: Option<String>,
}

impl AuditRequest {
    /// Site-only request.
    #[must_use]
    pub fn site(url: impl Into<String>) -> Self {
        Self {
            mode: AnalysisMode::Site,
            site: Some(url.into()),
            repo: None,
            competitors: Vec::new(),
            max_pages: None,
            workers: None,
            ci: false,
            baseline: None,
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
            Self::Usage(message) | Self::Crawl(CrawlError::Transport(message)) => {
                write!(formatter, "{message}")
            }
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
    Inventory {
        mode: AnalysisMode::Repo,
        snapshot_id: ContentHash::of_str(request.repo.as_deref().unwrap_or("repo")).hex(),
        site: None,
        repo: request.repo.clone(),
        hosts: Vec::new(),
        pages: Vec::new(),
        edges: Vec::new(),
        predicted_routes: Vec::new(),
        sitemap_discovered: 0,
        counts: InventoryCounts {
            crawled: 0,
            fetched: 0,
            redirected: 0,
            errors: 0,
            sitemap_urls: 0,
            indexable: 0,
        },
    }
}

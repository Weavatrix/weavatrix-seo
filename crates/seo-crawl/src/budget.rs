//! Crawl bounds. Missing bounds are refused rather than unbounded.

use std::time::Duration;
use weavatrix_seo_http::{FetchBudget, NetworkPolicy};

/// Hard limits for one crawl snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrawlBudget {
    /// Maximum URLs whose bodies are extracted.
    pub max_pages: usize,
    /// Maximum link depth from the seed.
    pub max_depth: u32,
    /// Maximum redirect hops per URL.
    pub max_redirects: u32,
    /// Maximum response body bytes.
    pub max_body_bytes: usize,
    /// Socket timeout.
    pub timeout: Duration,
    /// User-Agent product token.
    pub user_agent: String,
    /// Parallel fetch workers. `1` is sequential.
    pub workers: usize,
    /// When false, loopback/private/metadata destinations are refused.
    pub allow_private: bool,
}

impl Default for CrawlBudget {
    fn default() -> Self {
        Self {
            max_pages: 200,
            max_depth: 8,
            max_redirects: 5,
            max_body_bytes: 1_048_576,
            timeout: Duration::from_secs(15),
            user_agent: format!("weavatrix-seo/{}", env!("CARGO_PKG_VERSION")),
            workers: 5,
            allow_private: true,
        }
    }
}

impl CrawlBudget {
    /// Overrides the page cap.
    #[must_use]
    pub const fn with_max_pages(mut self, max_pages: usize) -> Self {
        self.max_pages = max_pages;
        self
    }

    /// Overrides fetch worker count. Values below 1 become 1.
    #[must_use]
    pub const fn with_workers(mut self, workers: usize) -> Self {
        self.workers = if workers == 0 { 1 } else { workers };
        self
    }

    /// Public-only network policy (MCP / competitor).
    #[must_use]
    pub const fn public_only(mut self) -> Self {
        self.allow_private = false;
        self
    }

    /// Transport budget for `weavatrix-seo-http`.
    #[must_use]
    pub fn fetch_budget(&self) -> FetchBudget {
        FetchBudget {
            max_redirects: self.max_redirects,
            max_body_bytes: self.max_body_bytes,
            timeout: self.timeout,
            user_agent: self.user_agent.clone(),
            pool_size: self.workers.max(1),
            policy: if self.allow_private {
                NetworkPolicy::allow_private()
            } else {
                NetworkPolicy::public_only()
            },
            max_retries: 2,
        }
    }
}

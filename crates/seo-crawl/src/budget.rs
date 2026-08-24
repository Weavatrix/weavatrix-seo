//! Crawl bounds. Missing bounds are refused rather than unbounded.

use std::time::Duration;

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
}

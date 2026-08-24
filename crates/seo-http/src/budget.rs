//! Fetch limits. Page caps live in the crawl crate.

use std::time::Duration;

/// Socket, redirect, and body bounds for one fetcher.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchBudget {
    /// Maximum redirect hops per URL.
    pub max_redirects: u32,
    /// Maximum decoded body bytes.
    pub max_body_bytes: usize,
    /// Connect and read timeout.
    pub timeout: Duration,
    /// User-Agent product token.
    pub user_agent: String,
    /// Idle keep-alive sockets to retain.
    pub pool_size: usize,
}

impl Default for FetchBudget {
    fn default() -> Self {
        Self {
            max_redirects: 5,
            max_body_bytes: 1_048_576,
            timeout: Duration::from_secs(15),
            user_agent: format!("weavatrix-seo/{}", env!("CARGO_PKG_VERSION")),
            pool_size: 5,
        }
    }
}

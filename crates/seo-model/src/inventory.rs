//! Search-surface inventory.

use crate::{AbsoluteUrl, ExtractedPage, GraphEdge};
use serde::{Deserialize, Serialize};

/// How the run was invoked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisMode {
    /// Public or staging site only.
    Site,
    /// Repository only.
    Repo,
    /// Repository plus live/staging site.
    Hybrid,
    /// Owned site versus public competitor sites.
    Compare,
}

/// Compact inventory totals.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InventoryCounts {
    /// URLs requested during the crawl.
    pub crawled: usize,
    /// Final success responses.
    pub fetched: usize,
    /// Redirected URLs.
    pub redirected: usize,
    /// 4xx/5xx.
    pub errors: usize,
    /// Sitemap loc entries discovered.
    pub sitemap_urls: usize,
    /// Pages classified indexable from response signals.
    pub indexable: usize,
}

/// Complete site/repo/hybrid inventory for one run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Inventory {
    /// Run mode.
    pub mode: AnalysisMode,
    /// Snapshot identity.
    pub snapshot_id: String,
    /// Seed site when present.
    pub site: Option<String>,
    /// Seed repository when present.
    pub repo: Option<String>,
    /// Hosts in scope.
    pub hosts: Vec<String>,
    /// Extracted pages keyed by final URL string.
    pub pages: Vec<ExtractedPage>,
    /// Graph edges.
    pub edges: Vec<GraphEdge>,
    /// Route patterns predicted from source.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub predicted_routes: Vec<String>,
    /// Loc entries discovered from sitemaps, before page cap.
    #[serde(default)]
    pub sitemap_discovered: usize,
    /// Totals.
    pub counts: InventoryCounts,
}

impl Inventory {
    /// Page matching a URL string.
    #[must_use]
    pub fn page(&self, url: &AbsoluteUrl) -> Option<&ExtractedPage> {
        self.pages.iter().find(|page| page.url == *url)
    }

    /// Rebuilds counts from pages.
    #[must_use]
    pub fn with_counts(mut self) -> Self {
        self.counts = InventoryCounts {
            crawled: self.pages.len(),
            fetched: self
                .pages
                .iter()
                .filter(|page| (200..400).contains(&page.status))
                .count(),
            redirected: self
                .pages
                .iter()
                .filter(|page| !page.redirects.is_empty())
                .count(),
            errors: self.pages.iter().filter(|page| page.status >= 400).count(),
            sitemap_urls: if self.sitemap_discovered == 0 {
                self.pages.iter().filter(|page| page.in_sitemap).count()
            } else {
                self.sitemap_discovered
            },
            indexable: self
                .pages
                .iter()
                .filter(|page| page.indexability == crate::Indexability::Indexable)
                .count(),
        };
        self
    }
}

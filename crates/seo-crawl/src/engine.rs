//! Deterministic BFS crawl over one origin.

use crate::assemble::{mark_inbound, page, record_links};
use crate::discover::{fetch_robots, fetch_sitemaps};
use crate::extract::extract_html;
use crate::frontier::Frontier;
use crate::schedule::{fetch_batch, pop_allowed};
use crate::{CrawlBudget, Fetcher, Result};
use std::collections::BTreeSet;
use weavatrix_seo_model::{
    AbsoluteUrl, AnalysisMode, ContentHash, Evidence, GraphEdge, Inventory, InventoryCounts,
    Relation,
};

/// Crawl invocation.
#[derive(Debug, Clone)]
pub struct CrawlConfig {
    /// Seed URL.
    pub seed: AbsoluteUrl,
    /// Budget.
    pub budget: CrawlBudget,
}

/// Prepared crawl.
#[derive(Debug, Clone)]
pub struct Crawl {
    config: CrawlConfig,
}

impl Crawl {
    /// Builds a crawl.
    #[must_use]
    pub const fn new(config: CrawlConfig) -> Self {
        Self { config }
    }

    /// Runs a site-only inventory.
    ///
    /// # Errors
    ///
    /// Returns a transport error when the seed cannot be fetched at all.
    pub fn inventory(&self) -> Result<Inventory> {
        let fetcher = Fetcher::new(self.config.budget.fetch_budget());
        let seed = &self.config.seed;
        let robots = fetch_robots(&fetcher, seed);
        let sitemap_urls = fetch_sitemaps(&fetcher, seed, &robots);
        let sitemap_set: BTreeSet<AbsoluteUrl> = sitemap_urls.iter().cloned().collect();
        let mut frontier = Frontier::default();
        frontier.seed(seed.clone());
        for url in &sitemap_urls {
            frontier.push_sitemap(url.clone());
        }
        let mut pages = Vec::new();
        let mut edges = Vec::new();
        for url in &sitemap_urls {
            edges.push(GraphEdge::new(
                seed.clone(),
                url.clone(),
                Relation::ListedInSitemap,
                Evidence::sitemap(),
            ));
        }
        let workers = self.config.budget.workers.max(1);
        while pages.len() < self.config.budget.max_pages {
            let remaining = self.config.budget.max_pages - pages.len();
            let batch = pop_allowed(&mut frontier, &robots, workers.min(remaining));
            if batch.is_empty() {
                break;
            }
            for (url, depth, fetched) in fetch_batch(&fetcher, batch) {
                let Ok(response) = fetched else {
                    continue;
                };
                let draft = extract_html(&response.body);
                let in_sitemap = sitemap_set.contains(&response.url) || sitemap_set.contains(&url);
                let extracted = page(&response, draft, in_sitemap).finalize();
                record_links(
                    &extracted,
                    seed,
                    depth,
                    self.config.budget.max_depth,
                    &mut frontier,
                    &mut edges,
                );
                pages.push(extracted);
            }
        }
        mark_inbound(&mut pages, &edges);
        pages.sort_by(|left, right| left.url.to_string().cmp(&right.url.to_string()));
        Ok(Inventory {
            mode: AnalysisMode::Site,
            snapshot_id: ContentHash::of_str(&seed.to_string()).hex(),
            site: Some(seed.to_string()),
            repo: None,
            hosts: vec![seed.host().to_owned()],
            pages,
            edges,
            predicted_routes: Vec::new(),
            sitemap_discovered: sitemap_set.len(),
            counts: InventoryCounts {
                crawled: 0,
                fetched: 0,
                redirected: 0,
                errors: 0,
                sitemap_urls: sitemap_set.len(),
                indexable: 0,
            },
        }
        .with_counts())
    }
}

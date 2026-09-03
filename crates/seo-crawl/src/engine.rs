//! Deterministic BFS crawl over one origin.

use crate::assemble::{mark_inbound, page, record_links, redirect_page};
use crate::discover::{fetch_llms_txt, fetch_robots, fetch_sitemaps};
use crate::extract::extract_html;
use crate::frontier::Frontier;
use crate::schedule::fetch_batch;
use crate::{CrawlBudget, Fetcher, Result};
use std::collections::{BTreeMap, BTreeSet};
use weavatrix_seo_model::{
    AbsoluteUrl, AiSurface, AnalysisMode, DiscoverySource, Evidence, ExtractedPage,
    FetchObservation, FetchOutcome, GraphEdge, Inventory, MediaKind, Relation, new_run_id,
};

/// Crawl invocation.
#[derive(Debug, Clone)]
pub struct CrawlConfig {
    /// Seed URL.
    pub seed: AbsoluteUrl,
    /// Budget.
    pub budget: CrawlBudget,
    /// Extra URLs from GSC, logs, citations, or a previous snapshot.
    pub extra_seeds: Vec<(AbsoluteUrl, DiscoverySource)>,
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

    /// Runs a site-only inventory. Fetch failures stay as observations.
    ///
    /// # Errors
    ///
    /// Returns a crawl error only when the seed URL is unusable as identity.
    #[allow(clippy::too_many_lines)]
    pub fn inventory(&self) -> Result<Inventory> {
        let fetcher = Fetcher::new(self.config.budget.fetch_budget());
        let seed = &self.config.seed;
        let run_id = new_run_id(&seed.to_string());
        let robots = fetch_robots(&fetcher, seed);
        let ai_surface = AiSurface {
            llms_txt_status: fetch_llms_txt(&fetcher, seed),
            robots_disallow_all: robots.ai_disallow_all.clone(),
            agent_matrix: robots.agent_matrix(seed),
        };
        let sitemap_urls = fetch_sitemaps(&fetcher, seed, &robots);
        let sitemap_set: BTreeSet<AbsoluteUrl> = sitemap_urls.iter().cloned().collect();
        let mut frontier = Frontier::default();
        let mut discovery: BTreeMap<String, DiscoverySource> = BTreeMap::new();
        frontier.seed(seed.clone());
        remember(&mut discovery, seed, DiscoverySource::Explicit);
        for url in &sitemap_urls {
            frontier.push_sitemap(url.clone());
            remember(&mut discovery, url, DiscoverySource::Sitemap);
        }
        for (url, source) in &self.config.extra_seeds {
            if url.host() != seed.host() {
                continue;
            }
            frontier.push_observed(url.clone());
            remember(&mut discovery, url, *source);
        }
        let mut pages = Vec::new();
        let mut edges = Vec::new();
        let mut observations = Vec::new();
        for url in &sitemap_urls {
            edges.push(GraphEdge::new(
                seed.clone(),
                url.clone(),
                Relation::ListedInSitemap,
                Evidence::sitemap(),
            ));
        }
        let workers = self.config.budget.workers.max(1);
        while pages.len() + observations.len() < self.config.budget.max_pages {
            let remaining = self.config.budget.max_pages - pages.len() - observations.len();
            let raw = frontier.pop_batch(workers.min(remaining));
            if raw.is_empty() {
                break;
            }
            let mut batch = Vec::new();
            for (url, depth) in raw {
                if robots.allows(&url) {
                    batch.push((url, depth));
                } else {
                    observations.push(FetchObservation::new(
                        url.to_string(),
                        FetchOutcome::RobotsBlocked,
                        "robots.txt",
                    ));
                }
            }
            for (url, depth, fetched) in fetch_batch(&fetcher, batch) {
                match fetched {
                    Ok(response) => record_fetch(
                        seed,
                        &url,
                        depth,
                        &response,
                        &sitemap_set,
                        self.config.budget.max_depth,
                        &mut frontier,
                        &mut pages,
                        &mut edges,
                    ),
                    Err(error) => observations.push(FetchObservation::new(
                        url.to_string(),
                        error.outcome(),
                        error.to_string(),
                    )),
                }
            }
        }
        mark_inbound(&mut pages, &edges);
        for page in &pages {
            if page.linked_from_page {
                remember(&mut discovery, &page.url, DiscoverySource::InternalLink);
            }
        }
        pages.sort_by(|left, right| left.url.to_string().cmp(&right.url.to_string()));
        Ok(Inventory {
            mode: AnalysisMode::Site,
            snapshot_id: String::new(),
            run_id: String::new(),
            policy_version: String::new(),
            semantics: None,
            config_digest: String::new(),
            repo_revision: None,
            site: Some(seed.to_string()),
            repo: None,
            hosts: vec![seed.host().to_owned()],
            pages,
            edges,
            nodes: Vec::new(),
            facts: Vec::new(),
            observations,
            predicted_routes: Vec::new(),
            producers: Vec::new(),
            policy: None,
            policy_error: None,
            sitemap_discovered: sitemap_set.len(),
            counts: weavatrix_seo_model::InventoryCounts::default(),
            discovery,
            ai_surface: Some(ai_surface),
        }
        .bind_run(&run_id, &seed.to_string())
        .with_counts())
    }
}

fn remember(
    discovery: &mut BTreeMap<String, DiscoverySource>,
    url: &AbsoluteUrl,
    source: DiscoverySource,
) {
    let key = url.to_string();
    discovery
        .entry(key)
        .and_modify(|current| *current = current.stronger(source))
        .or_insert(source);
}

#[allow(clippy::too_many_arguments)]
fn record_fetch(
    seed: &AbsoluteUrl,
    requested: &AbsoluteUrl,
    depth: u32,
    response: &crate::FetchResponse,
    sitemap_set: &BTreeSet<AbsoluteUrl>,
    max_depth: u32,
    frontier: &mut Frontier,
    pages: &mut Vec<ExtractedPage>,
    edges: &mut Vec<GraphEdge>,
) {
    for hop in &response.redirects {
        let Ok(from) = AbsoluteUrl::parse(&hop.from) else {
            continue;
        };
        let Ok(to) = AbsoluteUrl::parse(&hop.to) else {
            continue;
        };
        frontier.remember(from.clone());
        if !pages.iter().any(|page| page.url == from) {
            let in_sitemap = sitemap_set.contains(&from);
            pages.push(redirect_page(&from, &to, hop.status, in_sitemap));
            edges.push(GraphEdge::new(
                from,
                to,
                Relation::RedirectsTo,
                Evidence::http(),
            ));
        }
    }
    frontier.remember(response.url.clone());
    if pages.iter().any(|page| page.url == response.url) {
        return;
    }
    if !seed.same_origin(&response.url) {
        return;
    }
    let media = MediaKind::classify(response.header("content-type"), &response.body);
    let draft = if media.is_html() {
        extract_html(&response.body)
    } else {
        crate::extract::ExtractedPageDraft::default()
    };
    let in_sitemap = sitemap_set.contains(&response.url) || sitemap_set.contains(requested);
    let extracted = page(response, draft, in_sitemap).finalize();
    record_links(&extracted, seed, depth, max_depth, frontier, edges);
    pages.push(extracted);
}

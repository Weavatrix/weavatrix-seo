//! Deterministic BFS crawl over one origin.

use crate::extract::{ExtractedPageDraft, extract_html};
use crate::frontier::Frontier;
use crate::{CrawlBudget, FetchResponse, Fetcher, Result, Robots, parse_sitemap};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use weavatrix_seo_model::{
    AbsoluteUrl, AnalysisMode, ContentHash, Evidence, ExtractedPage, GraphEdge, Indexability,
    Inventory, InventoryCounts, Relation,
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
        let fetcher = Fetcher::new(self.config.budget.clone());
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
        while let Some((url, depth)) = frontier.pop() {
            if pages.len() >= self.config.budget.max_pages {
                break;
            }
            if !robots.allows(&url) {
                continue;
            }
            let Ok(response) = fetcher.get(&url) else {
                continue;
            };
            let draft = extract_html(&response.body);
            let in_sitemap = sitemap_set.contains(&response.url) || sitemap_set.contains(&url);
            let page = assemble(&response, draft, in_sitemap).finalize();
            record_links(
                &page,
                seed,
                depth,
                self.config.budget.max_depth,
                &mut frontier,
                &mut edges,
            );
            pages.push(page);
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

fn fetch_robots(fetcher: &Fetcher, seed: &AbsoluteUrl) -> Robots {
    let Ok(url) = AbsoluteUrl::parse(&format!("{}/robots.txt", seed.origin())) else {
        return Robots::default();
    };
    fetcher
        .get(&url)
        .ok()
        .filter(|response| response.status == 200)
        .map_or_else(Robots::default, |response| {
            Robots::parse(&response.body, "weavatrix-seo")
        })
}

fn fetch_sitemaps(fetcher: &Fetcher, seed: &AbsoluteUrl, robots: &Robots) -> Vec<AbsoluteUrl> {
    let mut declared = robots.sitemaps.clone();
    if declared.is_empty() {
        declared.push(format!("{}/sitemap.xml", seed.origin()));
        declared.push(format!("{}/sitemap_index.xml", seed.origin()));
    }
    let mut queue = VecDeque::new();
    for item in declared {
        if let Ok(url) = AbsoluteUrl::parse(&item).or_else(|_| seed.join(&item)) {
            queue.push_back(url);
        }
    }
    let mut visited = BTreeSet::new();
    let mut locs = Vec::new();
    let mut documents = 0_usize;
    while let Some(url) = queue.pop_front() {
        if documents >= 64 || !visited.insert(url.clone()) {
            continue;
        }
        let Ok(response) = fetcher.get(&url) else {
            continue;
        };
        if response.status != 200 {
            continue;
        }
        documents += 1;
        let body = &response.body;
        let nested = parse_sitemap(body, seed);
        if is_sitemap_index(body) {
            for child in nested {
                queue.push_back(child);
            }
            continue;
        }
        if is_urlset(body) || !nested.is_empty() {
            locs.extend(nested);
        }
    }
    locs.sort();
    locs.dedup();
    locs
}

fn is_sitemap_index(body: &str) -> bool {
    body.contains("<sitemapindex") || body.contains("<sitemapindex ")
}

fn is_urlset(body: &str) -> bool {
    body.contains("<urlset")
}

fn assemble(fetched: &FetchResponse, draft: ExtractedPageDraft, in_sitemap: bool) -> ExtractedPage {
    let mut robots = draft.robots;
    if let Some(header) = fetched.header("x-robots-tag") {
        robots.push(header.to_owned());
    }
    ExtractedPage {
        url: fetched.url.clone(),
        requested: fetched.requested.clone(),
        status: fetched.status,
        redirects: fetched.redirects.clone(),
        content_type: fetched.header("content-type").map(ToOwned::to_owned),
        canonical: draft.canonical,
        robots,
        title: draft.title,
        description: draft.description,
        html_lang: draft.html_lang,
        alternates: draft.alternates,
        headings: draft.headings,
        links: draft.links,
        images: draft.images,
        json_ld: draft.json_ld,
        text: draft.text,
        payload: draft.payload,
        content_hash: ContentHash::of(&[]),
        indexability: Indexability::Indexable,
        in_sitemap,
        linked_from_page: false,
        evidence: Evidence::http(),
    }
}

fn record_links(
    page: &ExtractedPage,
    seed: &AbsoluteUrl,
    depth: u32,
    max_depth: u32,
    frontier: &mut Frontier,
    edges: &mut Vec<GraphEdge>,
) {
    if let Some(canonical) = &page.canonical
        && let Ok(target) = AbsoluteUrl::parse(canonical).or_else(|_| page.url.join(canonical))
    {
        edges.push(GraphEdge::new(
            page.url.clone(),
            target,
            Relation::CanonicalTo,
            Evidence::http(),
        ));
    }
    for href in &page.links {
        let Ok(target) = page.url.join(href) else {
            continue;
        };
        if !seed.same_origin(&target) {
            continue;
        }
        edges.push(GraphEdge::new(
            page.url.clone(),
            target.clone(),
            Relation::LinksTo,
            Evidence::http(),
        ));
        if depth < max_depth {
            frontier.push_link(target, depth + 1);
        }
    }
}

fn mark_inbound(pages: &mut [ExtractedPage], edges: &[GraphEdge]) {
    let mut inbound = BTreeMap::new();
    for edge in edges
        .iter()
        .filter(|edge| edge.relation == Relation::LinksTo)
    {
        *inbound.entry(edge.target.clone()).or_insert(0_usize) += 1;
    }
    for page in pages {
        page.linked_from_page = inbound.get(&page.url).copied().unwrap_or(0) > 0;
        if page.url.path() == "/" {
            page.linked_from_page = true;
        }
    }
}

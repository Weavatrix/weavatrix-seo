//! Depth, orphans, and internal authority.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use weavatrix_graph::{
    Confidence, Edge, EdgeKind, EvidenceKind, GraphBuilder, Node, NodeKind, Provenance, page_rank,
};
use weavatrix_seo_model::{
    AbsoluteUrl, Evidence, Finding, FindingFamily, Indexability, Inventory, Locator, Relation,
    Severity,
};

/// Architecture metrics for one URL.
#[derive(Debug, Clone, PartialEq)]
pub struct PageArchitecture {
    /// URL.
    pub url: AbsoluteUrl,
    /// Shortest internal-link depth from the seed. `None` when unreachable.
    pub depth: Option<u32>,
    /// Inbound internal links.
    pub inbound: usize,
    /// Outbound internal links.
    pub outbound: usize,
    /// PageRank-like internal authority.
    pub authority: f64,
    /// Indexable and not internally reachable from the seed.
    pub orphan: bool,
}

/// Architecture pass.
#[derive(Debug, Clone, PartialEq)]
pub struct Architecture {
    /// Per-page metrics, sorted by URL.
    pub pages: Vec<PageArchitecture>,
}

/// Computes architecture and emits orphan/depth findings.
#[must_use]
pub fn analyze(inventory: &Inventory) -> (Architecture, Vec<Finding>) {
    let seed = inventory
        .site
        .as_deref()
        .and_then(|site| AbsoluteUrl::parse(site).ok());
    let depths = depths(inventory, seed.as_ref());
    let inbound = counts(inventory, true);
    let outbound = counts(inventory, false);
    let authority = authority(inventory);
    let mut pages = Vec::new();
    let mut findings = Vec::new();
    for page in &inventory.pages {
        let depth = depths.get(&page.url).copied();
        let inbound_count = inbound.get(&page.url).copied().unwrap_or(0);
        let orphan = page.indexability == Indexability::Indexable
            && page.status == 200
            && depth.is_none()
            && page.url.path() != "/";
        let metrics = PageArchitecture {
            url: page.url.clone(),
            depth,
            inbound: inbound_count,
            outbound: outbound.get(&page.url).copied().unwrap_or(0),
            authority: authority.get(&page.url).copied().unwrap_or(0.0),
            orphan,
        };
        if orphan {
            findings.push(
                Finding::new(
                    FindingFamily::Link,
                    2,
                    Severity::Warn,
                    &page.url.to_string(),
                    format!("{} is an orphan indexable URL", page.url),
                    Locator::url(&page.url),
                    Evidence::http(),
                )
                .explained(
                    "The URL is not reachable by internal links from the seed.",
                    "Add an internal link from a crawlable template.",
                    "Shortest internal path from the homepage is finite.",
                ),
            );
        }
        if let Some(depth) = depth
            && depth > 3
            && page.indexability == Indexability::Indexable
        {
            findings.push(
                Finding::new(
                    FindingFamily::Link,
                    3,
                    Severity::Info,
                    &page.url.to_string(),
                    format!("{} is {depth} hops from the seed", page.url),
                    Locator::url(&page.url),
                    Evidence::http(),
                )
                .explained(
                    "Deep URLs are harder to discover.",
                    "Raise internal links toward this page family.",
                    "Depth from the seed is three hops or fewer.",
                ),
            );
        }
        pages.push(metrics);
    }
    (Architecture { pages }, findings)
}

fn depths(inventory: &Inventory, seed: Option<&AbsoluteUrl>) -> BTreeMap<AbsoluteUrl, u32> {
    let Some(seed) = seed else {
        return BTreeMap::new();
    };
    let mut adj: BTreeMap<AbsoluteUrl, Vec<AbsoluteUrl>> = BTreeMap::new();
    for edge in inventory
        .edges
        .iter()
        .filter(|edge| edge.relation == Relation::LinksTo)
    {
        adj.entry(edge.source.clone())
            .or_default()
            .push(edge.target.clone());
    }
    let mut depths = BTreeMap::from([(seed.clone(), 0_u32)]);
    let mut queue = VecDeque::from([seed.clone()]);
    while let Some(url) = queue.pop_front() {
        let depth = depths[&url];
        for next in adj.get(&url).into_iter().flatten() {
            if depths.contains_key(next) {
                continue;
            }
            depths.insert(next.clone(), depth + 1);
            queue.push_back(next.clone());
        }
    }
    depths
}

fn counts(inventory: &Inventory, inbound: bool) -> BTreeMap<AbsoluteUrl, usize> {
    let mut counts = BTreeMap::new();
    for edge in inventory
        .edges
        .iter()
        .filter(|edge| edge.relation == Relation::LinksTo)
    {
        let key = if inbound {
            edge.target.clone()
        } else {
            edge.source.clone()
        };
        *counts.entry(key).or_insert(0) += 1;
    }
    counts
}

fn authority(inventory: &Inventory) -> BTreeMap<AbsoluteUrl, f64> {
    let mut builder = GraphBuilder::new();
    let mut ids = BTreeSet::new();
    for page in &inventory.pages {
        ids.insert(page.url.to_string());
    }
    for edge in inventory
        .edges
        .iter()
        .filter(|edge| edge.relation == Relation::LinksTo)
    {
        ids.insert(edge.source.to_string());
        ids.insert(edge.target.to_string());
    }
    for id in &ids {
        let Ok(node) = Node::new(id.clone(), id.clone(), NodeKind::Endpoint) else {
            continue;
        };
        let _ = builder.add_node(node);
    }
    let Ok(provenance) = Provenance::new(
        "weavatrix-seo-architecture",
        EvidenceKind::Extracted,
        Confidence::Exact,
    ) else {
        return BTreeMap::new();
    };
    for edge in inventory
        .edges
        .iter()
        .filter(|edge| edge.relation == Relation::LinksTo)
    {
        let Ok(source) = weavatrix_graph::NodeId::new(edge.source.to_string()) else {
            continue;
        };
        let Ok(target) = weavatrix_graph::NodeId::new(edge.target.to_string()) else {
            continue;
        };
        let _ = builder.add_edge(Edge::new(
            source,
            target,
            EdgeKind::References,
            provenance.clone(),
        ));
    }
    let Ok(graph) = builder.build() else {
        return BTreeMap::new();
    };
    let Ok(ranks) = page_rank(&graph, 0.85, 20) else {
        return BTreeMap::new();
    };
    let mut out = BTreeMap::new();
    for (index, score) in ranks {
        if let Some(node) = graph.node_at(index)
            && let Ok(url) = AbsoluteUrl::parse(node.id.as_str())
        {
            out.insert(url, score);
        }
    }
    out
}

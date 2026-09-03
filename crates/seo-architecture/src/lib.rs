//! Depth, orphans, and internal authority.

#![forbid(unsafe_code)]

mod rank;
mod template;

use std::collections::{BTreeMap, VecDeque};
use weavatrix_seo_model::{
    AbsoluteUrl, Evidence, Finding, FindingFamily, Indexability, Inventory, LinkLocation, Locator,
    Relation,
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
    /// Weighted internal `PageRank`. Body links count more than chrome.
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
    let authority = rank::weighted(inventory);
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
                Finding::from_rule(
                    FindingFamily::Link,
                    2,
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
                Finding::from_rule(
                    FindingFamily::Link,
                    3,
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
    findings.extend(equity_leaks(inventory));
    (Architecture { pages }, findings)
}

fn equity_leaks(inventory: &Inventory) -> Vec<Finding> {
    let pages: BTreeMap<&AbsoluteUrl, &weavatrix_seo_model::ExtractedPage> = inventory
        .pages
        .iter()
        .map(|page| (&page.url, page))
        .collect();
    let mut findings = Vec::new();
    for edge in inventory
        .edges
        .iter()
        .filter(|edge| edge.relation == Relation::LinksTo)
    {
        let location = edge.location.unwrap_or(LinkLocation::Contextual);
        if !matches!(location, LinkLocation::Nav | LinkLocation::Footer) {
            continue;
        }
        let Some(target) = pages.get(&edge.target) else {
            continue;
        };
        let leak = target.status >= 400 || target.indexability != Indexability::Indexable;
        if !leak {
            continue;
        }
        findings.push(
            Finding::from_rule(
                FindingFamily::Link,
                5,
                &edge.source.to_string(),
                format!(
                    "{} {} link to {} leaks internal equity",
                    edge.source,
                    location.as_str(),
                    edge.target
                ),
                Locator::url(&edge.source),
                Evidence::http(),
            )
            .explained(
                "Navigation and footer links pass residual authority. Pointing them at errors or noindex pages wastes that equity.",
                "Point chrome links at live indexable URLs, or drop the link.",
                "The target is 200 and indexable.",
            ),
        );
    }
    findings
}

/// Marks repeated template links on the crawl graph.
pub fn annotate_templates(inventory: &mut Inventory) {
    template::annotate(inventory);
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

#[cfg(test)]
mod tests {
    use super::analyze;
    use weavatrix_seo_model::{
        AbsoluteUrl, AnalysisMode, ContentHash, Evidence, ExtractedPage, GraphEdge, Heading,
        Inventory, LinkLocation, MediaKind, Relation,
    };

    fn url(path: &str) -> AbsoluteUrl {
        AbsoluteUrl::parse(&format!("https://x.test{path}")).expect("url")
    }

    fn page(path: &str, status: u16) -> ExtractedPage {
        let parsed = url(path);
        ExtractedPage {
            url: parsed.clone(),
            requested: parsed,
            status,
            redirects: Vec::new(),
            content_type: Some("text/html".into()),
            media: MediaKind::Html,
            canonical: None,
            robots: Vec::new(),
            title: Some(path.into()),
            description: None,
            html_lang: Some("en".into()),
            alternates: Vec::new(),
            headings: vec![Heading {
                level: 1,
                text: path.into(),
            }],
            links: Vec::new(),
            link_refs: Vec::new(),
            images: Vec::new(),
            json_ld: Vec::new(),
            text: path.into(),
            heading_text: path.into(),
            main_text: String::new(),
            payload: String::new(),
            arbitrary_script: String::new(),
            og_title: None,
            og_description: None,
            og_image: None,
            headers: Vec::new(),
            csp_meta: None,
            body_bytes: 1,
            fetch_ms: 1,
            has_main: true,
            unlabeled_controls: 0,
            content_hash: ContentHash::of_str(path),
            indexability: weavatrix_seo_model::Indexability::Indexable,
            in_sitemap: true,
            linked_from_page: true,
            evidence: Evidence::http(),
        }
        .finalize()
    }

    fn edge(from: &str, to: &str, location: LinkLocation) -> GraphEdge {
        GraphEdge::new(url(from), url(to), Relation::LinksTo, Evidence::http()).with_link(
            None,
            None,
            Some(location),
            None,
        )
    }

    #[test]
    fn nav_to_error_leaks_equity() {
        let mut inventory = Inventory::blank(AnalysisMode::Site);
        inventory.site = Some("https://x.test/".into());
        inventory.pages = vec![page("/", 200), page("/gone", 404)];
        inventory.edges = vec![edge("/", "/gone", LinkLocation::Nav)];
        let (_architecture, findings) = analyze(&inventory);
        let leak = findings
            .iter()
            .find(|finding| finding.code == "WVX-SEO-LINK-005")
            .expect("equity leak");
        assert!(leak.summary.contains("nav"), "{}", leak.summary);
        assert!(!leak.summary.contains("Nav"), "{}", leak.summary);
    }

    #[test]
    fn contextual_links_outrank_footer_chrome() {
        let mut inventory = Inventory::blank(AnalysisMode::Site);
        inventory.site = Some("https://x.test/".into());
        inventory.pages = vec![page("/", 200), page("/body", 200), page("/chrome", 200)];
        inventory.edges = vec![
            edge("/", "/body", LinkLocation::Contextual),
            edge("/", "/chrome", LinkLocation::Footer),
        ];
        let (architecture, _) = analyze(&inventory);
        let body = architecture
            .pages
            .iter()
            .find(|item| item.url.path() == "/body")
            .expect("body");
        let chrome = architecture
            .pages
            .iter()
            .find(|item| item.url.path() == "/chrome")
            .expect("chrome");
        assert!(
            body.authority > chrome.authority,
            "body {} vs chrome {}",
            body.authority,
            chrome.authority
        );
    }
}

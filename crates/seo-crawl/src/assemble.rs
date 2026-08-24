//! Page assembly from a fetch + HTML draft.

use crate::extract::ExtractedPageDraft;
use crate::frontier::Frontier;
use crate::FetchResponse;
use std::collections::BTreeMap;
use weavatrix_seo_model::{
    AbsoluteUrl, ContentHash, Evidence, ExtractedPage, GraphEdge, Indexability, Relation,
};

const KEPT_HEADERS: &[&str] = &[
    "cache-control",
    "content-encoding",
    "content-security-policy",
    "permissions-policy",
    "referrer-policy",
    "strict-transport-security",
    "x-content-type-options",
    "x-frame-options",
    "x-robots-tag",
];

pub fn page(fetched: &FetchResponse, draft: ExtractedPageDraft, in_sitemap: bool) -> ExtractedPage {
    let mut robots = draft.robots;
    if let Some(header) = fetched.header("x-robots-tag") {
        robots.push(header.to_owned());
    }
    let headers = fetched
        .headers
        .iter()
        .filter(|(name, _)| KEPT_HEADERS.contains(&name.as_str()))
        .cloned()
        .collect();
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
        og_title: draft.og_title,
        og_description: draft.og_description,
        og_image: draft.og_image,
        headers,
        body_bytes: fetched.body.len(),
        fetch_ms: fetched.fetch_ms,
        has_main: draft.has_main,
        unlabeled_controls: draft.unlabeled_controls,
        content_hash: ContentHash::of(&[]),
        indexability: Indexability::Indexable,
        in_sitemap,
        linked_from_page: false,
        evidence: Evidence::http(),
    }
}

pub fn record_links(
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

pub fn mark_inbound(pages: &mut [ExtractedPage], edges: &[GraphEdge]) {
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

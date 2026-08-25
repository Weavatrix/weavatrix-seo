//! Page assembly from a fetch + HTML draft.

use crate::FetchResponse;
use crate::extract::ExtractedPageDraft;
use crate::frontier::Frontier;
use std::collections::BTreeMap;
use weavatrix_seo_model::{
    AbsoluteUrl, ContentHash, Evidence, ExtractedPage, GraphEdge, Indexability, MediaKind, Relation,
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
    let media = MediaKind::classify(fetched.header("content-type"), &fetched.body);
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
        requested: fetched.url.clone(),
        status: fetched.status,
        redirects: Vec::new(),
        content_type: fetched.header("content-type").map(ToOwned::to_owned),
        media,
        canonical: draft.canonical,
        robots,
        title: draft.title,
        description: draft.description,
        html_lang: draft.html_lang,
        alternates: draft.alternates,
        headings: draft.headings,
        links: draft.links,
        link_refs: draft.link_refs,
        images: draft.images,
        json_ld: draft.json_ld,
        text: draft.text,
        heading_text: draft.heading_text,
        main_text: draft.main_text,
        payload: draft.payload,
        arbitrary_script: draft.arbitrary_script,
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

pub fn redirect_page(
    from: &AbsoluteUrl,
    to: &AbsoluteUrl,
    status: u16,
    in_sitemap: bool,
) -> ExtractedPage {
    ExtractedPage {
        url: from.clone(),
        requested: from.clone(),
        status,
        redirects: vec![weavatrix_seo_model::RedirectHop {
            from: from.to_string(),
            to: to.to_string(),
            status,
        }],
        content_type: None,
        media: MediaKind::Other,
        canonical: None,
        robots: Vec::new(),
        title: None,
        description: None,
        html_lang: None,
        alternates: Vec::new(),
        headings: Vec::new(),
        links: Vec::new(),
        link_refs: Vec::new(),
        images: Vec::new(),
        json_ld: Vec::new(),
        text: String::new(),
        heading_text: String::new(),
        main_text: String::new(),
        payload: String::new(),
        arbitrary_script: String::new(),
        og_title: None,
        og_description: None,
        og_image: None,
        headers: Vec::new(),
        body_bytes: 0,
        fetch_ms: 0,
        has_main: false,
        unlabeled_controls: 0,
        content_hash: ContentHash::of(&[]),
        indexability: Indexability::Redirected,
        in_sitemap,
        linked_from_page: false,
        evidence: Evidence::http(),
    }
    .finalize()
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
    let links: Vec<&weavatrix_seo_model::LinkRef> = page.link_refs.iter().collect();
    if links.is_empty() {
        for href in &page.links {
            push_link(page, href, None, seed, depth, max_depth, frontier, edges);
        }
        return;
    }
    for link in links {
        push_link(
            page,
            &link.href,
            Some(link),
            seed,
            depth,
            max_depth,
            frontier,
            edges,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn push_link(
    page: &ExtractedPage,
    href: &str,
    link: Option<&weavatrix_seo_model::LinkRef>,
    seed: &AbsoluteUrl,
    depth: u32,
    max_depth: u32,
    frontier: &mut Frontier,
    edges: &mut Vec<GraphEdge>,
) {
    let Ok(target) = page.url.join(href) else {
        return;
    };
    if !seed.same_origin(&target) {
        return;
    }
    let mut edge = GraphEdge::new(
        page.url.clone(),
        target.clone(),
        Relation::LinksTo,
        Evidence::http(),
    );
    if let Some(link) = link {
        let rel = if link.rel.is_empty() {
            None
        } else {
            Some(link.rel.join(" "))
        };
        edge = edge.with_link(
            link.anchor.clone(),
            rel,
            Some(link.location),
            link.context.clone(),
        );
    }
    edges.push(edge);
    if depth < max_depth {
        frontier.push_link(target, depth + 1);
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

//! Deterministic audit rules over a crawl inventory.

#![forbid(unsafe_code)]

use weavatrix_seo_model::{
    AbsoluteUrl, Evidence, ExtractedPage, Finding, FindingFamily, Indexability, Inventory, Locator,
    Relation, Severity,
};

/// Runs the site-only deterministic rule pack.
#[must_use]
pub fn audit(inventory: &Inventory) -> Vec<Finding> {
    let mut findings = Vec::new();
    crawl_status(inventory, &mut findings);
    canonical(inventory, &mut findings);
    sitemap(inventory, &mut findings);
    metadata(inventory, &mut findings);
    hreflang(inventory, &mut findings);
    schema(inventory, &mut findings);
    links(inventory, &mut findings);
    findings.sort_by(|left, right| {
        right
            .severity
            .cmp(&left.severity)
            .then_with(|| left.code.cmp(&right.code))
            .then_with(|| left.fingerprint.cmp(&right.fingerprint))
    });
    findings
}

fn crawl_status(inventory: &Inventory, findings: &mut Vec<Finding>) {
    for page in &inventory.pages {
        let url = page.url.to_string();
        if (400..500).contains(&page.status) {
            findings.push(
                Finding::new(
                    FindingFamily::Crawl,
                    1,
                    Severity::Error,
                    &url,
                    format!("{} returned {}", page.url, page.status),
                    Locator::url(&page.url),
                    page.evidence.clone(),
                )
                .explained(
                    "Search engines cannot index a client-error URL.",
                    "Restore the route or update internal links and sitemaps.",
                    "HTTP status is 200 and the URL is linked or listed as intended.",
                ),
            );
        }
        if page.status >= 500 {
            findings.push(
                Finding::new(
                    FindingFamily::Crawl,
                    2,
                    Severity::Error,
                    &url,
                    format!("{} returned {}", page.url, page.status),
                    Locator::url(&page.url),
                    page.evidence.clone(),
                )
                .explained(
                    "A server error hides the URL from discovery.",
                    "Fix the origin response for this URL.",
                    "HTTP status is in the 2xx range.",
                ),
            );
        }
        if page.redirects.len() > 1 {
            findings.push(
                Finding::new(
                    FindingFamily::Crawl,
                    3,
                    Severity::Warn,
                    &url,
                    format!(
                        "{} has a redirect chain of {}",
                        page.url,
                        page.redirects.len()
                    ),
                    Locator::url(&page.url),
                    page.evidence.clone(),
                )
                .explained(
                    "Redirect chains waste crawl budget and dilute signals.",
                    "Point the first hop at the final URL.",
                    "A single hop, or none, reaches the canonical URL.",
                ),
            );
        }
    }
}

fn canonical(inventory: &Inventory, findings: &mut Vec<Finding>) {
    for page in inventory
        .pages
        .iter()
        .filter(|page| page.indexability == Indexability::Indexable && page.status == 200)
    {
        match &page.canonical {
            None => findings.push(
                Finding::new(
                    FindingFamily::Canon,
                    1,
                    Severity::Warn,
                    &page.url.to_string(),
                    format!("{} has no canonical", page.url),
                    Locator::dom(&page.url, "link[rel=canonical]"),
                    page.evidence.clone(),
                )
                .explained(
                    "Indexable pages should declare a self-canonical.",
                    "Emit a self-referencing canonical on this template.",
                    "A canonical href matches the final URL.",
                ),
            ),
            Some(href) => {
                if let Ok(target) = AbsoluteUrl::parse(href).or_else(|_| page.url.join(href))
                    && let Some(dest) = inventory.page(&target)
                    && dest.status >= 400
                {
                    findings.push(
                        Finding::new(
                            FindingFamily::Canon,
                            2,
                            Severity::Error,
                            &page.url.to_string(),
                            format!("{} canonical points at {}", page.url, dest.status),
                            Locator::dom(&page.url, "link[rel=canonical]"),
                            page.evidence.clone(),
                        )
                        .explained(
                            "A canonical must resolve to a reachable URL.",
                            "Point the canonical at a live indexable URL.",
                            "The canonical target returns 200.",
                        ),
                    );
                }
            }
        }
    }
}

fn sitemap(inventory: &Inventory, findings: &mut Vec<Finding>) {
    for page in inventory.pages.iter().filter(|page| page.in_sitemap) {
        if page.status >= 400 {
            findings.push(
                Finding::new(
                    FindingFamily::Sitemap,
                    1,
                    Severity::Error,
                    &page.url.to_string(),
                    format!("sitemap lists unreachable {}", page.url),
                    Locator::Sitemap {
                        sitemap: inventory.site.clone().unwrap_or_default(),
                        loc: page.url.to_string(),
                    },
                    Evidence::sitemap(),
                )
                .explained(
                    "Sitemap loc values must exist.",
                    "Remove the loc or restore the URL.",
                    "The listed URL returns 200.",
                ),
            );
        }
        if page.indexability == Indexability::Noindex {
            findings.push(
                Finding::new(
                    FindingFamily::Sitemap,
                    2,
                    Severity::Warn,
                    &page.url.to_string(),
                    format!("sitemap lists noindex URL {}", page.url),
                    Locator::url(&page.url),
                    Evidence::sitemap(),
                )
                .explained(
                    "Sitemaps should list canonical indexable URLs only.",
                    "Drop the loc or drop the noindex signal.",
                    "Sitemap membership matches indexability.",
                ),
            );
        }
    }
}

fn metadata(inventory: &Inventory, findings: &mut Vec<Finding>) {
    let mut titles: Vec<(&ExtractedPage, String)> = Vec::new();
    for page in inventory
        .pages
        .iter()
        .filter(|page| page.status == 200 && page.indexability == Indexability::Indexable)
    {
        match &page.title {
            Some(title) if !title.trim().is_empty() => titles.push((page, title.clone())),
            _ => findings.push(
                Finding::new(
                    FindingFamily::Meta,
                    1,
                    Severity::Error,
                    &page.url.to_string(),
                    format!("{} is missing a title", page.url),
                    Locator::dom(&page.url, "title"),
                    page.evidence.clone(),
                )
                .explained(
                    "Indexable pages need a unique title.",
                    "Set a title on this template.",
                    "A non-empty title is present.",
                ),
            ),
        }
        if page.description.as_ref().is_none_or(String::is_empty) {
            findings.push(
                Finding::new(
                    FindingFamily::Meta,
                    3,
                    Severity::Info,
                    &page.url.to_string(),
                    format!("{} has no meta description", page.url),
                    Locator::dom(&page.url, "meta[name=description]"),
                    page.evidence.clone(),
                )
                .explained(
                    "A missing description is a display risk, not a ranking claim.",
                    "Add a unique description when the template owns one.",
                    "A description exists or the absence is intentional.",
                ),
            );
        }
    }
    titles.sort_by(|left, right| left.1.cmp(&right.1));
    let mut index = 0;
    while index < titles.len() {
        let mut end = index + 1;
        while end < titles.len() && titles[end].1 == titles[index].1 {
            end += 1;
        }
        if end - index > 1 {
            let urls: Vec<String> = titles[index..end]
                .iter()
                .map(|(page, _)| page.url.to_string())
                .collect();
            findings.push(
                Finding::new(
                    FindingFamily::Meta,
                    2,
                    Severity::Warn,
                    &titles[index].1,
                    format!(
                        "title {:?} is reused on {} URLs",
                        titles[index].1,
                        urls.len()
                    ),
                    Locator::url(&titles[index].0.url),
                    titles[index].0.evidence.clone(),
                )
                .with_affected(urls)
                .explained(
                    "Duplicate titles collapse distinct pages in search results.",
                    "Give each page family a unique title template.",
                    "No two indexable URLs share the same title.",
                ),
            );
        }
        index = end;
    }
}

fn hreflang(inventory: &Inventory, findings: &mut Vec<Finding>) {
    for page in &inventory.pages {
        for alternate in &page.alternates {
            let Ok(target) =
                AbsoluteUrl::parse(&alternate.href).or_else(|_| page.url.join(&alternate.href))
            else {
                continue;
            };
            let Some(dest) = inventory.page(&target) else {
                continue;
            };
            let returns = dest.alternates.iter().any(|item| {
                AbsoluteUrl::parse(&item.href)
                    .or_else(|_| dest.url.join(&item.href))
                    .is_ok_and(|href| href == page.url)
            });
            if !returns {
                findings.push(
                    Finding::new(
                        FindingFamily::I18n,
                        1,
                        Severity::Warn,
                        &page.url.to_string(),
                        format!(
                            "{} hreflang {} is not reciprocal",
                            page.url, alternate.hreflang
                        ),
                        Locator::dom(&page.url, "link[rel=alternate]"),
                        page.evidence.clone(),
                    )
                    .explained(
                        "Hreflang annotations must be reciprocal.",
                        "Add the return alternate on the target locale.",
                        "Each locale in the set lists every other locale.",
                    ),
                );
            }
        }
    }
}

fn schema(inventory: &Inventory, findings: &mut Vec<Finding>) {
    for page in &inventory.pages {
        for block in page.json_ld.iter().filter(|block| !block.valid_json) {
            findings.push(
                Finding::new(
                    FindingFamily::Schema,
                    1,
                    Severity::Warn,
                    &page.url.to_string(),
                    format!("{} has invalid JSON-LD", page.url),
                    Locator::JsonLd {
                        url: page.url.to_string(),
                        path: block.raw.chars().take(40).collect(),
                    },
                    page.evidence.clone(),
                )
                .explained(
                    "Structured data must be valid JSON.",
                    "Fix the JSON-LD generator for this template.",
                    "Each JSON-LD script parses as JSON.",
                ),
            );
        }
    }
}

fn links(inventory: &Inventory, findings: &mut Vec<Finding>) {
    for edge in inventory
        .edges
        .iter()
        .filter(|edge| edge.relation == Relation::LinksTo)
    {
        if let Some(target) = inventory.page(&edge.target)
            && target.status >= 400
        {
            findings.push(
                Finding::new(
                    FindingFamily::Link,
                    1,
                    Severity::Error,
                    &format!("{}->{}", edge.source, edge.target),
                    format!("{} links to {}", edge.source, edge.target),
                    Locator::url(&edge.source),
                    edge.evidence.clone(),
                )
                .with_affected([edge.target.to_string()])
                .explained(
                    "Broken internal links waste crawl budget.",
                    "Update or remove the href in the owning component.",
                    "The target returns 200.",
                ),
            );
        }
    }
}

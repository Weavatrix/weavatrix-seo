//! Query-parameter URLs that cannibalize path landings.

use weavatrix_seo_model::{
    AbsoluteUrl, Finding, FindingFamily, Indexability, Inventory, Locator, Severity,
};

pub fn audit(inventory: &Inventory, findings: &mut Vec<Finding>) {
    query_next_to_path(inventory, findings);
    city_path_redirects_to_query(inventory, findings);
}

fn query_next_to_path(inventory: &Inventory, findings: &mut Vec<Finding>) {
    for page in inventory.pages.iter().filter(|page| {
        page.status == 200 && page.indexability != Indexability::Noindex && page.media.is_html()
    }) {
        let Some(query) = page.url.query() else {
            continue;
        };
        let Some(city) = query_value(query, "city") else {
            continue;
        };
        if city.is_empty() {
            continue;
        }
        let Some(other) = inventory.pages.iter().find(|candidate| {
            candidate.status == 200
                && candidate.indexability == Indexability::Indexable
                && candidate.url.query().is_none()
                && candidate.url.path().contains(city)
        }) else {
            continue;
        };
        findings.push(
            Finding::new(
                FindingFamily::Cann,
                2,
                Severity::Warn,
                &page.url.to_string(),
                format!(
                    "{} uses ?city={city} while {} already occupies that city path",
                    page.url, other.url
                ),
                Locator::url(&page.url),
                page.evidence.clone(),
            )
            .with_affected([other.url.to_string()])
            .explained(
                "A query-parameter URL duplicates a pretty city landing in the same crawl.",
                "Canonicalise or noindex the query URL, and keep internal links on the path URL.",
                "One indexable URL remains for that city intent.",
            ),
        );
    }
}

fn city_path_redirects_to_query(inventory: &Inventory, findings: &mut Vec<Finding>) {
    for page in inventory
        .pages
        .iter()
        .filter(|page| page.indexability == Indexability::Redirected)
    {
        let Some(hop) = page.redirects.last() else {
            continue;
        };
        let Ok(target) = AbsoluteUrl::parse(&hop.to).or_else(|_| page.url.join(&hop.to)) else {
            continue;
        };
        let Some(query) = target.query() else {
            continue;
        };
        let Some(city) = query_value(query, "city") else {
            continue;
        };
        if !city_shaped(page.url.path()) && !page.url.path().contains(city) {
            continue;
        }
        findings.push(
            Finding::new(
                FindingFamily::Cann,
                3,
                Severity::Warn,
                &page.url.to_string(),
                format!("{} redirects a city path to {}", page.url, target),
                Locator::url(&page.url),
                page.evidence.clone(),
            )
            .with_affected([target.to_string()])
            .explained(
                "A pretty city URL hands the search identity to a query-parameter listing.",
                "Keep the city path as the 200 canonical, or 301 it to a path landing, not ?city=.",
                "The city URL is indexable on a path, or it is gone from internal links.",
            ),
        );
    }
}

fn city_shaped(path: &str) -> bool {
    path.contains("/cities/")
        || path.contains("/city/")
        || path
            .rsplit('/')
            .next()
            .is_some_and(|segment| segment.contains('-') && segment.len() > 4)
}

fn query_value<'a>(query: &'a str, name: &str) -> Option<&'a str> {
    query.split('&').find_map(|pair| {
        let (key, value) = pair.split_once('=')?;
        (key == name).then_some(value)
    })
}

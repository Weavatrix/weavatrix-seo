//! Query-parameter URLs that cannibalize path landings.

use weavatrix_seo_model::{Finding, FindingFamily, Indexability, Inventory, Locator, Severity};

pub fn audit(inventory: &Inventory, findings: &mut Vec<Finding>) {
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

fn query_value<'a>(query: &'a str, name: &str) -> Option<&'a str> {
    query.split('&').find_map(|pair| {
        let (key, value) = pair.split_once('=')?;
        (key == name).then_some(value)
    })
}

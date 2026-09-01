//! Reciprocal hreflang and locale twins that never declared a cluster.

use std::collections::BTreeMap;
use weavatrix_seo_model::{
    AbsoluteUrl, ExtractedPage, Finding, FindingFamily, Indexability, Inventory, Locator, Severity,
};

const LOCALES: &[&str] = &[
    "en", "ru", "he", "uk", "ar", "es", "de", "fr", "pt", "it", "pl", "nl",
];

pub fn audit(inventory: &Inventory, findings: &mut Vec<Finding>) {
    reciprocal(inventory, findings);
    missing_clusters(inventory, findings);
}

fn reciprocal(inventory: &Inventory, findings: &mut Vec<Finding>) {
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
            if dest.status >= 400 {
                findings.push(
                    Finding::new(
                        FindingFamily::I18n,
                        4,
                        Severity::Error,
                        &page.url.to_string(),
                        format!(
                            "{} hreflang {} points at {}",
                            page.url, alternate.hreflang, dest.status
                        ),
                        Locator::dom(&page.url, "link[rel=alternate]"),
                        page.evidence.clone(),
                    )
                    .explained(
                        "Hreflang must name a live locale URL.",
                        "Fix or drop the alternate that returns an error.",
                        "Every hreflang href in the crawl returns 200.",
                    ),
                );
                continue;
            }
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

fn missing_clusters(inventory: &Inventory, findings: &mut Vec<Finding>) {
    let mut groups: BTreeMap<String, Vec<&ExtractedPage>> = BTreeMap::new();
    for page in inventory.pages.iter().filter(|page| {
        page.status == 200 && page.indexability == Indexability::Indexable && page.media.is_html()
    }) {
        let key = locale_rest(page.url.path());
        groups.entry(key).or_default().push(page);
    }
    for (rest, pages) in groups {
        if pages.len() < 2 {
            continue;
        }
        let mut locales: Vec<String> = pages
            .iter()
            .map(|page| locale_of(page.url.path()).unwrap_or("default"))
            .map(str::to_owned)
            .collect();
        locales.sort();
        locales.dedup();
        if locales.len() < 2 {
            continue;
        }
        let urls: Vec<String> = pages.iter().map(|page| page.url.to_string()).collect();
        if pages.iter().any(|page| page.alternates.is_empty()) {
            findings.push(
                Finding::new(
                    FindingFamily::I18n,
                    2,
                    Severity::Warn,
                    &rest,
                    format!(
                        "locale variants of {rest} exist ({}) without an hreflang cluster",
                        locales.join(", ")
                    ),
                    Locator::url(&pages[0].url),
                    pages[0].evidence.clone(),
                )
                .with_affected(urls.clone())
                .explained(
                    "The crawl measured multiple locales of the same path, but pages do not declare alternates.",
                    "Emit reciprocal hreflang (and x-default) on every locale variant.",
                    "Each locale lists every other locale in the set.",
                ),
            );
        }
        let wants_default = inventory
            .policy
            .as_ref()
            .and_then(|policy| policy.international.x_default.as_deref())
            .is_some();
        let has_default = pages.iter().any(|page| {
            page.alternates
                .iter()
                .any(|item| item.hreflang.eq_ignore_ascii_case("x-default"))
        });
        if wants_default && !has_default {
            findings.push(
                Finding::new(
                    FindingFamily::I18n,
                    3,
                    Severity::Warn,
                    &rest,
                    format!("locale variants of {rest} have no x-default hreflang"),
                    Locator::url(&pages[0].url),
                    pages[0].evidence.clone(),
                )
                .with_affected(urls)
                .explained(
                    "The repository policy requires x-default when locale twins exist.",
                    "Add rel=alternate hreflang=x-default on the default locale URL.",
                    "The cluster includes an x-default alternate.",
                ),
            );
        }
    }
}

fn locale_of(path: &str) -> Option<&str> {
    let head = path.trim_start_matches('/').split('/').next().unwrap_or("");
    LOCALES.contains(&head).then_some(head)
}

fn locale_rest(path: &str) -> String {
    match locale_of(path) {
        None => path.to_owned(),
        Some(locale) => {
            let rest = path
                .trim_start_matches('/')
                .strip_prefix(locale)
                .unwrap_or(path)
                .trim_start_matches('/');
            if rest.is_empty() {
                "/".into()
            } else {
                format!("/{rest}")
            }
        }
    }
}

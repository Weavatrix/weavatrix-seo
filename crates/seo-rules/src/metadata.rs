//! Titles and descriptions.

use weavatrix_seo_model::{
    ExtractedPage, Finding, FindingFamily, Indexability, Inventory, Locator, Severity,
};

pub fn audit(inventory: &Inventory, findings: &mut Vec<Finding>) {
    let mut titles: Vec<(&ExtractedPage, String)> = Vec::new();
    for page in inventory.pages.iter().filter(|page| {
        page.status == 200 && page.indexability == Indexability::Indexable && page.media.is_html()
    }) {
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
    duplicate_descriptions(inventory, findings);
}

fn duplicate_descriptions(inventory: &Inventory, findings: &mut Vec<Finding>) {
    let mut rows: Vec<(&ExtractedPage, String)> = Vec::new();
    for page in inventory.pages.iter().filter(|page| {
        page.status == 200 && page.indexability == Indexability::Indexable && page.media.is_html()
    }) {
        if let Some(description) = page
            .description
            .as_ref()
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty())
        {
            rows.push((page, description));
        }
    }
    rows.sort_by(|left, right| left.1.cmp(&right.1));
    let mut index = 0;
    while index < rows.len() {
        let mut end = index + 1;
        while end < rows.len() && rows[end].1 == rows[index].1 {
            end += 1;
        }
        if end - index > 1 {
            let urls: Vec<String> = rows[index..end]
                .iter()
                .map(|(page, _)| page.url.to_string())
                .collect();
            findings.push(
                Finding::new(
                    FindingFamily::Meta,
                    6,
                    Severity::Warn,
                    &rows[index].1,
                    format!("meta description is reused on {} URLs", urls.len()),
                    Locator::url(&rows[index].0.url),
                    rows[index].0.evidence.clone(),
                )
                .with_affected(urls)
                .explained(
                    "Duplicate descriptions collapse snippets the same way duplicate titles do.",
                    "Give each indexable family its own description template.",
                    "No two indexable URLs share the same description.",
                ),
            );
        }
        index = end;
    }
}

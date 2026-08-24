//! Exact duplicate detection. Near-duplicate clustering stays unmeasured here.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use weavatrix_seo_model::{
    ContentHash, Evidence, Finding, FindingFamily, Indexability, Inventory, Locator, Severity,
};

/// Duplicate classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DuplicateClass {
    /// Distinct bodies.
    Unique,
    /// Byte-identical main content.
    Exact,
}

/// Groups exact-duplicate indexable pages.
#[must_use]
pub fn exact_duplicates(inventory: &Inventory) -> Vec<Finding> {
    let mut groups: BTreeMap<ContentHash, Vec<String>> = BTreeMap::new();
    for page in inventory.pages.iter().filter(|page| {
        page.status == 200
            && page.indexability == Indexability::Indexable
            && !page.text.trim().is_empty()
    }) {
        groups
            .entry(page.content_hash)
            .or_default()
            .push(page.url.to_string());
    }
    let mut findings = Vec::new();
    for urls in groups.into_values().filter(|urls| urls.len() > 1) {
        let subject = urls.join(" ");
        findings.push(
            Finding::new(
                FindingFamily::Dup,
                1,
                Severity::Warn,
                &subject,
                format!("{} URLs share identical main content", urls.len()),
                Locator::Url(urls[0].clone()),
                Evidence::http(),
            )
            .with_affected(urls.clone())
            .explained(
                "Exact duplicates split crawl attention unless they are intentional variants.",
                "Canonicalize, consolidate, or differentiate the bodies.",
                "Each remaining indexable URL has distinct main content or a canonical cluster.",
            ),
        );
    }
    findings
}

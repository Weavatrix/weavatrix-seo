//! Exact duplicates, near-duplicates, content profiles, and family decomposition.
//!
//! Exact-byte identity stays the default detector. Near-duplicate clustering,
//! lexical profiles, and programmatic family decomposition are additive.

#![forbid(unsafe_code)]

mod chunk;
mod family;
mod intent;
mod near;
mod profile;
mod tokens;

use std::collections::BTreeMap;
use weavatrix_seo_model::{
    ContentHash, Evidence, Finding, FindingFamily, Indexability, Inventory, Locator, Severity,
};

pub use chunk::chunks;
pub use family::decompose;
pub use intent::{fanout, fanout_subject};
pub use near::near_duplicates;
pub use profile::profiles;

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

/// Combined content pass: exact dups stay, near-dups and family findings are additive.
#[must_use]
pub fn audit(inventory: &Inventory) -> ContentPass {
    let exact = exact_duplicates(inventory);
    let (near_groups, near_findings) = near::near_duplicates(inventory);
    let profiles = profile::profiles(inventory);
    let (families, family_findings) = family::decompose(inventory);
    let chunks = chunk::chunks(inventory);
    let mut intents = intent::fanout(inventory);
    for page in inventory.pages.iter().filter(|page| {
        page.status == 200
            && page.indexability == Indexability::Indexable
            && !page.visible_text().trim().is_empty()
    }) {
        intents.extend(intent::fanout_subject(
            &page.url.to_string(),
            "url",
            &page.visible_text(),
        ));
    }
    for family in &families {
        let hay = inventory
            .pages
            .iter()
            .filter(|page| page.url.path().contains(&family.family))
            .map(weavatrix_seo_model::ExtractedPage::visible_text)
            .collect::<Vec<_>>()
            .join("\n");
        intents.extend(intent::fanout_subject(&family.family, "route_family", &hay));
    }
    let mut findings = exact;
    findings.extend(near_findings);
    findings.extend(family_findings);
    ContentPass {
        findings,
        profiles,
        families,
        chunks,
        intents,
        near_duplicates: near_groups,
    }
}

/// Output of the content intelligence pass.
pub struct ContentPass {
    /// Findings (exact dups, near-dups, thin families).
    pub findings: Vec<Finding>,
    /// Per-page profiles.
    pub profiles: Vec<weavatrix_seo_model::ContentProfile>,
    /// Family decompositions.
    pub families: Vec<weavatrix_seo_model::FamilyContent>,
    /// Chunks.
    pub chunks: Vec<weavatrix_seo_model::Chunk>,
    /// Intent fanout.
    pub intents: Vec<weavatrix_seo_model::IntentCoverage>,
    /// Near-duplicate groups.
    pub near_duplicates: Vec<weavatrix_seo_model::NearDuplicateGroup>,
}

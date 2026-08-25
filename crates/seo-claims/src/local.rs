//! Local geo binding on city families. Types without an address stay unlocated.

use weavatrix_seo_model::{
    Evidence, EvidenceKind, ExtractedPage, Finding, FindingFamily, Indexability,
    Inventory, Locator, Severity, glob_match,
};

const GEO_MARKERS: &[&str] = &[
    "areaserved",
    "addresslocality",
    "addressregion",
    "postaladdress",
    "postalcode",
    "\"geo\"",
    "latitude",
    "longitude",
];

/// City URLs that declare a place type but never bind it to a geography.
#[must_use]
pub fn audit(inventory: &Inventory) -> Vec<Finding> {
    let mut findings = Vec::new();
    for page in city_pages(inventory) {
        if !has_place_type(page) || has_geo(page) {
            continue;
        }
        findings.push(
            Finding::new(
                FindingFamily::Local,
                1,
                Severity::Warn,
                &page.url.to_string(),
                format!(
                    "{} declares Place/Service JSON-LD without areaServed or an address",
                    page.url
                ),
                Locator::dom(&page.url, "script[type='application/ld+json']"),
                Evidence {
                    kind: EvidenceKind::Deterministic,
                    source: page.evidence.source,
                    confidence: weavatrix_seo_model::Confidence::High,
                    snapshot_id: Some(inventory.snapshot_id.clone()),
                    revision: inventory.repo_revision.clone(),
                    policy_version: Some(inventory.policy_version.clone()),
                },
            )
            .explained(
                "A city landing is only local-search evidence when the schema names a place.",
                "Add areaServed, PostalAddress, or geo on the Place/Service node.",
                "The JSON-LD for this URL includes an address or areaServed value.",
            ),
        );
    }
    findings
}

fn city_pages(inventory: &Inventory) -> impl Iterator<Item = &ExtractedPage> {
    let patterns: Vec<String> = inventory
        .predicted_routes
        .iter()
        .filter(|pattern| pattern.contains(":city"))
        .cloned()
        .collect();
    inventory.pages.iter().filter(move |page| {
        page.status == 200
            && page.indexability == Indexability::Indexable
            && page.media.is_html()
            && is_city(page.url.path(), &patterns)
    })
}

fn is_city(path: &str, patterns: &[String]) -> bool {
    if patterns
        .iter()
        .any(|pattern| glob_match(pattern, path))
    {
        return true;
    }
    if !patterns.is_empty() {
        return false;
    }
    let lower = path.to_ascii_lowercase();
    lower.contains("/cities/") || lower.contains("/city/")
}

fn has_place_type(page: &ExtractedPage) -> bool {
    page.json_ld.iter().any(|block| {
        block.types.iter().any(|kind| {
            matches!(
                kind.to_ascii_lowercase().as_str(),
                "place"
                    | "city"
                    | "localbusiness"
                    | "service"
                    | "professionalservice"
                    | "homeandconstructionbusiness"
            )
        })
    })
}

fn has_geo(page: &ExtractedPage) -> bool {
    page.json_ld.iter().any(|block| {
        let compact: String = block
            .raw
            .chars()
            .filter(|ch| !ch.is_whitespace())
            .flat_map(char::to_lowercase)
            .collect();
        GEO_MARKERS.iter().any(|marker| compact.contains(marker))
    })
}

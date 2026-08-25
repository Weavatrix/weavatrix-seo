//! Pack entities named on the page but never declared as schema.

use crate::market::{contains_token, infer_market, page_haystack};
use crate::pack;
use weavatrix_seo_model::{
    Evidence, EvidenceKind, EvidenceSource, ExtractedPage, Finding, FindingFamily, Indexability,
    Inventory, Locator, Severity, glob_match,
};

const PLACE_TYPES: &[&str] = &[
    "Place",
    "City",
    "LocalBusiness",
    "Service",
    "Offer",
    "AggregateOffer",
    "ProfessionalService",
    "HomeAndConstructionBusiness",
];

/// Entity graph pass: undeclared pack names, and city families without a place type.
#[must_use]
pub fn audit(inventory: &Inventory) -> Vec<Finding> {
    let mut findings = Vec::new();
    undeclared(inventory, &mut findings);
    city_families(inventory, &mut findings);
    findings
}

fn undeclared(inventory: &Inventory, findings: &mut Vec<Finding>) {
    for page in indexable(inventory) {
        let hay = page_haystack(page);
        let market = infer_market(&page.url, page.html_lang.as_deref(), &hay);
        let Some(pack) = pack::for_market(market) else {
            continue;
        };
        let visible = visible_copy(page);
        let schema = schema_copy(page);
        let mut missing: Vec<&str> = pack
            .entities
            .iter()
            .filter(|entity| {
                contains_token(&visible, entity.token) && !contains_token(&schema, entity.token)
            })
            .map(|entity| entity.label)
            .collect();
        if missing.is_empty() {
            continue;
        }
        missing.sort_unstable();
        missing.dedup();
        let subject = format!("{}:{}", page.url, missing.join(","));
        findings.push(
            Finding::new(
                FindingFamily::Entity,
                1,
                Severity::Warn,
                &subject,
                format!(
                    "{} names {:?} without declaring those entities in JSON-LD",
                    page.url, missing
                ),
                Locator::dom(&page.url, "script[type='application/ld+json']"),
                evidence(inventory, page.evidence.source),
            )
            .explained(
                "The page market pack owns these entities, but structured data does not name them.",
                "Emit Organization, Place, or Service JSON-LD that uses the same entity labels as the visible copy.",
                "Each named pack entity appears in a JSON-LD node on the page.",
            ),
        );
    }
}

fn city_families(inventory: &Inventory, findings: &mut Vec<Finding>) {
    let patterns: Vec<&String> = inventory
        .predicted_routes
        .iter()
        .filter(|pattern| pattern.contains(":city"))
        .collect();
    if patterns.is_empty() {
        for page in indexable(inventory).filter(|page| is_city_path(page.url.path())) {
            if !has_place_type(page) {
                findings.push(city_finding(inventory, page.url.path(), page));
            }
        }
        return;
    }
    for pattern in patterns {
        let live: Vec<&ExtractedPage> = indexable(inventory)
            .filter(|page| glob_match(pattern, page.url.path()))
            .collect();
        if live.is_empty() {
            if !has_jsonld_producer(inventory, pattern) {
                findings.push(family_finding(inventory, pattern));
            }
            continue;
        }
        for page in live {
            if !has_place_type(page) {
                findings.push(city_finding(inventory, pattern, page));
            }
        }
    }
}

fn indexable(inventory: &Inventory) -> impl Iterator<Item = &ExtractedPage> {
    inventory.pages.iter().filter(|page| {
        page.status == 200 && page.indexability == Indexability::Indexable && page.media.is_html()
    })
}

fn visible_copy(page: &ExtractedPage) -> String {
    let mut out = String::new();
    if let Some(title) = &page.title {
        out.push_str(title);
        out.push(' ');
    }
    for heading in &page.headings {
        out.push_str(&heading.text);
        out.push(' ');
    }
    out.push_str(&page.text);
    out.push(' ');
    out.push_str(&page.heading_text);
    out
}

fn schema_copy(page: &ExtractedPage) -> String {
    page.json_ld
        .iter()
        .map(|block| block.raw.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_city_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.contains("/cities/") || lower.contains("/city/")
}

fn has_place_type(page: &ExtractedPage) -> bool {
    page.json_ld.iter().any(|block| {
        block.types.iter().any(|kind| {
            PLACE_TYPES
                .iter()
                .any(|wanted| kind.eq_ignore_ascii_case(wanted))
        })
    })
}

fn has_jsonld_producer(inventory: &Inventory, pattern: &str) -> bool {
    inventory.producers.iter().any(|producer| {
        producer.families.iter().any(|family| family == pattern) && looks_jsonld(producer)
    })
}

fn looks_jsonld(producer: &weavatrix_seo_model::ProducerFact) -> bool {
    let hay = format!("{} {}", producer.name, producer.path)
        .to_ascii_lowercase()
        .replace(['-', '_'], "");
    hay.contains("jsonld") || hay.contains("structureddata")
}

fn city_finding(inventory: &Inventory, subject: &str, page: &ExtractedPage) -> Finding {
    Finding::new(
        FindingFamily::Entity,
        2,
        Severity::Warn,
        subject,
        format!("{} is a city family URL without Place/Service JSON-LD", page.url),
        Locator::dom(&page.url, "script[type='application/ld+json']"),
        evidence(inventory, page.evidence.source),
    )
    .explained(
        "City landings are a place graph, not a generic HTML page.",
        "Emit Place, LocalBusiness, or Service JSON-LD with areaServed for this family.",
        "The live city URL declares a place or service type.",
    )
}

fn family_finding(inventory: &Inventory, pattern: &str) -> Finding {
    let path = inventory
        .producers
        .iter()
        .find(|producer| producer.families.iter().any(|family| family == pattern))
        .map_or("", |producer| producer.path.as_str());
    Finding::new(
        FindingFamily::Entity,
        2,
        Severity::Warn,
        pattern,
        format!("route {pattern} has no JSON-LD producer for city pages"),
        Locator::source_span(path, None, None),
        evidence(inventory, EvidenceSource::Repo),
    )
    .explained(
        "The App Router predicts a :city family, but no structured-data producer is bound to it.",
        "Add a JSON-LD helper (Place/Service) imported by the city page module.",
        "The family lists a jsonld/structuredData producer, or live city URLs declare a place type.",
    )
}

fn evidence(inventory: &Inventory, source: EvidenceSource) -> Evidence {
    Evidence {
        kind: EvidenceKind::Deterministic,
        source,
        confidence: weavatrix_seo_model::Confidence::High,
        snapshot_id: Some(inventory.snapshot_id.clone()),
        revision: inventory.repo_revision.clone(),
        policy_version: Some(inventory.policy_version.clone()),
    }
}

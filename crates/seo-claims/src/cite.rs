//! AI-search citation identity. Not llms.txt folklore.

use weavatrix_seo_model::{
    Evidence, EvidenceKind, EvidenceSource, ExtractedPage, Finding, FindingFamily, Indexability,
    Inventory, Locator, Severity, glob_match,
};

/// Citation pass: publisher identity, FAQ answers, FAQ producers without schema.
#[must_use]
pub fn audit(inventory: &Inventory) -> Vec<Finding> {
    let mut findings = Vec::new();
    publisher_identity(inventory, &mut findings);
    faq_pages(inventory, &mut findings);
    faq_producers(inventory, &mut findings);
    findings
}

fn publisher_identity(inventory: &Inventory, findings: &mut Vec<Finding>) {
    let mut sample: Option<&ExtractedPage> = None;
    let mut cited = false;
    for page in indexable(inventory) {
        for block in &page.json_ld {
            if !block.types.iter().any(|kind| is_cite_type(kind)) {
                continue;
            }
            if sample.is_none() {
                sample = Some(page);
            }
            if !block.ids.is_empty() || !block.same_as.is_empty() {
                cited = true;
            }
        }
    }
    let Some(page) = sample else {
        return;
    };
    if cited {
        return;
    }
    findings.push(
        Finding::new(
            FindingFamily::Ai,
            1,
            Severity::Warn,
            page.url.host(),
            format!(
                "{} declares Organization/WebSite JSON-LD without @id or sameAs",
                page.url.host()
            ),
            Locator::dom(&page.url, "script[type='application/ld+json']"),
            evidence(inventory, page.evidence.source),
        )
        .explained(
            "AI search cites a publisher by a stable entity id, not by a type name alone.",
            "Add @id (and sameAs when a public profile exists) on the Organization/WebSite node.",
            "An Organization or WebSite node on the origin has @id or sameAs.",
        ),
    );
}

fn faq_pages(inventory: &Inventory, findings: &mut Vec<Finding>) {
    for page in indexable(inventory) {
        if !faq_shaped(page) || has_faq_schema(page) {
            continue;
        }
        findings.push(
            Finding::new(
                FindingFamily::Ai,
                2,
                Severity::Warn,
                &page.url.to_string(),
                format!("{} has FAQ/Q&A copy without FAQPage JSON-LD", page.url),
                Locator::dom(&page.url, "h1,h2,h3"),
                evidence(inventory, page.evidence.source),
            )
            .explained(
                "Question headings are citation-ready answers only when they are also FAQPage nodes.",
                "Emit FAQPage JSON-LD for the questions already on the page.",
                "The page declares FAQPage, QAPage, or Question.",
            ),
        );
    }
}

fn faq_producers(inventory: &Inventory, findings: &mut Vec<Finding>) {
    let mut seen = Vec::new();
    for producer in &inventory.producers {
        if !looks_faq(&producer.name, &producer.path)
            || looks_faq_schema(&producer.name, &producer.path)
        {
            continue;
        }
        for family in &producer.families {
            if seen.iter().any(|item| item == family) {
                continue;
            }
            let live: Vec<&ExtractedPage> = indexable(inventory)
                .filter(|page| glob_match(family, page.url.path()))
                .collect();
            if live.iter().any(|page| has_faq_schema(page)) || !live.is_empty() {
                continue;
            }
            seen.push(family.clone());
            findings.push(
                Finding::new(
                    FindingFamily::Ai,
                    3,
                    Severity::Warn,
                    family,
                    format!("route {family} has an FAQ producer without FAQPage JSON-LD"),
                    Locator::source_span(producer.path.clone(), None, None),
                    evidence(inventory, EvidenceSource::Repo),
                )
                .explained(
                    "The source graph names an FAQ helper, but the family has no FAQPage producer or live node.",
                    "Bind the FAQ helper to FAQPage JSON-LD on this family.",
                    "Live URLs declare FAQPage, or the family lists a faqSchema/jsonld producer.",
                ),
            );
        }
    }
}

fn indexable(inventory: &Inventory) -> impl Iterator<Item = &ExtractedPage> {
    inventory.pages.iter().filter(|page| {
        page.status == 200 && page.indexability == Indexability::Indexable && page.media.is_html()
    })
}

fn is_cite_type(kind: &str) -> bool {
    let short = kind.rsplit('/').next().unwrap_or(kind);
    short.eq_ignore_ascii_case("organization") || short.eq_ignore_ascii_case("website")
}

fn faq_shaped(page: &ExtractedPage) -> bool {
    let faq_heading = page.headings.iter().any(|heading| {
        let text = heading.text.to_ascii_lowercase();
        text.contains("faq") || text.contains("frequently asked")
    });
    let questions = page
        .headings
        .iter()
        .filter(|heading| heading.text.trim_end().ends_with('?'))
        .count();
    faq_heading || questions >= 2
}

fn has_faq_schema(page: &ExtractedPage) -> bool {
    page.json_ld.iter().any(|block| {
        block.types.iter().any(|kind| {
            let short = kind.rsplit('/').next().unwrap_or(kind);
            short.eq_ignore_ascii_case("faqpage")
                || short.eq_ignore_ascii_case("qapage")
                || short.eq_ignore_ascii_case("question")
        })
    })
}

fn looks_faq(name: &str, path: &str) -> bool {
    let hay = format!("{name} {path}").to_ascii_lowercase();
    hay.contains("faq")
}

fn looks_faq_schema(name: &str, path: &str) -> bool {
    let hay = format!("{name} {path}")
        .to_ascii_lowercase()
        .replace(['-', '_'], "");
    hay.contains("faq")
        && (hay.contains("schema") || hay.contains("jsonld") || hay.contains("structureddata"))
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

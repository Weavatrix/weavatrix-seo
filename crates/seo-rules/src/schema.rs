//! JSON-LD parse evidence.

use weavatrix_seo_model::{Finding, FindingFamily, Inventory, Locator, Severity};

pub fn audit(inventory: &Inventory, findings: &mut Vec<Finding>) {
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

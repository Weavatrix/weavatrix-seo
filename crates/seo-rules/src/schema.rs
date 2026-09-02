//! JSON-LD parse evidence and search-feature eligibility.

use weavatrix_seo_model::{
    Finding, FindingFamily, Inventory, Locator, SchemaProvider, Severity, schema_features,
    schema_missing,
};

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
        required_fields(page, findings);
    }
}

fn required_fields(page: &weavatrix_seo_model::ExtractedPage, findings: &mut Vec<Finding>) {
    for block in page.json_ld.iter().filter(|block| block.valid_json) {
        for node in &block.nodes {
            for ty in &node.types {
                let short = short_type(ty);
                for profile in schema_features()
                    .iter()
                    .filter(|profile| profile.applies_to.eq_ignore_ascii_case(short))
                {
                    let missing = schema_missing(profile.required, &node.properties);
                    if missing.is_empty() {
                        continue;
                    }
                    let (number, severity) = match profile.provider {
                        SchemaProvider::Google => (2, Severity::Warn),
                        SchemaProvider::SchemaOrg => (3, Severity::Info),
                    };
                    let label = profile.feature;
                    findings.push(
                        Finding::new(
                            FindingFamily::Schema,
                            number,
                            severity,
                            &page.url.to_string(),
                            format!(
                                "{} `{short}` fails {label}: missing {}",
                                page.url,
                                missing.join(" OR ")
                            ),
                            Locator::JsonLd {
                                url: page.url.to_string(),
                                path: format!("{short}/{label}"),
                            },
                            page.evidence.clone(),
                        )
                        .explained(
                            "Rich-result eligibility is not the same as schema.org validity.",
                            "Emit the missing fields from first-party facts, not invented values.",
                            "The declared type satisfies the documented feature profile, or the feature is not claimed.",
                        ),
                    );
                }
            }
        }
    }
}

fn short_type(ty: &str) -> &str {
    ty.rsplit('/')
        .next()
        .unwrap_or(ty)
        .rsplit(':')
        .next()
        .unwrap_or(ty)
}

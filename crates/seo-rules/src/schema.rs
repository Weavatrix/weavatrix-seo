//! JSON-LD parse evidence and search-feature eligibility.

use weavatrix_seo_model::{
    FeatureStatus, Finding, FindingFamily, Inventory, Locator, SchemaProvider, schema_features,
    schema_missing,
};

pub fn audit(inventory: &Inventory, findings: &mut Vec<Finding>) {
    for page in &inventory.pages {
        for block in page.json_ld.iter().filter(|block| !block.valid_json) {
            findings.push(
                Finding::from_rule(
                    FindingFamily::Schema,
                    1,
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
                    let google = profile.provider == SchemaProvider::Google;
                    let active = profile.status.strong_eligibility();
                    let number = if google && active {
                        2
                    } else if google {
                        4
                    } else {
                        3
                    };
                    let label = profile.feature;
                    let status = profile.status.as_str();
                    let summary = if google && !active {
                        format!(
                            "{} `{short}` {label} is {status} in Google Search; missing {} is not a current eligibility failure",
                            page.url,
                            missing.join(" OR ")
                        )
                    } else {
                        format!(
                            "{} `{short}` fails {label}: missing {}",
                            page.url,
                            missing.join(" OR ")
                        )
                    };
                    findings.push(
                        Finding::from_rule(
                            FindingFamily::Schema,
                            number,
                            &page.url.to_string(),
                            summary,
                            Locator::JsonLd {
                                url: page.url.to_string(),
                                path: format!("{short}/{label}"),
                            },
                            page.evidence.clone(),
                        )
                        .explained(
                            if matches!(profile.status, FeatureStatus::Removed | FeatureStatus::Deprecated) {
                                "A retired rich-result contract is historical compatibility, not a current Warn."
                            } else {
                                "Rich-result eligibility is not the same as schema.org validity."
                            },
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

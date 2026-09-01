//! JSON-LD parse evidence and required properties.

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
        required_fields(page, findings);
    }
}

fn required_fields(page: &weavatrix_seo_model::ExtractedPage, findings: &mut Vec<Finding>) {
    for block in page.json_ld.iter().filter(|block| block.valid_json) {
        for node in &block.nodes {
            for ty in &node.types {
                let missing: Vec<&str> = required_for(ty)
                    .iter()
                    .copied()
                    .filter(|field| {
                        !node
                            .properties
                            .iter()
                            .any(|name| name.eq_ignore_ascii_case(field))
                    })
                    .collect();
                if missing.is_empty() {
                    continue;
                }
                let label = short_type(ty);
                findings.push(
                    Finding::new(
                        FindingFamily::Schema,
                        2,
                        Severity::Warn,
                        &page.url.to_string(),
                        format!(
                            "{} `{label}` JSON-LD is missing {}",
                            page.url,
                            missing.join(", ")
                        ),
                        Locator::JsonLd {
                            url: page.url.to_string(),
                            path: label.to_owned(),
                        },
                        page.evidence.clone(),
                    )
                    .explained(
                        "Search-engine rich results require those properties on the typed node.",
                        "Emit the missing fields from first-party facts, not invented values.",
                        "Each declared type has the documented required properties.",
                    ),
                );
            }
        }
    }
}

fn required_for(ty: &str) -> &'static [&'static str] {
    match short_type(ty) {
        "FAQPage" | "QAPage" => &["mainEntity"],
        "HowTo" => &["name", "step"],
        "Offer" => &["price"],
        "LocalBusiness" => &["name", "address"],
        "Article" | "BlogPosting" | "NewsArticle" => &["headline"],
        "BreadcrumbList" => &["itemListElement"],
        "Question" | "Product" | "Organization" | "Person" | "Service" => &["name"],
        _ => &[],
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

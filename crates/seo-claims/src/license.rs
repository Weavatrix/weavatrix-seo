//! Public license claims versus `license_verified` facts.

use weavatrix_seo_model::{
    Evidence, EvidenceKind, EvidenceSource, Finding, FindingFamily, Inventory, Locator, Severity,
};

const CLAIM_PHRASES: &[&str] = &[
    "license verified",
    "licenseverification",
    "licenseverified",
    "licensed professional",
    "licensed electrician",
    "licensed contractor",
    "document/license verification",
    "license verification badges",
];

/// Scans crawled pages for public license claims.
#[must_use]
pub fn page_claims(inventory: &Inventory) -> Vec<(String, String)> {
    let mut claims = Vec::new();
    for page in &inventory.pages {
        if page.status >= 400 {
            continue;
        }
        let mut hay = page.text.clone();
        hay.push(' ');
        hay.push_str(&page.payload);
        if let Some(title) = &page.title {
            hay.push(' ');
            hay.push_str(title);
        }
        for block in &page.json_ld {
            hay.push(' ');
            hay.push_str(&block.raw);
        }
        let hay_compact = compact(&hay);
        for phrase in CLAIM_PHRASES {
            if hay_compact.contains(&compact(phrase)) {
                claims.push((page.url.to_string(), (*phrase).to_owned()));
                break;
            }
        }
    }
    claims
}

/// Repo facts: `license_verified` assigned false.
#[must_use]
pub fn false_facts(source: &str) -> bool {
    let compact = compact(source);
    compact.contains("license_verified:false")
        || compact.contains("license_verified=false")
        || compact.contains("licenseverified:false")
}

/// Builds claim findings from live pages and optional repo source.
#[must_use]
pub fn audit_claims(inventory: &Inventory, repo_false: bool, repo_field: bool) -> Vec<Finding> {
    let claims = page_claims(inventory);
    if claims.is_empty() && !repo_field {
        return Vec::new();
    }
    let mut findings = Vec::new();
    if repo_false && (!claims.is_empty() || repo_field) {
        let subject = claims.first().map_or_else(
            || "license_verified:false".into(),
            |(url, phrase)| format!("{url}:{phrase}"),
        );
        let locator = claims.first().map_or(
            Locator::Source {
                path: String::new(),
                start_line: None,
            },
            |(url, _)| Locator::Url(url.clone()),
        );
        findings.push(
            Finding::new(
                FindingFamily::Claim,
                1,
                Severity::Error,
                &subject,
                "public license claim is contradicted by license_verified=false",
                locator,
                Evidence {
                    kind: EvidenceKind::Deterministic,
                    source: EvidenceSource::Repo,
                    confidence: weavatrix_seo_model::Confidence::High,
                    snapshot_id: None,
                    revision: None,
                    policy_version: None,
                },
            )
            .explained(
                "The public surface talks about verified/licensed trades while source data can set license_verified to false.",
                "Stop emitting verified language unless the underlying fact is true, or hide the badge when the field is false.",
                "No indexable page claims license verification unless the domain fact is true.",
            ),
        );
    }
    findings
}

fn compact(text: &str) -> String {
    text.chars()
        .filter(|ch| !ch.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{false_facts, page_claims};
    use weavatrix_seo_model::{
        AbsoluteUrl, AnalysisMode, ContentHash, Evidence, ExtractedPage, Indexability, Inventory,
        InventoryCounts,
    };

    #[test]
    fn detects_false_fact_and_public_phrase() {
        assert!(false_facts("license_verified: false"));
        let page = ExtractedPage {
            url: AbsoluteUrl::parse("https://kablay.us/category/electrician").unwrap(),
            requested: AbsoluteUrl::parse("https://kablay.us/category/electrician").unwrap(),
            status: 200,
            redirects: Vec::new(),
            content_type: None,
            canonical: None,
            robots: Vec::new(),
            title: Some("Electrician".into()),
            description: None,
            html_lang: Some("en".into()),
            alternates: Vec::new(),
            headings: Vec::new(),
            links: Vec::new(),
            images: Vec::new(),
            json_ld: Vec::new(),
            text: "document/license verification badges".into(),
            payload: String::new(),
            og_title: None,
            og_description: None,
            og_image: None,
            headers: Vec::new(),
            body_bytes: 0,
            fetch_ms: 0,
            has_main: false,
            unlabeled_controls: 0,
            content_hash: ContentHash::of(b"x"),
            indexability: Indexability::Indexable,
            in_sitemap: true,
            linked_from_page: true,
            evidence: Evidence::http(),
        };
        let inventory = Inventory {
            mode: AnalysisMode::Site,
            snapshot_id: "x".into(),
            site: Some("https://kablay.us/".into()),
            repo: None,
            hosts: vec!["kablay.us".into()],
            pages: vec![page],
            edges: Vec::new(),
            predicted_routes: Vec::new(),
            sitemap_discovered: 0,
            counts: InventoryCounts {
                crawled: 1,
                fetched: 1,
                redirected: 0,
                errors: 0,
                sitemap_urls: 0,
                indexable: 1,
            },
        };
        assert!(!page_claims(&inventory).is_empty());
    }
}

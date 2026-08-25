//! Public claims bound to pack facts, not a repo-wide boolean.

use crate::market::{infer_market, page_haystack};
use crate::pack::{self, ClaimRule, PolicyPack};
use weavatrix_seo_model::{
    Evidence, EvidenceKind, EvidenceSource, Finding, FindingFamily, Inventory, Locator, Severity,
};

/// Scans crawled pages for public claims from the owning pack.
#[must_use]
pub fn page_claims(inventory: &Inventory) -> Vec<(String, String, &'static PolicyPack, ClaimRule)> {
    let mut claims = Vec::new();
    for page in &inventory.pages {
        if page.status >= 400 {
            continue;
        }
        let hay = page_haystack(page);
        let market = infer_market(&page.url, page.html_lang.as_deref(), &hay);
        let Some(pack) = pack::for_market(market) else {
            continue;
        };
        let hay_compact = compact(&hay);
        for rule in pack.claims {
            if rule
                .phrases
                .iter()
                .any(|phrase| hay_compact.contains(&compact(phrase)))
            {
                claims.push((page.url.to_string(), rule.id.to_owned(), pack, *rule));
                break;
            }
        }
    }
    claims
}

/// Whether source assigns the pack fact to false.
#[must_use]
pub fn false_facts(source: &str) -> bool {
    let compact = compact(source);
    pack::all().iter().any(|pack| {
        pack.facts.iter().any(|fact| {
            fact.false_literals
                .iter()
                .any(|literal| compact.contains(literal))
        })
    })
}

/// Whether `source` assigns `field` to false.
#[must_use]
pub fn fact_is_false(source: &str, field: &str) -> bool {
    let compact = compact(source);
    pack::all().iter().any(|pack| {
        pack.facts.iter().any(|fact| {
            fact.field == field
                && fact
                    .false_literals
                    .iter()
                    .any(|literal| compact.contains(literal))
        })
    })
}

/// Builds claim findings: live claim of pack P vs false fact in a file of pack P.
#[must_use]
pub fn audit_claims(
    inventory: &Inventory,
    pack_false: &[(&'static str, String, Option<u32>)],
) -> Vec<Finding> {
    let claims = page_claims(inventory);
    if claims.is_empty() {
        return Vec::new();
    }
    let mut findings = Vec::new();
    for (url, phrase, pack, rule) in claims {
        let Some((_, path, line)) = pack_false.iter().find(|(id, _, _)| *id == pack.id) else {
            continue;
        };
        findings.push(
            Finding::new(
                FindingFamily::Claim,
                1,
                Severity::Error,
                &format!("{url}:{phrase}"),
                format!(
                    "public {} claim on {url} is contradicted by {}=false in pack {}",
                    rule.id, rule.requires_fact, pack.id
                ),
                Locator::Url(url.clone()),
                Evidence {
                    kind: EvidenceKind::Deterministic,
                    source: EvidenceSource::Repo,
                    confidence: weavatrix_seo_model::Confidence::High,
                    snapshot_id: Some(inventory.snapshot_id.clone()),
                    revision: inventory.repo_revision.clone(),
                    policy_version: Some(inventory.policy_version.clone()),
                },
            )
            .with_affected([path.clone()])
            .explained(
                "The public surface of this market pack claims a credential that source data marks false.",
                "Stop emitting verified language unless the underlying fact is true, or hide the badge when the field is false.",
                "No indexable page in this pack claims the credential unless the domain fact is true.",
            ),
        );
        let _ = line;
    }
    findings
}

pub(crate) fn compact(text: &str) -> String {
    text.chars()
        .filter(|ch| !ch.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{audit_claims, false_facts, page_claims};
    use weavatrix_seo_model::{
        AbsoluteUrl, AnalysisMode, ContentHash, Evidence, ExtractedPage, Indexability, Inventory,
        InventoryCounts, MediaKind,
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
            media: MediaKind::Html,
            canonical: None,
            robots: Vec::new(),
            title: Some("Electrician".into()),
            description: None,
            html_lang: Some("en".into()),
            alternates: Vec::new(),
            headings: Vec::new(),
            links: Vec::new(),
            link_refs: Vec::new(),
            images: Vec::new(),
            json_ld: Vec::new(),
            text: "document/license verification badges".into(),
            heading_text: String::new(),
            main_text: String::new(),
            payload: String::new(),
            arbitrary_script: String::new(),
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
        let mut inventory = Inventory::blank(AnalysisMode::Site);
        inventory.snapshot_id = "x".into();
        inventory.site = Some("https://kablay.us/".into());
        inventory.hosts = vec!["kablay.us".into()];
        inventory.pages = vec![page];
        inventory.counts = InventoryCounts {
            crawled: 1,
            fetched: 1,
            redirected: 0,
            errors: 0,
            sitemap_urls: 0,
            indexable: 1,
            incomplete: 0,
        };
        assert!(!page_claims(&inventory).is_empty());
        assert!(
            audit_claims(
                &inventory,
                &[("marketplace.contractor.il", "il.ts".into(), None)]
            )
            .is_empty()
        );
        assert!(
            !audit_claims(
                &inventory,
                &[("marketplace.contractor.us-wa", "data.ts".into(), Some(12))]
            )
            .is_empty()
        );
    }

    #[test]
    fn repo_field_alone_is_not_a_contradiction() {
        let inventory = Inventory::blank(AnalysisMode::Site);
        assert!(
            audit_claims(
                &inventory,
                &[("marketplace.contractor.us-wa", "x.ts".into(), None)]
            )
            .is_empty()
        );
    }
}

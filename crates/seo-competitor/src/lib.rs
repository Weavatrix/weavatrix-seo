//! Public competitor graph comparison. Copies no competitor prose.

#![forbid(unsafe_code)]

mod capability;
mod shape;

use std::collections::BTreeSet;
use weavatrix_seo_model::{Evidence, EvidenceSource, Inventory, Opportunity};

pub use capability::{Artifact, score as score_artifacts, site_backed_ids, tally};
pub use shape::Shape;

/// Comparison request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompareRequest {
    /// Owned site.
    pub site: String,
    /// Public competitor origins.
    pub competitors: Vec<String>,
}

/// Compares an owned inventory with crawled public competitor inventories.
#[must_use]
pub fn compare_inventories(owned: &Inventory, others: &[(String, Inventory)]) -> Vec<Opportunity> {
    let ours = shape::of(owned);
    let mut items = Vec::new();
    let mut schema_missing: BTreeSet<String> = BTreeSet::new();
    let mut seen_archetype = BTreeSet::new();
    for (origin, inventory) in others {
        let theirs = shape::of(inventory);
        for archetype in theirs.archetypes.difference(&ours.archetypes) {
            if !seen_archetype.insert(archetype.clone()) {
                continue;
            }
            items.push(Opportunity::unmeasured_demand(
                "cluster_gap",
                format!("{origin}:{archetype}"),
                format!("owned site lacks the `{archetype}` page archetype seen on a public competitor origin"),
                "The gap is structural. Do not copy competitor copy.",
                format!("Decide whether a first-party `{archetype}` family belongs in the target architecture."),
            ));
        }
        for prefix in theirs.prefixes.difference(&ours.prefixes) {
            if prefix.contains("/blog/") || prefix.contains("/category/") {
                continue;
            }
            if instance_prefix(prefix) {
                continue;
            }
            items.push(Opportunity::unmeasured_demand(
                "cluster_gap",
                format!("{origin}:prefix:{prefix}"),
                format!(
                    "owned site has no `{prefix}` URL family observed on a public competitor origin"
                ),
                "Prefix gaps are structural, not content to clone.",
                "Add the family only when first-party demand and facts exist.",
            ));
        }
        for schema in theirs.schema_types.difference(&ours.schema_types) {
            schema_missing.insert(schema.clone());
        }
        for locale in theirs.locales.difference(&ours.locales) {
            items.push(Opportunity::unmeasured_demand(
                "market_gap",
                format!("{origin}:locale:{locale}"),
                format!("owned site has no observed `{locale}` locale while a public competitor origin does"),
                "Locale coverage is observed from hreflang and html lang.",
                "Add the locale only when the market and content exist.",
            ));
        }
        cardinality_gap(origin, &ours, &theirs, &mut items);
        if theirs.internal_links > ours.internal_links.saturating_mul(2)
            && theirs.internal_links >= 8
        {
            items.push(Opportunity::unmeasured_demand(
                "link_gap",
                origin.clone(),
                format!(
                    "public competitor origin has {} internal links vs owned {}",
                    theirs.internal_links, ours.internal_links
                ),
                "Graph leverage is structural. Do not copy competitor anchors.",
                "Add first-party contextual links toward owned landings and families.",
            ));
        }
        if theirs.indexable > 0
            && ours.indexable > 0
            && theirs.with_h1 * 2 > theirs.indexable
            && ours.with_h1 * 2 < ours.indexable
        {
            items.push(Opportunity::unmeasured_demand(
                "content_gap",
                origin.clone(),
                "owned indexable pages are missing H1 coverage that a public competitor origin has",
                "H1 coverage is a template fact, not a license to copy competitor headings.",
                "Emit one H1 from the owned templates that lack it.",
            ));
        }
    }
    for schema in schema_missing {
        items.push(Opportunity::unmeasured_demand(
            "schema_gap",
            schema.clone(),
            format!(
                "owned site does not emit `{schema}` JSON-LD observed on a public competitor origin"
            ),
            "Schema must stay backed by first-party facts.",
            "Add the type only when domain facts support every required field.",
        ));
    }
    if others.is_empty() {
        items.push(Opportunity::unmeasured_demand(
            "cluster_gap",
            "compare",
            "Competitor comparison is unmeasured",
            "No public competitor origin was crawled.",
            "Pass --competitor URL.",
        ));
    }
    items
}

fn instance_prefix(prefix: &str) -> bool {
    let parts: Vec<&str> = prefix.split('/').filter(|part| !part.is_empty()).collect();
    matches!(parts.as_slice(), [_, second] if instance_segment(second))
}

fn instance_segment(segment: &str) -> bool {
    !segment.is_empty()
        && (segment.chars().all(|ch| ch.is_ascii_digit())
            || (segment.len() >= 8 && segment.bytes().all(|byte| byte.is_ascii_hexdigit())))
}

fn cardinality_gap(origin: &str, ours: &Shape, theirs: &Shape, items: &mut Vec<Opportunity>) {
    for (archetype, count) in &theirs.archetype_counts {
        let owned = ours.archetype_counts.get(archetype).copied().unwrap_or(0);
        if *count >= 4 && *count > owned.saturating_mul(2).saturating_add(2) {
            items.push(Opportunity::unmeasured_demand(
                "cluster_gap",
                format!("{origin}:{archetype}:cardinality"),
                format!(
                    "owned `{archetype}` inventory is {owned} URLs vs {count} on a public competitor origin"
                ),
                "Cardinality is not a reason to generate thin pages.",
                "Expand the family only when each URL has unique facts.",
            ));
        }
    }
}

/// Evidence for an unrun comparison.
#[must_use]
pub fn unmeasured_evidence() -> Evidence {
    Evidence::unmeasured(EvidenceSource::Http)
}

#[cfg(test)]
mod tests {
    use super::compare_inventories;
    use weavatrix_seo_model::{
        AbsoluteUrl, Alternate, AnalysisMode, ContentHash, Evidence, ExtractedPage, Heading,
        Indexability, Inventory, JsonLd, MediaKind, Relation,
    };

    fn page(path: &str, h1: &str) -> ExtractedPage {
        page_on("https://x.test", path, h1)
    }

    fn page_on(origin: &str, path: &str, h1: &str) -> ExtractedPage {
        let url = AbsoluteUrl::parse(&format!("{origin}{path}")).unwrap();
        ExtractedPage {
            url: url.clone(),
            requested: url,
            status: 200,
            redirects: Vec::new(),
            content_type: Some("text/html".into()),
            media: MediaKind::Html,
            canonical: None,
            robots: Vec::new(),
            title: Some(h1.into()),
            description: None,
            html_lang: Some("en".into()),
            alternates: Vec::new(),
            headings: vec![Heading {
                level: 1,
                text: h1.into(),
            }],
            links: Vec::new(),
            link_refs: Vec::new(),
            images: Vec::new(),
            json_ld: Vec::new(),
            text: h1.into(),
            heading_text: h1.into(),
            main_text: String::new(),
            payload: String::new(),
            arbitrary_script: String::new(),
            og_title: None,
            og_description: None,
            og_image: None,
            headers: Vec::new(),
            csp_meta: None,
            body_bytes: 1,
            fetch_ms: 1,
            has_main: false,
            unlabeled_controls: 0,
            content_hash: ContentHash::of_str(h1),
            indexability: Indexability::Indexable,
            in_sitemap: true,
            linked_from_page: true,
            evidence: Evidence::http(),
        }
        .finalize()
    }

    fn without_h1(mut page: ExtractedPage) -> ExtractedPage {
        page.headings.clear();
        page.heading_text.clear();
        page.finalize()
    }

    fn with_schema(mut page: ExtractedPage, ty: &str) -> ExtractedPage {
        page.json_ld.push(JsonLd {
            raw: format!(r#"{{"@type":"{ty}"}}"#),
            types: vec![ty.into()],
            valid_json: true,
            ..JsonLd::default()
        });
        page
    }

    fn with_lang(mut page: ExtractedPage, lang: &str) -> ExtractedPage {
        page.html_lang = Some(lang.into());
        page.alternates.push(Alternate {
            hreflang: lang.into(),
            href: page.url.to_string(),
        });
        page
    }

    fn kinds(items: &[weavatrix_seo_model::Opportunity]) -> Vec<&str> {
        items.iter().map(|item| item.kind.as_str()).collect()
    }

    #[test]
    fn flags_missing_faq_archetype() {
        let mut owned = Inventory::blank(AnalysisMode::Site);
        owned.pages = vec![page("/", "Home")];
        let mut other = Inventory::blank(AnalysisMode::Site);
        other.pages = vec![page("/faq", "FAQ")];
        let items = compare_inventories(&owned, &[("https://c.test/".into(), other)]);
        assert!(
            items.iter().any(|item| item.summary.contains("faq")),
            "{items:?}"
        );
    }

    #[test]
    fn flags_faqpage_schema_without_copying_prose() {
        let secret = "SECRET COMPETITOR COPY UNIQUE PHRASE";
        let mut owned = Inventory::blank(AnalysisMode::Site);
        owned.pages = vec![page("/", "Home")];
        let mut competitor = page_on("https://c.test", "/", "Home");
        competitor.text = secret.into();
        competitor = with_schema(competitor, "FAQPage");
        let mut other = Inventory::blank(AnalysisMode::Site);
        other.pages = vec![competitor];
        let items = compare_inventories(&owned, &[("https://c.test/".into(), other)]);
        assert!(
            items
                .iter()
                .any(|item| item.kind == "schema_gap" && item.summary.contains("FAQPage")),
            "{items:?}"
        );
        assert!(
            items.iter().all(|item| !item.summary.contains(secret)
                && !item.why.contains(secret)
                && !item.action.contains(secret)),
            "compare must not copy competitor prose: {items:?}"
        );
    }

    #[test]
    fn flags_hreflang_locale_gap() {
        let mut owned = Inventory::blank(AnalysisMode::Site);
        owned.pages = vec![page("/", "Home")];
        let mut other = Inventory::blank(AnalysisMode::Site);
        other.pages = vec![with_lang(page_on("https://c.test", "/", "Home"), "he-il")];
        let items = compare_inventories(&owned, &[("https://c.test/".into(), other)]);
        assert!(
            items
                .iter()
                .any(|item| item.kind == "market_gap" && item.summary.contains("he-il")),
            "{items:?}"
        );
    }

    #[test]
    fn flags_service_cardinality() {
        let mut owned = Inventory::blank(AnalysisMode::Site);
        owned.pages = vec![page("/", "Home"), page("/service/one", "One")];
        let mut other = Inventory::blank(AnalysisMode::Site);
        other.pages = (0..6)
            .map(|index| page_on("https://c.test", &format!("/service/{index}"), "Service"))
            .collect();
        let items = compare_inventories(&owned, &[("https://c.test/".into(), other)]);
        assert!(
            items.iter().any(|item| item.kind == "cluster_gap"
                && (item.subject.contains("cardinality") || item.summary.contains("inventory is"))),
            "{items:?}"
        );
    }

    #[test]
    fn skips_numeric_instance_prefixes() {
        let mut owned = Inventory::blank(AnalysisMode::Site);
        owned.pages = vec![page("/", "Home")];
        let mut other = Inventory::blank(AnalysisMode::Site);
        other.pages = (0..6)
            .map(|index| page_on("https://c.test", &format!("/service/{index}"), "Service"))
            .collect();
        let items = compare_inventories(&owned, &[("https://c.test/".into(), other)]);
        assert!(
            items
                .iter()
                .any(|item| item.subject.contains("cardinality")),
            "{items:?}"
        );
        assert!(
            items
                .iter()
                .all(|item| !item.summary.contains("/service/0")),
            "instance prefixes should collapse into cardinality: {items:?}"
        );
    }

    #[test]
    fn flags_guide_prefix_family() {
        let mut owned = Inventory::blank(AnalysisMode::Site);
        owned.pages = vec![page("/", "Home")];
        let mut other = Inventory::blank(AnalysisMode::Site);
        other.pages = vec![page_on("https://c.test", "/guides/how-to-permit", "Guide")];
        let items = compare_inventories(&owned, &[("https://c.test/".into(), other)]);
        assert!(
            items
                .iter()
                .any(|item| item.kind == "cluster_gap" && item.summary.contains("/guides/")),
            "{items:?}"
        );
    }

    #[test]
    fn flags_h1_coverage_gap() {
        let mut owned = Inventory::blank(AnalysisMode::Site);
        owned.pages = vec![
            page("/", "Home"),
            without_h1(page("/a", "A")),
            without_h1(page("/b", "B")),
            without_h1(page("/c", "C")),
        ];
        let mut other = Inventory::blank(AnalysisMode::Site);
        other.pages = vec![
            page_on("https://c.test", "/", "Home"),
            page_on("https://c.test", "/a", "A"),
            page_on("https://c.test", "/b", "B"),
        ];
        let items = compare_inventories(&owned, &[("https://c.test/".into(), other)]);
        assert!(kinds(&items).contains(&"content_gap"), "{items:?}");
    }

    #[test]
    fn flags_internal_link_leverage() {
        let mut owned = Inventory::blank(AnalysisMode::Site);
        owned.pages = vec![page("/", "Home")];
        let mut other = Inventory::blank(AnalysisMode::Site);
        other.pages = vec![page("/", "Home"), page("/a", "A")];
        let src = AbsoluteUrl::parse("https://x.test/").unwrap();
        let dst = AbsoluteUrl::parse("https://x.test/a").unwrap();
        other.edges = (0..8)
            .map(|_| {
                weavatrix_seo_model::GraphEdge::new(
                    src.clone(),
                    dst.clone(),
                    Relation::LinksTo,
                    Evidence::http(),
                )
            })
            .collect();
        let items = compare_inventories(&owned, &[("https://c.test/".into(), other)]);
        assert!(
            items.iter().any(|item| item.kind == "link_gap"),
            "{items:?}"
        );
    }

    #[test]
    fn empty_compare_stays_unmeasured() {
        let owned = Inventory::blank(AnalysisMode::Site);
        let items = compare_inventories(&owned, &[]);
        assert!(
            items.iter().any(|item| item.summary.contains("unmeasured")),
            "{items:?}"
        );
    }
}

//! AI-search citation identity.

use weavatrix_seo_claims::audit_cite;
use weavatrix_seo_model::{
    AbsoluteUrl, AnalysisMode, ContentHash, Evidence, ExtractedPage, Heading, Indexability,
    Inventory, JsonLd, MediaKind, ProducerFact,
};

fn page(url: &str, json_ld: Vec<JsonLd>, headings: Vec<Heading>) -> ExtractedPage {
    ExtractedPage {
        url: AbsoluteUrl::parse(url).unwrap(),
        requested: AbsoluteUrl::parse(url).unwrap(),
        status: 200,
        redirects: Vec::new(),
        content_type: None,
        media: MediaKind::Html,
        canonical: None,
        robots: Vec::new(),
        title: Some("Page".into()),
        description: None,
        html_lang: Some("en".into()),
        alternates: Vec::new(),
        headings,
        links: Vec::new(),
        link_refs: Vec::new(),
        images: Vec::new(),
        json_ld,
        text: String::new(),
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
    }
}

fn codes(inventory: &Inventory) -> Vec<String> {
    audit_cite(inventory)
        .into_iter()
        .map(|item| item.code)
        .collect()
}

#[test]
fn organization_without_id_is_ai_001() {
    let mut inventory = Inventory::blank(AnalysisMode::Site);
    inventory.pages = vec![page(
        "https://kablay.us/",
        vec![JsonLd {
            raw: r#"{"@type":"Organization","name":"Kablay"}"#.into(),
            types: vec!["Organization".into()],
            valid_json: true,
            ..JsonLd::default()
        }],
        Vec::new(),
    )];
    assert!(
        codes(&inventory)
            .iter()
            .any(|code| code == "WVX-SEO-AI-001"),
        "{:?}",
        codes(&inventory)
    );
}

#[test]
fn organization_with_id_is_quiet() {
    let mut inventory = Inventory::blank(AnalysisMode::Site);
    inventory.pages = vec![page(
        "https://kablay.us/",
        vec![JsonLd {
            raw: r#"{"@type":"Organization","@id":"https://kablay.us/#org"}"#.into(),
            types: vec!["Organization".into()],
            valid_json: true,
            ids: vec!["https://kablay.us/#org".into()],
            ..JsonLd::default()
        }],
        Vec::new(),
    )];
    assert!(
        codes(&inventory)
            .iter()
            .all(|code| code != "WVX-SEO-AI-001")
    );
}

#[test]
fn faq_headings_without_faqpage_are_ai_002() {
    let mut inventory = Inventory::blank(AnalysisMode::Site);
    inventory.pages = vec![page(
        "https://kablay.us/help",
        Vec::new(),
        vec![
            Heading {
                level: 2,
                text: "FAQ".into(),
            },
            Heading {
                level: 3,
                text: "How do I book?".into(),
            },
        ],
    )];
    assert_eq!(codes(&inventory), ["WVX-SEO-AI-002"]);
}

#[test]
fn faq_producer_without_schema_is_ai_003() {
    let mut inventory = Inventory::blank(AnalysisMode::Repo);
    inventory.predicted_routes = vec!["/:locale/help".into()];
    inventory.producers = vec![ProducerFact {
        path: "src/lib/faq.ts".into(),
        name: "faqItems".into(),
        content_hash: ContentHash::of(b"x"),
        families: vec!["/:locale/help".into()],
    }];
    assert_eq!(codes(&inventory), ["WVX-SEO-AI-003"]);
}

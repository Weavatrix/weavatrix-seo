//! Pack entity graph: named copy vs JSON-LD, city families vs producers.

use weavatrix_seo_claims::audit_entities;
use weavatrix_seo_model::{
    AbsoluteUrl, AnalysisMode, ContentHash, Evidence, ExtractedPage, Indexability, Inventory,
    JsonLd, MediaKind, ProducerFact,
};

fn page(url: &str, text: &str, json_ld: Vec<JsonLd>) -> ExtractedPage {
    ExtractedPage {
        url: AbsoluteUrl::parse(url).unwrap(),
        requested: AbsoluteUrl::parse(url).unwrap(),
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
        json_ld,
        text: text.into(),
        heading_text: String::new(),
        main_text: String::new(),
        payload: String::new(),
        arbitrary_script: String::new(),
        og_title: None,
        og_description: None,
        og_image: None,
        headers: Vec::new(),
        csp_meta: None,
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

#[test]
fn names_pack_entity_without_json_ld() {
    let mut inventory = Inventory::blank(AnalysisMode::Site);
    inventory.pages = vec![page(
        "https://kablay.us/category/electrician",
        "Licensed work in Clark County, Southwest Washington.",
        Vec::new(),
    )];
    let codes: Vec<_> = audit_entities(&inventory)
        .into_iter()
        .map(|item| item.code)
        .collect();
    assert!(
        codes.iter().any(|code| code == "WVX-SEO-ENTITY-001"),
        "{codes:?}"
    );
}

#[test]
fn schema_that_names_the_entity_is_quiet() {
    let mut inventory = Inventory::blank(AnalysisMode::Site);
    inventory.pages = vec![page(
        "https://kablay.us/category/electrician",
        "Licensed work in Clark County, Southwest Washington.",
        vec![JsonLd {
            raw: r#"{"@type":"Service","areaServed":"Clark County, Southwest Washington"}"#.into(),
            types: vec!["Service".into()],
            valid_json: true,
            ..JsonLd::default()
        }],
    )];
    assert!(
        audit_entities(&inventory)
            .iter()
            .all(|item| item.code != "WVX-SEO-ENTITY-001")
    );
}

#[test]
fn shared_chrome_entities_collapse_to_origin() {
    let mut inventory = Inventory::blank(AnalysisMode::Site);
    inventory.pages = vec![
        page(
            "https://kablay.us/",
            "Southwest Washington. Vancouver WA.",
            Vec::new(),
        ),
        page(
            "https://kablay.us/blog",
            "Southwest Washington. Vancouver WA.",
            Vec::new(),
        ),
        page(
            "https://kablay.us/about",
            "Southwest Washington. Vancouver WA.",
            Vec::new(),
        ),
        page(
            "https://kablay.us/help",
            "Southwest Washington. Vancouver WA.",
            Vec::new(),
        ),
    ];
    let items: Vec<_> = audit_entities(&inventory)
        .into_iter()
        .filter(|item| item.code == "WVX-SEO-ENTITY-001")
        .collect();
    assert_eq!(items.len(), 1, "{items:?}");
    assert!(items[0].summary.contains("shared chrome"), "{:?}", items[0]);
}

#[test]
fn city_family_without_producer_is_entity_002() {
    let mut inventory = Inventory::blank(AnalysisMode::Repo);
    inventory.predicted_routes = vec!["/:locale/category/:city".into()];
    inventory.producers = vec![ProducerFact {
        path: "src/app/[locale]/category/[city]/page.tsx".into(),
        name: "Page".into(),
        content_hash: ContentHash::of(b"x"),
        families: vec!["/:locale/category/:city".into()],
        symbol_hash: None,
        start_line: None,
        end_line: None,
    }];
    let codes: Vec<_> = audit_entities(&inventory)
        .into_iter()
        .map(|item| item.code)
        .collect();
    assert_eq!(codes, ["WVX-SEO-ENTITY-002"]);
}

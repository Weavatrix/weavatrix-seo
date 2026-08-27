//! City Place/Service JSON-LD must bind a geography.

use weavatrix_seo_claims::audit_local;
use weavatrix_seo_model::{
    AbsoluteUrl, AnalysisMode, ContentHash, Evidence, ExtractedPage, Indexability, Inventory,
    JsonLd, MediaKind,
};

fn page(url: &str, raw: &str, types: &[&str]) -> ExtractedPage {
    ExtractedPage {
        url: AbsoluteUrl::parse(url).unwrap(),
        requested: AbsoluteUrl::parse(url).unwrap(),
        status: 200,
        redirects: Vec::new(),
        content_type: None,
        media: MediaKind::Html,
        canonical: None,
        robots: Vec::new(),
        title: Some("City".into()),
        description: None,
        html_lang: Some("en".into()),
        alternates: Vec::new(),
        headings: Vec::new(),
        links: Vec::new(),
        link_refs: Vec::new(),
        images: Vec::new(),
        json_ld: vec![JsonLd {
            raw: raw.into(),
            types: types.iter().map(|item| (*item).to_owned()).collect(),
            valid_json: true,
            ..JsonLd::default()
        }],
        text: "City landing".into(),
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
fn city_service_without_address_is_local() {
    let mut inventory = Inventory::blank(AnalysisMode::Site);
    inventory.pages = vec![page(
        "https://kablay.us/cities/vancouver",
        r#"{"@type":"Service","name":"Electrician"}"#,
        &["Service"],
    )];
    let codes: Vec<_> = audit_local(&inventory)
        .into_iter()
        .map(|item| item.code)
        .collect();
    assert_eq!(codes, ["WVX-SEO-LOCAL-001"]);
}

#[test]
fn city_service_with_area_served_is_quiet() {
    let mut inventory = Inventory::blank(AnalysisMode::Site);
    inventory.pages = vec![page(
        "https://kablay.us/cities/vancouver",
        r#"{"@type":"Service","areaServed":{"@type":"City","name":"Vancouver"}}"#,
        &["Service", "City"],
    )];
    assert!(audit_local(&inventory).is_empty());
}

#[test]
fn default_locale_city_family_is_local() {
    let mut inventory = Inventory::blank(AnalysisMode::Hybrid);
    inventory.predicted_routes = vec!["/:locale/category/:slug/:city".into()];
    inventory.pages = vec![page(
        "https://kablay.us/category/cleaning/camas-wa",
        r#"{"@type":"Service","name":"Cleaning"}"#,
        &["Service"],
    )];
    let codes: Vec<_> = audit_local(&inventory)
        .into_iter()
        .map(|item| item.code)
        .collect();
    assert_eq!(codes, ["WVX-SEO-LOCAL-001"]);
}

#[test]
fn non_city_url_without_address_is_quiet() {
    let mut inventory = Inventory::blank(AnalysisMode::Site);
    inventory.pages = vec![page(
        "https://kablay.us/about",
        r#"{"@type":"Organization","name":"Kablay"}"#,
        &["Organization"],
    )];
    assert!(audit_local(&inventory).is_empty());
}

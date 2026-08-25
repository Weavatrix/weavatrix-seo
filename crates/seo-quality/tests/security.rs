//! Origin HSTS/CSP facts: values and splits, not folklore max-age.

use weavatrix_seo_model::{
    AbsoluteUrl, AnalysisMode, ContentHash, Evidence, ExtractedPage, Indexability, Inventory,
    MediaKind,
};
use weavatrix_seo_quality::audit;

fn page(url: &str, headers: Vec<(&str, &str)>) -> ExtractedPage {
    ExtractedPage {
        url: AbsoluteUrl::parse(url).unwrap(),
        requested: AbsoluteUrl::parse(url).unwrap(),
        status: 200,
        redirects: Vec::new(),
        content_type: Some("text/html".into()),
        media: MediaKind::Html,
        canonical: None,
        robots: Vec::new(),
        title: Some("Home".into()),
        description: None,
        html_lang: Some("en".into()),
        alternates: Vec::new(),
        headings: Vec::new(),
        links: Vec::new(),
        link_refs: Vec::new(),
        images: Vec::new(),
        json_ld: Vec::new(),
        text: "Home".into(),
        heading_text: String::new(),
        main_text: String::new(),
        payload: String::new(),
        arbitrary_script: String::new(),
        og_title: None,
        og_description: None,
        og_image: None,
        headers: headers
            .into_iter()
            .map(|(name, value)| (name.to_owned(), value.to_owned()))
            .collect(),
        body_bytes: 4,
        fetch_ms: 1,
        has_main: true,
        unlabeled_controls: 0,
        content_hash: ContentHash::of(b"x"),
        indexability: Indexability::Indexable,
        in_sitemap: true,
        linked_from_page: true,
        evidence: Evidence::http(),
    }
}

fn codes(pages: Vec<ExtractedPage>) -> Vec<String> {
    let mut inventory = Inventory::blank(AnalysisMode::Site);
    inventory.pages = pages;
    audit(&inventory)
        .into_iter()
        .filter(|item| item.code.starts_with("WVX-SEO-SEC-"))
        .map(|item| item.code)
        .collect()
}

#[test]
fn https_without_hsts_is_sec_001() {
    let found = codes(vec![page("https://kablay.us/", vec![])]);
    assert!(found.contains(&"WVX-SEO-SEC-001".into()), "{found:?}");
}

#[test]
fn hsts_max_age_zero_is_sec_006() {
    let found = codes(vec![page(
        "https://kablay.us/",
        vec![("strict-transport-security", "max-age=0")],
    )]);
    assert!(found.contains(&"WVX-SEO-SEC-006".into()), "{found:?}");
    assert!(!found.contains(&"WVX-SEO-SEC-001".into()), "{found:?}");
}

#[test]
fn csp_frame_ancestors_quiets_xfo() {
    let found = codes(vec![page(
        "https://kablay.us/",
        vec![
            ("strict-transport-security", "max-age=300"),
            ("content-security-policy", "frame-ancestors 'self'"),
            ("x-content-type-options", "nosniff"),
            ("referrer-policy", "no-referrer"),
        ],
    )]);
    assert!(
        !found.iter().any(|code| code == "WVX-SEO-SEC-004"),
        "{found:?}"
    );
}

#[test]
fn mixed_hsts_is_origin_split() {
    let found = codes(vec![
        page(
            "https://kablay.us/",
            vec![("strict-transport-security", "max-age=300")],
        ),
        page("https://kablay.us/about", vec![]),
    ]);
    assert!(found.contains(&"WVX-SEO-SEC-007".into()), "{found:?}");
    assert!(!found.contains(&"WVX-SEO-SEC-001".into()), "{found:?}");
}

#[test]
fn report_only_csp_is_not_enforcing() {
    let found = codes(vec![page(
        "https://kablay.us/",
        vec![("content-security-policy-report-only", "default-src 'self'")],
    )]);
    assert!(found.contains(&"WVX-SEO-SEC-003".into()), "{found:?}");
}
//! Canonical chains, hreflang errors, duplicate descriptions.

use weavatrix_seo_model::{
    AbsoluteUrl, Alternate, AnalysisMode, ContentHash, Evidence, ExtractedPage, Indexability,
    Inventory, MediaKind,
};
use weavatrix_seo_rules::audit;

fn page(
    url: &str,
    status: u16,
    canonical: Option<&str>,
    description: Option<&str>,
) -> ExtractedPage {
    let parsed = AbsoluteUrl::parse(url).unwrap();
    ExtractedPage {
        url: parsed.clone(),
        requested: parsed,
        status,
        redirects: Vec::new(),
        content_type: Some("text/html".into()),
        media: MediaKind::Html,
        canonical: canonical.map(ToOwned::to_owned),
        robots: Vec::new(),
        title: Some("T".into()),
        description: description.map(ToOwned::to_owned),
        html_lang: Some("en".into()),
        alternates: Vec::new(),
        headings: Vec::new(),
        links: Vec::new(),
        link_refs: Vec::new(),
        images: Vec::new(),
        json_ld: Vec::new(),
        text: "body".into(),
        heading_text: String::new(),
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
        content_hash: ContentHash::of_str(url),
        indexability: Indexability::Indexable,
        in_sitemap: true,
        linked_from_page: true,
        evidence: Evidence::http(),
    }
    .finalize()
}

fn codes(inventory: &Inventory) -> Vec<String> {
    audit(inventory)
        .into_iter()
        .map(|finding| finding.code)
        .collect()
}

#[test]
fn canonical_chain_is_canon_003() {
    let a = page("https://x.test/a", 200, Some("https://x.test/b"), None);
    let b = page("https://x.test/b", 200, Some("https://x.test/c"), None);
    let c = page("https://x.test/c", 200, Some("https://x.test/c"), None);
    let mut inventory = Inventory::blank(AnalysisMode::Site);
    inventory.pages = vec![a, b, c];
    let codes = codes(&inventory);
    assert!(
        codes.iter().any(|code| code == "WVX-SEO-CANON-003"),
        "{codes:?}"
    );
}

#[test]
fn hreflang_to_404_is_i18n_004() {
    let mut en = page("https://x.test/en", 200, Some("https://x.test/en"), None);
    en.alternates.push(Alternate {
        hreflang: "he".into(),
        href: "https://x.test/he".into(),
    });
    let he = page("https://x.test/he", 404, None, None);
    let mut inventory = Inventory::blank(AnalysisMode::Site);
    inventory.pages = vec![en, he];
    let codes = codes(&inventory);
    assert!(
        codes.iter().any(|code| code == "WVX-SEO-I18N-004"),
        "{codes:?}"
    );
}

#[test]
fn reused_description_is_meta_006() {
    let a = page(
        "https://x.test/a",
        200,
        Some("https://x.test/a"),
        Some("Same snippet"),
    );
    let b = page(
        "https://x.test/b",
        200,
        Some("https://x.test/b"),
        Some("Same snippet"),
    );
    let mut inventory = Inventory::blank(AnalysisMode::Site);
    inventory.pages = vec![a, b];
    let codes = codes(&inventory);
    assert!(
        codes.iter().any(|code| code == "WVX-SEO-META-006"),
        "{codes:?}"
    );
}

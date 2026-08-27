//! Redirect and relative canonical classification.

use weavatrix_seo_model::{
    AbsoluteUrl, ContentHash, Evidence, ExtractedPage, Indexability, MediaKind, RedirectHop,
};

fn base(
    url: &str,
    status: u16,
    canonical: Option<&str>,
    redirects: Vec<RedirectHop>,
) -> ExtractedPage {
    let parsed = AbsoluteUrl::parse(url).unwrap();
    ExtractedPage {
        url: parsed.clone(),
        requested: parsed,
        status,
        redirects,
        content_type: Some("text/html".into()),
        media: MediaKind::Html,
        canonical: canonical.map(ToOwned::to_owned),
        robots: Vec::new(),
        title: None,
        description: None,
        html_lang: None,
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
        body_bytes: 4,
        fetch_ms: 1,
        has_main: false,
        unlabeled_controls: 0,
        content_hash: ContentHash::of(b"x"),
        indexability: Indexability::Indexable,
        in_sitemap: false,
        linked_from_page: false,
        evidence: Evidence::http(),
    }
    .finalize()
}

#[test]
fn redirect_hops_do_not_make_final_page_redirected() {
    let hops = vec![RedirectHop {
        from: "https://example.com/old".into(),
        to: "https://example.com/new".into(),
        status: 301,
    }];
    let final_page = base("https://example.com/new", 200, None, hops);
    assert_eq!(final_page.indexability, Indexability::Indexable);
    let hop = base("https://example.com/old", 301, None, Vec::new());
    assert_eq!(hop.indexability, Indexability::Redirected);
}

#[test]
fn relative_canonical_is_resolved() {
    let page = base("https://example.com/old", 200, Some("/new"), Vec::new());
    assert_eq!(page.indexability, Indexability::Canonicalized);
}

#[test]
fn self_relative_canonical_stays_indexable() {
    let page = base("https://example.com/new", 200, Some("/new"), Vec::new());
    assert_eq!(page.indexability, Indexability::Indexable);
}

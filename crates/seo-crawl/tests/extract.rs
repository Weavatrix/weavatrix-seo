//! HTML extraction from the weavatrix-parse token stream.

use weavatrix_seo_crawl::extract_html;

#[test]
fn keeps_query_string_in_href() {
    let draft = extract_html(
        r#"<html><body><a href="/specialists?city=vancouver-wa">City</a></body></html>"#,
    );
    assert!(
        draft
            .links
            .iter()
            .any(|href| href.contains("city=vancouver-wa")),
        "{:?}",
        draft.links
    );
}

#[test]
fn reads_title_canonical_links_and_json_ld() {
    let html = r#"<!doctype html>
<html lang="en">
<head>
  <title> Guide </title>
  <link rel="canonical" href="https://x.test/guide">
  <link rel="alternate" hreflang="ru" href="https://x.test/ru/guide">
  <meta name="description" content="A guide.">
  <meta name="robots" content="index,follow">
  <meta property="og:title" content="OG Guide">
  <meta property="og:image" content="https://x.test/og.png">
  <script type="application/ld+json">{"@type":"Article","name":"Guide"}</script>
</head>
<body>
  <nav><a href="/nav">Nav</a></nav>
  <h1>Heading</h1>
  <p>Body text.</p>
  <a href="/about">About</a>
  <img src="/x.png" alt="diagram">
  <footer>ignore</footer>
</body>
</html>"#;
    let draft = extract_html(html);
    assert_eq!(draft.title.as_deref(), Some("Guide"));
    assert_eq!(draft.canonical.as_deref(), Some("https://x.test/guide"));
    assert_eq!(draft.html_lang.as_deref(), Some("en"));
    assert_eq!(draft.description.as_deref(), Some("A guide."));
    assert!(draft.links.iter().any(|href| href == "/about"));
    assert_eq!(draft.headings[0].text, "Heading");
    assert!(
        draft
            .json_ld
            .iter()
            .any(|block| block.types.contains(&"Article".to_owned())),
        "{:?}",
        draft.json_ld
    );
    assert!(draft.text.contains("Body text"));
    assert!(!draft.text.contains("ignore"));
    assert_eq!(draft.og_title.as_deref(), Some("OG Guide"));
    assert_eq!(draft.og_image.as_deref(), Some("https://x.test/og.png"));
}

#[test]
fn reads_csp_http_equiv() {
    let draft = extract_html(
        r#"<html><head><meta http-equiv="Content-Security-Policy" content="default-src self"></head><body><h1>Home</h1></body></html>"#,
    );
    assert_eq!(draft.csp_meta.as_deref(), Some("default-src self"));
}

#[test]
fn reads_organization_citation_id() {
    let draft = extract_html(
        r#"<html><head><script type="application/ld+json">{"@type":"Organization","@id":"https://x.test/#org","sameAs":["https://x.com/x"]}</script></head><body><h1>Home</h1></body></html>"#,
    );
    let block = draft.json_ld.first().expect("jsonld");
    assert!(block.types.iter().any(|kind| kind == "Organization"));
    assert_eq!(block.ids, ["https://x.test/#org"]);
    assert_eq!(block.same_as, ["https://x.com/x"]);
}

#[test]
fn empty_alt_is_present_missing_alt_is_none() {
    let draft = extract_html(
        r#"<html><body>
          <img src="/icon.svg" alt="">
          <img src="/hero.png">
          <img src="/skip.png" aria-hidden="true">
          <main><p>Body</p></main>
        </body></html>"#,
    );
    assert_eq!(draft.images[0].alt.as_deref(), Some(""));
    assert!(draft.images[1].alt.is_none());
    assert!(draft.images[2].hidden);
    assert!(draft.has_main);
}

#[test]
fn button_inner_text_counts_as_accessible_name() {
    let draft = extract_html(
        r#"<html lang="en"><body>
          <main>
            <button type="button">English</button>
            <button type="submit">Send</button>
            <textarea placeholder="Describe the work"></textarea>
          </main>
        </body></html>"#,
    );
    assert_eq!(draft.unlabeled_controls, 1);
}

#[test]
fn heading_and_body_buffers_stay_separate() {
    let draft = extract_html(
        "<html><body><p>One</p><h2>Mid</h2><p>Two</p><h3>End</h3><p>Three</p></body></html>",
    );
    assert_eq!(draft.heading_text, "Mid End");
    assert!(
        draft.text.contains("One") && draft.text.contains("Two") && draft.text.contains("Three")
    );
    assert!(!draft.text.contains("Mid"));
}

#[test]
fn link_keeps_anchor_rel_and_nav_location() {
    let draft = extract_html(
        r#"<html><body><nav><a href="/about" rel="nofollow">About us</a></nav><main><a href="/post">Read</a></main></body></html>"#,
    );
    let nav = draft
        .link_refs
        .iter()
        .find(|link| link.href == "/about")
        .expect("nav link");
    assert_eq!(nav.anchor.as_deref(), Some("About us"));
    assert!(nav.rel.iter().any(|token| token == "nofollow"));
    assert_eq!(nav.location, weavatrix_seo_model::LinkLocation::Nav);
    let body = draft
        .link_refs
        .iter()
        .find(|link| link.href == "/post")
        .expect("body link");
    assert_eq!(body.location, weavatrix_seo_model::LinkLocation::Contextual);
}

#[test]
fn link_context_is_last_heading() {
    let draft = extract_html(
        r#"<html><body><h2>Guides</h2><p>See <a href="/guide">the guide</a>.</p></body></html>"#,
    );
    let link = draft
        .link_refs
        .iter()
        .find(|item| item.href == "/guide")
        .expect("link");
    assert_eq!(link.context.as_deref(), Some("Guides"));
}

#[test]
fn arbitrary_script_is_not_payload_rsc_is() {
    let draft = extract_html(
        r#"<html><body>
        <script>const unitPiece = "IEC";</script>
        <script id="__NEXT_DATA__" type="application/json">{"iec":"Hevrat"}</script>
        <p>Visible</p>
        </body></html>"#,
    );
    assert!(draft.arbitrary_script.contains("IEC"));
    assert!(!draft.payload.contains("IEC"));
    assert!(draft.payload.contains("Hevrat"));
    assert!(draft.text.contains("Visible"));
}

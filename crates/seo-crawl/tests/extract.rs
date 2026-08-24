//! HTML extraction from the weavatrix-parse token stream.

use weavatrix_seo_crawl::extract_html;

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

//! URL identity: trailing slash, query join, IPv6 brackets.

use weavatrix_seo_model::AbsoluteUrl;

#[test]
fn trailing_slash_stays_distinct() {
    let plain = AbsoluteUrl::parse("https://example.com/foo").unwrap();
    let slash = AbsoluteUrl::parse("https://example.com/foo/").unwrap();
    assert_ne!(plain, slash);
    assert_eq!(plain.path(), "/foo");
    assert_eq!(slash.path(), "/foo/");
    assert_eq!(plain.to_string(), "https://example.com/foo");
    assert_eq!(slash.to_string(), "https://example.com/foo/");
}

#[test]
fn query_only_join_keeps_current_path() {
    let base = AbsoluteUrl::parse("https://example.com/dir/page").unwrap();
    let joined = base.join("?page=2").unwrap();
    assert_eq!(joined.to_string(), "https://example.com/dir/page?page=2");
    assert_eq!(joined.path(), "/dir/page");
    assert_eq!(joined.query(), Some("page=2"));
}

#[test]
fn ipv6_origin_uses_brackets() {
    let url = AbsoluteUrl::parse("http://[2001:db8::1]:8080/x").unwrap();
    assert_eq!(url.host(), "2001:db8::1");
    assert_eq!(url.origin(), "http://[2001:db8::1]:8080");
    assert_eq!(url.to_string(), "http://[2001:db8::1]:8080/x");
}

#[test]
fn drops_fragment_and_default_port() {
    let url = AbsoluteUrl::parse("HTTPS://Example.COM:443/a/./b/../c#frag").unwrap();
    assert_eq!(url.to_string(), "https://example.com/a/c");
}

#[test]
fn joins_relative_and_root_paths() {
    let base = AbsoluteUrl::parse("http://example.com/dir/page").unwrap();
    assert_eq!(
        base.join("other").unwrap().to_string(),
        "http://example.com/dir/other"
    );
    assert_eq!(
        base.join("/root").unwrap().to_string(),
        "http://example.com/root"
    );
}

#[test]
fn rejects_credentials_and_non_http() {
    assert!(AbsoluteUrl::parse("http://user:pass@example.com/").is_err());
    assert!(AbsoluteUrl::parse("ftp://example.com/").is_err());
    let base = AbsoluteUrl::parse("http://example.com/").unwrap();
    assert!(base.join("mailto:a@b.c").is_err());
}

#[test]
fn relative_canonical_resolves() {
    let page = AbsoluteUrl::parse("https://example.com/old").unwrap();
    assert_eq!(
        page.join("/new").unwrap().to_string(),
        "https://example.com/new"
    );
}

//! Eleven-tool catalog and CLI parity of the crawl-backed schema.

use mcport::ConcurrentToolServer;
use weavatrix_seo_mcp::{Roots, seo_server};

#[test]
fn catalog_has_eleven_tools() {
    let server = seo_server(20, &Roots::new(&[]));
    let catalog = server.catalog().to_string();
    for name in [
        "seo_inventory",
        "seo_audit",
        "seo_opportunities",
        "seo_plan",
        "seo_compare",
        "seo_links",
        "seo_vectors",
        "seo_diff",
        "seo_gate",
        "seo_explain",
        "seo_observations",
    ] {
        assert!(catalog.contains(name), "{catalog}");
    }
}

#[test]
fn crawl_schema_accepts_every_cli_evidence_import() {
    let server = seo_server(20, &Roots::new(&[]));
    let catalog = server.catalog().to_string();
    for field in ["gsc", "observations", "history", "workers", "render"] {
        assert!(catalog.contains(field), "missing {field} in {catalog}");
    }
}

#[test]
fn allow_root_is_a_host_option() {
    let options = weavatrix_seo_mcp::parse_host_args(&[
        "--allow-root".into(),
        ".".into(),
        "--max-pages".into(),
        "10".into(),
    ])
    .expect("parse");
    assert_eq!(options.max_pages, 10);
    assert_eq!(options.roots, vec![".".to_owned()]);
}

#[test]
fn a_server_without_declared_roots_still_has_a_boundary() {
    let roots = Roots::new(&[]);
    assert!(
        !roots.allowed().is_empty(),
        "an empty allow-list must fall back to the working directory, not to the whole disk"
    );
    assert!(
        roots.resolve("repo", "C:\\Windows\\System32").is_err()
            || roots.resolve("repo", "/etc").is_err(),
        "a system path outside the working directory must be refused"
    );
}

#[test]
fn no_tool_advertises_an_unimplemented_scope() {
    let server = seo_server(20, &Roots::new(&[]));
    let catalog = server.catalog().to_string();
    assert!(!catalog.contains("\"scope\""), "{catalog}");
}

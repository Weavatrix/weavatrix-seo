//! Eleven-tool catalog and CLI parity of the crawl-backed schema.

use mcport::ConcurrentToolServer;
use weavatrix_seo_mcp::seo_server;

#[test]
fn catalog_has_eleven_tools() {
    let server = seo_server(20);
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
    let server = seo_server(20);
    let catalog = server.catalog().to_string();
    for field in ["gsc", "observations", "history", "workers", "render"] {
        assert!(catalog.contains(field), "missing {field} in {catalog}");
    }
}

#[test]
fn no_tool_advertises_an_unimplemented_scope() {
    let server = seo_server(20);
    let catalog = server.catalog().to_string();
    assert!(!catalog.contains("\"scope\""), "{catalog}");
}

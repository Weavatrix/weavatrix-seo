//! Eight-tool catalog.

use mcport::ConcurrentToolServer;
use weavatrix_seo_mcp::seo_server;

#[test]
fn catalog_has_eight_tools() {
    let server = seo_server(20);
    let catalog = server.catalog().to_string();
    for name in [
        "seo_inventory",
        "seo_audit",
        "seo_opportunities",
        "seo_plan",
        "seo_compare",
        "seo_diff",
        "seo_explain",
        "seo_observations",
    ] {
        assert!(catalog.contains(name), "{catalog}");
    }
}

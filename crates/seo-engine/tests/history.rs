//! Compact history and revision-bound `seo_diff`.

use std::collections::BTreeMap;
use weavatrix_seo::{AuditRequest, diff_paths, run_audit, save_history};

mod common;

use common::{html, page, spawn};

#[test]
fn history_roundtrip_diffs_added_url() {
    let mut pages = BTreeMap::new();
    pages.insert(
        "/".into(),
        page(
            200,
            html(
                "Home",
                "<link rel=\"canonical\" href=\"/\">",
                "<h1>Home</h1><p>Hi.</p><a href=\"/about\">About</a>",
            ),
        ),
    );
    pages.insert(
        "/about".into(),
        page(
            200,
            html(
                "About",
                "<link rel=\"canonical\" href=\"/about\">",
                "<h1>About</h1><p>Later.</p>",
            ),
        ),
    );
    let site = spawn(pages);
    let origin = format!("{}/", site.base);
    let small = run_audit(&AuditRequest {
        site: Some(origin.clone()),
        max_pages: Some(1),
        workers: Some(1),
        ..AuditRequest::default()
    })
    .expect("small");
    let full = run_audit(&AuditRequest {
        site: Some(origin),
        max_pages: Some(8),
        workers: Some(1),
        ..AuditRequest::default()
    })
    .expect("full");
    let dir = std::env::temp_dir().join(format!("wvx-seo-hist-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let base = save_history(dir.to_string_lossy().as_ref(), &small).expect("save base");
    let head = save_history(dir.to_string_lossy().as_ref(), &full).expect("save head");
    let delta = diff_paths(&base, &head).expect("diff");
    assert!(delta.comparable, "{delta:?}");
    assert!(
        delta.urls_added.iter().any(|url| url.contains("/about")),
        "{delta:?}"
    );
    assert_ne!(small.inventory.snapshot_id, full.inventory.snapshot_id);
    let _ = std::fs::remove_dir_all(dir);
}

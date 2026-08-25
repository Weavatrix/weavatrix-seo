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

#[test]
fn worktree_dirs_diff_predicted_routes() {
    let fixture =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../seo-nextjs/tests/fixtures");
    let head_dir = std::env::temp_dir().join(format!("wvx-seo-wt-head-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&head_dir);
    copy_tree(&fixture, &head_dir);
    let extra = head_dir.join("src/app/about/page.tsx");
    std::fs::create_dir_all(extra.parent().expect("parent")).expect("about dir");
    std::fs::write(&extra, "export default function About() { return null; }\n")
        .expect("about page");
    let delta = diff_paths(
        fixture.to_string_lossy().as_ref(),
        head_dir.to_string_lossy().as_ref(),
    )
    .expect("diff");
    assert!(delta.comparable, "{delta:?}");
    assert!(!delta.unmeasured, "{delta:?}");
    assert!(
        delta
            .routes_added
            .iter()
            .any(|route| route.contains("about")),
        "{delta:?}"
    );
    let _ = std::fs::remove_dir_all(head_dir);
}

#[test]
fn helper_edit_impacts_city_family() {
    let fixture =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../seo-nextjs/tests/fixtures");
    let head_dir = std::env::temp_dir().join(format!("wvx-seo-wt-helper-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&head_dir);
    copy_tree(&fixture, &head_dir);
    std::fs::write(
        head_dir.join("src/lib/citySeo.ts"),
        "export function cityTitle(): string { return \"Changed City\"; }\n",
    )
    .expect("helper");
    let delta = diff_paths(
        fixture.to_string_lossy().as_ref(),
        head_dir.to_string_lossy().as_ref(),
    )
    .expect("diff");
    assert!(delta.comparable, "{delta:?}");
    assert!(
        delta
            .producers_changed
            .iter()
            .any(|item| item.contains("citySeo")),
        "{delta:?}"
    );
    assert!(
        delta
            .families_impacted
            .iter()
            .any(|item| item.contains("category") && item.contains("city")),
        "{delta:?}"
    );
    let _ = std::fs::remove_dir_all(head_dir);
}

#[test]
fn import_edit_impacts_city_family() {
    let fixture =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../seo-nextjs/tests/fixtures");
    let head_dir = std::env::temp_dir().join(format!("wvx-seo-wt-import-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&head_dir);
    copy_tree(&fixture, &head_dir);
    std::fs::write(
        head_dir.join("src/lib/cities.ts"),
        "export const CITY = \"Changed\";\n",
    )
    .expect("import");
    let delta = diff_paths(
        fixture.to_string_lossy().as_ref(),
        head_dir.to_string_lossy().as_ref(),
    )
    .expect("diff");
    assert!(
        delta
            .producers_changed
            .iter()
            .any(|item| item.contains("cities")),
        "{delta:?}"
    );
    assert!(
        delta
            .families_impacted
            .iter()
            .any(|item| item.contains("category") && item.contains("city")),
        "{delta:?}"
    );
    let _ = std::fs::remove_dir_all(&head_dir);
}

#[test]
fn git_shas_without_snapshots_stay_unmeasured() {
    let delta = diff_paths("aaaaaaaa", "bbbbbbbb").expect("diff");
    assert!(!delta.comparable);
    assert!(delta.unmeasured);
}

fn copy_tree(from: &std::path::Path, to: &std::path::Path) {
    std::fs::create_dir_all(to).expect("mkdir");
    for entry in std::fs::read_dir(from).expect("read") {
        let entry = entry.expect("entry");
        let dest = to.join(entry.file_name());
        if entry.file_type().expect("type").is_dir() {
            copy_tree(&entry.path(), &dest);
        } else {
            let _ = std::fs::copy(entry.path(), dest);
        }
    }
}

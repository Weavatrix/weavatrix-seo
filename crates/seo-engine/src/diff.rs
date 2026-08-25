//! Load snapshots, worktrees, or git SHAs for `seo_diff`.

use std::path::Path;
use weavatrix_seo_history::{SearchDiff, StoredSnapshot, diff};
use weavatrix_seo_model::{AnalysisMode, InventoryCounts, POLICY_VERSION};
use weavatrix_seo_nextjs::predict;

use crate::request::read_revision;

/// Diffs two snapshot files, worktree directories, or git SHAs.
///
/// Git SHAs without snapshot files stay unmeasured. Two worktrees compare
/// predicted routes only.
///
/// # Errors
///
/// Returns IO or JSON errors for existing files that cannot be parsed.
pub fn diff_paths(base: &str, head: &str) -> Result<SearchDiff, String> {
    let base_snap = load_side(base)?;
    let head_snap = load_side(head)?;
    let mut delta = diff(&base_snap, &head_snap);
    let empty = |snap: &StoredSnapshot| snap.pages.is_empty() && snap.predicted_routes.is_empty();
    if empty(&base_snap) && empty(&head_snap) {
        delta.comparable = false;
        delta.unmeasured = true;
    }
    Ok(delta)
}

fn load_side(path: &str) -> Result<StoredSnapshot, String> {
    let location = Path::new(path);
    if location.is_dir() {
        return Ok(from_worktree(path));
    }
    if location.is_file() {
        return weavatrix_seo_history::load(path);
    }
    if looks_like_revision(path) {
        return Ok(unmeasured_revision(path));
    }
    Err(format!(
        "diff path `{path}` is not a snapshot file or worktree"
    ))
}

fn from_worktree(repo: &str) -> StoredSnapshot {
    let surface = predict(repo);
    let revision = read_revision(repo);
    StoredSnapshot {
        schema: "weavatrix-seo-snapshot/v1".into(),
        snapshot_id: revision
            .clone()
            .unwrap_or_else(|| format!("worktree:{repo}")),
        run_id: format!("worktree:{repo}"),
        policy_version: POLICY_VERSION.to_owned(),
        config_digest: String::new(),
        mode: AnalysisMode::Repo,
        site: None,
        repo: Some(repo.to_owned()),
        repo_revision: revision,
        predicted_routes: surface.patterns(),
        pages: Vec::new(),
        findings: Vec::new(),
        counts: InventoryCounts::default(),
    }
}

fn unmeasured_revision(revision: &str) -> StoredSnapshot {
    StoredSnapshot {
        schema: "weavatrix-seo-snapshot/v1".into(),
        snapshot_id: format!("git:{revision}"),
        run_id: String::new(),
        policy_version: POLICY_VERSION.to_owned(),
        config_digest: String::new(),
        mode: AnalysisMode::Repo,
        site: None,
        repo: None,
        repo_revision: Some(revision.to_owned()),
        predicted_routes: Vec::new(),
        pages: Vec::new(),
        findings: Vec::new(),
        counts: InventoryCounts::default(),
    }
}

fn looks_like_revision(value: &str) -> bool {
    let trimmed = value.trim();
    let len = trimmed.len();
    (7..=64).contains(&len) && trimmed.bytes().all(|byte| byte.is_ascii_hexdigit())
}

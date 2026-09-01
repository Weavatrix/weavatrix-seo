//! Repo-only dogfood against sibling Weavatrix product repos when present.

use std::path::{Path, PathBuf};
use weavatrix_seo::{AnalysisMode, AuditRequest, run_audit};

fn sibling(name: &str) -> Option<PathBuf> {
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidate = here.join("..").join("..").join("..").join(name);
    candidate.exists().then_some(candidate)
}

fn repo_audit(path: &Path) {
    let report = run_audit(&AuditRequest {
        mode: AnalysisMode::Repo,
        repo: Some(path.to_string_lossy().into_owned()),
        max_pages: Some(1),
        ..AuditRequest::default()
    })
    .expect("repo audit");
    assert!(
        report.intelligence.is_some(),
        "repo-only runs still carry evidence semantics"
    );
    assert_eq!(report.inventory.policy_version, env!("CARGO_PKG_VERSION"));
    assert!(
        !report.inventory.predicted_routes.is_empty() || !report.inventory.producers.is_empty(),
        "expected Next.js families or producers in {}",
        path.display()
    );
}

#[test]
fn kablay_us_repo_only() {
    let Some(path) = sibling("kablay-us") else {
        return;
    };
    repo_audit(&path);
}

#[test]
fn nikogyps_or_kablay_il_repo_only() {
    for name in [
        "nikogyps",
        "niko-gyps",
        "nikogips",
        "NikoGyps",
        "nico-gyps",
        "profi",
        "kablay-il",
    ] {
        if let Some(path) = sibling(name) {
            repo_audit(&path);
            return;
        }
    }
}

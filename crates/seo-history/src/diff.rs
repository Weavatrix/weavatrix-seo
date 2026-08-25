//! Revision-bound search-surface diff.

use crate::StoredSnapshot;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use weavatrix_seo_model::Severity;

/// Identity of one side of a diff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffRef {
    /// Snapshot id.
    pub snapshot_id: String,
    /// Repo revision when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_revision: Option<String>,
    /// Site seed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub site: Option<String>,
}

/// Search-surface delta between two snapshots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchDiff {
    /// Base side.
    pub base: DiffRef,
    /// Head side.
    pub head: DiffRef,
    /// Origin/mode/policy matched.
    pub comparable: bool,
    /// URLs in head not in base.
    pub urls_added: Vec<String>,
    /// URLs in base not in head.
    pub urls_removed: Vec<String>,
    /// URLs whose content hash changed.
    pub urls_changed: Vec<String>,
    /// New error fingerprints.
    pub findings_added: Vec<String>,
    /// Error fingerprints that disappeared on still-measured URLs.
    pub findings_resolved: Vec<String>,
    /// Predicted routes added.
    pub routes_added: Vec<String>,
    /// Predicted routes removed.
    pub routes_removed: Vec<String>,
    /// True when the inputs could not be compared as SEO evidence.
    pub unmeasured: bool,
}

/// Diffs two compact snapshots.
#[must_use]
pub fn diff(base: &StoredSnapshot, head: &StoredSnapshot) -> SearchDiff {
    let comparable = base.site == head.site
        && (base.policy_version.is_empty()
            || head.policy_version.is_empty()
            || base.policy_version == head.policy_version);
    let base_pages: BTreeMap<&str, &crate::StoredPage> = base
        .pages
        .iter()
        .map(|page| (page.url.as_str(), page))
        .collect();
    let head_pages: BTreeMap<&str, &crate::StoredPage> = head
        .pages
        .iter()
        .map(|page| (page.url.as_str(), page))
        .collect();
    let mut urls_added = Vec::new();
    let mut urls_removed = Vec::new();
    let mut urls_changed = Vec::new();
    for url in head_pages.keys() {
        if !base_pages.contains_key(url) {
            urls_added.push((*url).to_owned());
        }
    }
    for (url, old) in &base_pages {
        match head_pages.get(url) {
            None => urls_removed.push((*url).to_owned()),
            Some(new) if new.content_hash != old.content_hash => {
                urls_changed.push((*url).to_owned());
            }
            Some(_) => {}
        }
    }
    let base_errors: BTreeSet<_> = base
        .findings
        .iter()
        .filter(|item| item.severity == Severity::Error)
        .map(|item| item.fingerprint.as_str())
        .collect();
    let head_errors: BTreeSet<_> = head
        .findings
        .iter()
        .filter(|item| item.severity == Severity::Error)
        .map(|item| item.fingerprint.as_str())
        .collect();
    let findings_added: Vec<String> = head_errors
        .difference(&base_errors)
        .map(|item| (*item).to_owned())
        .collect();
    let mut findings_resolved = Vec::new();
    for finding in base
        .findings
        .iter()
        .filter(|item| item.severity == Severity::Error)
    {
        if head_errors.contains(finding.fingerprint.as_str()) {
            continue;
        }
        if head_pages.contains_key(finding.url.as_str()) || finding.url.is_empty() {
            findings_resolved.push(finding.fingerprint.clone());
        }
    }
    let base_routes: BTreeSet<_> = base.predicted_routes.iter().collect();
    let head_routes: BTreeSet<_> = head.predicted_routes.iter().collect();
    SearchDiff {
        base: DiffRef {
            snapshot_id: base.snapshot_id.clone(),
            repo_revision: base.repo_revision.clone(),
            site: base.site.clone(),
        },
        head: DiffRef {
            snapshot_id: head.snapshot_id.clone(),
            repo_revision: head.repo_revision.clone(),
            site: head.site.clone(),
        },
        comparable,
        urls_added,
        urls_removed,
        urls_changed,
        findings_added,
        findings_resolved,
        routes_added: head_routes
            .difference(&base_routes)
            .map(|item| (*item).clone())
            .collect(),
        routes_removed: base_routes
            .difference(&head_routes)
            .map(|item| (*item).clone())
            .collect(),
        unmeasured: !comparable,
    }
}

/// Loads two snapshot files and diffs them.
///
/// # Errors
///
/// Returns IO or JSON errors.
pub fn diff_paths(base: &str, head: &str) -> Result<SearchDiff, String> {
    let base = crate::store::load(base)?;
    let head = crate::store::load(head)?;
    Ok(diff(&base, &head))
}

#[cfg(test)]
mod tests {
    use super::diff;
    use crate::{StoredFinding, StoredPage, StoredSnapshot};
    use weavatrix_seo_model::{AnalysisMode, ContentHash, Indexability, InventoryCounts, Severity};

    fn snap(site: &str, urls: &[&str], errors: &[&str]) -> StoredSnapshot {
        StoredSnapshot {
            schema: "weavatrix-seo-snapshot/v1".into(),
            snapshot_id: "s".into(),
            run_id: "r".into(),
            policy_version: "0.1.0".into(),
            config_digest: String::new(),
            mode: AnalysisMode::Site,
            site: Some(site.into()),
            repo: None,
            repo_revision: Some("abc".into()),
            predicted_routes: Vec::new(),
            pages: urls
                .iter()
                .map(|url| StoredPage {
                    url: (*url).into(),
                    status: 200,
                    indexability: Indexability::Indexable,
                    content_hash: ContentHash::of_str(url),
                    title: None,
                    h1: None,
                    in_sitemap: true,
                    linked_from_page: true,
                })
                .collect(),
            findings: errors
                .iter()
                .map(|fp| StoredFinding {
                    fingerprint: (*fp).into(),
                    code: "WVX-SEO-CONTENT-001".into(),
                    severity: Severity::Error,
                    summary: "x".into(),
                    url: urls.first().copied().unwrap_or("").into(),
                })
                .collect(),
            counts: InventoryCounts::default(),
        }
    }

    #[test]
    fn added_url_and_resolved_error() {
        let base = snap(
            "https://x.test/",
            &["https://x.test/", "https://x.test/old"],
            &["err-a"],
        );
        let mut head = snap(
            "https://x.test/",
            &["https://x.test/", "https://x.test/new"],
            &[],
        );
        head.snapshot_id = "h".into();
        let delta = diff(&base, &head);
        assert!(delta.comparable);
        assert!(delta.urls_added.iter().any(|url| url.ends_with("/new")));
        assert!(delta.urls_removed.iter().any(|url| url.ends_with("/old")));
        assert_eq!(delta.findings_resolved, vec!["err-a"]);
        assert!(!delta.unmeasured);
    }
}

//! Search-surface inventory.

use crate::{
    AbsoluteUrl, ExtractedPage, FetchObservation, GraphEdge, POLICY_VERSION, snapshot_digest,
};
use serde::{Deserialize, Serialize};
use std::fmt::Write as _;

/// How the run was invoked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisMode {
    /// Public or staging site only.
    Site,
    /// Repository only.
    Repo,
    /// Repository plus live/staging site.
    Hybrid,
    /// Owned site versus public competitor sites.
    Compare,
}

/// Compact inventory totals.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct InventoryCounts {
    /// URLs requested during the crawl.
    pub crawled: usize,
    /// Final success responses.
    pub fetched: usize,
    /// Redirected URLs.
    pub redirected: usize,
    /// 4xx/5xx.
    pub errors: usize,
    /// Sitemap loc entries discovered.
    pub sitemap_urls: usize,
    /// Pages classified indexable from response signals.
    pub indexable: usize,
    /// Fetch failures retained as observations.
    #[serde(default)]
    pub incomplete: usize,
}

/// Complete site/repo/hybrid inventory for one run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Inventory {
    /// Run mode.
    pub mode: AnalysisMode,
    /// Snapshot identity of this measured run.
    pub snapshot_id: String,
    /// Unique analysis-run identity.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub run_id: String,
    /// Policy identifier used for this run.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub policy_version: String,
    /// Crawl/config digest for CI comparability.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub config_digest: String,
    /// Git revision when a repository was in scope.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_revision: Option<String>,
    /// Seed site when present.
    pub site: Option<String>,
    /// Seed repository when present.
    pub repo: Option<String>,
    /// Hosts in scope.
    pub hosts: Vec<String>,
    /// Extracted pages keyed by final URL string.
    pub pages: Vec<ExtractedPage>,
    /// Graph edges.
    pub edges: Vec<GraphEdge>,
    /// Fetch attempts that did not yield a usable body.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observations: Vec<FetchObservation>,
    /// Route patterns predicted from source.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub predicted_routes: Vec<String>,
    /// Loc entries discovered from sitemaps, before page cap.
    #[serde(default)]
    pub sitemap_discovered: usize,
    /// Totals.
    pub counts: InventoryCounts,
}

impl Inventory {
    /// Empty inventory for a mode.
    #[must_use]
    pub fn blank(mode: AnalysisMode) -> Self {
        Self {
            mode,
            snapshot_id: String::new(),
            run_id: String::new(),
            policy_version: POLICY_VERSION.to_owned(),
            config_digest: String::new(),
            repo_revision: None,
            site: None,
            repo: None,
            hosts: Vec::new(),
            pages: Vec::new(),
            edges: Vec::new(),
            observations: Vec::new(),
            predicted_routes: Vec::new(),
            sitemap_discovered: 0,
            counts: InventoryCounts::default(),
        }
    }

    /// Page matching a URL string.
    #[must_use]
    pub fn page(&self, url: &AbsoluteUrl) -> Option<&ExtractedPage> {
        self.pages.iter().find(|page| page.url == *url)
    }

    /// URLs that were actually measured (pages + failed observations).
    #[must_use]
    pub fn measured_urls(&self) -> Vec<String> {
        let mut urls: Vec<String> = self.pages.iter().map(|page| page.url.to_string()).collect();
        urls.extend(self.observations.iter().map(|item| item.url.clone()));
        urls.sort();
        urls.dedup();
        urls
    }

    /// Binds run/snapshot/policy onto every page, edge, and observation.
    #[must_use]
    pub fn bind_run(mut self, run_id: &str, seed: &str) -> Self {
        let mut measured = String::new();
        for page in &self.pages {
            let _ = writeln!(
                measured,
                "{}:{}:{}",
                page.url, page.status, page.content_hash
            );
        }
        for item in &self.observations {
            let _ = writeln!(measured, "{}:{:?}", item.url, item.outcome);
        }
        run_id.clone_into(&mut self.run_id);
        self.snapshot_id = snapshot_digest(run_id, seed, &measured);
        POLICY_VERSION.clone_into(&mut self.policy_version);
        let snapshot = self.snapshot_id.clone();
        let revision = self.repo_revision.clone();
        for page in &mut self.pages {
            page.evidence.snapshot_id = Some(snapshot.clone());
            page.evidence.policy_version = Some(POLICY_VERSION.to_owned());
            if let Some(revision) = &revision {
                page.evidence.revision = Some(revision.clone());
            }
        }
        for edge in &mut self.edges {
            edge.evidence.snapshot_id = Some(snapshot.clone());
            edge.evidence.policy_version = Some(POLICY_VERSION.to_owned());
        }
        for observation in &mut self.observations {
            observation.evidence.snapshot_id = Some(snapshot.clone());
            observation.evidence.policy_version = Some(POLICY_VERSION.to_owned());
        }
        self
    }

    /// Rebuilds counts from pages and observations.
    #[must_use]
    pub fn with_counts(mut self) -> Self {
        self.counts = InventoryCounts {
            crawled: self.pages.len() + self.observations.len(),
            fetched: self
                .pages
                .iter()
                .filter(|page| (200..400).contains(&page.status))
                .count(),
            redirected: self
                .pages
                .iter()
                .filter(|page| page.indexability == crate::Indexability::Redirected)
                .count(),
            errors: self.pages.iter().filter(|page| page.status >= 400).count(),
            sitemap_urls: if self.sitemap_discovered == 0 {
                self.pages.iter().filter(|page| page.in_sitemap).count()
            } else {
                self.sitemap_discovered
            },
            indexable: self
                .pages
                .iter()
                .filter(|page| page.indexability == crate::Indexability::Indexable)
                .count(),
            incomplete: self.observations.len(),
        };
        self
    }
}

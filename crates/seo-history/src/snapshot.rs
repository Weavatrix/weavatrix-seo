//! Compact crawl snapshot. Page text and payloads are not stored.

use serde::{Deserialize, Serialize};
use weavatrix_seo_model::{
    AnalysisMode, AuditReport, ContentHash, ExtractedPage, Finding, Indexability, InventoryCounts,
    ProducerFact, Severity,
};

/// One measured URL without body text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredPage {
    /// Final URL.
    pub url: String,
    /// HTTP status.
    pub status: u16,
    /// Indexability.
    pub indexability: Indexability,
    /// Visible-text hash.
    pub content_hash: ContentHash,
    /// Title when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// First H1 when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub h1: Option<String>,
    /// Listed in a sitemap.
    #[serde(default)]
    pub in_sitemap: bool,
    /// Internally linked.
    #[serde(default)]
    pub linked_from_page: bool,
}

/// Compact finding row for history and diff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredFinding {
    /// Fingerprint.
    pub fingerprint: String,
    /// Catalogue code.
    pub code: String,
    /// Severity.
    pub severity: Severity,
    /// Summary.
    pub summary: String,
    /// Subject URL or path.
    pub url: String,
}

/// Revision-bound snapshot persisted to disk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredSnapshot {
    /// Artifact schema.
    pub schema: String,
    /// Crawl snapshot id.
    pub snapshot_id: String,
    /// Analysis run id.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub run_id: String,
    /// Policy version.
    #[serde(default)]
    pub policy_version: String,
    /// Config digest.
    #[serde(default)]
    pub config_digest: String,
    /// Mode.
    pub mode: AnalysisMode,
    /// Site seed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub site: Option<String>,
    /// Repo path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    /// Git revision when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_revision: Option<String>,
    /// Predicted route families.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub predicted_routes: Vec<String>,
    /// Source producers hashed for impact.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub producers: Vec<ProducerFact>,
    /// Measured pages.
    pub pages: Vec<StoredPage>,
    /// Error/warn findings.
    pub findings: Vec<StoredFinding>,
    /// Inventory totals.
    pub counts: InventoryCounts,
}

impl StoredSnapshot {
    /// Builds a compact snapshot from a full audit report.
    #[must_use]
    pub fn from_report(report: &AuditReport) -> Self {
        Self {
            schema: "weavatrix-seo-snapshot/v1".into(),
            snapshot_id: report.inventory.snapshot_id.clone(),
            run_id: report.inventory.run_id.clone(),
            policy_version: report.inventory.policy_version.clone(),
            config_digest: report.inventory.config_digest.clone(),
            mode: report.inventory.mode,
            site: report.inventory.site.clone(),
            repo: report.inventory.repo.clone(),
            repo_revision: report.inventory.repo_revision.clone(),
            predicted_routes: report.inventory.predicted_routes.clone(),
            producers: report.inventory.producers.clone(),
            pages: report.inventory.pages.iter().map(store_page).collect(),
            findings: report.findings.iter().map(store_finding).collect(),
            counts: report.inventory.counts.clone(),
        }
    }
}

fn store_page(page: &ExtractedPage) -> StoredPage {
    StoredPage {
        url: page.url.to_string(),
        status: page.status,
        indexability: page.indexability,
        content_hash: page.content_hash,
        title: page.title.clone(),
        h1: page
            .headings
            .iter()
            .find(|heading| heading.level == 1)
            .map(|heading| heading.text.clone()),
        in_sitemap: page.in_sitemap,
        linked_from_page: page.linked_from_page,
    }
}

fn store_finding(finding: &Finding) -> StoredFinding {
    StoredFinding {
        fingerprint: finding.fingerprint.clone(),
        code: finding.code.clone(),
        severity: finding.severity,
        summary: finding.summary.clone(),
        url: finding.locator.subject_url().to_owned(),
    }
}

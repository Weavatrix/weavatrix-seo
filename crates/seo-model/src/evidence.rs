//! Evidence attached to every fact and edge.

use serde::{Deserialize, Serialize};

/// How a fact was established.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EvidenceKind {
    /// Exact extraction or deterministic rule.
    Deterministic,
    /// Measured at runtime (HTTP, logs, GSC).
    Observed,
    /// Imported from an external provider.
    External,
    /// Model or similarity inference. Never upgraded to deterministic.
    Inferred,
    /// Required axis that was not measured.
    Unmeasured,
}

/// Where the bytes or symbols came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSource {
    /// Repository source.
    Repo,
    /// HTTP response.
    Http,
    /// Rendered DOM.
    RenderedDom,
    /// Sitemap document.
    Sitemap,
    /// Server or CDN logs.
    Logs,
    /// Search Console export.
    Gsc,
    /// Keyword/SERP/backlink provider.
    Provider,
    /// Semantic similarity layer.
    Semantic,
}

/// Confidence attached to evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    /// Exact parse or byte comparison.
    Exact,
    /// Strong but not bitwise.
    High,
    /// Useful but incomplete.
    Medium,
    /// Weak or inferred.
    Low,
}

/// Provenance for one recorded fact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evidence {
    /// How the fact was established.
    pub kind: EvidenceKind,
    /// Origin surface.
    pub source: EvidenceSource,
    /// Confidence of the extraction.
    pub confidence: Confidence,
    /// Snapshot identity when recorded from a crawl.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot_id: Option<String>,
    /// Git revision when recorded from a repository.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
    /// Policy identifier when a project contract applied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_version: Option<String>,
}

impl Evidence {
    /// Deterministic HTTP extraction.
    #[must_use]
    pub fn http() -> Self {
        Self {
            kind: EvidenceKind::Deterministic,
            source: EvidenceSource::Http,
            confidence: Confidence::Exact,
            snapshot_id: None,
            revision: None,
            policy_version: None,
        }
    }

    /// Sitemap document extraction.
    #[must_use]
    pub fn sitemap() -> Self {
        Self {
            kind: EvidenceKind::Deterministic,
            source: EvidenceSource::Sitemap,
            confidence: Confidence::Exact,
            snapshot_id: None,
            revision: None,
            policy_version: None,
        }
    }

    /// Explicitly unmeasured axis.
    #[must_use]
    pub fn unmeasured(source: EvidenceSource) -> Self {
        Self {
            kind: EvidenceKind::Unmeasured,
            source,
            confidence: Confidence::Low,
            snapshot_id: None,
            revision: None,
            policy_version: None,
        }
    }

    /// Observed HTTP fact (success or recorded failure).
    #[must_use]
    pub fn http_observed() -> Self {
        Self {
            kind: EvidenceKind::Observed,
            source: EvidenceSource::Http,
            confidence: Confidence::Exact,
            snapshot_id: None,
            revision: None,
            policy_version: None,
        }
    }

    /// Binds the evidence to a crawl snapshot.
    #[must_use]
    pub fn with_snapshot(mut self, snapshot_id: impl Into<String>) -> Self {
        self.snapshot_id = Some(snapshot_id.into());
        self
    }

    /// Binds a git revision / worktree digest.
    #[must_use]
    pub fn with_revision(mut self, revision: impl Into<String>) -> Self {
        self.revision = Some(revision.into());
        self
    }

    /// Binds the policy identifier.
    #[must_use]
    pub fn with_policy(mut self, policy_version: impl Into<String>) -> Self {
        self.policy_version = Some(policy_version.into());
        self
    }
}

/// Hybrid layer classification for one URL property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LayerState {
    /// Source, build, response, and render agree.
    Expected,
    /// Present only in source.
    SourceOnly,
    /// Present only in the build/route model.
    BuildOnly,
    /// Present only in the HTTP response.
    ResponseOnly,
    /// Present only after rendering.
    RenderOnly,
    /// Layers disagree.
    Contradicted,
    /// The layer was not measured.
    Unmeasured,
}

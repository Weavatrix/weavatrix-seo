//! Predicted search surface from a repository. Unmeasured until wired.

#![forbid(unsafe_code)]

use weavatrix_seo_model::{AnalysisMode, Evidence, EvidenceSource};

/// Predicted route family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteFamily {
    /// Pattern such as `/:locale/category/:city`.
    pub pattern: String,
    /// Owning source path when known.
    pub owner: Option<String>,
}

/// Repo-only inventory outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSurface {
    /// Analysis mode.
    pub mode: AnalysisMode,
    /// Predicted families.
    pub families: Vec<RouteFamily>,
    /// Evidence for the prediction.
    pub evidence: Evidence,
}

/// Repo analysis is not wired in 0.0.1.
#[must_use]
pub fn unmeasured(repo: &str) -> SourceSurface {
    let _ = repo;
    SourceSurface {
        mode: AnalysisMode::Repo,
        families: Vec::new(),
        evidence: Evidence::unmeasured(EvidenceSource::Repo),
    }
}

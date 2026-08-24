//! Predicted search surface from a repository.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use weavatrix_seo_model::{AnalysisMode, Evidence, EvidenceSource};

/// Predicted route family.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteFamily {
    /// Pattern such as `/:locale/category/:city`.
    pub pattern: String,
    /// Owning source path when known.
    pub owner: Option<String>,
    /// `generateMetadata` or `metadata` export is present.
    pub has_metadata: bool,
    /// `generateStaticParams` is present.
    pub has_static_params: bool,
}

/// Repo-only inventory outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSurface {
    /// Analysis mode.
    pub mode: AnalysisMode,
    /// Predicted families.
    pub families: Vec<RouteFamily>,
    /// `app/**/sitemap.ts` owners.
    pub sitemaps: Vec<String>,
    /// `app/**/robots.ts` owners.
    pub robots: Vec<String>,
    /// Middleware file when present.
    pub middleware: Option<String>,
    /// Evidence for the prediction.
    pub evidence: Evidence,
}

/// Repo analysis produced no surface.
#[must_use]
pub fn unmeasured(repo: &str) -> SourceSurface {
    let _ = repo;
    SourceSurface {
        mode: AnalysisMode::Repo,
        families: Vec::new(),
        sitemaps: Vec::new(),
        robots: Vec::new(),
        middleware: None,
        evidence: Evidence::unmeasured(EvidenceSource::Repo),
    }
}

impl SourceSurface {
    /// Patterns only.
    #[must_use]
    pub fn patterns(&self) -> Vec<String> {
        self.families
            .iter()
            .map(|family| family.pattern.clone())
            .collect()
    }
}

//! Predicted search surface from a repository.

#![forbid(unsafe_code)]

mod impact;

use serde::{Deserialize, Serialize};
use weavatrix_seo_model::{AnalysisMode, Evidence, EvidenceSource, Locator};

/// Source function, component, or export with an exact span.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSymbol {
    /// Repository-relative path.
    pub path: String,
    /// Declared name.
    pub name: String,
    /// Start line when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_line: Option<u32>,
    /// End line when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_line: Option<u32>,
}

impl SourceSymbol {
    /// Locator for this symbol.
    #[must_use]
    pub fn locator(&self) -> Locator {
        Locator::source_span(self.path.clone(), self.start_line, self.end_line)
    }
}

/// Redirect or rewrite from `next.config`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigHop {
    /// Incoming path pattern.
    pub source: String,
    /// Destination.
    pub destination: String,
    /// Redirect status when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
}

/// Extracted Next.js config facts.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NextConfig {
    /// Config file path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// `basePath`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_path: Option<String>,
    /// `trailingSlash`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trailing_slash: Option<bool>,
    /// Config redirects.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub redirects: Vec<ConfigHop>,
    /// Config rewrites.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rewrites: Vec<ConfigHop>,
}

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
    /// Default page export.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_symbol: Option<SourceSymbol>,
    /// Metadata producer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata_symbol: Option<SourceSymbol>,
    /// `generateStaticParams` producer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub static_params_symbol: Option<SourceSymbol>,
    /// JSON-LD producers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub json_ld_symbols: Vec<SourceSymbol>,
    /// Imported SEO helpers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub helpers: Vec<SourceSymbol>,
    /// Intercepting-route segment when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intercepting: Option<String>,
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
    /// Sitemap producer symbols.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sitemap_symbols: Vec<SourceSymbol>,
    /// `app/**/robots.ts` owners.
    pub robots: Vec<String>,
    /// Middleware file when present.
    pub middleware: Option<String>,
    /// Next config facts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_config: Option<NextConfig>,
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
        sitemap_symbols: Vec::new(),
        robots: Vec::new(),
        middleware: None,
        next_config: None,
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

//! Predicted search surface from a repository.

#![forbid(unsafe_code)]

mod impact;
mod policy;

use serde::{Deserialize, Serialize};
use weavatrix_seo_model::{AnalysisMode, Evidence, EvidenceKind, EvidenceSource, Locator};

pub use policy::{PolicyLoad, allows_family, load as load_policy};

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
    /// How completely this adapter measured source producers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<FrameworkCapabilities>,
}

/// Honesty about what a framework adapter can prove.
///
/// Values are `exact`, `high`, `partial`, or `unmeasured`. Source claims in
/// `seo_explain` / `seo_plan` must not outrun this.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameworkCapabilities {
    /// Route family prediction.
    pub route_prediction: String,
    /// Page/default export producer.
    pub page_producer: String,
    /// Metadata producer.
    pub metadata_producer: String,
    /// Schema/JSON-LD producer.
    pub schema_producer: String,
    /// Static generation / SSG evidence.
    #[serde(default)]
    pub static_generation: String,
    /// Helper import graph.
    pub helper_graph: String,
    /// Broader import graph (workspace, package exports).
    #[serde(default)]
    pub import_graph: String,
    /// Dataflow from domain facts to public copy.
    #[serde(default)]
    pub dataflow: String,
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
        capabilities: None,
    }
}

/// True when a route family is private chrome, not an indexable search URL.
#[must_use]
pub fn is_private_pattern(pattern: &str) -> bool {
    [
        "/admin",
        "/dashboard",
        "/auth",
        "/chats",
        "/settings",
        "/pro/",
        "/tasks/",
        "/profile",
    ]
    .iter()
    .any(|token| pattern.contains(token))
        || pattern.contains("/*")
}

impl SourceSurface {
    /// Merges another predicted surface. Later duplicates of a pattern are dropped.
    pub fn merge(&mut self, mut other: Self) {
        self.families.append(&mut other.families);
        self.sitemaps.append(&mut other.sitemaps);
        self.sitemap_symbols.append(&mut other.sitemap_symbols);
        self.robots.append(&mut other.robots);
        if self.middleware.is_none() {
            self.middleware = other.middleware;
        }
        if self.next_config.is_none() {
            self.next_config = other.next_config;
        }
        if self.capabilities.is_none() {
            self.capabilities = other.capabilities;
        }
        self.families
            .sort_by(|left, right| left.pattern.cmp(&right.pattern));
        self.families
            .dedup_by(|left, right| left.pattern == right.pattern);
        if self.evidence.kind == EvidenceKind::Unmeasured
            && other.evidence.kind != EvidenceKind::Unmeasured
        {
            self.evidence = other.evidence;
        }
    }

    /// Patterns only.
    #[must_use]
    pub fn patterns(&self) -> Vec<String> {
        self.families
            .iter()
            .map(|family| family.pattern.clone())
            .collect()
    }
}

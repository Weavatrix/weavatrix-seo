//! Next.js App Router adapter over weavatrix-scan and weavatrix-parse.

#![forbid(unsafe_code)]

mod route;

use std::fs;
use std::path::Path;
use weavatrix_parse::{Language, extract};
use weavatrix_scan::scan_repository;
use weavatrix_seo_model::{AnalysisMode, Evidence, EvidenceKind, EvidenceSource};
use weavatrix_seo_source::{RouteFamily, SourceSurface};

use crate::route::{app_prefix, pattern_from_page};

pub use route::matches as route_matches;

/// Predicts the Next.js search surface for `repo`.
#[must_use]
pub fn predict(repo: &str) -> SourceSurface {
    let Ok(report) = scan_repository(repo) else {
        return weavatrix_seo_source::unmeasured(repo);
    };
    let mut families = Vec::new();
    let mut sitemaps = Vec::new();
    let mut robots = Vec::new();
    let mut middleware = None;
    for file in &report.files {
        let relative = file.relative.replace('\\', "/");
        let name = relative.rsplit('/').next().unwrap_or(&relative);
        if name == "middleware.ts" || name == "middleware.js" {
            middleware = Some(relative.clone());
        }
        if name == "sitemap.ts" || name == "sitemap.js" {
            sitemaps.push(relative.clone());
        }
        if name == "robots.ts" || name == "robots.js" {
            robots.push(relative.clone());
        }
        let Some((_, rest)) = app_prefix(&relative) else {
            continue;
        };
        if !is_page(name) {
            continue;
        }
        let Some(pattern) = pattern_from_page(rest) else {
            continue;
        };
        let (has_metadata, has_static_params) = inspect_source(&file.absolute);
        families.push(RouteFamily {
            pattern,
            owner: Some(relative),
            has_metadata,
            has_static_params,
        });
    }
    families.sort_by(|left, right| left.pattern.cmp(&right.pattern));
    families.dedup_by(|left, right| left.pattern == right.pattern);
    SourceSurface {
        mode: AnalysisMode::Repo,
        families,
        sitemaps,
        robots,
        middleware,
        evidence: Evidence {
            kind: EvidenceKind::Deterministic,
            source: EvidenceSource::Repo,
            confidence: weavatrix_seo_model::Confidence::Exact,
            snapshot_id: None,
            revision: None,
            policy_version: None,
        },
    }
}

fn is_page(name: &str) -> bool {
    matches!(
        name,
        "page.tsx" | "page.ts" | "page.jsx" | "page.js" | "page.mdx"
    )
}

fn inspect_source(path: &Path) -> (bool, bool) {
    let Ok(source) = fs::read_to_string(path) else {
        return (false, false);
    };
    let facts = extract(&source, Language::TypeScript);
    let names: Vec<&str> = facts
        .declarations
        .iter()
        .map(|item| item.name.as_str())
        .collect();
    let has_metadata = names.contains(&"generateMetadata")
        || names.contains(&"metadata")
        || source.contains("generateMetadata")
        || source.contains("export const metadata");
    let has_static_params =
        names.contains(&"generateStaticParams") || source.contains("generateStaticParams");
    (has_metadata, has_static_params)
}

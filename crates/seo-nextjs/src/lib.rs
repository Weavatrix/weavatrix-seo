//! Next.js App Router adapter over weavatrix-scan and weavatrix-parse.

#![forbid(unsafe_code)]

mod config;
mod producers;
mod route;

use std::fs;
use weavatrix_scan::scan_repository;
use weavatrix_seo_model::{AnalysisMode, Evidence, EvidenceKind, EvidenceSource};
use weavatrix_seo_source::{RouteFamily, SourceSurface};

use crate::route::{app_prefix, intercepting_in, pattern_from_page};

pub use route::matches as route_matches;

/// Predicts the Next.js search surface for `repo`.
#[must_use]
pub fn predict(repo: &str) -> SourceSurface {
    let Ok(report) = scan_repository(repo) else {
        return weavatrix_seo_source::unmeasured(repo);
    };
    let mut families = Vec::new();
    let mut sitemaps = Vec::new();
    let mut sitemap_symbols = Vec::new();
    let mut robots = Vec::new();
    let mut middleware = None;
    let mut next_config = None;
    for file in &report.files {
        let relative = file.relative.replace('\\', "/");
        let name = relative.rsplit('/').next().unwrap_or(&relative);
        if is_next_config(name) {
            if let Ok(source) = fs::read_to_string(&file.absolute) {
                next_config = Some(config::parse(&relative, &source));
            }
            continue;
        }
        if name == "middleware.ts" || name == "middleware.js" {
            middleware = Some(relative.clone());
        }
        if name == "sitemap.ts" || name == "sitemap.js" {
            sitemaps.push(relative.clone());
            if let Ok(source) = fs::read_to_string(&file.absolute) {
                let producers = producers::inspect(&relative, &source);
                if let Some(page) = producers.page {
                    sitemap_symbols.push(page);
                }
            }
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
        let intercepting = intercepting_in(rest);
        let producers = fs::read_to_string(&file.absolute)
            .map(|source| producers::inspect(&relative, &source))
            .unwrap_or_default();
        families.push(RouteFamily {
            pattern,
            owner: Some(relative),
            has_metadata: producers.metadata.is_some(),
            has_static_params: producers.static_params.is_some(),
            page_symbol: producers.page,
            metadata_symbol: producers.metadata,
            static_params_symbol: producers.static_params,
            json_ld_symbols: producers.json_ld,
            helpers: producers.helpers,
            intercepting,
        });
    }
    families.sort_by(|left, right| left.pattern.cmp(&right.pattern));
    families.dedup_by(|left, right| left.pattern == right.pattern);
    SourceSurface {
        mode: AnalysisMode::Repo,
        families,
        sitemaps,
        sitemap_symbols,
        robots,
        middleware,
        next_config,
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

fn is_next_config(name: &str) -> bool {
    matches!(
        name,
        "next.config.js" | "next.config.ts" | "next.config.mjs" | "next.config.cjs"
    )
}

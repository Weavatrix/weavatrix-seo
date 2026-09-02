//! Framework search-surface adapters: Next.js App/Pages, Nuxt, Astro.

#![forbid(unsafe_code)]

mod astro;
mod config;
mod file_routes;
mod nuxt;
mod pages_router;
mod paths;
mod producers;
mod route;

use std::fs;
use std::path::Path;
use weavatrix_scan::scan_repository;
use weavatrix_seo_model::{AnalysisMode, Evidence, EvidenceKind, EvidenceSource};
use weavatrix_seo_source::{RouteFamily, SourceSurface};

use crate::route::{app_prefix, intercepting_in, pattern_from_page};

pub use route::matches as route_matches;

/// Predicts the search surface for `repo`.
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
    let aliases = paths::load(repo);
    ingest_scan(
        &report.files,
        &aliases,
        &mut families,
        &mut sitemaps,
        &mut sitemap_symbols,
        &mut robots,
        &mut middleware,
        &mut next_config,
    );
    for relative in extra_page_files(repo) {
        if families
            .iter()
            .any(|family| family.owner.as_deref() == Some(relative.as_str()))
        {
            continue;
        }
        if let Some(family) = pages_router::family(&relative, None)
            .or_else(|| nuxt::family(&relative))
            .or_else(|| astro::family(&relative))
        {
            families.push(family);
        }
    }
    families.sort_by(|left, right| left.pattern.cmp(&right.pattern));
    families.dedup_by(|left, right| left.pattern == right.pattern);
    let capabilities = surface_capabilities(&families);
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
        capabilities: Some(capabilities),
    }
}

fn surface_capabilities(families: &[RouteFamily]) -> weavatrix_seo_source::FrameworkCapabilities {
    let app = families.iter().any(|family| {
        family
            .owner
            .as_deref()
            .is_some_and(|path| path.contains("/app/"))
    });
    let ts = families.iter().any(|family| {
        family.owner.as_deref().is_some_and(|path| {
            let ext = std::path::Path::new(path)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("");
            ext.eq_ignore_ascii_case("tsx") || ext.eq_ignore_ascii_case("ts")
        })
    });
    let helpers = if families.iter().any(|family| !family.helpers.is_empty()) {
        "high"
    } else {
        "partial"
    };
    weavatrix_seo_source::FrameworkCapabilities {
        route_prediction: "exact".into(),
        page_producer: if app {
            "exact".into()
        } else if ts {
            "high".into()
        } else {
            "partial".into()
        },
        metadata_producer: if families.iter().any(|family| family.has_metadata) {
            "exact".into()
        } else {
            "unmeasured".into()
        },
        schema_producer: if families
            .iter()
            .any(|family| !family.json_ld_symbols.is_empty())
        {
            "high".into()
        } else {
            "unmeasured".into()
        },
        static_generation: if app { "high".into() } else { "partial".into() },
        helper_graph: helpers.into(),
        import_graph: helpers.into(),
        dataflow: "unmeasured".into(),
    }
}

#[allow(clippy::too_many_arguments)]
fn ingest_scan(
    files: &[weavatrix_scan::ScannedFile],
    aliases: &[(String, String)],
    families: &mut Vec<RouteFamily>,
    sitemaps: &mut Vec<String>,
    sitemap_symbols: &mut Vec<weavatrix_seo_source::SourceSymbol>,
    robots: &mut Vec<String>,
    middleware: &mut Option<String>,
    next_config: &mut Option<weavatrix_seo_source::NextConfig>,
) {
    for file in files {
        let relative = file.relative.replace('\\', "/");
        let name = relative.rsplit('/').next().unwrap_or(&relative);
        if is_next_config(name) {
            if let Ok(source) = fs::read_to_string(&file.absolute) {
                *next_config = Some(config::parse(&relative, &source));
            }
            continue;
        }
        if name == "middleware.ts" || name == "middleware.js" {
            *middleware = Some(relative.clone());
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
        let source = fs::read_to_string(&file.absolute).ok();
        if let Some((_, rest)) = app_prefix(&relative)
            && is_page(name)
            && let Some(pattern) = pattern_from_page(rest)
        {
            let intercepting = intercepting_in(rest);
            let producers = source
                .as_deref()
                .map(|body| producers::inspect_with_aliases(&relative, body, aliases))
                .unwrap_or_default();
            families.push(RouteFamily {
                pattern,
                owner: Some(relative.clone()),
                has_metadata: producers.metadata.is_some(),
                has_static_params: producers.static_params.is_some(),
                page_symbol: producers.page,
                metadata_symbol: producers.metadata,
                static_params_symbol: producers.static_params,
                json_ld_symbols: producers.json_ld,
                helpers: producers.helpers,
                intercepting,
            });
            continue;
        }
        if let Some(family) = pages_router::family(&relative, source.as_deref()) {
            families.push(family);
            continue;
        }
        if let Some(family) = nuxt::family(&relative) {
            families.push(family);
            continue;
        }
        if let Some(family) = astro::family(&relative) {
            families.push(family);
        }
    }
}

fn is_page(name: &str) -> bool {
    matches!(
        name,
        "page.tsx" | "page.ts" | "page.jsx" | "page.js" | "page.mdx"
    )
}

fn extra_page_files(repo: &str) -> Vec<String> {
    let mut files = Vec::new();
    for root in ["pages", "src/pages", "app/pages"] {
        collect_files(&Path::new(repo).join(root), repo, &mut files);
    }
    files.sort();
    files.dedup();
    files
}

fn collect_files(dir: &Path, repo: &str, files: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, repo, files);
            continue;
        }
        let Ok(relative) = path.strip_prefix(repo) else {
            continue;
        };
        files.push(relative.to_string_lossy().replace('\\', "/"));
    }
}

fn is_next_config(name: &str) -> bool {
    matches!(
        name,
        "next.config.js" | "next.config.ts" | "next.config.mjs" | "next.config.cjs"
    )
}

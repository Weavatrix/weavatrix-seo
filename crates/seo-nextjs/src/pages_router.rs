//! Next.js Pages Router (`pages/`, `src/pages/`).

use crate::file_routes::{pages_rest, pattern_from_file};
use crate::producers;
use weavatrix_seo_source::RouteFamily;

pub fn family(relative: &str, source: Option<&str>) -> Option<RouteFamily> {
    let rest = pages_rest(relative)?;
    if rest.starts_with("api/") {
        return None;
    }
    let ext = rest.rsplit('.').next().unwrap_or("");
    if !matches!(ext, "tsx" | "ts" | "jsx" | "js" | "mdx") {
        return None;
    }
    let pattern = pattern_from_file(rest)?;
    let producers = source.map_or_else(producers::Producers::default, |body| {
        producers::inspect(relative, body)
    });
    Some(RouteFamily {
        pattern,
        owner: Some(relative.to_owned()),
        has_metadata: producers.metadata.is_some(),
        has_static_params: producers.static_params.is_some(),
        page_symbol: producers.page,
        metadata_symbol: producers.metadata,
        static_params_symbol: producers.static_params,
        json_ld_symbols: producers.json_ld,
        helpers: producers.helpers,
        intercepting: None,
    })
}

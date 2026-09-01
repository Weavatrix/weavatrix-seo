//! Astro `src/pages/` routes.

use crate::file_routes::{pages_rest, pattern_from_file};
use weavatrix_seo_source::RouteFamily;

pub fn family(relative: &str) -> Option<RouteFamily> {
    let rest = pages_rest(relative)?;
    let ext = rest.rsplit('.').next().unwrap_or("");
    if !matches!(ext, "astro" | "md" | "mdx") {
        return None;
    }
    let pattern = pattern_from_file(rest)?;
    Some(RouteFamily {
        pattern,
        owner: Some(relative.to_owned()),
        has_metadata: false,
        has_static_params: false,
        page_symbol: None,
        metadata_symbol: None,
        static_params_symbol: None,
        json_ld_symbols: Vec::new(),
        helpers: Vec::new(),
        intercepting: None,
    })
}

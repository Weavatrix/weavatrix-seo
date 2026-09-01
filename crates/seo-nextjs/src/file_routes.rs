//! File-based route patterns shared by Pages Router, Nuxt, and Astro.

/// Converts a file path remainder such as `blog/[slug].vue` into `/blog/:slug`.
#[must_use]
pub fn pattern_from_file(rest: &str) -> Option<String> {
    let owned = rest.replace('\\', "/");
    let rest = owned.trim_start_matches('/');
    let without_ext = rest.rsplit_once('.').map_or(rest, |(head, _)| head);
    let mut parts = Vec::new();
    for segment in without_ext.split('/') {
        if segment.is_empty() {
            continue;
        }
        if segment.starts_with('_') || segment == "components" || segment == "layouts" {
            return None;
        }
        if segment == "index" {
            continue;
        }
        parts.push(normalize_segment(segment));
    }
    if parts.is_empty() {
        Some("/".to_owned())
    } else {
        Some(format!("/{}", parts.join("/")))
    }
}

fn normalize_segment(segment: &str) -> String {
    if let Some(inner) = segment.strip_prefix("[[...")
        && let Some(inner) = inner.strip_suffix("]]")
    {
        return format!("*{inner}");
    }
    if let Some(inner) = segment.strip_prefix("[...")
        && let Some(inner) = inner.strip_suffix(']')
    {
        return format!("*{inner}");
    }
    if let Some(inner) = segment.strip_prefix("[[")
        && let Some(inner) = inner.strip_suffix("]]")
    {
        return format!(":{inner}");
    }
    if let Some(inner) = segment.strip_prefix('[')
        && let Some(inner) = inner.strip_suffix(']')
    {
        return format!(":{inner}");
    }
    segment.to_owned()
}

/// Remainder after a `pages/` marker, if this is a file-based page.
#[must_use]
pub fn pages_rest(relative: &str) -> Option<&str> {
    for marker in ["/src/pages/", "/pages/", "/app/pages/"] {
        if let Some(index) = relative.find(marker) {
            return Some(&relative[index + marker.len()..]);
        }
    }
    for prefix in ["src/pages/", "pages/", "app/pages/"] {
        if let Some(rest) = relative.strip_prefix(prefix) {
            return Some(rest);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::pattern_from_file;

    #[test]
    fn vue_and_astro_files() {
        assert_eq!(pattern_from_file("index.vue").as_deref(), Some("/"));
        assert_eq!(
            pattern_from_file("blog/[slug].vue").as_deref(),
            Some("/blog/:slug")
        );
        assert_eq!(
            pattern_from_file("blog/[...slug].astro").as_deref(),
            Some("/blog/*slug")
        );
        assert_eq!(pattern_from_file("about.md").as_deref(), Some("/about"));
    }
}

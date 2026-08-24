//! App Router path → URL pattern.

/// Splits `apps/web/src/app/[locale]/page.tsx` into prefix and remainder.
#[must_use]
pub fn app_prefix(relative: &str) -> Option<(&str, &str)> {
    let markers = ["/src/app/", "/app/"];
    for marker in markers {
        if let Some(index) = relative.find(marker) {
            let start = index + marker.len();
            return Some((&relative[..start], &relative[start..]));
        }
    }
    None
}

/// Converts ` [locale]/category/[slug]/page.tsx ` into `/:locale/category/:slug`.
#[must_use]
pub fn pattern_from_page(rest: &str) -> Option<String> {
    let without_file = rest.rsplit_once('/')?.0;
    let mut parts = Vec::new();
    for segment in without_file.split('/') {
        if segment.is_empty() {
            continue;
        }
        if segment.starts_with('(') && segment.ends_with(')') {
            continue;
        }
        if segment.starts_with('@') {
            continue;
        }
        parts.push(normalize_segment(segment)?);
    }
    if parts.is_empty() {
        Some("/".to_owned())
    } else {
        Some(format!("/{}", parts.join("/")))
    }
}

fn normalize_segment(segment: &str) -> Option<String> {
    if let Some(inner) = segment.strip_prefix("[[...")
        && let Some(inner) = inner.strip_suffix("]]")
    {
        return Some(format!("*{inner}"));
    }
    if let Some(inner) = segment.strip_prefix("[...")
        && let Some(inner) = inner.strip_suffix(']')
    {
        return Some(format!("*{inner}"));
    }
    if let Some(inner) = segment.strip_prefix('[')
        && let Some(inner) = inner.strip_suffix(']')
    {
        return Some(format!(":{inner}"));
    }
    if segment.starts_with('_') {
        return None;
    }
    Some(segment.to_owned())
}

/// Whether `path` matches an App Router pattern.
#[must_use]
pub fn matches(pattern: &str, path: &str) -> bool {
    if matches_parts(pattern, path) {
        return true;
    }
    if let Some(rest) = pattern.strip_prefix("/:locale") {
        let rest = if rest.is_empty() { "/" } else { rest };
        return matches_parts(rest, path);
    }
    false
}

fn matches_parts(pattern: &str, path: &str) -> bool {
    let pattern = pattern.trim_end_matches('/');
    let path = path.trim_end_matches('/');
    let pattern = if pattern.is_empty() { "/" } else { pattern };
    let path = if path.is_empty() { "/" } else { path };
    let pattern_parts: Vec<&str> = pattern.split('/').filter(|part| !part.is_empty()).collect();
    let path_parts: Vec<&str> = path.split('/').filter(|part| !part.is_empty()).collect();
    if pattern_parts.len() != path_parts.len()
        && !pattern_parts
            .last()
            .is_some_and(|part| part.starts_with('*'))
    {
        return false;
    }
    let mut path_index = 0;
    for part in &pattern_parts {
        if part.starts_with('*') {
            return path_index <= path_parts.len();
        }
        if path_index >= path_parts.len() {
            return false;
        }
        if part.starts_with(':') || *part == path_parts[path_index] {
            path_index += 1;
            continue;
        }
        return false;
    }
    path_index == path_parts.len()
}

#[cfg(test)]
mod tests {
    use super::{app_prefix, matches, pattern_from_page};

    #[test]
    fn locale_category_city() {
        let relative = "apps/web/src/app/[locale]/category/[slug]/[city]/page.tsx";
        let rest = app_prefix(relative).unwrap().1;
        assert_eq!(
            pattern_from_page(rest).as_deref(),
            Some("/:locale/category/:slug/:city")
        );
        assert!(matches(
            "/:locale/category/:slug/:city",
            "/en/category/plumber/vancouver"
        ));
        assert!(matches("/:locale/about", "/about"));
        assert!(matches("/:locale", "/"));
    }
}

//! Deterministic extraction from `next.config.*` source. No JS eval.

use weavatrix_seo_source::{ConfigHop, NextConfig};

/// Parses string-literal config facts from a config file body.
#[must_use]
pub fn parse(path: &str, source: &str) -> NextConfig {
    NextConfig {
        path: Some(path.to_owned()),
        base_path: string_prop(source, "basePath"),
        trailing_slash: bool_prop(source, "trailingSlash"),
        redirects: hops(source, "redirects"),
        rewrites: hops(source, "rewrites"),
    }
}

fn string_prop(source: &str, name: &str) -> Option<String> {
    let needle = format!("{name}:");
    let rest = source.split(&needle).nth(1)?;
    let rest = rest.trim_start();
    let quote = rest.chars().next().filter(|ch| *ch == '\'' || *ch == '"')?;
    let inner = rest[1..].split(quote).next()?.to_owned();
    if inner.is_empty() { None } else { Some(inner) }
}

fn bool_prop(source: &str, name: &str) -> Option<bool> {
    let needle = format!("{name}:");
    let rest = source.split(&needle).nth(1)?.trim_start();
    if rest.starts_with("true") {
        Some(true)
    } else if rest.starts_with("false") {
        Some(false)
    } else {
        None
    }
}

fn hops(source: &str, section: &str) -> Vec<ConfigHop> {
    let Some(start) = source.find(section) else {
        return Vec::new();
    };
    let window = source
        .get(start..start.saturating_add(4000))
        .unwrap_or(&source[start..]);
    let mut hops = Vec::new();
    let mut rest = window;
    while let Some(at) = rest.find("source:") {
        rest = rest[at + 7..].trim_start();
        let Some(from) = quoted(rest) else {
            break;
        };
        let dest_at = rest.find("destination:").map(|index| index + 12);
        let Some(dest_at) = dest_at else {
            break;
        };
        let dest_rest = rest[dest_at..].trim_start();
        let Some(destination) = quoted(dest_rest) else {
            break;
        };
        let status = if window[..window.len().saturating_sub(rest.len())]
            .contains("permanent: true")
            || rest.contains("permanent: true")
        {
            Some(308)
        } else {
            None
        };
        hops.push(ConfigHop {
            source: from,
            destination,
            status,
        });
        rest = dest_rest;
        if hops.len() >= 32 {
            break;
        }
    }
    hops
}

fn quoted(rest: &str) -> Option<String> {
    let quote = rest.chars().next().filter(|ch| *ch == '\'' || *ch == '"')?;
    let inner = rest[1..].split(quote).next()?.to_owned();
    if inner.is_empty() { None } else { Some(inner) }
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn reads_base_path_slash_and_redirect() {
        let source = r"
            const nextConfig = {
              basePath: '/app',
              trailingSlash: true,
              async redirects() {
                return [{ source: '/old', destination: '/new', permanent: true }];
              },
            };
        ";
        let config = parse("next.config.ts", source);
        assert_eq!(config.base_path.as_deref(), Some("/app"));
        assert_eq!(config.trailing_slash, Some(true));
        assert_eq!(config.redirects[0].source, "/old");
        assert_eq!(config.redirects[0].destination, "/new");
        assert_eq!(config.redirects[0].status, Some(308));
    }
}

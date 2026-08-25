//! URL parse helpers. Kept separate so identity stays reviewable.

use crate::{Result, Scheme, SeoError};

pub(crate) fn split_scheme(raw: &str) -> Option<(Scheme, &str)> {
    let (scheme, rest) = raw.split_once("://")?;
    let scheme = match scheme.to_ascii_lowercase().as_str() {
        "http" => Scheme::Http,
        "https" => Scheme::Https,
        _ => return None,
    };
    Some((scheme, rest))
}

pub(crate) fn split_authority(rest: &str) -> (&str, &str) {
    match rest.find(['/', '?', '#']) {
        Some(index) => (&rest[..index], &rest[index..]),
        None => (rest, "/"),
    }
}

pub(crate) fn split_host_port(authority: &str, scheme: Scheme) -> Result<(String, Option<u16>)> {
    let authority = authority.trim().trim_matches('.');
    if authority.is_empty() {
        return Err(SeoError::InvalidUrl(authority.into()));
    }
    let (host, port) = if let Some(stripped) = authority.strip_prefix('[') {
        let (host, rest) = stripped
            .split_once(']')
            .ok_or_else(|| SeoError::InvalidUrl(authority.into()))?;
        let port = match rest.strip_prefix(':') {
            Some(port) => Some(parse_port(port)?),
            None if rest.is_empty() => None,
            None => return Err(SeoError::InvalidUrl(authority.into())),
        };
        (host.to_ascii_lowercase(), port)
    } else if let Some((host, port)) = authority.rsplit_once(':')
        && !host.is_empty()
        && port.chars().all(|ch| ch.is_ascii_digit())
    {
        (host.to_ascii_lowercase(), Some(parse_port(port)?))
    } else {
        (authority.to_ascii_lowercase(), None)
    };
    if host.is_empty() {
        return Err(SeoError::InvalidUrl(authority.into()));
    }
    let port = port.filter(|value| *value != scheme.default_port());
    Ok((host, port))
}

fn parse_port(value: &str) -> Result<u16> {
    value
        .parse()
        .map_err(|_| SeoError::InvalidUrl(format!(":{value}")))
}

pub(crate) fn split_path_query(path_and_query: &str) -> (String, Option<String>) {
    let without_fragment = path_and_query
        .split_once('#')
        .map_or(path_and_query, |part| part.0);
    match without_fragment.split_once('?') {
        Some((path, query)) => {
            let query = query.trim();
            (
                path.to_owned(),
                if query.is_empty() {
                    None
                } else {
                    Some(query.to_owned())
                },
            )
        }
        None => (without_fragment.to_owned(), None),
    }
}

/// Collapses `.` / `..` but keeps a trailing slash when the server URL had one.
pub(crate) fn normalize_path(path: &str) -> String {
    let trailing = path.ends_with('/') && path != "/" && !path.is_empty();
    let mut out = Vec::new();
    let source = if path.is_empty() { "/" } else { path };
    for segment in source.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    if out.is_empty() {
        "/".to_owned()
    } else if trailing {
        format!("/{}/", out.join("/"))
    } else {
        format!("/{}", out.join("/"))
    }
}

pub(crate) fn parent_path(path: &str) -> String {
    match path.rsplit_once('/') {
        Some(("", _)) | None => "/".to_owned(),
        Some((parent, _)) => format!("{parent}/"),
    }
}

pub(crate) fn host_for_origin(host: &str) -> String {
    if host.contains(':') {
        format!("[{host}]")
    } else {
        host.to_owned()
    }
}

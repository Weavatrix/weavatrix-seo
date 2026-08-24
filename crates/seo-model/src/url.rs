//! Absolute HTTP(S) URL identity used as the crawl key.

use crate::{Result, SeoError};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};

/// Scheme accepted by the crawler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scheme {
    /// `http://`
    Http,
    /// `https://`
    Https,
}

impl Scheme {
    /// Default TCP port for the scheme.
    #[must_use]
    pub const fn default_port(self) -> u16 {
        match self {
            Self::Http => 80,
            Self::Https => 443,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Https => "https",
        }
    }
}

/// Normalized absolute URL. Fragments are dropped. Default ports are omitted.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AbsoluteUrl {
    scheme: Scheme,
    host: String,
    port: Option<u16>,
    path: String,
    query: Option<String>,
}

impl AbsoluteUrl {
    /// Parses an absolute `http`/`https` URL.
    ///
    /// # Errors
    ///
    /// Returns [`SeoError::InvalidUrl`] when the locator is not an absolute
    /// HTTP(S) URL with a host.
    pub fn parse(raw: &str) -> Result<Self> {
        let trimmed = raw.trim();
        let (scheme, rest) =
            split_scheme(trimmed).ok_or_else(|| SeoError::InvalidUrl(raw.into()))?;
        let rest = rest.strip_prefix("//").unwrap_or(rest);
        let (authority, path_and_query) = split_authority(rest);
        if authority.is_empty() {
            return Err(SeoError::InvalidUrl(raw.into()));
        }
        if authority.contains('@') {
            return Err(SeoError::InvalidUrl(raw.into()));
        }
        let (host_port, _) = authority.split_once('/').unwrap_or((authority, ""));
        let (host, port) = split_host_port(host_port, scheme)?;
        let (path, query) = split_path_query(path_and_query);
        Ok(Self {
            scheme,
            host,
            port,
            path: normalize_path(&path),
            query,
        })
    }

    /// Resolves `href` against this URL.
    ///
    /// # Errors
    ///
    /// Returns [`SeoError::UnresolvableUrl`] when `href` is empty, a non-http
    /// scheme, or cannot be joined.
    pub fn join(&self, href: &str) -> Result<Self> {
        let href = href.trim();
        if href.is_empty() || href.starts_with('#') || href.starts_with("javascript:") {
            return Err(SeoError::UnresolvableUrl(href.into()));
        }
        if href.starts_with("mailto:") || href.starts_with("tel:") || href.starts_with("data:") {
            return Err(SeoError::UnresolvableUrl(href.into()));
        }
        if split_scheme(href).is_some() {
            return Self::parse(href).map_err(|_| SeoError::UnresolvableUrl(href.into()));
        }
        if let Some(rest) = href.strip_prefix("//") {
            return Self::parse(&format!("{}://{rest}", self.scheme.as_str()))
                .map_err(|_| SeoError::UnresolvableUrl(href.into()));
        }
        let joined = if href.starts_with('/') {
            format!("{}{href}", self.origin())
        } else {
            let base = parent_path(&self.path);
            format!("{}{base}{href}", self.origin())
        };
        let without_fragment = joined
            .split_once('#')
            .map_or(joined.as_str(), |part| part.0);
        Self::parse(without_fragment).map_err(|_| SeoError::UnresolvableUrl(href.into()))
    }

    /// `scheme://host[:port]`
    #[must_use]
    pub fn origin(&self) -> String {
        match self.port {
            Some(port) => format!("{}://{}:{port}", self.scheme.as_str(), self.host),
            None => format!("{}://{}", self.scheme.as_str(), self.host),
        }
    }

    /// Scheme.
    #[must_use]
    pub const fn scheme(&self) -> Scheme {
        self.scheme
    }

    /// Lowercased host.
    #[must_use]
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Explicit port when it is not the scheme default.
    #[must_use]
    pub const fn port(&self) -> Option<u16> {
        self.port
    }

    /// Effective TCP port.
    #[must_use]
    pub fn tcp_port(&self) -> u16 {
        self.port.unwrap_or_else(|| self.scheme.default_port())
    }

    /// Normalized path, always starting with `/`.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Raw query without `?`.
    #[must_use]
    pub fn query(&self) -> Option<&str> {
        self.query.as_deref()
    }

    /// Path used as the HTTP request-target (includes query).
    #[must_use]
    pub fn request_target(&self) -> String {
        match &self.query {
            Some(query) => format!("{}?{query}", self.path),
            None => self.path.clone(),
        }
    }

    /// Whether `other` is the same host (and port) as this URL.
    #[must_use]
    pub fn same_origin(&self, other: &Self) -> bool {
        self.scheme == other.scheme
            && self.host == other.host
            && self.tcp_port() == other.tcp_port()
    }
}

impl Display for AbsoluteUrl {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(formatter, "{}{}", self.origin(), self.request_target())
    }
}

fn split_scheme(raw: &str) -> Option<(Scheme, &str)> {
    let (scheme, rest) = raw.split_once("://")?;
    let scheme = match scheme.to_ascii_lowercase().as_str() {
        "http" => Scheme::Http,
        "https" => Scheme::Https,
        _ => return None,
    };
    Some((scheme, rest))
}

fn split_authority(rest: &str) -> (&str, &str) {
    match rest.find(['/', '?', '#']) {
        Some(index) => (&rest[..index], &rest[index..]),
        None => (rest, "/"),
    }
}

fn split_host_port(authority: &str, scheme: Scheme) -> Result<(String, Option<u16>)> {
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

fn split_path_query(path_and_query: &str) -> (String, Option<String>) {
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

fn normalize_path(path: &str) -> String {
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
    } else {
        format!("/{}", out.join("/"))
    }
}

fn parent_path(path: &str) -> String {
    match path.rsplit_once('/') {
        Some(("", _)) | None => "/".to_owned(),
        Some((parent, _)) => format!("{parent}/"),
    }
}

#[cfg(test)]
mod tests {
    use super::AbsoluteUrl;

    #[test]
    fn drops_fragment_and_default_port() {
        let url = AbsoluteUrl::parse("HTTPS://Example.COM:443/a/./b/../c#frag").unwrap();
        assert_eq!(url.to_string(), "https://example.com/a/c");
    }

    #[test]
    fn joins_relative_and_root_paths() {
        let base = AbsoluteUrl::parse("http://example.com/dir/page").unwrap();
        assert_eq!(
            base.join("other").unwrap().to_string(),
            "http://example.com/dir/other"
        );
        assert_eq!(
            base.join("/root").unwrap().to_string(),
            "http://example.com/root"
        );
    }

    #[test]
    fn rejects_credentials_and_non_http() {
        assert!(AbsoluteUrl::parse("http://user:pass@example.com/").is_err());
        assert!(AbsoluteUrl::parse("ftp://example.com/").is_err());
        let base = AbsoluteUrl::parse("http://example.com/").unwrap();
        assert!(base.join("mailto:a@b.c").is_err());
    }
}

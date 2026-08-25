//! Absolute HTTP(S) URL identity used as the crawl key.

use crate::url_parse::{
    host_for_origin, normalize_path, parent_path, split_authority, split_host_port, split_path_query,
    split_scheme,
};
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

    /// Wire name, `http` or `https`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Https => "https",
        }
    }
}

/// Absolute URL identity. Fragments are dropped. Default ports are omitted.
///
/// Trailing slashes are kept: `/foo` and `/foo/` are distinct until the server
/// itself redirects or canonicalizes them.
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
        let joined = if let Some(query) = href.strip_prefix('?') {
            format!("{}{}?{query}", self.origin(), self.path)
        } else if href.starts_with('/') {
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

    /// `scheme://host[:port]` with IPv6 hosts in brackets.
    #[must_use]
    pub fn origin(&self) -> String {
        let host = host_for_origin(&self.host);
        match self.port {
            Some(port) => format!("{}://{host}:{port}", self.scheme.as_str()),
            None => format!("{}://{host}", self.scheme.as_str()),
        }
    }

    /// Scheme.
    #[must_use]
    pub const fn scheme(&self) -> Scheme {
        self.scheme
    }

    /// Lowercased host without IPv6 brackets.
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

    /// Path, always starting with `/`. Trailing slash is significant.
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

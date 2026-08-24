//! Origin identity for DNS and keep-alive pooling.

use weavatrix_seo_model::{AbsoluteUrl, Scheme};

/// Scheme + host + port.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Origin {
    /// HTTP or HTTPS.
    pub scheme: Scheme,
    /// Lowercased host.
    pub host: String,
    /// Effective TCP port.
    pub port: u16,
}

impl Origin {
    /// Origin of `url`.
    #[must_use]
    pub fn of(url: &AbsoluteUrl) -> Self {
        Self {
            scheme: url.scheme(),
            host: url.host().to_owned(),
            port: url.tcp_port(),
        }
    }
}

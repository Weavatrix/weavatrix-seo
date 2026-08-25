//! Network safety policy. Default for MCP is public-only.

use crate::{HttpError, Result};
use std::net::IpAddr;
use weavatrix_seo_model::AbsoluteUrl;

/// Who may be contacted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetworkPolicy {
    /// When false, loopback/private/link-local/metadata are refused.
    pub allow_private: bool,
}

impl NetworkPolicy {
    /// MCP / competitor default: public unicast only.
    #[must_use]
    pub const fn public_only() -> Self {
        Self {
            allow_private: false,
        }
    }

    /// CLI / local fixture opt-in.
    #[must_use]
    pub const fn allow_private() -> Self {
        Self {
            allow_private: true,
        }
    }

    /// Checks a URL host before DNS. IP literals are checked immediately.
    ///
    /// # Errors
    ///
    /// Returns [`HttpError::Blocked`] when the destination is not permitted.
    pub fn check_url(self, url: &AbsoluteUrl) -> Result<()> {
        self.check_host(url.host())
    }

    /// Checks a hostname or IP literal.
    ///
    /// # Errors
    ///
    /// Returns [`HttpError::Blocked`] when the destination is not permitted.
    pub fn check_host(self, host: &str) -> Result<()> {
        if self.allow_private {
            return Ok(());
        }
        if blocked_hostname(host) {
            return Err(HttpError::Blocked(format!("blocked host {host}")));
        }
        if let Ok(ip) = host.parse::<IpAddr>() {
            self.check_ip(ip)?;
        }
        Ok(())
    }

    /// Checks one resolved address. Call for every A/AAAA and every redirect.
    ///
    /// # Errors
    ///
    /// Returns [`HttpError::Blocked`] for loopback, private, link-local, or
    /// cloud-metadata ranges.
    pub fn check_ip(self, ip: IpAddr) -> Result<()> {
        if self.allow_private || !is_restricted(ip) {
            Ok(())
        } else {
            Err(HttpError::Blocked(format!("blocked address {ip}")))
        }
    }
}

fn blocked_hostname(host: &str) -> bool {
    let host = host.trim_end_matches('.');
    matches!(
        host,
        "localhost"
            | "metadata.google.internal"
            | "metadata.internal"
            | "kubernetes.default"
            | "kubernetes.default.svc"
            | "instance-data"
    ) || tld_is(host, "localhost")
        || tld_is(host, "internal")
        || tld_is(host, "local")
}

fn tld_is(host: &str, label: &str) -> bool {
    host.rsplit('.')
        .next()
        .is_some_and(|part| part.eq_ignore_ascii_case(label))
        && host.contains('.')
}

fn is_restricted(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_unspecified()
                || v4.is_broadcast()
                || is_cgnat(v4.octets())
                || v4.octets() == [169, 254, 169, 254]
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6.is_unicast_link_local()
                || v6.is_unique_local()
                || v6.is_unspecified()
                || v6
                    .to_ipv4_mapped()
                    .is_some_and(|mapped| is_restricted(IpAddr::V4(mapped)))
        }
    }
}

fn is_cgnat(octets: [u8; 4]) -> bool {
    octets[0] == 100 && octets[1] >= 64 && octets[1] <= 127
}

#[cfg(test)]
mod tests {
    use super::NetworkPolicy;
    use std::net::{IpAddr, Ipv4Addr};
    use weavatrix_seo_model::AbsoluteUrl;

    #[test]
    fn public_only_blocks_loopback_and_metadata() {
        let policy = NetworkPolicy::public_only();
        let loopback = AbsoluteUrl::parse("http://127.0.0.1/").unwrap();
        assert!(policy.check_url(&loopback).is_err());
        let meta = AbsoluteUrl::parse("http://169.254.169.254/latest").unwrap();
        assert!(policy.check_url(&meta).is_err());
        let local = AbsoluteUrl::parse("http://localhost/").unwrap();
        assert!(policy.check_url(&local).is_err());
        assert!(
            policy
                .check_ip(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)))
                .is_err()
        );
        let public = AbsoluteUrl::parse("https://example.com/").unwrap();
        assert!(policy.check_url(&public).is_ok());
    }

    #[test]
    fn allow_private_permits_loopback() {
        let policy = NetworkPolicy::allow_private();
        let loopback = AbsoluteUrl::parse("http://127.0.0.1/").unwrap();
        assert!(policy.check_url(&loopback).is_ok());
    }
}

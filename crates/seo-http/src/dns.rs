//! Process-wide DNS cache. All A/AAAA records are retained.

use crate::{HttpError, Result};
use std::collections::BTreeMap;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Mutex;

const MAX_ADDRS: usize = 4;

/// Cached A/AAAA lookup.
#[derive(Debug)]
pub struct DnsCache {
    inner: Mutex<BTreeMap<(String, u16), Vec<SocketAddr>>>,
}

impl Default for DnsCache {
    fn default() -> Self {
        Self {
            inner: Mutex::new(BTreeMap::new()),
        }
    }
}

impl DnsCache {
    /// Resolves `host:port`, caching every address (capped).
    ///
    /// # Errors
    ///
    /// Returns [`HttpError::Dns`] when lookup fails or returns nothing.
    pub fn lookup(&self, host: &str, port: u16) -> Result<Vec<SocketAddr>> {
        let key = (host.to_owned(), port);
        if let Some(addrs) = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&key)
            .cloned()
        {
            return Ok(addrs);
        }
        let mut addrs: Vec<SocketAddr> = (host, port)
            .to_socket_addrs()
            .map_err(|error| HttpError::Dns(error.to_string()))?
            .collect();
        addrs.truncate(MAX_ADDRS);
        if addrs.is_empty() {
            return Err(HttpError::Dns(format!("no addresses for {host}")));
        }
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(key, addrs.clone());
        Ok(addrs)
    }
}

#[cfg(test)]
mod tests {
    use super::DnsCache;

    #[test]
    fn caches_localhost() {
        let cache = DnsCache::default();
        let first = cache.lookup("localhost", 80).expect("dns");
        let second = cache.lookup("localhost", 80).expect("dns");
        assert_eq!(first, second);
        assert!(!first.is_empty());
    }
}

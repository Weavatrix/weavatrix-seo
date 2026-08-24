//! Process-wide DNS cache. One lookup per origin per crawl.

use crate::{HttpError, Result};
use std::collections::BTreeMap;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Mutex;

/// Cached A/AAAA lookup.
#[derive(Debug)]
pub struct DnsCache {
    inner: Mutex<BTreeMap<(String, u16), SocketAddr>>,
}

impl Default for DnsCache {
    fn default() -> Self {
        Self {
            inner: Mutex::new(BTreeMap::new()),
        }
    }
}

impl DnsCache {
    /// Resolves `host:port`, caching the first address.
    ///
    /// # Errors
    ///
    /// Returns [`HttpError::Transport`] when DNS fails.
    pub fn lookup(&self, host: &str, port: u16) -> Result<SocketAddr> {
        let key = (host.to_owned(), port);
        if let Some(addr) = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&key)
            .copied()
        {
            return Ok(addr);
        }
        let addr = (host, port)
            .to_socket_addrs()
            .map_err(|error| HttpError::Transport(error.to_string()))?
            .next()
            .ok_or_else(|| HttpError::Transport(format!("no addresses for {host}")))?;
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(key, addr);
        Ok(addr)
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
    }
}

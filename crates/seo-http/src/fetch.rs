//! Redirect-following GET over the keep-alive pool.

use crate::connect::open;
use crate::decode::decode_body;
use crate::dns::DnsCache;
use crate::origin::Origin;
use crate::pool::{Conn, Pool};
use crate::request::write_get;
use crate::response::{ParsedResponse, read_response};
use crate::{FetchBudget, Result};
use std::sync::Arc;
use std::time::Instant;
use weavatrix_seo_model::{AbsoluteUrl, RedirectHop};

/// One fetched URL after following redirects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchResponse {
    /// Final URL.
    pub url: AbsoluteUrl,
    /// Original request URL.
    pub requested: AbsoluteUrl,
    /// Final status.
    pub status: u16,
    /// Redirect hops.
    pub redirects: Vec<RedirectHop>,
    /// Lowercased headers of the final response.
    pub headers: Vec<(String, String)>,
    /// Body as lossy UTF-8.
    pub body: String,
    /// Elapsed fetch time including redirects.
    pub fetch_ms: u32,
}

impl FetchResponse {
    /// Header value from the final response.
    #[must_use]
    pub fn header(&self, name: &str) -> Option<&str> {
        let name = name.to_ascii_lowercase();
        self.headers
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| value.as_str())
    }
}

/// HTTP fetcher with shared DNS cache and keep-alive pool.
#[derive(Clone)]
pub struct Fetcher {
    budget: FetchBudget,
    dns: Arc<DnsCache>,
    pool: Arc<Pool>,
}

impl Fetcher {
    /// Builds a fetcher. Clones share the same pool and DNS cache.
    #[must_use]
    pub fn new(budget: FetchBudget) -> Self {
        let pool_size = budget.pool_size.max(1);
        Self {
            budget,
            dns: Arc::new(DnsCache::default()),
            pool: Arc::new(Pool::new(pool_size)),
        }
    }

    /// GET with redirect following.
    ///
    /// # Errors
    ///
    /// Returns transport, TLS, or budget failure.
    pub fn get(&self, url: &AbsoluteUrl) -> Result<FetchResponse> {
        let started = Instant::now();
        let mut current = url.clone();
        let mut redirects = Vec::new();
        for _ in 0..=self.budget.max_redirects {
            let parsed = self.exchange(&current)?;
            if (300..400).contains(&parsed.status)
                && let Some(location) = parsed.header("location")
            {
                let next = current.join(location)?;
                if redirects
                    .iter()
                    .any(|hop: &RedirectHop| hop.to == next.to_string())
                {
                    return Err(crate::HttpError::Transport(format!(
                        "redirect loop at {current}"
                    )));
                }
                redirects.push(RedirectHop {
                    from: current.to_string(),
                    to: next.to_string(),
                    status: parsed.status,
                });
                current = next;
                continue;
            }
            let encoding = parsed.header("content-encoding").map(str::to_owned);
            let body = decode_body(
                encoding.as_deref(),
                parsed.body,
                self.budget.max_body_bytes,
            )?;
            return Ok(FetchResponse {
                url: current,
                requested: url.clone(),
                status: parsed.status,
                redirects,
                headers: parsed.headers,
                body: String::from_utf8_lossy(&body).into_owned(),
                fetch_ms: millis(started),
            });
        }
        Err(crate::HttpError::Transport("too many redirects".into()))
    }

    fn exchange(&self, url: &AbsoluteUrl) -> Result<ParsedResponse> {
        let origin = Origin::of(url);
        if let Some(mut conn) = self.pool.checkout(&origin)
            && let Ok(parsed) = self.roundtrip(&mut conn, url)
        {
            self.maybe_checkin(origin, conn, &parsed);
            return Ok(parsed);
        }
        let mut conn = open(&origin, &self.dns, self.budget.timeout)?;
        let parsed = self.roundtrip(&mut conn, url)?;
        self.maybe_checkin(origin, conn, &parsed);
        Ok(parsed)
    }

    fn roundtrip(&self, conn: &mut Conn, url: &AbsoluteUrl) -> Result<ParsedResponse> {
        write_get(conn, url, &self.budget.user_agent)?;
        read_response(conn, self.budget.max_body_bytes)
    }

    fn maybe_checkin(&self, origin: Origin, conn: Conn, parsed: &ParsedResponse) {
        let closed = parsed
            .header("connection")
            .is_some_and(|value| value.to_ascii_lowercase().contains("close"));
        if parsed.framed && !closed {
            self.pool.checkin(origin, conn);
        }
    }
}

fn millis(started: Instant) -> u32 {
    u32::try_from(started.elapsed().as_millis()).unwrap_or(u32::MAX)
}

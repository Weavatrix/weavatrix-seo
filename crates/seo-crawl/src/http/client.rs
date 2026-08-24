//! Blocking HTTP GET with bounded redirects.

use super::response::{ParsedResponse, read_response};
use crate::{CrawlBudget, CrawlError, Result};
use std::io::Write;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use weavatrix_seo_model::{AbsoluteUrl, RedirectHop, Scheme};

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

/// HTTP fetcher.
#[derive(Debug, Clone)]
pub struct Fetcher {
    budget: CrawlBudget,
}

impl Fetcher {
    /// Builds a fetcher from a crawl budget.
    #[must_use]
    pub const fn new(budget: CrawlBudget) -> Self {
        Self { budget }
    }

    /// GET with redirect following.
    ///
    /// # Errors
    ///
    /// Returns [`CrawlError`] on transport, TLS, or budget failure.
    pub fn get(&self, url: &AbsoluteUrl) -> Result<FetchResponse> {
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
                    return Err(CrawlError::Transport(format!("redirect loop at {current}")));
                }
                redirects.push(RedirectHop {
                    from: current.to_string(),
                    to: next.to_string(),
                    status: parsed.status,
                });
                current = next;
                continue;
            }
            return Ok(FetchResponse {
                url: current,
                requested: url.clone(),
                status: parsed.status,
                redirects,
                headers: parsed.headers,
                body: String::from_utf8_lossy(&parsed.body).into_owned(),
            });
        }
        Err(CrawlError::Transport("too many redirects".into()))
    }

    fn exchange(&self, url: &AbsoluteUrl) -> Result<ParsedResponse> {
        let addr = resolve(url)?;
        let stream = TcpStream::connect_timeout(&addr, self.budget.timeout)
            .map_err(|error| CrawlError::Transport(error.to_string()))?;
        stream
            .set_read_timeout(Some(self.budget.timeout))
            .map_err(|error| CrawlError::Transport(error.to_string()))?;
        stream
            .set_write_timeout(Some(self.budget.timeout))
            .map_err(|error| CrawlError::Transport(error.to_string()))?;
        match url.scheme() {
            Scheme::Http => self.write_and_read(url, stream),
            Scheme::Https => self.exchange_tls(url, stream),
        }
    }

    fn write_and_read(&self, url: &AbsoluteUrl, mut stream: TcpStream) -> Result<ParsedResponse> {
        write_request(&mut stream, url, &self.budget.user_agent)?;
        read_response(&mut stream, self.budget.max_body_bytes)
    }

    #[cfg(feature = "tls")]
    fn exchange_tls(&self, url: &AbsoluteUrl, stream: TcpStream) -> Result<ParsedResponse> {
        let mut stream = super::tls::wrap(url.host(), stream)?;
        write_request(&mut stream, url, &self.budget.user_agent)?;
        read_response(&mut stream, self.budget.max_body_bytes)
    }

    #[cfg(not(feature = "tls"))]
    fn exchange_tls(&self, _url: &AbsoluteUrl, _stream: TcpStream) -> Result<ParsedResponse> {
        Err(CrawlError::TlsDisabled)
    }
}

fn write_request(stream: &mut impl Write, url: &AbsoluteUrl, user_agent: &str) -> Result<()> {
    let host = match url.port() {
        Some(port) => format!("{}:{port}", url.host()),
        None => url.host().to_owned(),
    };
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {host}\r\nUser-Agent: {user_agent}\r\nAccept: */*\r\nAccept-Encoding: identity\r\nConnection: close\r\n\r\n",
        url.request_target()
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|error| CrawlError::Transport(error.to_string()))
}

fn resolve(url: &AbsoluteUrl) -> Result<SocketAddr> {
    let mut addrs = (url.host(), url.tcp_port())
        .to_socket_addrs()
        .map_err(|error| CrawlError::Transport(error.to_string()))?;
    addrs
        .next()
        .ok_or_else(|| CrawlError::Transport(format!("no addresses for {}", url.host())))
}

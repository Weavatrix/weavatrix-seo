//! Fresh TCP/TLS connection.

use crate::dns::DnsCache;
use crate::origin::Origin;
use crate::pool::Conn;
use crate::{HttpError, Result};
use std::net::TcpStream;
use std::time::Duration;
use weavatrix_seo_model::Scheme;

/// Opens a new socket to `origin`.
///
/// # Errors
///
/// Returns transport or TLS errors.
pub fn open(origin: &Origin, dns: &DnsCache, timeout: Duration) -> Result<Conn> {
    let addr = dns.lookup(&origin.host, origin.port)?;
    let stream = TcpStream::connect_timeout(&addr, timeout)
        .map_err(|error| HttpError::Transport(error.to_string()))?;
    stream
        .set_nodelay(true)
        .map_err(|error| HttpError::Transport(error.to_string()))?;
    stream
        .set_read_timeout(Some(timeout))
        .map_err(|error| HttpError::Transport(error.to_string()))?;
    stream
        .set_write_timeout(Some(timeout))
        .map_err(|error| HttpError::Transport(error.to_string()))?;
    match origin.scheme {
        Scheme::Http => Ok(Conn::Plain(stream)),
        Scheme::Https => wrap_tls(&origin.host, stream),
    }
}

#[cfg(feature = "tls")]
fn wrap_tls(host: &str, stream: TcpStream) -> Result<Conn> {
    Ok(Conn::Tls(Box::new(crate::tls::wrap(host, stream)?)))
}

#[cfg(not(feature = "tls"))]
fn wrap_tls(_host: &str, _stream: TcpStream) -> Result<Conn> {
    Err(HttpError::TlsDisabled)
}

//! Fresh TCP/TLS connection with address fallback.

use crate::dns::DnsCache;
use crate::origin::Origin;
use crate::pool::Conn;
use crate::{HttpError, NetworkPolicy, Result};
use std::io::ErrorKind;
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;
use weavatrix_seo_model::Scheme;

/// Opens a new socket to `origin`, trying every resolved address.
///
/// # Errors
///
/// Returns DNS, timeout, TLS, blocked, or transport errors.
pub fn open(
    origin: &Origin,
    dns: &DnsCache,
    timeout: Duration,
    policy: NetworkPolicy,
) -> Result<Conn> {
    policy.check_host(&origin.host)?;
    let addrs = dns.lookup(&origin.host, origin.port)?;
    let mut last = HttpError::Transport(format!("no usable address for {}", origin.host));
    for addr in addrs {
        if let Err(error) = policy.check_ip(addr.ip()) {
            last = error;
            continue;
        }
        match connect_one(origin, addr, timeout) {
            Ok(conn) => return Ok(conn),
            Err(error) => last = error,
        }
    }
    Err(last)
}

fn connect_one(origin: &Origin, addr: SocketAddr, timeout: Duration) -> Result<Conn> {
    let stream = TcpStream::connect_timeout(&addr, timeout).map_err(|error| map_io(&error))?;
    stream.set_nodelay(true).map_err(|error| map_io(&error))?;
    stream
        .set_read_timeout(Some(timeout))
        .map_err(|error| map_io(&error))?;
    stream
        .set_write_timeout(Some(timeout))
        .map_err(|error| map_io(&error))?;
    match origin.scheme {
        Scheme::Http => Ok(Conn::Plain(stream)),
        Scheme::Https => wrap_tls(&origin.host, stream),
    }
}

pub(crate) fn map_io(error: &std::io::Error) -> HttpError {
    match error.kind() {
        ErrorKind::TimedOut | ErrorKind::WouldBlock => HttpError::Timeout(error.to_string()),
        _ if error.to_string().to_ascii_lowercase().contains("timed out") => {
            HttpError::Timeout(error.to_string())
        }
        _ => HttpError::Transport(error.to_string()),
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

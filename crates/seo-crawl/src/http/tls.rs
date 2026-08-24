//! rustls client stream for HTTPS fetches.

use crate::{CrawlError, Result};
use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};
use std::net::TcpStream;
use std::sync::{Arc, OnceLock};

static CONFIG: OnceLock<Arc<ClientConfig>> = OnceLock::new();

fn client_config() -> Arc<ClientConfig> {
    CONFIG
        .get_or_init(|| {
            let mut roots = RootCertStore::empty();
            roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
            Arc::new(
                ClientConfig::builder()
                    .with_root_certificates(roots)
                    .with_no_client_auth(),
            )
        })
        .clone()
}

/// Wraps a connected TCP stream in TLS.
///
/// # Errors
///
/// Returns [`CrawlError::Transport`] when the handshake fails.
pub fn wrap(host: &str, stream: TcpStream) -> Result<StreamOwned<ClientConnection, TcpStream>> {
    let server_name = host
        .to_owned()
        .try_into()
        .map_err(|_| CrawlError::Transport(format!("invalid TLS host: {host}")))?;
    let connection = ClientConnection::new(client_config(), server_name)
        .map_err(|error| CrawlError::Transport(error.to_string()))?;
    Ok(StreamOwned::new(connection, stream))
}

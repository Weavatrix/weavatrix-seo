//! HTTP/1.1 request writer.

use crate::{HttpError, Result};
use std::io::Write;
use weavatrix_seo_model::AbsoluteUrl;

/// Writes a keep-alive GET that accepts gzip/deflate.
///
/// # Errors
///
/// Returns [`HttpError::Transport`] on write failure.
pub fn write_get(stream: &mut impl Write, url: &AbsoluteUrl, user_agent: &str) -> Result<()> {
    let host = match url.port() {
        Some(port) => format!("{}:{port}", url.host()),
        None => url.host().to_owned(),
    };
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {host}\r\nUser-Agent: {user_agent}\r\nAccept: */*\r\nAccept-Encoding: gzip, deflate\r\nConnection: keep-alive\r\n\r\n",
        url.request_target()
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|error| HttpError::Transport(error.to_string()))
}

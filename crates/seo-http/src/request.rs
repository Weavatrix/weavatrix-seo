//! HTTP/1.1 request writer.

use crate::Result;
use std::io::Write;
use weavatrix_seo_model::AbsoluteUrl;

/// Writes a keep-alive GET that accepts gzip/deflate.
///
/// # Errors
///
/// Returns [`HttpError::Transport`] on write failure.
pub fn write_get(stream: &mut impl Write, url: &AbsoluteUrl, user_agent: &str) -> Result<()> {
    let host = host_header(url);
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {host}\r\nUser-Agent: {user_agent}\r\nAccept: */*\r\nAccept-Encoding: gzip, deflate\r\nConnection: keep-alive\r\n\r\n",
        url.request_target()
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|error| crate::connect::map_io(&error))
}

fn host_header(url: &AbsoluteUrl) -> String {
    let host = if url.host().contains(':') {
        format!("[{}]", url.host())
    } else {
        url.host().to_owned()
    };
    match url.port() {
        Some(port) => format!("{host}:{port}"),
        None => host,
    }
}

//! HTTP/1.1 response decoder. Identity and chunked bodies.

use crate::connect::map_io;
use crate::{HttpError, Result};
use std::io::Read;

/// Decoded response before redirect handling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedResponse {
    /// Status code.
    pub status: u16,
    /// Lowercased header names.
    pub headers: Vec<(String, String)>,
    /// Body bytes, UTF-8 lossy later.
    pub body: Vec<u8>,
    /// True when the body was framed (keep-alive safe).
    pub framed: bool,
}

impl ParsedResponse {
    /// First header value, case-insensitive name.
    #[must_use]
    pub fn header(&self, name: &str) -> Option<&str> {
        let name = name.to_ascii_lowercase();
        self.headers
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| value.as_str())
    }
}

/// Reads one HTTP/1 response from `reader`.
///
/// # Errors
///
/// Returns [`HttpError`] when the status line, headers, or body are invalid.
pub fn read_response(reader: &mut impl Read, max_body: usize) -> Result<ParsedResponse> {
    let head = read_until(reader, b"\r\n\r\n", 64 * 1024)?;
    let text = std::str::from_utf8(&head)
        .map_err(|_| HttpError::Transport("response headers are not UTF-8".into()))?;
    let mut lines = text.split("\r\n");
    let status_line = lines
        .next()
        .ok_or_else(|| HttpError::Transport("empty response".into()))?;
    let status = parse_status(status_line)?;
    let mut headers = Vec::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let (name, value) = line
            .split_once(':')
            .ok_or_else(|| HttpError::Transport(format!("invalid header: {line}")))?;
        headers.push((name.trim().to_ascii_lowercase(), value.trim().to_owned()));
    }
    let parsed = ParsedResponse {
        status,
        headers,
        body: Vec::new(),
        framed: false,
    };
    let (body, framed) = read_body(reader, &parsed, max_body)?;
    Ok(ParsedResponse {
        body,
        framed,
        ..parsed
    })
}

fn parse_status(line: &str) -> Result<u16> {
    let mut parts = line.split_whitespace();
    let version = parts
        .next()
        .ok_or_else(|| HttpError::Transport("missing HTTP version".into()))?;
    if !version.starts_with("HTTP/") {
        return Err(HttpError::Transport(format!("invalid status line: {line}")));
    }
    let status = parts
        .next()
        .ok_or_else(|| HttpError::Transport("missing status code".into()))?;
    status
        .parse()
        .map_err(|_| HttpError::Transport(format!("invalid status code: {status}")))
}

fn read_body(
    reader: &mut impl Read,
    parsed: &ParsedResponse,
    max_body: usize,
) -> Result<(Vec<u8>, bool)> {
    if parsed
        .header("transfer-encoding")
        .is_some_and(|value| value.to_ascii_lowercase().contains("chunked"))
    {
        return Ok((read_chunked(reader, max_body)?, true));
    }
    if let Some(length) = parsed.header("content-length") {
        let length: usize = length
            .parse()
            .map_err(|_| HttpError::Transport("invalid content-length".into()))?;
        if length > max_body {
            return Err(HttpError::Budget(format!(
                "content-length {length} exceeds {max_body}"
            )));
        }
        let mut body = vec![0_u8; length];
        reader
            .read_exact(&mut body)
            .map_err(|error| map_io(&error))?;
        return Ok((body, true));
    }
    let mut body = Vec::new();
    reader
        .take(u64::try_from(max_body.saturating_add(1)).unwrap_or(u64::MAX))
        .read_to_end(&mut body)
        .map_err(|error| map_io(&error))?;
    if body.len() > max_body {
        return Err(HttpError::Budget("response body exceeds budget".into()));
    }
    Ok((body, false))
}

fn read_chunked(reader: &mut impl Read, max_body: usize) -> Result<Vec<u8>> {
    let mut body = Vec::new();
    loop {
        let line = read_until(reader, b"\r\n", 64)?;
        let line = std::str::from_utf8(&line)
            .map_err(|_| HttpError::Transport("chunk size is not UTF-8".into()))?
            .trim();
        let size = usize::from_str_radix(line.split(';').next().unwrap_or(line).trim(), 16)
            .map_err(|_| HttpError::Transport(format!("invalid chunk size: {line}")))?;
        if size == 0 {
            let _ = read_until(reader, b"\r\n", 64);
            break;
        }
        if body.len().saturating_add(size) > max_body {
            return Err(HttpError::Budget("chunked body exceeds budget".into()));
        }
        let mut chunk = vec![0_u8; size];
        reader
            .read_exact(&mut chunk)
            .map_err(|error| map_io(&error))?;
        body.extend_from_slice(&chunk);
        let mut crlf = [0_u8; 2];
        reader
            .read_exact(&mut crlf)
            .map_err(|error| map_io(&error))?;
    }
    Ok(body)
}

fn read_until(reader: &mut impl Read, needle: &[u8], max: usize) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    let mut byte = [0_u8; 1];
    while buffer.len() < max {
        let read = reader.read(&mut byte).map_err(|error| map_io(&error))?;
        if read == 0 {
            return Err(HttpError::Transport("unexpected end of response".into()));
        }
        buffer.push(byte[0]);
        if buffer.ends_with(needle) {
            buffer.truncate(buffer.len() - needle.len());
            return Ok(buffer);
        }
    }
    Err(HttpError::Budget("response head exceeds budget".into()))
}

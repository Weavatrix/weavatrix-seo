//! HTTP/1.1 response decoder. Identity and chunked bodies only.

use crate::{CrawlError, Result};
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
/// Returns [`CrawlError`] when the status line, headers, or body are invalid
/// or exceed `max_body`.
pub fn read_response(reader: &mut impl Read, max_body: usize) -> Result<ParsedResponse> {
    let head = read_until(reader, b"\r\n\r\n", 64 * 1024)?;
    let text = std::str::from_utf8(&head)
        .map_err(|_| CrawlError::Transport("response headers are not UTF-8".into()))?;
    let mut lines = text.split("\r\n");
    let status_line = lines
        .next()
        .ok_or_else(|| CrawlError::Transport("empty response".into()))?;
    let status = parse_status(status_line)?;
    let mut headers = Vec::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let (name, value) = line
            .split_once(':')
            .ok_or_else(|| CrawlError::Transport(format!("invalid header: {line}")))?;
        headers.push((name.trim().to_ascii_lowercase(), value.trim().to_owned()));
    }
    let parsed = ParsedResponse {
        status,
        headers,
        body: Vec::new(),
    };
    let body = read_body(reader, &parsed, max_body)?;
    Ok(ParsedResponse { body, ..parsed })
}

fn parse_status(line: &str) -> Result<u16> {
    let mut parts = line.split_whitespace();
    let version = parts
        .next()
        .ok_or_else(|| CrawlError::Transport("missing HTTP version".into()))?;
    if !version.starts_with("HTTP/") {
        return Err(CrawlError::Transport(format!(
            "invalid status line: {line}"
        )));
    }
    let status = parts
        .next()
        .ok_or_else(|| CrawlError::Transport("missing status code".into()))?;
    status
        .parse()
        .map_err(|_| CrawlError::Transport(format!("invalid status code: {status}")))
}

fn read_body(reader: &mut impl Read, parsed: &ParsedResponse, max_body: usize) -> Result<Vec<u8>> {
    if parsed
        .header("transfer-encoding")
        .is_some_and(|value| value.to_ascii_lowercase().contains("chunked"))
    {
        return read_chunked(reader, max_body);
    }
    if let Some(length) = parsed.header("content-length") {
        let length: usize = length
            .parse()
            .map_err(|_| CrawlError::Transport("invalid content-length".into()))?;
        if length > max_body {
            return Err(CrawlError::Budget(format!(
                "content-length {length} exceeds {max_body}"
            )));
        }
        let mut body = vec![0_u8; length];
        reader
            .read_exact(&mut body)
            .map_err(|error| CrawlError::Transport(error.to_string()))?;
        return Ok(body);
    }
    let mut body = Vec::new();
    reader
        .take(u64::try_from(max_body.saturating_add(1)).unwrap_or(u64::MAX))
        .read_to_end(&mut body)
        .map_err(|error| CrawlError::Transport(error.to_string()))?;
    if body.len() > max_body {
        return Err(CrawlError::Budget("response body exceeds budget".into()));
    }
    Ok(body)
}

fn read_chunked(reader: &mut impl Read, max_body: usize) -> Result<Vec<u8>> {
    let mut body = Vec::new();
    loop {
        let line = read_until(reader, b"\r\n", 64)?;
        let line = std::str::from_utf8(&line)
            .map_err(|_| CrawlError::Transport("chunk size is not UTF-8".into()))?
            .trim();
        let size = usize::from_str_radix(line.split(';').next().unwrap_or(line).trim(), 16)
            .map_err(|_| CrawlError::Transport(format!("invalid chunk size: {line}")))?;
        if size == 0 {
            let _ = read_until(reader, b"\r\n", 64);
            break;
        }
        if body.len().saturating_add(size) > max_body {
            return Err(CrawlError::Budget("chunked body exceeds budget".into()));
        }
        let mut chunk = vec![0_u8; size];
        reader
            .read_exact(&mut chunk)
            .map_err(|error| CrawlError::Transport(error.to_string()))?;
        body.extend_from_slice(&chunk);
        let mut crlf = [0_u8; 2];
        reader
            .read_exact(&mut crlf)
            .map_err(|error| CrawlError::Transport(error.to_string()))?;
    }
    Ok(body)
}

fn read_until(reader: &mut impl Read, needle: &[u8], max: usize) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    let mut byte = [0_u8; 1];
    while buffer.len() < max {
        let read = reader
            .read(&mut byte)
            .map_err(|error| CrawlError::Transport(error.to_string()))?;
        if read == 0 {
            return Err(CrawlError::Transport("unexpected end of response".into()));
        }
        buffer.push(byte[0]);
        if buffer.ends_with(needle) {
            buffer.truncate(buffer.len() - needle.len());
            return Ok(buffer);
        }
    }
    Err(CrawlError::Budget("response head exceeds budget".into()))
}

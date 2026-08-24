//! Content-Encoding decoder. Gzip and deflate only.

use crate::{HttpError, Result};
use flate2::read::{DeflateDecoder, GzDecoder};
use std::io::Read;

/// Decodes `body` according to `Content-Encoding`.
///
/// # Errors
///
/// Returns [`HttpError::Transport`] when the payload is not valid gzip/deflate.
pub fn decode_body(encoding: Option<&str>, body: Vec<u8>, max_body: usize) -> Result<Vec<u8>> {
    let Some(encoding) = encoding else {
        return Ok(body);
    };
    let encoding = encoding.to_ascii_lowercase();
    if encoding.contains("gzip") {
        return inflate(GzDecoder::new(body.as_slice()), max_body);
    }
    if encoding.contains("deflate") {
        return inflate(DeflateDecoder::new(body.as_slice()), max_body);
    }
    Ok(body)
}

fn inflate(decoder: impl Read, max_body: usize) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    decoder
        .take(u64::try_from(max_body.saturating_add(1)).unwrap_or(u64::MAX))
        .read_to_end(&mut out)
        .map_err(|error| HttpError::Transport(error.to_string()))?;
    if out.len() > max_body {
        return Err(HttpError::Budget("decoded body exceeds budget".into()));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::decode_body;
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use std::io::Write;

    #[test]
    fn roundtrips_gzip() {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(b"hello-seo").unwrap();
        let gz = encoder.finish().unwrap();
        let out = decode_body(Some("gzip"), gz, 1024).unwrap();
        assert_eq!(out, b"hello-seo");
    }
}

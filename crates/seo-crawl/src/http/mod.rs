//! HTTP/1.1 client used by the crawler.

mod client;
mod response;
#[cfg(feature = "tls")]
mod tls;

pub use client::{FetchResponse, Fetcher};

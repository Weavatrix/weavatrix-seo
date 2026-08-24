//! First-party HTTP/1.1 transport: DNS cache, keep-alive pool, gzip.

#![forbid(unsafe_code)]

mod budget;
mod connect;
mod decode;
mod dns;
mod error;
mod fetch;
mod origin;
mod pool;
mod request;
mod response;
#[cfg(feature = "tls")]
mod tls;

pub use budget::FetchBudget;
pub use error::{HttpError, Result};
pub use fetch::{FetchResponse, Fetcher};

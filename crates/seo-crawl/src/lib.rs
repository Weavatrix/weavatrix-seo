//! First-party bounded crawler: HTTP, robots, sitemap, HTML extraction.
//!
//! Rendering belongs in `weavatrix-seo-render`. This crate records the raw
//! HTTP response only.

#![forbid(unsafe_code)]

mod budget;
mod engine;
mod error;
mod extract;
mod http;
mod robots;
mod sitemap;

pub use budget::CrawlBudget;
pub use engine::{Crawl, CrawlConfig};
pub use error::{CrawlError, Result};
pub use extract::extract_html;
pub use http::{FetchResponse, Fetcher};
pub use robots::Robots;
pub use sitemap::parse_sitemap;

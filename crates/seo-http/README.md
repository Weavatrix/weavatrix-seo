# weavatrix-seo-http

[![crates.io](https://img.shields.io/crates/v/weavatrix-seo-http.svg)](https://crates.io/crates/weavatrix-seo-http)
[![docs.rs](https://docs.rs/weavatrix-seo-http/badge.svg)](https://docs.rs/weavatrix-seo-http)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/Weavatrix/weavatrix-seo/blob/main/LICENSE)

Keep-alive **HTTP/1.1** transport for [Weavatrix SEO](https://github.com/Weavatrix/weavatrix-seo).

DNS cache, connection pool, gzip (including gzip *files* without `Content-Encoding`), TLS (`tls` feature, default on). It is not a browser and not HTTP/2.

## API

- `Fetcher` / `FetchResponse` — one origin-aware client
- `FetchBudget` — byte/time caps
- `NetworkPolicy` — public-only vs allow-private
- `HttpError` — DNS, timeout, TLS, policy

```rust
use weavatrix_seo_http::{Fetcher, NetworkPolicy};

let fetcher = Fetcher::new(NetworkPolicy::PublicOnly);
// crawl layer owns robots, sitemaps, and HTML extraction
```

```toml
[dependencies]
weavatrix-seo-http = "0.6.2"
```

Crawling sits in [`weavatrix-seo-crawl`](https://crates.io/crates/weavatrix-seo-crawl). MIT.

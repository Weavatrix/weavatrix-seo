# weavatrix-seo-crawl

[![crates.io](https://img.shields.io/crates/v/weavatrix-seo-crawl.svg)](https://crates.io/crates/weavatrix-seo-crawl)
[![docs.rs](https://docs.rs/weavatrix-seo-crawl/badge.svg)](https://docs.rs/weavatrix-seo-crawl)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/Weavatrix/weavatrix-seo/blob/main/LICENSE)

Bounded **discovery crawl** over [`weavatrix-seo-http`](https://crates.io/crates/weavatrix-seo-http).

Robots groups, sitemaps (including `.xml.gz`), HTML extraction, landings before sitemap floods, first city URL per family sampled. Rendering belongs in [`weavatrix-seo-render`](https://crates.io/crates/weavatrix-seo-render) — this crate records the **raw HTTP** response only.

## API

- `Crawl` / `CrawlConfig` — workers, max pages, public-only
- `Robots::parse` — group-based `User-agent` / `Allow` / `Disallow`
- `parse_sitemap` — urlset and sitemap index
- `extract_html` — canonical, hreflang, JSON-LD nodes, links, images
- `CrawlBudget`

```toml
[dependencies]
weavatrix-seo-crawl = "0.6.2"
```

The product audit is [`weavatrix-seo`](https://crates.io/crates/weavatrix-seo). MIT.

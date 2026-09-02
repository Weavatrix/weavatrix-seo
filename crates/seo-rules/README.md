# weavatrix-seo-rules

[![crates.io](https://img.shields.io/crates/v/weavatrix-seo-rules.svg)](https://crates.io/crates/weavatrix-seo-rules)
[![docs.rs](https://docs.rs/weavatrix-seo-rules/badge.svg)](https://docs.rs/weavatrix-seo-rules)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/Weavatrix/weavatrix-seo/blob/main/LICENSE)

Deterministic **technical SEO rules** over a crawl inventory.

Canonical graph (including unmeasured targets), hreflang reciprocity, metadata, schema feature eligibility vs vocabulary, internal links, sitemaps, status, query strings, AI robots roles.

```rust
use weavatrix_seo_rules::audit;
let findings = audit(&inventory);
```

Findings use the per-code registry in [`weavatrix-seo-model`](https://crates.io/crates/weavatrix-seo-model): `WVX-SEO-CANON-004` is unmeasured canonical, `WVX-SEO-SCHEMA-002` is a Google rich-result miss, `WVX-SEO-SCHEMA-003` is a schema.org gap.

```toml
[dependencies]
weavatrix-seo-rules = "0.6.2"
```

Quality headers/H1/a11y live in [`weavatrix-seo-quality`](https://crates.io/crates/weavatrix-seo-quality). MIT.

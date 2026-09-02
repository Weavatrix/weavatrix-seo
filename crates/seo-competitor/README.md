# weavatrix-seo-competitor

[![crates.io](https://img.shields.io/crates/v/weavatrix-seo-competitor.svg)](https://crates.io/crates/weavatrix-seo-competitor)
[![docs.rs](https://docs.rs/weavatrix-seo-competitor/badge.svg)](https://docs.rs/weavatrix-seo-competitor)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/Weavatrix/weavatrix-seo/blob/main/LICENSE)

Public-site **structural comparison**. Schema types, hreflang locales, FAQ archetypes, service cardinality, guide prefixes, H1 coverage, internal-link shape.

Competitor **prose is never copied**.

```rust
use weavatrix_seo_competitor::compare_inventories;

let gaps = compare_inventories(&owned, &[("https://them.example/".into(), other)]);
```

CLI: `weavatrix-seo compare --site URL --competitor URL`. MCP: `seo_compare`. Live bot-walls stay unmeasured (one-page inventories).

```toml
[dependencies]
weavatrix-seo-competitor = "0.6.2"
```

MIT.

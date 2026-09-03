# weavatrix-seo-observation

[![crates.io](https://img.shields.io/crates/v/weavatrix-seo-observation.svg)](https://crates.io/crates/weavatrix-seo-observation)
[![docs.rs](https://docs.rs/weavatrix-seo-observation/badge.svg)](https://docs.rs/weavatrix-seo-observation)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/Weavatrix/weavatrix-seo/blob/main/LICENSE)

Imported **search-observation** contracts. No vendor crawlers.

Kinds: `search_performance`, `bot_crawl`, `ai_citation`, `serp_position`, `analytics`. Bot hits never become demand. An unknown provider stays `analytics`. Tagged `previous`/`current` windows drive decay; CTR and striking-distance views work on a single export. Expected CTR is inferred.

```rust
use weavatrix_seo_observation::{load_any, unmeasured, ObservationKind};

let snap = load_any("gsc.json").unwrap_or_else(|_| unmeasured());
```

CLI: `--gsc` / `--observations`. MCP: `seo_observations`. `ai_visibility` stays unmeasured until a generative-search citation is imported.

```toml
[dependencies]
weavatrix-seo-observation = "0.6.2"
```

MIT.

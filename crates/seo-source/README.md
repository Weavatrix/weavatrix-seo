# weavatrix-seo-source

[![crates.io](https://img.shields.io/crates/v/weavatrix-seo-source.svg)](https://crates.io/crates/weavatrix-seo-source)
[![docs.rs](https://docs.rs/weavatrix-seo-source/badge.svg)](https://docs.rs/weavatrix-seo-source)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/Weavatrix/weavatrix-seo/blob/main/LICENSE)

Repository **search-surface** contracts: predicted route families, producers, `.weavatrix/seo.json` policy, `FrameworkCapabilities`.

```rust
use weavatrix_seo_source::{FrameworkCapabilities, SourceSurface, load_policy};

let policy = load_policy("./repo");
```

`FrameworkCapabilities` values are `exact` / `high` / `partial` / `unmeasured`. `seo_explain` must not outrun them.

Framework file prediction lives in [`weavatrix-seo-nextjs`](https://crates.io/crates/weavatrix-seo-nextjs) (Next.js, Nuxt, Astro).

```toml
[dependencies]
weavatrix-seo-source = "0.6.2"
```

MIT.

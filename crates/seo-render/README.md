# weavatrix-seo-render

[![crates.io](https://img.shields.io/crates/v/weavatrix-seo-render.svg)](https://crates.io/crates/weavatrix-seo-render)
[![docs.rs](https://docs.rs/weavatrix-seo-render/badge.svg)](https://docs.rs/weavatrix-seo-render)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/Weavatrix/weavatrix-seo/blob/main/LICENSE)

Raw-versus-**rendered** evidence boundary. Weavatrix SEO does not own a browser.

Import a `weavatrix-seo-render/v1` snapshot from Weavatrix Quality / Playwright (`--render PATH`). Missing import is `unmeasured`, not a pass.

```rust
use weavatrix_seo_render::{load, reconcile, unmeasured};

let report = load("render.json").unwrap_or_else(|_| unmeasured());
```

API: `RenderSnapshot`, `RenderedPage`, `reconcile`, `RenderMode`.

```toml
[dependencies]
weavatrix-seo-render = "0.6.2"
```

MIT.

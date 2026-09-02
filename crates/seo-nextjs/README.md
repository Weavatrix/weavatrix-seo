# weavatrix-seo-nextjs

[![crates.io](https://img.shields.io/crates/v/weavatrix-seo-nextjs.svg)](https://crates.io/crates/weavatrix-seo-nextjs)
[![docs.rs](https://docs.rs/weavatrix-seo-nextjs/badge.svg)](https://docs.rs/weavatrix-seo-nextjs)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/Weavatrix/weavatrix-seo/blob/main/LICENSE)

**Next.js App Router, Pages Router, Nuxt `pages/`, Astro `src/pages/`** adapters that predict a `SourceSurface`.

```rust
use weavatrix_seo_nextjs::predict;

let surface = predict("./repo");
assert!(!surface.families.is_empty() || surface.capabilities.is_none());
```

Capabilities are honest: App Router metadata can be `exact`; Nuxt/Astro frontmatter is often `unmeasured`. `tsconfig` `paths` aliases resolve helpers beyond `@/` and `~/`.

```toml
[dependencies]
weavatrix-seo-nextjs = "0.6.2"
```

Surface types: [`weavatrix-seo-source`](https://crates.io/crates/weavatrix-seo-source). MIT.

# weavatrix-seo-semantic

[![crates.io](https://img.shields.io/crates/v/weavatrix-seo-semantic.svg)](https://crates.io/crates/weavatrix-seo-semantic)
[![docs.rs](https://docs.rs/weavatrix-seo-semantic/badge.svg)](https://docs.rs/weavatrix-seo-semantic)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/Weavatrix/weavatrix-seo/blob/main/LICENSE)

Adapter from SEO pages to **weavatrix-semantic** profiles. First-party lexical model `wvx-seo-lexhash-v1` (64-d). No embedding service.

```rust
use weavatrix_seo_semantic::{analyze, embed, link_inputs, MODEL};

assert_eq!(MODEL, "wvx-seo-lexhash-v1");
let pass = analyze(&inventory, &architecture);
```

`seo_retrieve` copies **lexical** scores. `semantic` stays `None` for this model. Directed `seo_links` recommendations are `INFERRED`, never a ranking proof.

```toml
[dependencies]
weavatrix-seo-semantic = "0.6.2"
```

MIT.

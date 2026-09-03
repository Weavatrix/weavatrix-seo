# weavatrix-seo-content

[![crates.io](https://img.shields.io/crates/v/weavatrix-seo-content.svg)](https://crates.io/crates/weavatrix-seo-content)
[![docs.rs](https://docs.rs/weavatrix-seo-content/badge.svg)](https://docs.rs/weavatrix-seo-content)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/Weavatrix/weavatrix-seo/blob/main/LICENSE)

Duplicate and **content-identity** analysis: exact duplicates, MinHash near-duplicates, per-page profiles, family template decomposition, heading chunks, intent fanout per URL and route family.

```rust
use weavatrix_seo_content::{audit, exact_duplicates, near_duplicates};

let pass = audit(&inventory);
```

Exact-duplicate detection is never replaced by near-duplicates. Authorship of synthetic-style bands stays `UNMEASURED`.

```toml
[dependencies]
weavatrix-seo-content = "0.6.2"
```

MIT.

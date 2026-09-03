# weavatrix-seo-architecture

[![crates.io](https://img.shields.io/crates/v/weavatrix-seo-architecture.svg)](https://crates.io/crates/weavatrix-seo-architecture)
[![docs.rs](https://docs.rs/weavatrix-seo-architecture/badge.svg)](https://docs.rs/weavatrix-seo-architecture)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/Weavatrix/weavatrix-seo/blob/main/LICENSE)

Internal-link **architecture** over the Search Evidence Graph: depth, orphans, weighted PageRank (body over chrome), equity leaks, template annotation.

```rust
use weavatrix_seo_architecture::{analyze, annotate_templates};

annotate_templates(&mut inventory);
let (architecture, findings) = analyze(&inventory);
```

This is site-graph architecture, not Weavatrix Core's repository architecture contract.

```toml
[dependencies]
weavatrix-seo-architecture = "0.6.2"
```

MIT.

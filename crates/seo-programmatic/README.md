# weavatrix-seo-programmatic

[![crates.io](https://img.shields.io/crates/v/weavatrix-seo-programmatic.svg)](https://crates.io/crates/weavatrix-seo-programmatic)
[![docs.rs](https://docs.rs/weavatrix-seo-programmatic/badge.svg)](https://docs.rs/weavatrix-seo-programmatic)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/Weavatrix/weavatrix-seo/blob/main/LICENSE)

Programmatic SEO **safety compiler**. City/service matrices, thin variants, `SAFE_TO_GENERATE` that requires fact coverage — not two unique samples.

```rust
use weavatrix_seo_programmatic::{compile, enrich, thin_city_variants, SafetyVerdict};

let matrices = enrich(compile(&inventory, &predicted), &families);
let thin = thin_city_variants(&inventory);
```

`SafetyVerdict::{SafeToGenerate, Unsafe, Unmeasured}`. Unmeasured requirements stay unmeasured.

```toml
[dependencies]
weavatrix-seo-programmatic = "0.6.2"
```

MIT.

# weavatrix-seo-quality

[![crates.io](https://img.shields.io/crates/v/weavatrix-seo-quality.svg)](https://crates.io/crates/weavatrix-seo-quality)
[![docs.rs](https://docs.rs/weavatrix-seo-quality/badge.svg)](https://docs.rs/weavatrix-seo-quality)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/Weavatrix/weavatrix-seo/blob/main/LICENSE)

Live-document **quality evidence**: H1, Open Graph, accessibility of the HTML, origin security headers, fetch cost, mixed-content **subresources**.

```rust
use weavatrix_seo_quality::audit;
let findings = audit(&inventory);
```

`WVX-SEO-SEC-008` fires on `http://` images and `og:image` on an HTTPS page. Navigation `<a href="http://…">` is not mixed content.

This is not Weavatrix Quality (WVQ). Browser network proof is imported separately via [`weavatrix-seo-render`](https://crates.io/crates/weavatrix-seo-render).

```toml
[dependencies]
weavatrix-seo-quality = "0.6.2"
```

MIT.

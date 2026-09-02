# weavatrix-seo-claims

[![crates.io](https://img.shields.io/crates/v/weavatrix-seo-claims.svg)](https://crates.io/crates/weavatrix-seo-claims)
[![docs.rs](https://docs.rs/weavatrix-seo-claims/badge.svg)](https://docs.rs/weavatrix-seo-claims)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/Weavatrix/weavatrix-seo/blob/main/LICENSE)

Public **claim → domain fact** graph. Built-in contractor packs (US-WA, IL) plus `.weavatrix/seo.pack.yaml`.

```rust
use weavatrix_seo_claims::{audit_with_graph, pack_digest};

let (findings, domain) = audit_with_graph(&inventory, Some("./repo"));
let _ = pack_digest(); // hashes pack *content*, not only ids
```

License phrases without a true `license_verified` fact are contradictions. Foreign-market entities on the wrong origin are contamination. Extra packs participate in `policy_pack_digest`.

```toml
[dependencies]
weavatrix-seo-claims = "0.6.2"
```

MIT.

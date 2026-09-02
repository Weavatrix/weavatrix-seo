# weavatrix-seo-history

[![crates.io](https://img.shields.io/crates/v/weavatrix-seo-history.svg)](https://crates.io/crates/weavatrix-seo-history)
[![docs.rs](https://docs.rs/weavatrix-seo-history/badge.svg)](https://docs.rs/weavatrix-seo-history)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/Weavatrix/weavatrix-seo/blob/main/LICENSE)

Revision-bound crawl **snapshots** and `seo_diff`. Page bodies are not stored.

```rust
use weavatrix_seo_history::{diff, load, save, StoredSnapshot};

let snap = StoredSnapshot::from_report(&report);
save("./seo-history", &snap)?;
let delta = diff(&base, &head);
```

Producer diff prefers `symbol_hash` when both sides have it, else file hash. Missing `EvidenceSemantics` is `legacy_semantics`, not full equivalence. `index.jsonl` lists runs without re-reading every snapshot.

CLI: `weavatrix-seo audit --history DIR` and `weavatrix-seo diff --base/--head`. MCP: `seo_diff`.

```toml
[dependencies]
weavatrix-seo-history = "0.6.2"
```

MIT.

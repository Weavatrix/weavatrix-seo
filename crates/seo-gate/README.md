# weavatrix-seo-gate

[![crates.io](https://img.shields.io/crates/v/weavatrix-seo-gate.svg)](https://crates.io/crates/weavatrix-seo-gate)
[![docs.rs](https://docs.rs/weavatrix-seo-gate/badge.svg)](https://docs.rs/weavatrix-seo-gate)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/Weavatrix/weavatrix-seo/blob/main/LICENSE)

Evidence-graph **CI gate**: fail on error findings, regress against a comparable baseline.

CLI: `weavatrix-seo audit --ci --baseline previous.json`. MCP: `seo_gate`.

```rust
use weavatrix_seo_gate::{evaluate, from_report, load_baseline};

let baseline = load_baseline("previous.json")?;
let verdict = evaluate(&baseline, &report);
```

Incomparable origin/mode/semantics do not resolve findings. A smaller crawl is coverage change, not a set of resolved errors. `GateVerdict` is the MCP return; the CLI maps it to an exit code.

```toml
[dependencies]
weavatrix-seo-gate = "0.6.2"
```

MIT.

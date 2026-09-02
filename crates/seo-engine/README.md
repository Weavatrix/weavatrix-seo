# weavatrix-seo

[![crates.io](https://img.shields.io/crates/v/weavatrix-seo.svg)](https://crates.io/crates/weavatrix-seo)
[![docs.rs](https://docs.rs/weavatrix-seo/badge.svg)](https://docs.rs/weavatrix-seo)
[![npm](https://img.shields.io/npm/v/weavatrix-seo.svg)](https://www.npmjs.com/package/weavatrix-seo)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/Weavatrix/weavatrix-seo/blob/main/LICENSE)

Source-aware **Search Intelligence engine**. Library crate for the [Weavatrix SEO](https://github.com/Weavatrix/weavatrix-seo) product.

The **CLI and MCP host are the same native binary** (`weavatrix-seo` / `weavatrix-seo mcp`). Install those from [`weavatrix-seo-cli`](https://crates.io/crates/weavatrix-seo-cli) or [npm `weavatrix-seo`](https://www.npmjs.com/package/weavatrix-seo). This crate is the embeddable engine those hosts call.

## Embed

```toml
[dependencies]
weavatrix-seo = "0.6.2"
```

```rust
use weavatrix_seo::{AuditRequest, run_audit, retrieve, parse_query, run_on_report};

let request = AuditRequest { /* site, repo, max_pages, … */ };
let report = run_audit(&request)?;
let pages = retrieve(&report, "iphone screen repair", 10);
```

## Engine API

| Function | Role |
|---|---|
| `run_audit` / `run_inventory` | Crawl + assemble the Search Evidence Graph |
| `parse_query` / `run_on_report` | Bounded `FROM … WHERE … LIMIT` |
| `retrieve` / `similar` / `chunks_for` | Lexical candidate pages and chunks |
| `explain_chain` | Finding → URL → route → symbol |
| `plan_from` | Search architecture plan + Refactor handoff |
| `evaluate_gate` | CI verdict vs baseline |
| `link_inputs` | Page vectors for `seo_links` |
| `render_html` / `render_text` | Reports |

Full CLI commands, 15 MCP tools, agent configs, and examples: the [GitHub README](https://github.com/Weavatrix/weavatrix-seo).

MIT.

# weavatrix-seo-cli

[![crates.io](https://img.shields.io/crates/v/weavatrix-seo-cli.svg)](https://crates.io/crates/weavatrix-seo-cli)
[![npm](https://img.shields.io/npm/v/weavatrix-seo.svg)](https://www.npmjs.com/package/weavatrix-seo)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/Weavatrix/weavatrix-seo/blob/main/LICENSE)

Command-line surface for [Weavatrix SEO](https://github.com/Weavatrix/weavatrix-seo). Binary name: **`weavatrix-seo`**.

`weavatrix-seo mcp` is the MCP host — not a second product. Same engine as [`weavatrix-seo-mcp`](https://crates.io/crates/weavatrix-seo-mcp) and npm `weavatrix-seo`.

```bash
cargo install weavatrix-seo-cli --locked
# or: npm install -g weavatrix-seo

weavatrix-seo audit --site https://example.com --json
weavatrix-seo audit --site https://example.com --repo . --html report.html
weavatrix-seo compare --site https://example.com --competitor https://them.example --json
weavatrix-seo query --site https://example.com --q "FROM urls WHERE inbound_links = 0 AND indexable = true LIMIT 20" --json
weavatrix-seo query --history ./seo-history --q "FROM urls WHERE clicks_delta_28d < -30 AND producer_changed = true LIMIT 20" --json
weavatrix-seo retrieve --site https://example.com --q "licensed electrician vancouver" --json
weavatrix-seo plan --site https://example.com --json
weavatrix-seo mcp --allow-root .
```

| Command | MCP tool | Purpose |
|---|---|---|
| `audit` | `seo_audit` | Findings by axis, severity, authority |
| `inventory` | `seo_inventory` | Measured URLs and producers |
| `opportunities` | `seo_opportunities` | Gaps to build |
| `plan` | `seo_plan` | Architecture plan |
| `compare` | `seo_compare` | Competitor structure |
| `query` | `seo_query` | Bounded DSL |
| `retrieve` | `seo_retrieve` | Candidate pages |
| `explain` | `seo_explain` | Evidence chain |
| `diff` | `seo_diff` | Snapshot delta |
| `mcp` | *(host)* | All 15 tools over stdio |

Flags: `--json`, `--max-pages`, `--workers`, `--html`, `--ci`, `--baseline`, `--public-only`, `--gsc`, `--observations`, `--render`, `--history`.

Examples, agent configs, and evidence rules: [github.com/Weavatrix/weavatrix-seo](https://github.com/Weavatrix/weavatrix-seo). MIT.

# weavatrix-seo-mcp

[![crates.io](https://img.shields.io/crates/v/weavatrix-seo-mcp.svg)](https://crates.io/crates/weavatrix-seo-mcp)
[![npm](https://img.shields.io/npm/v/weavatrix-seo.svg)](https://www.npmjs.com/package/weavatrix-seo)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/Weavatrix/weavatrix-seo/blob/main/LICENSE)

Bounded **MCP host** for [Weavatrix SEO](https://github.com/Weavatrix/weavatrix-seo). Transport: [mcport](https://github.com/Weavatrix/mcport).

This is **not a separate product**. `weavatrix-seo mcp` in [`weavatrix-seo-cli`](https://crates.io/crates/weavatrix-seo-cli) and npm bin `weavatrix-seo-mcp` are the same host. Fifteen tools, no shell, paths confined to `--allow-root`.

```bash
cargo install weavatrix-seo-mcp --locked
# or: npm install -g weavatrix-seo

weavatrix-seo-mcp --allow-root /path/to/repo
```

### Claude Code

```bash
claude mcp add weavatrix-seo -- npx -y weavatrix-seo mcp --allow-root .
```

### Codex

```toml
[mcp_servers.weavatrix-seo]
command = "npx"
args = ["-y", "weavatrix-seo", "mcp", "--allow-root", "."]
```

## Tools (15)

| Tool | What it returns |
|---|---|
| `seo_inventory` | Search surface inventory |
| `seo_audit` | Findings by axis / severity / authority |
| `seo_opportunities` | Construction gaps |
| `seo_plan` | Architecture plan + Refactor handoff |
| `seo_compare` | Public competitor structural gaps |
| `seo_links` | Directed internal-link recommendations |
| `seo_vectors` | Lexical page vectors |
| `seo_diff` | Snapshot delta |
| `seo_gate` | CI verdict |
| `seo_explain` | Evidence chain |
| `seo_observations` | GSC / logs / citations |
| `seo_query` | Bounded DSL |
| `seo_retrieve` | Candidate pages |
| `seo_similar` | Pages like a URL |
| `seo_chunks` | Answering chunks |

```json
{ "name": "seo_audit", "arguments": { "site": "https://example.com", "max_pages": 80 } }
```

MCP crawls are public-only. Missing evidence is `unmeasured`. Full examples: [github.com/Weavatrix/weavatrix-seo](https://github.com/Weavatrix/weavatrix-seo). MIT.

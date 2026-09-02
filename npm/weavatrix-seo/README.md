# weavatrix-seo

[![npm](https://img.shields.io/npm/v/weavatrix-seo.svg)](https://www.npmjs.com/package/weavatrix-seo)
[![crates.io](https://img.shields.io/crates/v/weavatrix-seo.svg)](https://crates.io/crates/weavatrix-seo)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/Weavatrix/weavatrix-seo/blob/main/LICENSE)

Native [Weavatrix SEO](https://github.com/Weavatrix/weavatrix-seo) **CLI and MCP host**. One binary, two sockets.

**Audit the site you shipped, understand the code that produced it, and build the search architecture you are missing.**

This npm package is a zero-dependency Node launcher around prebuilt `weavatrix-seo` binaries for Windows, macOS, and Linux on x64 and arm64. Installation does not compile, download, or run lifecycle scripts. `weavatrix-seo-mcp` is the same native binary with `mcp` as the first argument.

## Install

```bash
npm install -g weavatrix-seo
# or
npx -y weavatrix-seo audit --site https://example.com --json
```

Rust:

```bash
cargo install weavatrix-seo-cli --locked
```

## 30-second CLI

```bash
weavatrix-seo --version
weavatrix-seo audit --site https://example.com --json
weavatrix-seo audit --site https://example.com --html report.html
weavatrix-seo audit --site https://example.com --ci --baseline previous.json
weavatrix-seo inventory --site https://example.com --json
weavatrix-seo opportunities --site https://example.com --json
weavatrix-seo plan --site https://example.com --json
weavatrix-seo compare --site https://example.com --competitor https://competitor.example --json
weavatrix-seo query --site https://example.com --q "FROM urls WHERE inbound_links = 0 AND indexable = true LIMIT 20" --json
weavatrix-seo retrieve --site https://example.com --q "licensed electrician vancouver" --json
weavatrix-seo explain WVX-SEO-META-001:abcd1234 --site https://example.com --json
weavatrix-seo diff --base ./seo-history/a.json --head ./seo-history/b.json --json
```

| Command | What it does |
|---|---|
| `audit` | Crawl + findings by axis, severity, authority |
| `inventory` | Measured URLs, routes, producers |
| `opportunities` | Gaps to build, not current errors |
| `plan` | Target search architecture + Refactor handoff |
| `compare` | Owned site vs public competitor origins |
| `query` | Bounded DSL over the last audit |
| `retrieve` | Candidate pages (lexical, not an embedding API) |
| `explain` | URL → route → symbol chain |
| `diff` | Two snapshots, audit JSON files, or worktrees |
| `mcp` | Same engine over stdio MCP |

Hybrid (live site vs the repo that should have built it):

```bash
weavatrix-seo audit --site https://example.com --repo . --json
```

## Add MCP (same binary)

MCP is not a second package. Point the agent at `weavatrix-seo mcp`.

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

### Cursor

```json
{
  "mcpServers": {
    "weavatrix-seo": {
      "command": "npx",
      "args": ["-y", "weavatrix-seo", "mcp", "--allow-root", "."]
    }
  }
}
```

Paths a tool accepts stay inside `--allow-root` (cwd when omitted). MCP crawls are public-only. Missing evidence is `unmeasured`.

## MCP tools (15)

| Tool | What the agent gets |
|---|---|
| `seo_inventory` | Measured URLs, routes, producers |
| `seo_audit` | Findings by axis, severity, authority |
| `seo_opportunities` | Construction gaps |
| `seo_plan` | Architecture plan + read-only Refactor handoff |
| `seo_compare` | Structural gaps vs public competitors |
| `seo_links` | Directed internal-link recommendations (inferred) |
| `seo_vectors` | Page vectors, lexical model `wvx-seo-lexhash-v1` |
| `seo_diff` | Snapshot / worktree delta |
| `seo_gate` | CI verdict (`--ci` / `--baseline`) |
| `seo_explain` | Evidence chain for one fingerprint |
| `seo_observations` | Imported GSC / logs / AI citations |
| `seo_query` | `FROM urls\|findings\|… WHERE … LIMIT n` |
| `seo_retrieve` | Ranked candidate pages |
| `seo_similar` | Pages similar to a URL |
| `seo_chunks` | Chunks that best answer a query |

```json
{ "name": "seo_audit", "arguments": { "site": "https://example.com", "max_pages": 80 } }
```

```json
{ "name": "seo_query", "arguments": { "site": "https://example.com", "q": "FROM urls WHERE inbound_links = 0 AND indexable = true LIMIT 20" } }
```

```json
{ "name": "seo_retrieve", "arguments": { "site": "https://example.com", "q": "iphone screen repair haifa" } }
```

What an agent can ask:

```text
Which indexable URLs have zero internal inlinks?
Is this canonical target measured, or UNMEASURED because it was off-budget?
What source symbol emits the missing FAQPage.mainEntity?
Which competitor locales or schema types are we missing?
```

## Why not another crawler

- **Code-aware.** Next.js, Nuxt, and Astro families from source. A finding can name the helper that emitted the title.
- **Evidence-honest.** Google rich-result ≠ schema.org vocabulary. Off-crawl canonicals are `UNMEASURED`. Mixed content is subresources, not navigation links.
- **Agent-native.** Fifteen MCP tools are the CLI, not a wrapper around a report file.
- **Read-only.** No page writes, no generated articles, no patches.
- **No model on the default path.** Lexical vectors, no embedding API, no browser unless you import a WVQ snapshot.

Full engine contract, finding families, and crate map: [github.com/Weavatrix/weavatrix-seo](https://github.com/Weavatrix/weavatrix-seo).

MIT. See [LICENSE](LICENSE).

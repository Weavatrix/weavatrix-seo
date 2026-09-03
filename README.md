# Weavatrix SEO

[![CI](https://github.com/Weavatrix/weavatrix-seo/actions/workflows/ci.yml/badge.svg)](https://github.com/Weavatrix/weavatrix-seo/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/weavatrix-seo.svg)](https://crates.io/crates/weavatrix-seo)
[![npm](https://img.shields.io/npm/v/weavatrix-seo.svg)](https://www.npmjs.com/package/weavatrix-seo)
[![docs.rs](https://docs.rs/weavatrix-seo/badge.svg)](https://docs.rs/weavatrix-seo)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.88-blue.svg)](Cargo.toml)

Source-aware Search Intelligence for the [Weavatrix ecosystem](https://weavatrix.com/ecosystem).

**Audit the site you shipped, understand the code that produced it, and build the search architecture you are missing.**

Weavatrix SEO is not Screaming Frog with a JSON dump. It is a **Search Evidence Graph**: live HTTP, repository routes, schema, claims, and search observations bound to one revision. A finding on `/iphone-repair/haifa` can name the Next.js helper that emitted the title. Missing evidence stays `unmeasured`. It never turns green by silence.

```text
repository source
    + live / raw / rendered website
    + sitemap / canonical / hreflang / schema
    + internal links
    + GSC / logs / AI citations
    + domain / legal / business facts
    + git revision
                     │
                     ▼
              Search Evidence Graph
                     │
        AUDIT  ·  OPPORTUNITY  ·  BUILD
                     │
         CLI commands  =  MCP tools
                     │
         source-level fix location
```

The **CLI** (`weavatrix-seo`) and the **MCP host** (`weavatrix-seo mcp` / `weavatrix-seo-mcp`) are the **same native binary**. MCP is not a second product. It is the agent socket for the same inventory, audit, plan, query, and retrieve pipeline.

## Install in 30 seconds

```bash
npm install -g weavatrix-seo
# or: npx -y weavatrix-seo
# or: cargo install weavatrix-seo-cli --locked

weavatrix-seo --version
weavatrix-seo audit --site https://example.com --json
```

Prebuilt binaries cover Windows, macOS, and Linux on x64 and arm64. The npm package has **zero dependencies** and **no install scripts**.

| Surface | How you run it |
|---|---|
| CLI | `weavatrix-seo <command>` |
| MCP | `weavatrix-seo mcp` or `weavatrix-seo-mcp` |
| Library | `weavatrix-seo = "0.6.2"` |

## Why this is not another crawler

- **Code-aware.** Next.js App/Pages, Nuxt, and Astro families are predicted from source. Hybrid mode classifies SOURCE_ONLY vs RESPONSE_ONLY against the crawl budget.
- **Evidence-honest.** Authority, method, and scope sit on every finding. A Google rich-result miss is not a schema.org vocabulary miss. A target outside the crawl is `UNMEASURED`, not healthy.
- **Agent-native.** Fifteen MCP tools match the CLI. An agent can `seo_query` orphans, `seo_retrieve` a service page, and `seo_explain` a fingerprint without a shell.
- **Read-only.** SEO never writes pages, never generates articles, never applies patches. `seo_plan` can hand a location to Weavatrix Refactor; SEO still does not mutate source.
- **No model on the default path.** Similarity is first-party lexical (`wvx-seo-lexhash-v1`). There is no embedding API and no browser unless you import a WVQ render snapshot.

## CLI commands

Every command takes `--json` for structured output. `--max-pages N` bounds the crawl. `--workers N` sets parallel fetches (default 5). `--public-only` refuses loopback and private destinations (MCP default).

| Command | What it does |
|---|---|
| `audit` | Crawl + deterministic findings by axis, severity, authority |
| `inventory` | Measured URLs, routes, producers — no ranking |
| `opportunities` | Gaps to *build*, not current errors |
| `plan` | Target search architecture + Refactor handoff |
| `compare` | Owned site vs public competitor origins |
| `query` | Bounded DSL over the last audit |
| `retrieve` | Candidate pages for a query (lexical) |
| `explain` | URL → route → symbol chain for one fingerprint |
| `diff` | Two snapshots, audit JSON files, or worktrees |
| `opportunities` | Ranked construction work |
| `mcp` | Same engine over stdio MCP |

### First audit

```bash
weavatrix-seo audit --site https://example.com --json
weavatrix-seo audit --site https://example.com --html report.html
weavatrix-seo audit --site https://example.com --ci --baseline previous.json
```

`--ci` exits non-zero on error findings. `--baseline` treats missing URLs as coverage regressions, not resolved issues.

### Hybrid: production vs the repo that should have built it

```bash
weavatrix-seo audit --site https://kablay.us --repo ./kablay-us --json
```

The edge is `COMPARED_AGAINST`, never `CHANGED_BY`. Hybrid does not prove the live site was built from this worktree.

### Compare

```bash
weavatrix-seo compare --site https://kablay.us --competitor https://kablay.co.il --json
```

Structural gaps only: schema types, hreflang locales, FAQ archetypes, service cardinality, H1 coverage. Competitor prose is never copied.

### Query and retrieve

```bash
weavatrix-seo query --site https://example.com --q "FROM urls WHERE inbound_links = 0 AND indexable = true LIMIT 20" --json
weavatrix-seo retrieve --site https://example.com --q "licensed electrician vancouver" --json
weavatrix-seo explain WVX-SEO-CRAWL-001:abcd1234 --site https://example.com --json
```

`retrieve` returns `lexical` scores. `semantic` is `null` for `wvx-seo-lexhash-v1`. That is intentional.

### History and CI

```bash
weavatrix-seo audit --site https://example.com --history ./seo-history --json
weavatrix-seo diff --base ./seo-history/aaa.json --head ./seo-history/bbb.json --json
weavatrix-seo audit --site https://example.com --gsc gsc.json --observations logs.json --render render.json --json
```

## Add the MCP host

MCP is the same binary. Point the agent at `weavatrix-seo mcp` (or `weavatrix-seo-mcp`). Paths a tool accepts are confined to `--allow-root` (cwd when omitted).

### Claude Code

```bash
claude mcp add weavatrix-seo -- npx -y weavatrix-seo mcp --allow-root .
```

### Codex

Plugin (icon included):

```text
codex plugin marketplace add Weavatrix/weavatrix-seo --sparse .agents/plugins plugins/weavatrix-seo
codex plugin add weavatrix-seo@weavatrix-seo
```

Or MCP only:

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

### Cargo

```bash
cargo install weavatrix-seo-cli --locked
weavatrix-seo mcp --allow-root /path/to/site-repo
```

## MCP tools (15)

CLI command on the left, MCP tool on the right. Same engine.

| MCP tool | CLI | What the agent gets |
|---|---|---|
| `seo_inventory` | `inventory` | Measured URLs, routes, producers |
| `seo_audit` | `audit` | Findings by axis, severity, authority |
| `seo_opportunities` | `opportunities` | Construction gaps, not current errors |
| `seo_plan` | `plan` | Architecture plan + read-only Refactor handoff |
| `seo_compare` | `compare` | Structural gaps vs public competitors |
| `seo_links` | *(audit vectors)* | Directed internal-link recommendations |
| `seo_vectors` | *(audit vectors)* | Page vectors, lexical model id |
| `seo_diff` | `diff` | Snapshot / worktree delta |
| `seo_gate` | `audit --ci` | Gate verdict instead of an exit code |
| `seo_explain` | `explain` | Evidence chain for one fingerprint |
| `seo_observations` | `--gsc` / `--observations` | Imported GSC / logs / citations |
| `seo_query` | `query` | Bounded `FROM … WHERE … LIMIT` (live crawl or `--history` SQLite) |
| `seo_retrieve` | `retrieve` | Ranked candidate pages |
| `seo_similar` | *(retrieve)* | Pages similar to a URL |
| `seo_chunks` | *(retrieve)* | Chunks that best answer a query |

Crawl-backed tools accept `site`, `repo`, `mode`, `max_pages`, `workers`, `gsc`, `observations`, `render`, `history`. MCP crawls are public-only.

Example calls:

```json
{ "name": "seo_audit", "arguments": { "site": "https://example.com", "max_pages": 80 } }
```

```json
{ "name": "seo_query", "arguments": { "site": "https://example.com", "query": "FROM urls WHERE inbound_links = 0 AND indexable = true LIMIT 20" } }
```

```json
{ "name": "seo_retrieve", "arguments": { "site": "https://example.com", "q": "iphone screen repair haifa" } }
```

```json
{ "name": "seo_explain", "arguments": { "id": "WVX-SEO-META-001:abcd1234", "site": "https://example.com", "repo": "." } }
```

What an agent can ask:

```text
Which indexable URLs have zero internal inlinks?
Where does this canonical chain end, and is the target measured?
What source symbol emits the missing FAQPage.mainEntity?
Which competitor locales or schema types are we missing?
Hand me a plan for the city family that is SOURCE_ONLY.
```

## Sample output

```json
{
  "inventory": {
    "mode": "hybrid",
    "site": "https://example.com/",
    "counts": { "pages": 80, "indexable": 61, "errors": 3 }
  },
  "findings": [
    {
      "code": "WVX-SEO-LINK-001",
      "severity": "error",
      "authority": "search_engine_documented",
      "summary": "https://example.com/old is a broken internal link",
      "fingerprint": "WVX-SEO-LINK-001:a1b2c3d4"
    }
  ]
}
```

Missing evidence is never a pass. `semantic` on retrieve is `null` until a real embedding model is bound.

## Modes

| Mode | Input | What it can prove |
|---|---|---|
| Site-only | `--site URL` | Live crawl, technical audit, architecture, duplicates |
| Repo-only | `--repo PATH` | Next.js / Nuxt / Astro families, sitemap/robots owners |
| Hybrid | `--repo` + `--site` | Source intent versus HTTP inventory |
| Compare | `--site` + `--competitor` | Structural archetype / schema / locale / H1 gaps |

## Library crates

Install the engine:

```toml
[dependencies]
weavatrix-seo = "0.6.2"
```

Or compose a layer:

| Crate | Owns |
|---|---|
| [`weavatrix-seo-model`](https://crates.io/crates/weavatrix-seo-model) | Graph types, findings, rule registry |
| [`weavatrix-seo-http`](https://crates.io/crates/weavatrix-seo-http) | Keep-alive HTTP/1.1 |
| [`weavatrix-seo-crawl`](https://crates.io/crates/weavatrix-seo-crawl) | Bounded discovery crawl |
| [`weavatrix-seo-rules`](https://crates.io/crates/weavatrix-seo-rules) | Deterministic technical rules |
| [`weavatrix-seo-source`](https://crates.io/crates/weavatrix-seo-source) / [`-nextjs`](https://crates.io/crates/weavatrix-seo-nextjs) | Repo surface + framework adapters |
| [`weavatrix-seo-cli`](https://crates.io/crates/weavatrix-seo-cli) / [`-mcp`](https://crates.io/crates/weavatrix-seo-mcp) | CLI and MCP hosts |

Each crate README on crates.io names its API. Nothing is a silent re-export of the product README.

## Evidence rules

- crawler success is not proof of indexation
- a sitemap loc is not proof a page is indexed
- lexical similarity is never a ranking claim
- mixed content is subresources, not navigation `<a href>`
- canonical/hreflang targets outside the crawl are `UNMEASURED`
- AI crawler tokens have roles (training vs search discovery vs citation)

## License

MIT. See [LICENSE](LICENSE).

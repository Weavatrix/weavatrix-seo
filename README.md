# Weavatrix SEO

[![CI](https://github.com/Weavatrix/weavatrix-seo/actions/workflows/ci.yml/badge.svg)](https://github.com/Weavatrix/weavatrix-seo/actions/workflows/ci.yml)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.88-blue.svg)](Cargo.toml)

Source-aware Search Intelligence for the [Weavatrix ecosystem](https://weavatrix.com/ecosystem).

**Audit the site you shipped, understand the code that produced it, and build the search architecture you are missing.**

Weavatrix SEO turns repository source, live HTTP, rendered DOM, sitemaps, entities, claims, and search observations into one revision-bound Search Evidence Graph. It is not a detached crawler report and not a keyword database.

```text
repository source
    + build/runtime routes
    + live/raw/rendered website
    + sitemap/canonical/hreflang/schema
    + internal links
    + search observations
    + domain/legal/business facts
    + git revision
                     │
                     ▼
              Search Evidence Graph
                     │
        AUDIT  ·  OPPORTUNITY  ·  BUILD
                     │
              EXPLAIN / DIFF / PLAN
                     │
         source-level fix location
```

## Status

`0.6.0` is additive intelligence on top of the 0.5.0 domain graph. Findings now carry `RuleAuthority` (protocol vs search-engine documented vs experiment). Snapshot comparability uses `EvidenceSemantics` so crate version and rule meaning cannot drift apart. Unknown confidence/risk stays unknown in ranking. Content intelligence adds near-duplicates, per-page profiles, family template decomposition, chunks, and intent fanout — without replacing exact-duplicate detection. `SAFE_TO_GENERATE` now requires fact coverage, not two unique samples. MCP/CLI gain `seo_query`, `seo_retrieve`, `seo_similar`, and `seo_chunks`. Existing tools, findings, and modes stay. SEO does not own a browser.

- bounded first-party HTTP crawl with keep-alive workers (default 5)
- robots and sitemap discovery, landings before sitemap loc floods, first city URL per family sampled
- response metadata, canonical, hreflang, schema, links, headings, images, Open Graph
- deterministic technical audit plus H1 / a11y / origin security-header values / performance checks
- internal-link architecture (depth, orphans, authority)
- exact-duplicate detection, near-duplicate MinHash clusters, and thin programmatic city variants
- content profiles, family template/fact decomposition, chunks, and intent fanout
- bounded `query` DSL and candidate-page `retrieve` (Rust computes similarity)
- HTML report (`--html PATH`) plus JSON
- directed internal-link recommendations from first-party page vectors, no embedding service
- market-entity contamination, license claim/fact contradictions, undeclared pack entities, and AI citation identity

`.weavatrix/seo.json` is optional. When present, `indexability.include` / `exclude` decide which route families may be CREATE/SOURCE_ONLY; otherwise private chrome (`/admin`, `/auth`, …) is excluded.

Repo-only Next.js App Router prediction is live. Hybrid classifies SOURCE_ONLY / RESPONSE_ONLY against the crawl budget. Compare crawls public competitor origins for structural gaps. Rendered DOM is measured only when `--render PATH` supplies a WVQ/Playwright snapshot. GSC/Bing/log imports use `--gsc` / `--observations`. Next.js RSC payloads are captured from HTTP. Missing evidence is never green.

The ordinary audit path does not call a model.

## Modes

| Mode | Input | What it can prove |
|---|---|---|
| Site-only | `--site URL` | Live crawl, technical audit, architecture, duplicates |
| Repo-only | `--repo PATH` | Next.js App Router families, sitemap/robots owners, programmatic flags |
| Hybrid | `--repo` + `--site` | Source intent versus HTTP inventory |
| Compare | `--site` + `--competitor` | Public-site structural archetype/schema/locale/cardinality/H1 gaps |

## Install

Nothing is on crates.io yet: every crate in this workspace is `publish = false`.
Build from a checkout:

```bash
cargo install --path apps/seo-cli --locked
```

Or straight from git:

```bash
cargo install --git https://github.com/Weavatrix/weavatrix-seo weavatrix-seo-cli --locked
```

The library is consumed the same way until the crates are published:

```toml
[dependencies]
weavatrix-seo = { git = "https://github.com/Weavatrix/weavatrix-seo" }
```

## CLI

```bash
weavatrix-seo audit --site https://example.com
weavatrix-seo audit --site https://example.com --workers 5 --html report.html --ci --baseline previous.json
weavatrix-seo audit --site https://example.com --public-only --json
weavatrix-seo plan --site https://example.com --gsc gsc.json --json
weavatrix-seo audit --site https://example.com --render render.json --json
weavatrix-seo audit --site https://example.com --history ./seo-history --json
weavatrix-seo diff --base ./seo-history/aaa.json --head ./seo-history/bbb.json --json
weavatrix-seo diff --base ./worktree-a --head ./worktree-b --json
weavatrix-seo inventory --site https://example.com
weavatrix-seo opportunities --site https://example.com
weavatrix-seo plan --site https://example.com
weavatrix-seo explain WVX-SEO-CRAWL-001:abcd1234
weavatrix-seo query --site https://example.com --q "FROM urls WHERE inbound_links = 0 AND indexable = true LIMIT 20"
weavatrix-seo retrieve --site https://example.com --q "licensed electrician vancouver"
weavatrix-seo mcp
```

`--json` prints the structured report. `--max-pages N` bounds the crawl. `--workers N` sets parallel fetches (default 5). `--html PATH` writes a standalone HTML report. `--ci` fails on error findings. `--baseline PATH` compares a previous audit or compact baseline; missing URLs are coverage regressions, not resolved. `--public-only` refuses loopback, private, and metadata destinations (MCP default). CLI still allows private/staging unless this flag is set. `--gsc PATH` or `--observations PATH` imports provider JSON. Each row carries a `kind` (`search_performance`, `bot_crawl`, `ai_citation`, `serp_position`, `analytics`) — declared per row or per file, otherwise implied by a known provider name. An unrecognised provider stays `analytics`, so it never becomes search demand. `--render PATH` imports a `weavatrix-seo-render/v1` snapshot from WVQ/Playwright; missing import is `unmeasured`, not a pass. `--history DIR` writes a compact snapshot (no page text). `diff --base/--head` compares two snapshots, audit JSON files, or worktree directories and reports producer impact: families whose helper, metadata, or imported data module changed even if the route pattern did not. Git SHAs without snapshot files stay unmeasured. `explain` prints the URL → route → symbol chain.

## Benchmarks

First-party throughput, no Criterion:

```bash
cargo bench -p weavatrix-seo
```

`crawl` measures loopback audit + query/retrieve. `content` profiles a synthetic
inventory. `query` repeats the DSL. `compare` diffs two loopback origins and
prints the first-party artifact matrix (evidence graph, chunks, authority,
unknown-stays-unknown). If `siteone-crawler` or `screamingfrogseospider` is on
`PATH`, those binaries are spawned as optional URL-list baselines — they are
never a product dependency.

Live competitor dogfood (network, public-only). Thumbtack is often a 1-page
bot wall; `kablay.co.il` is the structural baseline:

```bash
WEAVATRIX_SEO_LIVE=1 cargo test -p weavatrix-seo --test live_compare -- --nocapture
```

Evidence is snapshot-bound. `/foo` and `/foo/` stay distinct until the server redirects or canonicalizes. Redirect hops are graph edges; the final 200 is indexable. HTML-only rules skip PDF/JSON/image. Arbitrary inline script is not public copy.

## MCP

Existing tools stay. Four analytical primitives are additive:

```text
seo_inventory
seo_audit
seo_opportunities
seo_plan
seo_compare
seo_links
seo_vectors
seo_diff
seo_gate
seo_explain
seo_observations
seo_query
seo_retrieve
seo_similar
seo_chunks
```

Run `weavatrix-seo mcp` or the `weavatrix-seo-mcp` binary.

The crawl-backed tools accept every evidence import the CLI accepts: `gsc`, `observations`, `render`, `history`, `workers`, `max_pages`. `seo_gate` is `--ci` / `--baseline` and returns the verdict instead of an exit code. `seo_observations` reads a provider export; without one it stays unmeasured. MCP crawls are public-only.

Every path a tool accepts is confined to an allow-list. `weavatrix-seo mcp --allow-root PATH` is repeatable; with none declared the boundary is the working directory. Paths are canonicalised before the check, so `..` segments and symlinks cannot escape.

`seo_links` returns directed internal-link recommendations, and `seo_vectors` returns the page vectors and link profiles they were computed from. The embedding model is `wvx-seo-lexhash-v1`: first-party, deterministic, 64-dimension, and **lexical**. It needs no embedding service, and it cannot match synonyms or cross-language pairs. Node identities are `page:<url>`, so these are page-graph inputs, not repository-graph inputs. Similarity is `INFERRED` and is never upgraded to a ranking claim.

## Evidence

Every fact carries:

```text
kind:            DETERMINISTIC | OBSERVED | EXTERNAL | INFERRED | UNMEASURED
source:          repo | http | rendered_dom | sitemap | logs | gsc | provider | semantic
locator:         URL, header, DOM, source span
confidence:      exact | high | medium | low
snapshot_id:     measured crawl, not the seed URL
run_id:          one analysis invocation
policy_version:  finding semantics for this release (engine version)
                 plus EvidenceSemantics (artifact schema + rule digest)
authority:       protocol | search-engine | contract | jurisdiction |
                 practice | heuristic | opportunity
revision:        the worktree a source fact came from, never a live response
graph:           URL ─RENDERED_BY→ route ─METADATA_FROM→ symbol@span
                 URL ─COMPARED_AGAINST→ revision
                 URL ─CLAIMS→ claim ─REQUIRES→ field ─DEFINED_AT→ span
                 claim ─GOVERNED_BY→ policy · URL ─ABOUT→ entity / market
```

Rules:

- crawler success is not proof of indexation
- a sitemap entry is not proof a page is indexed
- semantic similarity is never upgraded to deterministic truth
- missing evidence is `unmeasured`, not a pass
- an observation says what it measured; crawler hits, analytics sessions, and AI
  citations are never read as search demand
- retrieval readiness is inferable, AI visibility is not: `ai_visibility` stays
  unmeasured until a generative-search citation is imported
- a hybrid run compares production against a worktree; it does not prove
  production was built from it, so the edge is `COMPARED_AGAINST` and never
  `CHANGED_BY`
- an HTTP response is never the provenance of a source fact
- an explicit `.weavatrix/seo.*` contract beats the built-in private-path
  heuristic; a contract that is present but unreadable is an error, not silence

## Findings

Stable families and fingerprints:

```text
WVX-SEO-CRAWL-*      crawl/discovery
WVX-SEO-IDX-*        indexability
WVX-SEO-CANON-*      canonical
WVX-SEO-SITEMAP-*    sitemap
WVX-SEO-I18N-*       hreflang/locale
WVX-SEO-RENDER-*     raw/render drift
WVX-SEO-META-*       metadata
WVX-SEO-SCHEMA-*     structured data
WVX-SEO-LINK-*       internal links
WVX-SEO-DUP-*        duplication
WVX-SEO-CANN-*       cannibalization
WVX-SEO-CONTENT-*    content coverage
WVX-SEO-ENTITY-*     entities
WVX-SEO-MARKET-*     market/jurisdiction
WVX-SEO-CLAIM-*      public claim integrity
WVX-SEO-PROG-*       programmatic SEO
WVX-SEO-PERF-*       performance
WVX-SEO-A11Y-*       accessibility
WVX-SEO-SEC-*        security headers
WVX-SEO-LOCAL-*      local SEO
WVX-SEO-AI-*         AI-search readiness
WVX-SEO-OBS-*        imported observations
WVX-SEO-COMP-*       competitive gaps
```

## Ecosystem

```text
weavatrix-rust / Weavatrix Core   source graph, spans, impact
weavatrix-semantic                similarity, SEO link policy, anchors
weavatrix-seo                     search evidence, audit, architecture, plan
Weavatrix Quality                 runtime/browser proof
weavatrix-refactor                guarded source mutation after explicit approval
```

Weavatrix SEO is read-only. It does not write pages, generate articles, or apply patches.

## License

MIT. See [LICENSE](LICENSE).

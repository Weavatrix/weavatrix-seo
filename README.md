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

`0.1.11` collapses shared chrome pack entities into one origin `ENTITY-001`. HSTS/CSP origin facts from 0.1.10 stay. SEO does not own a browser.

- bounded first-party HTTP crawl with keep-alive workers (default 5)
- robots and sitemap discovery, landings before sitemap loc floods, first city URL per family sampled
- response metadata, canonical, hreflang, schema, links, headings, images, Open Graph
- deterministic technical audit plus H1 / a11y / origin security-header values / performance checks
- internal-link architecture (depth, orphans, authority)
- exact-duplicate detection and thin programmatic city variants
- HTML report (`--html PATH`) plus JSON
- `seo_inventory`, `seo_audit`, `seo_explain`, `seo_opportunities`
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
| Compare | `--site` + `--competitor` | Public-site structural archetype/schema/locale gaps |

## Install

```bash
cargo install --path apps/seo-cli --locked
```

Library:

```toml
[dependencies]
weavatrix-seo = "0.1.11"
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
weavatrix-seo mcp
```

`--json` prints the structured report. `--max-pages N` bounds the crawl. `--workers N` sets parallel fetches (default 5). `--html PATH` writes a standalone HTML report. `--ci` fails on error findings. `--baseline PATH` compares a previous audit or compact baseline; missing URLs are coverage regressions, not resolved. `--public-only` refuses loopback, private, and metadata destinations (MCP default). CLI still allows private/staging unless this flag is set. `--gsc PATH` or `--observations PATH` imports GSC/Bing/bot-log JSON. `--render PATH` imports a `weavatrix-seo-render/v1` snapshot from WVQ/Playwright; missing import is `unmeasured`, not a pass. `--history DIR` writes a compact snapshot (no page text). `diff --base/--head` compares two snapshots, audit JSON files, or worktree directories and reports producer impact: families whose helper, metadata, or imported data module changed even if the route pattern did not. Git SHAs without snapshot files stay unmeasured. `explain` prints the URL → route → symbol chain.

Evidence is snapshot-bound. `/foo` and `/foo/` stay distinct until the server redirects or canonicalizes. Redirect hops are graph edges; the final 200 is indexable. HTML-only rules skip PDF/JSON/image. Arbitrary inline script is not public copy.

## MCP

Eight tools, no shell:

```text
seo_inventory
seo_audit
seo_opportunities
seo_plan
seo_compare
seo_diff
seo_explain
seo_observations
```

Run `weavatrix-seo mcp` or the `weavatrix-seo-mcp` binary.

## Evidence

Every fact carries:

```text
kind:            DETERMINISTIC | OBSERVED | EXTERNAL | INFERRED | UNMEASURED
source:          repo | http | rendered_dom | sitemap | logs | gsc | provider | semantic
locator:         URL, header, DOM, source span
confidence:      exact | high | medium | low
snapshot_id:     measured crawl, not the seed URL
run_id:          one analysis invocation
policy_version:  finding semantics for this release
revision:        git worktree when a repo is in scope
graph:           URL ─RENDERED_BY→ route ─METADATA_FROM→ symbol@span
```

Rules:

- crawler success is not proof of indexation
- a sitemap entry is not proof a page is indexed
- semantic similarity is never upgraded to deterministic truth
- missing evidence is `unmeasured`, not a pass

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

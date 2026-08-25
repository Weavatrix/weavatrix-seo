# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

## 0.1.2 - 2026-08-25

- Source impact cone: producer file hashes travel with snapshots. A helper/metadata edit names `families_impacted` even when the route pattern did not change.
- `seo_diff` reports `producers_changed`, `families_impacted`, and measured `urls_impacted`.
- `seo_explain` returns the evidence chain URL → route → symbol@span → revision. Repo-only explain is valid without a live site.

## 0.1.1 - 2026-08-25

- Render proof: ingest WVQ/Playwright JSON (`weavatrix-seo-render/v1`) via `--render PATH`. HTTP versus rendered title/canonical/H1/JSON-LD is `WVX-SEO-RENDER-003`–`007`. SEO still does not own a browser.
- `render_reconciliation` stays unmeasured until a render snapshot is present. A repo path is not render evidence.
- `seo_diff` accepts two worktree directories (predicted routes) or two snapshot files. Git SHAs without snapshots stay unmeasured.
- Loopback bench covers two origins. Live fixture origins stay out of CI.

## 0.1.0 - 2026-08-25

- Beta: compact crawl history (`--history DIR`) and revision-bound `seo_diff` (`diff --base PATH --head PATH`, MCP `seo_diff`).
- Provider adapters: GSC, Bing, and bot-log JSON via `--observations PATH`. Missing import stays unmeasured.
- Competitor compare now covers prefix families, indexable cardinality, internal-link leverage, and H1 coverage. Still no competitor prose.
- Snapshots omit page text/payload. Git SHAs without snapshot files stay unmeasured.

## 0.0.9 - 2026-08-25

- Semantic pass: first-party lexical embeddings through `weavatrix-semantic` (SEO policy, directed links, AnchorMatcher). Evidence is `INFERRED`.
- Near-duplicate intent is cannibalization (`WVX-SEO-CANN-001`); missing topical links are `WVX-SEO-LINK-004`.
- GSC is the first observation provider via `--gsc PATH` JSON export. Demand and visibility-gap axes rank opportunities. Uncrawled GSC URLs are `WVX-SEO-OBS-001`.
- `seo_plan` compiles CREATE/IMPROVE/CONSOLIDATE/LINK/NOINDEX/DELETE with evidence, acceptance, and verification. Still read-only.
- Programmatic compiler scores route families (`SAFE_TO_GENERATE` … `UNMEASURED`) from predicted patterns and measured URLs, including sitemap-only variants.
- Rendered DOM stays unmeasured. Do not treat this release as a browser crawler.

## 0.0.8 - 2026-08-25

- Heterogeneous Search Evidence Graph: URLs bind to route families, source symbols, schema objects, and revisions (`RENDERED_BY`, `GENERATED_BY`, `METADATA_FROM`, `DECLARES`, `CHANGED_BY`).
- Policy packs (`marketplace.contractor.us-wa` / `.il`) own entities and claims. Kablay is the first fixture pack, not core engine regexes. A false fact only contradicts claims of the same pack.
- Next.js adapter reads `next.config.*` (`basePath`, `trailingSlash`, redirects/rewrites), records metadata/`generateStaticParams`/JSON-LD/helper spans, and distinguishes intercepting routes from route groups.
- Internal links keep surrounding heading context and template frequency when the same shape repeats.

## 0.0.7 - 2026-08-25

- Snapshot, run, policy, and revision identities: every HTTP fact is bound to the measured crawl, not the seed URL.
- Fetch failures (DNS, TLS, timeout, body-limit, robots, SSRF) stay as observations and increment incomplete coverage.
- URL identity keeps `/foo` vs `/foo/`; query-only joins resolve against the current path; IPv6 hosts keep brackets.
- Redirect hops are their own pages/edges; the final 200 stays indexable. Relative canonicals resolve against the page URL.
- MCP/competitor fetches are public-only; CLI loopback/staging needs no extra flag, `--public-only` tightens it.
- DNS tries multiple addresses; `429`/`503` honour a capped `Retry-After`.
- HTML-only findings do not run on PDF/JSON/image bodies. Claim/market haystacks use visible text, JSON-LD, and recognized RSC — not arbitrary script.
- `license_verified=false` elsewhere in the repo is not a contradiction without a live public claim.
- CI baseline is comparable (origin/mode/policy/measured URLs); unmeasured errors are coverage regressions, not resolved.

## 0.0.6 - 2026-08-25

- Accessible-name: button inner text and submit `value` count; empty `alt` stays decorative.
- Shared unlabelled chrome is one origin finding, not a per-URL dump.
- Sample two city URLs per family so uniqueness can actually be measured.
- Origin `Referrer-Policy` evidence.

## 0.0.5 - 2026-08-25

- Split transport (`weavatrix-seo-http`), live quality (`weavatrix-seo-quality`), and evidence CI (`weavatrix-seo-gate`) out of the crawl/engine crates.
- Keep-alive pool, DNS cache, and gzip/deflate decode on the HTTP path.
- Origin-level security headers; alt absence is not empty decorative alt.
- Sample the first city URL per family so uniqueness is measured inside a small budget.
- `--ci` / `--baseline PATH` compare error fingerprints, not a fake score.

## 0.0.4 - 2026-08-25

- Parallel crawl workers (`--workers N`, default 5) without mixing landing and sitemap lanes.
- HTML report via `--html PATH`.
- Live quality axes: H1, Open Graph, accessibility, security headers, and fetch size/time.
- Programmatic uniqueness: city variants that only swap the city token (`WVX-SEO-PROG-002`).

## 0.0.3 - 2026-08-24

- Detect cross-market entity contamination (`WVX-SEO-MARKET-001`) on crawled pages and in Washington source packs.
- Detect public license claims contradicted by `license_verified=false` (`WVX-SEO-CLAIM-001`).
- Crawl linked landings before sitemap loc floods so category pages are measured inside a small budget.

## 0.0.2 - 2026-08-24

- Expand sitemap indexes into nested urlsets instead of treating index loc values as pages.
- Predict Next.js App Router route families, sitemap/robots/middleware owners, and metadata/`generateStaticParams` from the repository.
- Repo-only and hybrid audits: source-only / response-only classification plus programmatic family findings.
- Compare mode crawls public competitor origins and reports structural archetype, schema, and locale gaps without copying prose.
- Treat unprefixed default-locale URLs as matching `/:locale` App Router families.

## 0.0.1 - 2026-08-24

- Initial public workspace for Weavatrix SEO.
- Site-only inventory, audit, explain, and opportunity pass over a bounded first-party crawler.
- Deterministic Search Evidence Graph with explicit evidence kinds and stable finding fingerprints.
- CLI (`weavatrix-seo`) and MCP (`weavatrix-seo mcp` / `weavatrix-seo-mcp`) surfaces.
- Repo, hybrid, render, claim, programmatic, observation, and compare contracts are present and return `unmeasured` until those layers are wired.

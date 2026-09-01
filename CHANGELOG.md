# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

## 0.6.0 - 2026-09-01

Additive search intelligence. Nothing existing is removed.

- Evidence semantics identity: `POLICY_VERSION` tracks the crate version, and
  every inventory carries `EvidenceSemantics` (`engine_version`,
  `artifact_schema_version`, `rule_semantics_digest`, `policy_pack_digest`) so
  two snapshots with different finding meaning cannot look comparable.
- `RuleAuthority` on every finding. Severity still gates CI; authority tells an
  agent whether the rule is a protocol MUST, a documented search-engine SHOULD,
  a project contract, a jurisdiction requirement, or an experiment.
- Unknown confidence/risk is a ranking bucket, not 100/0. Trusted+measured,
  trusted+partial, unknown, then known-untrusted.
- Unique tempdirs for tests so parallel CI jobs do not clobber fixtures.
- Content intelligence beside exact duplicates: MinHash/LSH near-duplicates
  (`WVX-SEO-DUP-002`), per-page `ContentProfile` (MATTR, entropy, fact density,
  water/readability diagnostics, synthetic-style bands with authorship always
  `UNMEASURED`), family template decomposition, heading-bounded chunks, and
  intent fanout coverage.
- Programmatic `PageMatrix` v2: unique samples are necessary, not sufficient.
  `SAFE_TO_GENERATE` now requires fact coverage and distinctness; otherwise
  `SAFE_IF_REQUIREMENTS_MET` with `unmet_requirements`.
- Outcome metrics (`citation_rate`, `prompts_observed`, `search_clicks`) sit
  beside findings. Missing providers stay unmeasured, never zero.
- Opportunity axes gain raw impressions/clicks, recoverable clicks, and
  difficulty-to-build. `seo_plan` fills real dependencies.
- Chunk nodes bind `URL ─CONTAINS→ Chunk` on the evidence graph.
- MCP/CLI: `seo_query` (bounded DSL), `seo_retrieve`, `seo_similar`,
  `seo_chunks`. Existing eleven tools remain. Rust computes similarity.

## 0.5.0 - 2026-08-28

The domain layer of the graph. `SearchNodeKind` has declared `Claim`,
`DataField`, `Entity`, `Market`, and `Policy` since 0.0.8, and the builder never
created one. The detectors already established these facts to raise findings, so
the graph was narrower than its own type system.

- Public claims, the fields they require, and the source that defines them are
  bound as nodes and edges:

  ```text
  URL ─CLAIMS→ Claim ─REQUIRES→ DataField ─DEFINED_AT→ source span
                 └─GOVERNED_BY→ Policy
  URL ─ABOUT→ Entity / Market
  ```

- Two relations are new: `GOVERNED_BY` and `DEFINED_AT`. An entity keeps its own
  jurisdiction even when a page of another market names it, which is what makes
  contamination legible in the graph and not only in a finding.
- `seo_explain` walks that chain. Explaining a claim contradiction now names the
  claim, the field it needs, the policy that requires it, and the file and line
  that set the field false.
- `EvidenceSource::Policy` separates a shipped pack rule from repository
  evidence. The pack says a claim requires a fact; whether the analysed project
  satisfies it is a different measurement with a different source.
- The domain graph and the integrity findings come from one repository scan
  (`audit_with_graph`), so an explanation cannot be built from a different read
  of the source than the finding it explains.
- Market classification stays `INFERRED`: it is a heuristic over host, path,
  language, and copy.

## 0.4.0 - 2026-08-28

A filesystem boundary for the agent surface, plus the first exact graph
identities.

- The MCP confines every caller-supplied path. `repo`, `gsc`, `observations`,
  `render`, `history`, `baseline`, and both `seo_diff` sides are canonicalised
  and checked against an allow-list, so `..` segments and symlinks cannot point
  outside it. `--allow-root PATH` is repeatable; with none declared the boundary
  is the working directory, which is where a plugin launcher starts the server.
  "No shell" was never "no filesystem capability".
- JSON-LD nodes keep their own identity. A block is parsed into `JsonLdNode`
  values carrying `@id`, `@type`, and `sameAs` together, and the flat `types` /
  `ids` / `same_as` fields derive from them. `Organization #org` beside
  `WebSite #site` used to leave two ids and two types with no mapping, and the
  graph labelled every id with the first type in the document.
- `generateStaticParams` and JSON-LD producers reach the graph. Both were
  extracted and used for impact, but neither was ever bound as a node or edge.
- A UTF-8 byte-order mark no longer breaks an import. Windows tooling writes one
  by default, and every JSON loader — observations, render, baseline, history,
  search contract — rejected such a file with `expected \`{\` at line 1 column 1`.
- The README no longer shows a crates.io dependency. Every crate here is
  `publish = false`, so the example could not work; the git and `--path` routes
  are the real ones.

## 0.3.0 - 2026-08-28

Typed observations and honest ranking. An observation now says what it measured,
and nothing else is allowed to stand in for search demand.

- Observations carry an `ObservationKind`: `search_performance`, `bot_crawl`,
  `ai_citation`, `serp_position`, or `analytics`. A row can declare `kind`
  explicitly; otherwise a known provider name implies it, and an unrecognised
  provider stays `analytics` rather than being guessed into demand.
- Bot hits are no longer folded into impressions. `hits` and `impressions` are
  separate fields, and only `search_performance` rows feed the demand and
  visibility axes. 500 crawler requests used to become 50 points of demand and
  promote a page up the opportunity list.
- `WVX-SEO-OBS-001` fires only on measured search demand. A bot hit on an
  uncrawled URL is a crawl-budget fact, not a search-coverage gap.
- Average position is `Option<f32>`. Search Console reports fractions, and 12.4
  was previously truncated to 12 on import. The visibility gap rounds instead of
  truncating.
- `ai_search` is split. `ai_retrieval_readiness` keeps the schema, content, and
  architecture findings that can be inferred from a crawl. The new
  `ai_visibility` axis stays `unmeasured` until a generative-search citation is
  actually imported, because readiness is not visibility.
- `PageMatrix.cardinality` is now `measured_urls`. It always held the number of
  URLs this crawl measured, never the size of the generated matrix. Estimating
  real cardinality needs the route generators, which the compiler still does not
  read; `SAFE_TO_GENERATE` remains unchanged and still deserves stricter gates.
- Opportunity ranking uses the axes it declares. Ordering is lexicographic over
  trust, measured demand, visibility gap, business value, conversion potential,
  graph leverage, and topical fit, with implementation cost only as a
  tie-breaker — not one opaque score. A low-confidence or high-risk item sinks
  below everything trustworthy instead of being dropped. An axis nobody scored
  is not treated as a low score.

## 0.2.0 - 2026-08-28

Evidence semantics. Every change here removes a claim the code could not support.
Output shape changes, so this is a minor bump.

- A live response no longer carries the worktree revision. Crawled pages are not
  stamped with a repository SHA, and the graph edge is
  `url --COMPARED_AGAINST--> revision` instead of `CHANGED_BY`. `CHANGED_BY`
  stays reserved for a deployment or build proof that the engine cannot produce
  yet. `seo_explain` walks both.
- Source facts carry repository provenance. Route families and source symbols
  used to inherit the evidence of the first crawled page, so a
  `generateMetadata` symbol reported `source: http`. They now use
  `Evidence::repo()` with the worktree revision. A URL matching a predicted
  route pattern is a cross-layer inference and is labelled `INFERRED`.
- Findings are stamped. Every finding now carries `snapshot_id` and
  `policy_version`; `revision` is attached only when a repository parser
  established the fact.
- An explicit `.weavatrix/seo.*` contract wins over the built-in private-path
  heuristic. A project that declares `/profile/:username` indexable is no longer
  overruled by a guess. The heuristic still applies when nothing is declared.
- A present-but-unreadable contract is `WVX-SEO-IDX-001` (error) instead of
  silently reading as "no contract". This can fail a `--ci` run that used to
  pass on a typo.
- `frame-ancestors` delivered through `meta http-equiv` no longer satisfies the
  X-Frame-Options check, per CSP Level 3. Meta CSP is kept in `csp_meta` rather
  than merged into response headers, and still counts as the origin having a
  policy.
- One `EvidenceScope` owns comparability for the gate and the history diff.
  `seo_diff` now checks mode, as its contract always claimed, and reports
  `config_changed`; an origin-level finding no longer resolves when the crawl
  budget shrank.
- `seo_plan` reports the real `PageMatrix` verdict. The field was previously a
  substring guess that always answered `REVIEW`.

## 0.1.13 - 2026-08-27

- MCP parity: crawl-backed tools accept `gsc`, `observations`, `render`, `history`, and `workers`, so demand and visibility axes rank the same way they do on the CLI.
- `seo_observations` reads a provider export instead of always answering `unmeasured`. It filters by `provider` and caps returned rows.
- `seo_gate` is the `--ci` / `--baseline` evidence gate as a tool. It returns new errors, resolved fingerprints, coverage regressions, and comparability.
- `seo_links` returns directed internal-link recommendations. `seo_vectors` returns the `wvx-seo-lexhash-v1` page vectors and SEO link profiles behind them, so a link pass needs no embedding service. Lexical, so no synonym or cross-language matching. Node identities are `page:<url>`.
- `scope` is gone from the MCP schema. It was advertised and never read.
- Vectors and profiles now come from one producer (`link_inputs`), so the analysis pass and the exported payload cannot drift.

## 0.1.12 - 2026-08-25

- `/:locale` is optional in search-policy globs, so default-locale URLs (`/category/:slug/:city`) match city families. Live kablay.us city landings can be `LOCAL-001` / `ENTITY-002`.

## 0.1.11 - 2026-08-25

- Shared chrome entities (nav/footer city names on most pages) collapse to one origin `WVX-SEO-ENTITY-001`. Page-unique undeclared entities stay per URL.

## 0.1.10 - 2026-08-25

- HSTS/CSP origin facts: `max-age=0` or missing max-age is `WVX-SEO-SEC-006`. Mixed HTML responses are `SEC-007`.
- CSP `frame-ancestors` satisfies clickjacking, so missing X-Frame-Options is quiet. Meta `http-equiv` CSP counts. Report-Only is not enforcing.

## 0.1.9 - 2026-08-25

- Impact cone follows relative imports of SEO helpers (`weavatrix-parse`). Editing `cities.ts` imported by `citySeo.ts` names the `:city` family even though the helper file did not change.
- Next.js page/layout modules are not walked, so the cone does not hash the whole UI tree.

## 0.1.8 - 2026-08-25

- AI-search citation: Organization/WebSite without `@id`/`sameAs` is `WVX-SEO-AI-001`. FAQ copy without FAQPage is `AI-002`. FAQ source producers without schema are `AI-003`.
- JSON-LD extraction keeps Organization/WebSite `@id` and `sameAs`; the evidence graph binds those ids.

## 0.1.7 - 2026-08-25

- Local graph: city URLs that declare Place/Service JSON-LD without `areaServed` or an address are `WVX-SEO-LOCAL-001`. Missing types stay `ENTITY-002`.

## 0.1.6 - 2026-08-25

- Entity graph: pack entities named in visible copy but missing from JSON-LD are `WVX-SEO-ENTITY-001`.
- `:city` families without a Place/Service type (or JSON-LD producer) are `WVX-SEO-ENTITY-002`. Foreign entities stay `MARKET-001`.

## 0.1.5 - 2026-08-25

- Repository search contract: `.weavatrix/seo.json` or a tiny `.weavatrix/seo.yaml` subset. `include`/`exclude` globs own CREATE and SOURCE_ONLY. Missing file keeps the private-family default.
- Policy `international.x_default` emits `WVX-SEO-I18N-003` when locale twins have no `x-default` alternate.

## 0.1.4 - 2026-08-25

- City path that 301s onto `?city=` is `WVX-SEO-CANN-003` (live Profi `/ru/cities/yavne` → `/ru/specialists?city=yavne`).
- Same-city different-service URLs are not cannibalization (`/category/plumber/x` vs `/category/electrician/x`).
- Markdown docs are not market-pack evidence.
- Live site/hybrid on kablay.us and kablay.co.il: CANN-003 and inverse MARKET confirmed; I18N-002 stays quiet when live pages already emit hreflang.

## 0.1.3 - 2026-08-25

- Market packs are bidirectional: Israel files/pages that name Vancouver WA / Clark County are `MARKET-001`, not only the US→Israel direction.
- Locale twins (`/` and `/ru`) without hreflang are `WVX-SEO-I18N-002`. SiteOne does not score this.
- Query city URLs (`?city=`) that sit beside a pretty city path are `WVX-SEO-CANN-002`.
- Attribute values keep `?query` across tokenizer splits, so faceted URLs are crawled as their own identity.
- `seo_plan` no longer CREATE-proposes admin/auth/dashboard/catch-all families.

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

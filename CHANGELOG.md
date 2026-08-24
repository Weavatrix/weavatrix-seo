# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

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

---
name: weavatrix-seo
description: >-
  Use Weavatrix SEO for search-surface evidence: crawl, GSC, logs, AI
  citations, bounded seo_query, retrieve, and a read-only plan. Skip it for
  source edits (use Weavatrix Refactor) and for repository graphs (use
  Weavatrix core).
---

# Weavatrix SEO

Read-only Search Intelligence. Prefer `seo_query` and `seo_retrieve` over raw
vectors. Missing evidence is `unmeasured`, never a pass.

## Minimal workflow

1. `seo_audit` or `seo_inventory` with `site` and optional `repo`.
2. `seo_query` for orphans, CTR gaps, or history (`history` without a crawl).
3. `seo_explain` a fingerprint when the agent needs the source chain.
4. `seo_plan` for the DAG + Refactor handoff. Do not apply the handoff here.

Import GSC, logs, or AI-visibility JSON via `observations` / `gsc`. Combined
nginx lines need `origin` and `format: combined`. Prompt files use `prompts[]`.

## Safety

- SEO never writes pages or source.
- Bot hits are not search demand.
- If `seo_plan.handoff.repo_revision` or `symbol_hash` drifted, refuse the
  Refactor apply until SEO re-runs.

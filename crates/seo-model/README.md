# weavatrix-seo-model

[![crates.io](https://img.shields.io/crates/v/weavatrix-seo-model.svg)](https://crates.io/crates/weavatrix-seo-model)
[![docs.rs](https://docs.rs/weavatrix-seo-model/badge.svg)](https://docs.rs/weavatrix-seo-model)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/Weavatrix/weavatrix-seo/blob/main/LICENSE)

Search Evidence Graph **types** for [Weavatrix SEO](https://github.com/Weavatrix/weavatrix-seo).

This crate owns identities, evidence, findings, extracted pages, inventories, rule registry, schema feature profiles, and AI crawler roles. It does **not** crawl, parse HTML, fetch HTTP, or rank opportunities.

## Why it exists

Every other `weavatrix-seo-*` crate speaks this language. If a finding says `required`, the registry and `SchemaFeatureProfile` say *who required it* (Google rich-result vs schema.org vocabulary). If two snapshots look comparable, `EvidenceSemantics` says whether the rule meaning actually matches.

## What you get

- `AbsoluteUrl`, `Inventory`, `ExtractedPage`, `Finding`, `AuditReport`
- `RuleDefinition` / `rules()` / `rule_authority()` — per-code catalogue
- `SchemaFeatureProfile` — `required` trees (`Path`, `AnyOf`, `All`)
- `AiAgentDefinition` / `AiAgentRole` — training vs search discovery vs citation
- `EvidenceSemantics`, `EvidenceScope`, `RuleAuthority`, `Severity`
- Graph nodes and relations (`CanonicalTo`, `AlternateOf`, …)

```rust
use weavatrix_seo_model::{FindingFamily, rule_authority, rules};

let meta_title = rules().iter().find(|r| r.family == FindingFamily::Meta && r.number == 1);
assert!(meta_title.is_some());
let _ = rule_authority(FindingFamily::Ai, 4); // experimental llms.txt
```

```toml
[dependencies]
weavatrix-seo-model = "0.6.2"
```

Product CLI/MCP: [`weavatrix-seo`](https://crates.io/crates/weavatrix-seo). MIT.

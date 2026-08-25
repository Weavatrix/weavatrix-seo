//! Hash source producers so a helper edit can name affected families.

use crate::{RouteFamily, SourceSurface, SourceSymbol};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use weavatrix_seo_model::{ContentHash, ProducerFact};

impl SourceSurface {
    /// File hashes for page/metadata/helper/sitemap producers.
    #[must_use]
    pub fn producer_facts(&self, repo: &str) -> Vec<ProducerFact> {
        let mut grouped: BTreeMap<(String, String), BTreeSet<String>> = BTreeMap::new();
        for family in &self.families {
            for symbol in family.symbols() {
                grouped
                    .entry((symbol.path.clone(), symbol.name.clone()))
                    .or_default()
                    .insert(family.pattern.clone());
            }
            if let Some(owner) = &family.owner {
                grouped
                    .entry((owner.clone(), "module".into()))
                    .or_default()
                    .insert(family.pattern.clone());
            }
        }
        for path in self.sitemaps.iter().chain(self.robots.iter()) {
            grouped
                .entry((path.clone(), "module".into()))
                .or_default()
                .insert("sitemap".into());
        }
        if let Some(path) = &self.middleware {
            grouped
                .entry((path.clone(), "middleware".into()))
                .or_default()
                .insert("middleware".into());
        }
        if let Some(config) = &self.next_config
            && let Some(path) = &config.path
        {
            grouped
                .entry((path.clone(), "next.config".into()))
                .or_default()
                .insert("config".into());
        }
        grouped
            .into_iter()
            .map(|((path, name), families)| ProducerFact {
                content_hash: file_hash(repo, &path),
                path,
                name,
                families: families.into_iter().collect(),
            })
            .collect()
    }
}

impl RouteFamily {
    fn symbols(&self) -> Vec<&SourceSymbol> {
        let mut out = Vec::new();
        out.extend(self.page_symbol.as_ref());
        out.extend(self.metadata_symbol.as_ref());
        out.extend(self.static_params_symbol.as_ref());
        out.extend(self.json_ld_symbols.iter());
        out.extend(self.helpers.iter());
        out
    }
}

fn file_hash(repo: &str, relative: &str) -> ContentHash {
    let relative = relative.replace('\\', "/");
    let root = Path::new(repo);
    let mut candidates = vec![root.join(&relative)];
    let named = relative.rsplit('/').next().unwrap_or(&relative);
    if !named.contains('.') {
        for ext in [".ts", ".tsx", ".js", ".jsx"] {
            candidates.push(root.join(format!("{relative}{ext}")));
        }
        candidates.push(root.join(&relative).join("index.ts"));
        candidates.push(root.join(&relative).join("index.tsx"));
    }
    for path in candidates {
        if let Ok(bytes) = std::fs::read(path) {
            return ContentHash::of(&bytes);
        }
    }
    ContentHash::of_str(&relative)
}

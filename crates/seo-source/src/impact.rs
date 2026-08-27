//! Hash source producers so a helper edit can name affected families.

use crate::{RouteFamily, SourceSurface, SourceSymbol};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use weavatrix_parse::{Language, extract};
use weavatrix_seo_model::{ContentHash, ProducerFact};

const MAX_CONE: usize = 64;
const MAX_DEPTH: u8 = 4;

impl SourceSurface {
    /// File hashes for page/metadata/helper/sitemap producers and their import cone.
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
        expand_imports(repo, &mut grouped);
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

fn expand_imports(repo: &str, grouped: &mut BTreeMap<(String, String), BTreeSet<String>>) {
    let mut queue: Vec<(String, BTreeSet<String>, u8)> = grouped
        .iter()
        .map(|((path, _), families)| (path.clone(), families.clone(), 0))
        .collect();
    let mut seen: BTreeSet<String> = grouped.keys().map(|(path, _)| stem(path)).collect();
    let mut index = 0;
    while index < queue.len() && grouped.len() < MAX_CONE {
        let (path, families, depth) = queue[index].clone();
        index += 1;
        if depth >= MAX_DEPTH || is_route_module(&path) {
            continue;
        }
        let Some((file, source)) = read_source(repo, &path) else {
            continue;
        };
        let facts = extract(&source, language(&file));
        for import in facts.imports {
            if import.type_only || !is_relative(&import.specifier) {
                continue;
            }
            let resolved = join_relative(&file, &import.specifier);
            let Some((imported, _)) = read_source(repo, &resolved) else {
                continue;
            };
            if !seen.insert(stem(&imported)) {
                merge_families(grouped, &imported, &families);
                continue;
            }
            grouped
                .entry((imported.clone(), "import".into()))
                .or_default()
                .extend(families.iter().cloned());
            queue.push((imported, families.clone(), depth + 1));
        }
    }
}

fn merge_families(
    grouped: &mut BTreeMap<(String, String), BTreeSet<String>>,
    path: &str,
    families: &BTreeSet<String>,
) {
    if let Some((_, set)) = grouped
        .iter_mut()
        .find(|((existing, _), _)| stem(existing) == stem(path))
    {
        set.extend(families.iter().cloned());
    }
}

fn is_relative(specifier: &str) -> bool {
    specifier.starts_with("./") || specifier.starts_with("../")
}

fn is_route_module(path: &str) -> bool {
    let name = path.replace('\\', "/");
    let name = name.rsplit('/').next().unwrap_or(&name);
    let stem = stem(name);
    matches!(
        stem.as_str(),
        "page"
            | "layout"
            | "route"
            | "template"
            | "default"
            | "loading"
            | "error"
            | "not-found"
            | "sitemap"
            | "robots"
            | "middleware"
            | "next.config"
    ) || name.starts_with("next.config.")
}

fn language(path: &str) -> Language {
    match path.rsplit('.').next() {
        Some("js" | "jsx" | "mjs" | "cjs") => Language::JavaScript,
        _ => Language::TypeScript,
    }
}

fn stem(path: &str) -> String {
    let path = path.replace('\\', "/");
    for ext in [".tsx", ".ts", ".jsx", ".js", ".mjs", ".cjs"] {
        if let Some(stem) = path.strip_suffix(ext) {
            return stem.to_owned();
        }
    }
    path
}

fn join_relative(from_file: &str, specifier: &str) -> String {
    let parent = from_file.replace('\\', "/");
    let parent = parent.rsplit_once('/').map_or("", |(head, _)| head);
    let mut parts: Vec<&str> = parent.split('/').filter(|part| !part.is_empty()).collect();
    let specifier = specifier.replace('\\', "/");
    for segment in specifier.split('/') {
        match segment {
            "." | "" => {}
            ".." => {
                let _ = parts.pop();
            }
            other => parts.push(other),
        }
    }
    parts.join("/")
}

fn read_source(repo: &str, relative: &str) -> Option<(String, String)> {
    let relative = relative.replace('\\', "/");
    let root = Path::new(repo);
    let mut candidates = vec![relative.clone()];
    let named = relative.rsplit('/').next().unwrap_or(&relative);
    if !named.contains('.') {
        for ext in [".ts", ".tsx", ".js", ".jsx"] {
            candidates.push(format!("{relative}{ext}"));
        }
        candidates.push(format!("{relative}/index.ts"));
        candidates.push(format!("{relative}/index.tsx"));
    }
    for candidate in candidates {
        let Ok(bytes) = std::fs::read(root.join(&candidate)) else {
            continue;
        };
        let source = String::from_utf8_lossy(&bytes).into_owned();
        return Some((candidate, source));
    }
    None
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

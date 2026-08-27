//! Deterministic link inputs: page vectors plus SEO link profiles.
//!
//! These rows are the whole input a directed internal-link pass needs. They are
//! produced from the measured inventory, so no embedding model is required.

use crate::embed::{self, DIM, MODEL};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use weavatrix_seo_architecture::Architecture;
use weavatrix_seo_model::{Indexability, Inventory, Relation};

/// One page vector. `node` is `page:<url>`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VectorRow {
    /// Graph node identity.
    pub node: String,
    /// Embedding components.
    pub values: Vec<f32>,
}

/// One SEO link profile for the node of the same name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct PageRow {
    /// Graph node identity.
    pub node: String,
    /// Host. Cross-site links are refused on this value.
    pub site: String,
    /// Canonical content identity.
    pub canonical: String,
    /// Content language when the page declared one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// Whether the page may be a recommendation source.
    pub source_eligible: bool,
    /// Whether the page may be a recommendation target.
    pub target_eligible: bool,
    /// Authority or root page.
    pub cornerstone: bool,
    /// No measured internal inbound link.
    pub orphan: bool,
    /// Caller priority. Unused by this producer.
    pub target_priority: u32,
    /// Targets this page already links to.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub existing_targets: Vec<String>,
}

/// Vectors and profiles for one measured inventory.
///
/// `vectors` and `pages` cover the same node set: indexable `200` HTML pages
/// with non-empty visible text.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinkInputs {
    /// Embedding model identity.
    pub model: String,
    /// Embedding width.
    pub dimension: usize,
    /// Page vectors.
    pub vectors: Vec<VectorRow>,
    /// Page profiles.
    pub pages: Vec<PageRow>,
}

/// Builds vectors and profiles from measured evidence.
#[must_use]
pub fn link_inputs(inventory: &Inventory, architecture: &Architecture) -> LinkInputs {
    let authority: BTreeMap<String, f64> = architecture
        .pages
        .iter()
        .map(|page| (page.url.to_string(), page.authority))
        .collect();
    let mut outgoing: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for edge in inventory
        .edges
        .iter()
        .filter(|edge| edge.relation == Relation::LinksTo)
    {
        outgoing
            .entry(edge.source.to_string())
            .or_default()
            .insert(edge.target.to_string());
    }
    let mut vectors = Vec::new();
    let mut pages = Vec::new();
    for page in &inventory.pages {
        if page.status != 200 || page.indexability != Indexability::Indexable {
            continue;
        }
        let Some(values) = embed::embed(&page.visible_text()) else {
            continue;
        };
        let url = page.url.to_string();
        let node = format!("page:{url}");
        let cornerstone =
            authority.get(&url).is_some_and(|score| *score > 0.15) || url.ends_with('/');
        vectors.push(VectorRow {
            node: node.clone(),
            values,
        });
        pages.push(PageRow {
            node,
            site: page.url.host().to_owned(),
            canonical: page.canonical.clone().unwrap_or_else(|| url.clone()),
            language: page.html_lang.clone(),
            source_eligible: true,
            target_eligible: true,
            cornerstone,
            orphan: !page.linked_from_page && page.url.path() != "/",
            target_priority: 0,
            existing_targets: outgoing
                .get(&url)
                .map(|targets| targets.iter().map(|to| format!("page:{to}")).collect())
                .unwrap_or_default(),
        });
    }
    LinkInputs {
        model: MODEL.to_owned(),
        dimension: DIM,
        vectors,
        pages,
    }
}

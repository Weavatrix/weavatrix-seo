//! Heterogeneous Search Evidence Graph nodes and fact edges.

use crate::{Evidence, Locator, Relation};
use serde::{Deserialize, Serialize};

/// Kind of a search-graph node. Distinct from crawl URL identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchNodeKind {
    /// HTTP URL.
    Url,
    /// App Router / route family.
    RouteFamily,
    /// Source function, component, or export.
    SourceSymbol,
    /// Domain data field.
    DataField,
    /// JSON-LD or schema object.
    SchemaObject,
    /// Public claim.
    Claim,
    /// Named entity.
    Entity,
    /// Topic cluster placeholder.
    Topic,
    /// Market / jurisdiction pack.
    Market,
    /// Legal or credential requirement.
    LegalRequirement,
    /// Imported search observation.
    SearchObservation,
    /// Git revision.
    Revision,
    /// Policy pack.
    Policy,
}

/// One node in the Search Evidence Graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchNode {
    /// Stable id, for example `url:https://x.test/` or `symbol:app/page.tsx#Page`.
    pub id: String,
    /// Node kind.
    pub kind: SearchNodeKind,
    /// Human label.
    pub label: String,
    /// Exact locator when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locator: Option<Locator>,
}

impl SearchNode {
    /// Builds a node.
    #[must_use]
    pub fn new(kind: SearchNodeKind, id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            kind,
            label: label.into(),
            locator: None,
        }
    }

    /// Attaches a locator.
    #[must_use]
    pub fn at(mut self, locator: Locator) -> Self {
        self.locator = Some(locator);
        self
    }
}

/// Heterogeneous fact edge. URL-to-URL crawl links stay on [`crate::GraphEdge`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactEdge {
    /// Source node id.
    pub source: String,
    /// Target node id.
    pub target: String,
    /// Source kind.
    pub source_kind: SearchNodeKind,
    /// Target kind.
    pub target_kind: SearchNodeKind,
    /// Relation.
    pub relation: Relation,
    /// Evidence.
    pub evidence: Evidence,
    /// Locator when the fact has a precise span.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locator: Option<Locator>,
}

impl FactEdge {
    /// Builds a fact edge.
    #[must_use]
    pub fn new(
        source: impl Into<String>,
        source_kind: SearchNodeKind,
        target: impl Into<String>,
        target_kind: SearchNodeKind,
        relation: Relation,
        evidence: Evidence,
    ) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
            source_kind,
            target_kind,
            relation,
            evidence,
            locator: None,
        }
    }

    /// Attaches a locator.
    #[must_use]
    pub fn at(mut self, locator: Locator) -> Self {
        self.locator = Some(locator);
        self
    }
}

/// URL node id.
#[must_use]
pub fn url_id(url: &str) -> String {
    format!("url:{url}")
}

/// Route-family node id.
#[must_use]
pub fn route_id(pattern: &str) -> String {
    format!("route:{pattern}")
}

/// Source-symbol node id.
#[must_use]
pub fn symbol_id(path: &str, name: &str) -> String {
    format!("symbol:{path}#{name}")
}

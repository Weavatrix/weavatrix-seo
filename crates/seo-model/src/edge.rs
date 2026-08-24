//! Typed Search Evidence Graph edges.

use crate::{AbsoluteUrl, Evidence};
use serde::{Deserialize, Serialize};

/// Relation between two search-surface nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Relation {
    /// Internal or extracted hyperlink.
    LinksTo,
    /// Canonical target.
    CanonicalTo,
    /// Hreflang alternate.
    AlternateOf,
    /// Redirect.
    RedirectsTo,
    /// Listed in a sitemap.
    ListedInSitemap,
    /// Blocked by robots.
    BlockedBy,
}

/// One recorded graph edge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Source URL.
    pub source: AbsoluteUrl,
    /// Target URL.
    pub target: AbsoluteUrl,
    /// Relation.
    pub relation: Relation,
    /// Evidence for the relation.
    pub evidence: Evidence,
    /// Anchor text when the relation is a hyperlink.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor: Option<String>,
}

impl GraphEdge {
    /// Builds a relation edge.
    #[must_use]
    pub fn new(
        source: AbsoluteUrl,
        target: AbsoluteUrl,
        relation: Relation,
        evidence: Evidence,
    ) -> Self {
        Self {
            source,
            target,
            relation,
            evidence,
            anchor: None,
        }
    }
}

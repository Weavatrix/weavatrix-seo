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
    /// `rel` tokens joined by space.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rel: Option<String>,
    /// Document location of the link.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<crate::LinkLocation>,
    /// Surrounding context.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
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
            rel: None,
            location: None,
            context: None,
        }
    }

    /// Attaches hyperlink semantics.
    #[must_use]
    pub fn with_link(
        mut self,
        anchor: Option<String>,
        rel: Option<String>,
        location: Option<crate::LinkLocation>,
        context: Option<String>,
    ) -> Self {
        self.anchor = anchor;
        self.rel = rel;
        self.location = location;
        self.context = context;
        self
    }
}

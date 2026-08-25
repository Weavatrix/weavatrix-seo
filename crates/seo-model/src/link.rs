//! First-class hyperlink evidence.

use serde::{Deserialize, Serialize};

/// Where a link sits in the document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkLocation {
    /// Primary navigation.
    Nav,
    /// Footer.
    Footer,
    /// Header chrome other than nav.
    Header,
    /// Breadcrumb trail.
    Breadcrumb,
    /// In-content / contextual.
    Contextual,
}

impl LinkLocation {
    /// Infers placement from the open-element stack.
    #[must_use]
    pub fn from_stack(stack: &[String]) -> Self {
        for name in stack.iter().rev() {
            match name.as_str() {
                "nav" => return Self::Nav,
                "footer" => return Self::Footer,
                "header" => return Self::Header,
                _ => {}
            }
        }
        Self::Contextual
    }
}

/// One extracted `a[href]` with semantics kept for the graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkRef {
    /// Raw href.
    pub href: String,
    /// Visible anchor text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<String>,
    /// Nearby context when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    /// `rel` tokens.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rel: Vec<String>,
    /// Document location.
    pub location: LinkLocation,
}

impl LinkRef {
    /// Builds a contextual link from an href only.
    #[must_use]
    pub fn href(href: impl Into<String>) -> Self {
        Self {
            href: href.into(),
            anchor: None,
            context: None,
            rel: Vec::new(),
            location: LinkLocation::Contextual,
        }
    }
}

//! Source producers bound to route families.

use crate::ContentHash;
use serde::{Deserialize, Serialize};

/// One source file/symbol that can change a search family.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProducerFact {
    /// Repository-relative path or import specifier.
    pub path: String,
    /// Symbol or module name.
    pub name: String,
    /// File bytes when the path could be read.
    pub content_hash: ContentHash,
    /// Route families that import or declare this producer.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub families: Vec<String>,
}

impl ProducerFact {
    /// `path#name` identity.
    #[must_use]
    pub fn key(&self) -> String {
        format!("{}#{}", self.path, self.name)
    }
}

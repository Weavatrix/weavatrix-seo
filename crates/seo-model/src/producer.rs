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
    /// Hash of the symbol extent when the span was known. A change in an
    /// unrelated function in the same file does not flip this.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol_hash: Option<ContentHash>,
    /// Start line of the symbol extent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_line: Option<u32>,
    /// End line of the symbol extent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_line: Option<u32>,
}

impl ProducerFact {
    /// `path#name` identity.
    #[must_use]
    pub fn key(&self) -> String {
        format!("{}#{}", self.path, self.name)
    }
}

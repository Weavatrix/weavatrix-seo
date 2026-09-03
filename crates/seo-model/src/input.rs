//! User-supplied evidence files are never silent absence.

use serde::{Deserialize, Serialize};

/// How an imported evidence file was resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputStateKind {
    /// No file was supplied.
    Absent,
    /// File parsed and contained rows.
    Connected,
    /// File parsed and contained zero rows.
    Empty,
    /// File existed but could not be parsed.
    Invalid,
    /// File parsed but is older than the allowed window.
    Stale,
    /// File parsed with recoverable warnings.
    Partial,
}

impl Default for InputStateKind {
    fn default() -> Self {
        Self::Absent
    }
}

/// Typed load result for GSC, logs, render snapshots, packs, and baselines.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputState {
    /// Discriminator.
    pub kind: InputStateKind,
    /// `NO_GSC_SUPPLIED`, `GSC_INVALID`, …
    pub label: String,
    /// Parse or freshness error. Never a finding summary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl InputState {
    /// No file was passed.
    #[must_use]
    pub fn absent(prefix: &str) -> Self {
        Self {
            kind: InputStateKind::Absent,
            label: format!("NO_{prefix}_SUPPLIED"),
            error: None,
        }
    }

    /// File parsed with rows.
    #[must_use]
    pub fn connected(prefix: &str) -> Self {
        Self {
            kind: InputStateKind::Connected,
            label: format!("{prefix}_CONNECTED"),
            error: None,
        }
    }

    /// File parsed with zero rows.
    #[must_use]
    pub fn empty(prefix: &str) -> Self {
        Self {
            kind: InputStateKind::Empty,
            label: format!("{prefix}_EMPTY"),
            error: None,
        }
    }

    /// File could not be used as evidence.
    #[must_use]
    pub fn invalid(prefix: &str, error: impl Into<String>) -> Self {
        Self {
            kind: InputStateKind::Invalid,
            label: format!("{prefix}_INVALID"),
            error: Some(error.into()),
        }
    }

    /// Whether rows in this state may influence findings.
    #[must_use]
    pub const fn usable(&self) -> bool {
        matches!(
            self.kind,
            InputStateKind::Connected | InputStateKind::Empty | InputStateKind::Partial
        )
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::absent("GSC")
    }
}

#[cfg(test)]
mod tests {
    use super::{InputState, InputStateKind};

    #[test]
    fn invalid_is_not_absence() {
        let state = InputState::invalid("GSC", "expected value");
        assert_eq!(state.kind, InputStateKind::Invalid);
        assert_eq!(state.label, "GSC_INVALID");
        assert!(!state.usable());
        assert_ne!(state.label, InputState::absent("GSC").label);
    }
}

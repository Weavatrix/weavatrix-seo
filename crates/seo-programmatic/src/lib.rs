//! Programmatic page-matrix safety verdicts.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Safety verdict for one generated page family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SafetyVerdict {
    /// Unique value and evidence exist.
    SafeToGenerate,
    /// Generate only after listed requirements.
    SafeIfRequirementsMet,
    /// Merge into an existing URL.
    Consolidate,
    /// Keep for users, keep out of the index.
    NoindexByDefault,
    /// Do not generate.
    RejectLowValue,
    /// Human review required.
    Review,
    /// Not enough evidence to decide.
    Unmeasured,
}

/// One proposed family.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageMatrix {
    /// Family identity.
    pub family: String,
    /// Estimated cardinality when known.
    pub cardinality: Option<u64>,
    /// Verdict.
    pub verdict: SafetyVerdict,
}

/// Default compiler output before route generators are wired.
#[must_use]
pub fn unmeasured(family: impl Into<String>) -> PageMatrix {
    PageMatrix {
        family: family.into(),
        cardinality: None,
        verdict: SafetyVerdict::Unmeasured,
    }
}

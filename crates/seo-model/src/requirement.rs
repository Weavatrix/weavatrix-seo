//! Typed programmatic-matrix gates. String `unmet_requirements` stay beside them.

use serde::{Deserialize, Serialize};

/// One compiler gate. Names match the historical unmet-requirement strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequirementKind {
    /// At least two unique body samples in the family.
    SampleDiversity,
    /// Local-fact coverage from content intelligence.
    FactCoverage,
    /// Semantic distinctness, not only unique hashes.
    SemanticDistinctness,
    /// Canonical strategy for the family.
    CanonicalStrategy,
    /// Discovery beyond sitemap-only URLs.
    DiscoverySupport,
    /// Claim integrity against the owned graph.
    ClaimIntegrity,
    /// Measured search demand (never keyword-tool volume).
    DemandEvidence,
    /// Query cannibalization risk across the family.
    CannibalizationRisk,
    /// Internal-link support for the generated URLs.
    InternalLinkSupport,
}

impl RequirementKind {
    /// Every known gate, in stable report order.
    pub const ALL: [Self; 9] = [
        Self::SampleDiversity,
        Self::FactCoverage,
        Self::SemanticDistinctness,
        Self::CanonicalStrategy,
        Self::DiscoverySupport,
        Self::ClaimIntegrity,
        Self::DemandEvidence,
        Self::CannibalizationRisk,
        Self::InternalLinkSupport,
    ];

    /// Historical unmet-requirement label. Keep these strings stable.
    #[must_use]
    pub const fn unmet_label(self) -> &'static str {
        match self {
            Self::SampleDiversity => "sufficient sample diversity",
            Self::FactCoverage => "fact coverage",
            Self::SemanticDistinctness => "semantic distinctness",
            Self::CanonicalStrategy => "canonical strategy",
            Self::DiscoverySupport => "discovery support beyond sitemap",
            Self::ClaimIntegrity => "claim integrity",
            Self::DemandEvidence => "demand evidence",
            Self::CannibalizationRisk => "cannibalization risk",
            Self::InternalLinkSupport => "internal link support",
        }
    }

    /// Gates that must be [`RequirementState::Passed`] for `SAFE_TO_GENERATE`.
    #[must_use]
    pub const fn required_for_generate(self) -> bool {
        matches!(
            self,
            Self::SampleDiversity
                | Self::FactCoverage
                | Self::SemanticDistinctness
                | Self::CanonicalStrategy
        )
    }
}

/// Measurement of one gate. Unmeasured is not a pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RequirementState {
    /// The gate was measured and holds.
    Passed,
    /// The gate was measured and does not hold.
    Failed,
    /// The axis was not measured. Never treated as a pass.
    Unmeasured,
}

/// One typed requirement on a page matrix.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequirementResult {
    /// Which gate.
    pub kind: RequirementKind,
    /// Pass / fail / unmeasured.
    pub state: RequirementState,
    /// Numeric witness when the compiler had one (ratio, coverage, …).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<u16>,
    /// Short evidence note. Not a provenance graph.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
}

impl RequirementResult {
    /// Builds a result with no numeric witness.
    #[must_use]
    pub fn new(kind: RequirementKind, state: RequirementState) -> Self {
        Self {
            kind,
            state,
            value: None,
            evidence: None,
        }
    }

    /// Attaches a 0–100 witness.
    #[must_use]
    pub fn with_value(mut self, value: u16) -> Self {
        self.value = Some(value);
        self
    }

    /// Attaches a short evidence note.
    #[must_use]
    pub fn with_evidence(mut self, evidence: impl Into<String>) -> Self {
        self.evidence = Some(evidence.into());
        self
    }

    /// Whether this result blocks `SAFE_TO_GENERATE`.
    #[must_use]
    pub const fn blocks_generate(&self) -> bool {
        match self.state {
            RequirementState::Passed => false,
            RequirementState::Failed => true,
            RequirementState::Unmeasured => self.kind.required_for_generate(),
        }
    }
}

/// Human unmet-requirement strings derived from typed results.
///
/// Failed gates always appear. Unmeasured gates appear only when they are
/// required for generate. Passed gates never appear. This keeps the historical
/// `unmet_requirements` field beside the typed list.
#[must_use]
pub fn unmet_labels(requirements: &[RequirementResult]) -> Vec<String> {
    requirements
        .iter()
        .filter(|item| item.blocks_generate())
        .map(|item| item.kind.unmet_label().to_owned())
        .collect()
}

/// True when every required generate gate is present and passed.
#[must_use]
pub fn required_gates_passed(requirements: &[RequirementResult]) -> bool {
    RequirementKind::ALL
        .iter()
        .filter(|kind| kind.required_for_generate())
        .all(|kind| {
            requirements
                .iter()
                .any(|item| item.kind == *kind && item.state == RequirementState::Passed)
        })
}

#[cfg(test)]
mod tests {
    use super::{
        RequirementKind, RequirementResult, RequirementState, required_gates_passed, unmet_labels,
    };

    #[test]
    fn unique_samples_are_not_enough_to_generate() {
        let requirements = vec![
            RequirementResult::new(RequirementKind::SampleDiversity, RequirementState::Passed)
                .with_value(2),
            RequirementResult::new(RequirementKind::FactCoverage, RequirementState::Unmeasured),
            RequirementResult::new(
                RequirementKind::SemanticDistinctness,
                RequirementState::Unmeasured,
            ),
            RequirementResult::new(
                RequirementKind::CanonicalStrategy,
                RequirementState::Unmeasured,
            ),
        ];
        assert!(!required_gates_passed(&requirements));
        assert!(
            unmet_labels(&requirements)
                .iter()
                .any(|item| item == "fact coverage")
        );
    }

    #[test]
    fn unmeasured_optional_gates_do_not_nag() {
        let requirements = vec![RequirementResult::new(
            RequirementKind::ClaimIntegrity,
            RequirementState::Unmeasured,
        )];
        assert!(unmet_labels(&requirements).is_empty());
    }
}

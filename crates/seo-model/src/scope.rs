//! Comparability of two evidence runs.
//!
//! The gate, the history diff, and any future comparison must agree on what
//! "comparable" means. One type owns that rule so a doc comment and the code
//! cannot drift apart.

use crate::AnalysisMode;
use serde::{Deserialize, Serialize};

/// Identity that decides whether two runs may be compared as SEO evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceScope {
    /// Seed origin.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
    /// Analysis mode.
    pub mode: AnalysisMode,
    /// Policy identifier.
    #[serde(default)]
    pub policy_version: String,
    /// Crawl and request configuration digest.
    #[serde(default)]
    pub config_digest: String,
    /// Rule-semantics digest. Empty means a legacy snapshot.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub rule_semantics_digest: String,
    /// Policy-pack digest. Empty means a legacy snapshot.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub policy_pack_digest: String,
}

impl EvidenceScope {
    /// Builds a scope.
    #[must_use]
    pub fn new(
        origin: Option<String>,
        mode: AnalysisMode,
        policy_version: impl Into<String>,
        config_digest: impl Into<String>,
    ) -> Self {
        Self {
            origin,
            mode,
            policy_version: policy_version.into(),
            config_digest: config_digest.into(),
            rule_semantics_digest: String::new(),
            policy_pack_digest: String::new(),
        }
    }

    /// Attaches evidence-semantics identity.
    #[must_use]
    pub fn with_semantics(mut self, semantics: Option<&crate::EvidenceSemantics>) -> Self {
        if let Some(semantics) = semantics {
            self.rule_semantics_digest
                .clone_from(&semantics.rule_semantics_digest);
            self.policy_pack_digest
                .clone_from(&semantics.policy_pack_digest);
        }
        self
    }

    /// True when origin, mode, and policy allow a comparison.
    ///
    /// A site-only run and a hybrid run over the same origin measure the same
    /// live surface, so they stay comparable. An empty policy on either side
    /// means "not declared", not "different".
    #[must_use]
    pub fn comparable_with(&self, other: &Self) -> bool {
        self.origin == other.origin
            && modes_match(self.mode, other.mode)
            && (self.policy_version.is_empty()
                || other.policy_version.is_empty()
                || self.policy_version == other.policy_version)
            && (self.rule_semantics_digest.is_empty()
                || other.rule_semantics_digest.is_empty()
                || self.rule_semantics_digest == other.rule_semantics_digest)
            && (self.policy_pack_digest.is_empty()
                || other.policy_pack_digest.is_empty()
                || self.policy_pack_digest == other.policy_pack_digest)
    }

    /// True when either side is a legacy snapshot without semantics identity.
    #[must_use]
    pub fn legacy_semantics(&self, other: &Self) -> bool {
        self.rule_semantics_digest.is_empty() || other.rule_semantics_digest.is_empty()
    }

    /// True when the crawl configuration differs between two comparable runs.
    ///
    /// Comparability does not imply equal coverage. A caller that ignores this
    /// will read a smaller crawl as a set of resolved findings.
    #[must_use]
    pub fn config_changed(&self, other: &Self) -> bool {
        !self.config_digest.is_empty()
            && !other.config_digest.is_empty()
            && self.config_digest != other.config_digest
    }
}

fn modes_match(left: AnalysisMode, right: AnalysisMode) -> bool {
    left == right
        || matches!(
            (left, right),
            (AnalysisMode::Site, AnalysisMode::Hybrid) | (AnalysisMode::Hybrid, AnalysisMode::Site)
        )
}

#[cfg(test)]
mod tests {
    use super::EvidenceScope;
    use crate::AnalysisMode;

    fn scope(origin: &str, mode: AnalysisMode, policy: &str, config: &str) -> EvidenceScope {
        EvidenceScope::new(Some(origin.into()), mode, policy, config)
    }

    #[test]
    fn site_and_hybrid_over_one_origin_stay_comparable() {
        let left = scope("https://x.test/", AnalysisMode::Site, "0.1.0", "a");
        let right = scope("https://x.test/", AnalysisMode::Hybrid, "0.1.0", "a");
        assert!(left.comparable_with(&right));
    }

    #[test]
    fn compare_mode_is_not_a_site_run() {
        let left = scope("https://x.test/", AnalysisMode::Site, "0.1.0", "a");
        let right = scope("https://x.test/", AnalysisMode::Compare, "0.1.0", "a");
        assert!(!left.comparable_with(&right));
    }

    #[test]
    fn different_origin_is_never_comparable() {
        let left = scope("https://x.test/", AnalysisMode::Site, "0.1.0", "a");
        let right = scope("https://y.test/", AnalysisMode::Site, "0.1.0", "a");
        assert!(!left.comparable_with(&right));
    }

    #[test]
    fn budget_change_is_comparable_but_reported() {
        let small = scope("https://x.test/", AnalysisMode::Site, "0.1.0", "max=100");
        let large = scope("https://x.test/", AnalysisMode::Site, "0.1.0", "max=1000");
        assert!(small.comparable_with(&large));
        assert!(small.config_changed(&large));
    }

    #[test]
    fn undeclared_policy_is_not_a_mismatch() {
        let declared = scope("https://x.test/", AnalysisMode::Site, "0.1.0", "a");
        let blank = scope("https://x.test/", AnalysisMode::Site, "", "a");
        assert!(declared.comparable_with(&blank));
    }
}

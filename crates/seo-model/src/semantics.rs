//! Comparability identity for finding semantics.
//!
//! Package version is not enough: two snapshots can share a crate version and
//! still disagree on rule meaning. [`EvidenceSemantics`] hashes the catalogue,
//! authorities, and thresholds so history/diff can refuse a false comparison.

use crate::{ContentHash, FindingFamily, RuleAuthority, Severity, registry, schema_feature};
use std::fmt::Write as _;

/// Artifact schema for the current report shape.
pub const ARTIFACT_SCHEMA_VERSION: &str = "seo-artifact/2";

/// Engine version bound into every snapshot. Kept in lockstep with the crate.
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Thresholds that participate in ranking and programmatic verdicts.
/// Changing these changes [`EvidenceSemantics::rule_semantics_digest`].
pub const MIN_CONFIDENCE: u16 = 40;
/// Risk above this sinks an opportunity below trusted work.
pub const MAX_RISK: u16 = 70;
/// Unique-body samples that used to unlock `SAFE_TO_GENERATE` on their own.
pub const LEGACY_UNIQUE_SAMPLE_FLOOR: u16 = 2;

/// Identity of the evidence semantics used to produce findings.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EvidenceSemantics {
    /// Crate / engine version.
    pub engine_version: String,
    /// Report artifact schema.
    pub artifact_schema_version: String,
    /// Digest of finding codes, severity, authority, and thresholds.
    pub rule_semantics_digest: String,
    /// Digest of shipped policy-pack identifiers.
    pub policy_pack_digest: String,
}

impl EvidenceSemantics {
    /// Semantics shipped with this binary.
    #[must_use]
    pub fn current() -> Self {
        Self {
            engine_version: ENGINE_VERSION.to_owned(),
            artifact_schema_version: ARTIFACT_SCHEMA_VERSION.to_owned(),
            rule_semantics_digest: rule_semantics_digest(),
            policy_pack_digest: policy_pack_digest(),
        }
    }
}

impl Default for EvidenceSemantics {
    fn default() -> Self {
        Self::current()
    }
}

/// Digest of rule identity: registered codes, severity, authority, thresholds, features.
#[must_use]
pub fn rule_semantics_digest() -> String {
    let mut material = String::new();
    for rule in registry::all() {
        let _ = writeln!(
            material,
            "{}-{:03}:{}:{}:{}:{}",
            rule.family.prefix(),
            rule.number,
            severity_name(rule.default_severity),
            authority_token(rule.authority),
            rule.semantics_version,
            rule.provider_or_standard
        );
    }
    for family in FindingFamily::ALL {
        let _ = writeln!(
            material,
            "fallback:{}:{}:{}",
            family.prefix(),
            severity_token(family),
            authority_token(RuleAuthority::for_family(family, 1))
        );
    }
    for profile in schema_feature::profiles() {
        let _ = writeln!(
            material,
            "feature:{}:{}:{}:{}:{}:{}:{}:{}",
            profile.feature,
            profile.applies_to,
            profile.docs_revision,
            profile.status.as_str(),
            profile.required.canonical(),
            profile.recommended.join(","),
            profile.effective_until.unwrap_or("-"),
            profile.docs_checked_at
        );
    }
    let _ = write!(
        material,
        "min_confidence={MIN_CONFIDENCE}\nmax_risk={MAX_RISK}\nlegacy_unique={LEGACY_UNIQUE_SAMPLE_FLOOR}\nnormalization=whitespace\n"
    );
    ContentHash::of_str(&material).hex()
}

/// Digest of built-in policy pack identifiers. Pack *content* lives in claims;
/// this identity only records which packs the engine knows about.
#[must_use]
pub fn policy_pack_digest() -> String {
    ContentHash::of_str("marketplace.contractor.us-wa\nmarketplace.contractor.il\n").hex()
}

fn severity_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warn => "warn",
        Severity::Info => "info",
    }
}

fn severity_token(family: FindingFamily) -> &'static str {
    match family {
        FindingFamily::Claim | FindingFamily::Idx | FindingFamily::Crawl => "error",
        FindingFamily::Ai | FindingFamily::Obs | FindingFamily::Comp => "info",
        _ => "warn",
    }
}

fn authority_token(authority: RuleAuthority) -> &'static str {
    match authority {
        RuleAuthority::ProtocolRequirement => "protocol",
        RuleAuthority::SearchEngineDocumented => "search",
        RuleAuthority::ProjectContract => "contract",
        RuleAuthority::JurisdictionRequirement => "jurisdiction",
        RuleAuthority::IndustryBestPractice => "practice",
        RuleAuthority::ExperimentalHeuristic => "heuristic",
        RuleAuthority::InferredOpportunity => "opportunity",
    }
}

#[cfg(test)]
mod tests {
    use super::{ARTIFACT_SCHEMA_VERSION, ENGINE_VERSION, EvidenceSemantics};

    #[test]
    fn current_semantics_are_stable_within_a_process() {
        let left = EvidenceSemantics::current();
        let right = EvidenceSemantics::current();
        assert_eq!(left, right);
        assert_eq!(left.engine_version, ENGINE_VERSION);
        assert_eq!(left.artifact_schema_version, ARTIFACT_SCHEMA_VERSION);
        assert_eq!(left.rule_semantics_digest.len(), 32);
        assert_eq!(left.policy_pack_digest.len(), 32);
    }
}

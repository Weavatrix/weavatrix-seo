//! Why a rule is legitimate. Distinct from [`crate::EvidenceKind`], which says how a fact was established.

use crate::FindingFamily;
use serde::{Deserialize, Serialize};

/// Authority of the rule that produced a finding.
///
/// Severity still gates CI. Authority tells an agent whether the rule is a
/// protocol MUST, a search-engine documented SHOULD, a project contract, or an
/// experiment. Missing evidence stays unmeasured; authority never upgrades it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleAuthority {
    /// RFC / HTML / HTTP / robots syntax.
    ProtocolRequirement,
    /// Documented search-engine behaviour (canonical, hreflang, sitemap).
    SearchEngineDocumented,
    /// Explicit `.weavatrix/seo` or repository contract.
    ProjectContract,
    /// Jurisdiction or licensed-trade requirement.
    JurisdictionRequirement,
    /// Widely accepted industry practice.
    IndustryBestPractice,
    /// Heuristic that is useful but not proven.
    ExperimentalHeuristic,
    /// Opportunity inferred from gaps, not a defect.
    InferredOpportunity,
}

impl Default for RuleAuthority {
    fn default() -> Self {
        Self::ExperimentalHeuristic
    }
}

impl RuleAuthority {
    /// Catalogue authority for a finding family and number.
    #[must_use]
    pub const fn for_family(family: FindingFamily, number: u16) -> Self {
        match family {
            FindingFamily::Crawl | FindingFamily::A11y | FindingFamily::Security => {
                Self::ProtocolRequirement
            }
            FindingFamily::Idx if number == 1 => Self::ProjectContract,
            FindingFamily::Idx
            | FindingFamily::Canon
            | FindingFamily::Sitemap
            | FindingFamily::I18n
            | FindingFamily::Meta
            | FindingFamily::Schema
            | FindingFamily::Dup
            | FindingFamily::Cann
            | FindingFamily::Local => Self::SearchEngineDocumented,
            FindingFamily::Render
            | FindingFamily::Link
            | FindingFamily::Content
            | FindingFamily::Entity
            | FindingFamily::Prog
            | FindingFamily::Perf => Self::IndustryBestPractice,
            FindingFamily::Market | FindingFamily::Claim => Self::JurisdictionRequirement,
            FindingFamily::Ai => Self::ExperimentalHeuristic,
            FindingFamily::Obs | FindingFamily::Comp => Self::InferredOpportunity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RuleAuthority;
    use crate::FindingFamily;

    #[test]
    fn protocol_and_jurisdiction_stay_distinct() {
        assert_eq!(
            RuleAuthority::for_family(FindingFamily::Crawl, 1),
            RuleAuthority::ProtocolRequirement
        );
        assert_eq!(
            RuleAuthority::for_family(FindingFamily::Claim, 1),
            RuleAuthority::JurisdictionRequirement
        );
        assert_eq!(
            RuleAuthority::for_family(FindingFamily::Idx, 1),
            RuleAuthority::ProjectContract
        );
        assert_eq!(
            RuleAuthority::for_family(FindingFamily::Ai, 1),
            RuleAuthority::ExperimentalHeuristic
        );
    }
}

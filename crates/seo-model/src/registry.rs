//! Per-code rule registry. Family defaults stay as fallback.

use crate::{FindingFamily, RuleAuthority, Severity};

/// One catalogue rule. Hashed into [`crate::rule_semantics_digest`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuleDefinition {
    /// Family.
    pub family: FindingFamily,
    /// Catalogue number.
    pub number: u16,
    /// Default gate severity.
    pub default_severity: Severity,
    /// Why the rule is legitimate.
    pub authority: RuleAuthority,
    /// Short title.
    pub title: &'static str,
    /// Standard or provider, for example `google-search` or `rfc9309`.
    pub provider_or_standard: &'static str,
    /// Semantics version of this rule body.
    pub semantics_version: &'static str,
}

impl RuleDefinition {
    /// Catalogue code such as `WVX-SEO-META-001`.
    #[must_use]
    pub fn code(self) -> String {
        format!("{}-{:03}", self.family.prefix(), self.number)
    }
}

/// Looks up a registered rule. Unknown numbers fall back to family defaults.
#[must_use]
pub fn lookup(family: FindingFamily, number: u16) -> Option<&'static RuleDefinition> {
    RULES
        .iter()
        .find(|rule| rule.family == family && rule.number == number)
}

/// Authority for a finding. Registered rules win over family defaults.
#[must_use]
pub fn authority(family: FindingFamily, number: u16) -> RuleAuthority {
    lookup(family, number).map_or_else(
        || RuleAuthority::for_family(family, number),
        |rule| rule.authority,
    )
}

/// Every registered rule, sorted by code. Used by the semantics digest.
#[must_use]
pub fn all() -> &'static [RuleDefinition] {
    RULES
}

const RULES: &[RuleDefinition] = &[
    def(
        FindingFamily::Crawl,
        1,
        Severity::Error,
        RuleAuthority::ProtocolRequirement,
        "client error status",
        "http",
        "1",
    ),
    def(
        FindingFamily::Crawl,
        2,
        Severity::Error,
        RuleAuthority::ProtocolRequirement,
        "server error status",
        "http",
        "1",
    ),
    def(
        FindingFamily::Crawl,
        3,
        Severity::Warn,
        RuleAuthority::SearchEngineDocumented,
        "redirect chain",
        "google-search",
        "1",
    ),
    def(
        FindingFamily::Canon,
        1,
        Severity::Warn,
        RuleAuthority::SearchEngineDocumented,
        "missing canonical",
        "google-search",
        "1",
    ),
    def(
        FindingFamily::Canon,
        2,
        Severity::Error,
        RuleAuthority::SearchEngineDocumented,
        "canonical target error",
        "google-search",
        "1",
    ),
    def(
        FindingFamily::Canon,
        3,
        Severity::Warn,
        RuleAuthority::SearchEngineDocumented,
        "canonical chain",
        "google-search",
        "1",
    ),
    def(
        FindingFamily::Canon,
        4,
        Severity::Info,
        RuleAuthority::SearchEngineDocumented,
        "canonical target unmeasured",
        "google-search",
        "1",
    ),
    def(
        FindingFamily::I18n,
        1,
        Severity::Warn,
        RuleAuthority::SearchEngineDocumented,
        "hreflang not reciprocal",
        "google-search",
        "1",
    ),
    def(
        FindingFamily::I18n,
        2,
        Severity::Warn,
        RuleAuthority::SearchEngineDocumented,
        "locale twins without cluster",
        "google-search",
        "1",
    ),
    def(
        FindingFamily::I18n,
        3,
        Severity::Warn,
        RuleAuthority::ProjectContract,
        "missing x-default",
        "project-policy",
        "1",
    ),
    def(
        FindingFamily::I18n,
        4,
        Severity::Error,
        RuleAuthority::SearchEngineDocumented,
        "hreflang target error",
        "google-search",
        "1",
    ),
    def(
        FindingFamily::I18n,
        5,
        Severity::Info,
        RuleAuthority::SearchEngineDocumented,
        "hreflang target unmeasured",
        "google-search",
        "1",
    ),
    def(
        FindingFamily::Meta,
        1,
        Severity::Error,
        RuleAuthority::SearchEngineDocumented,
        "missing title",
        "google-search",
        "1",
    ),
    def(
        FindingFamily::Meta,
        2,
        Severity::Warn,
        RuleAuthority::SearchEngineDocumented,
        "duplicate title",
        "google-search",
        "1",
    ),
    def(
        FindingFamily::Meta,
        3,
        Severity::Info,
        RuleAuthority::IndustryBestPractice,
        "missing description",
        "google-search",
        "1",
    ),
    def(
        FindingFamily::Meta,
        4,
        Severity::Info,
        RuleAuthority::IndustryBestPractice,
        "missing og:title",
        "open-graph",
        "1",
    ),
    def(
        FindingFamily::Meta,
        5,
        Severity::Info,
        RuleAuthority::IndustryBestPractice,
        "missing og:image",
        "open-graph",
        "1",
    ),
    def(
        FindingFamily::Meta,
        6,
        Severity::Warn,
        RuleAuthority::IndustryBestPractice,
        "duplicate description",
        "google-search",
        "1",
    ),
    def(
        FindingFamily::Schema,
        1,
        Severity::Warn,
        RuleAuthority::ProtocolRequirement,
        "invalid JSON-LD",
        "json",
        "1",
    ),
    def(
        FindingFamily::Schema,
        2,
        Severity::Warn,
        RuleAuthority::SearchEngineDocumented,
        "rich-result required field",
        "google-search",
        "1",
    ),
    def(
        FindingFamily::Schema,
        3,
        Severity::Info,
        RuleAuthority::IndustryBestPractice,
        "schema.org vocabulary gap",
        "schema.org",
        "1",
    ),
    def(
        FindingFamily::Schema,
        4,
        Severity::Info,
        RuleAuthority::SearchEngineDocumented,
        "retired rich-result feature",
        "google-search",
        "1",
    ),
    def(
        FindingFamily::Link,
        1,
        Severity::Error,
        RuleAuthority::SearchEngineDocumented,
        "broken internal link",
        "google-search",
        "1",
    ),
    def(
        FindingFamily::Link,
        2,
        Severity::Warn,
        RuleAuthority::SearchEngineDocumented,
        "orphan indexable URL",
        "google-search",
        "1",
    ),
    def(
        FindingFamily::Link,
        3,
        Severity::Info,
        RuleAuthority::IndustryBestPractice,
        "deep URL",
        "google-search",
        "1",
    ),
    def(
        FindingFamily::Link,
        4,
        Severity::Info,
        RuleAuthority::InferredOpportunity,
        "high internal authority without demand",
        "gsc",
        "1",
    ),
    def(
        FindingFamily::Link,
        5,
        Severity::Warn,
        RuleAuthority::SearchEngineDocumented,
        "internal equity leak",
        "google-search",
        "1",
    ),
    def(
        FindingFamily::Dup,
        1,
        Severity::Warn,
        RuleAuthority::SearchEngineDocumented,
        "exact duplicate",
        "google-search",
        "1",
    ),
    def(
        FindingFamily::Dup,
        2,
        Severity::Warn,
        RuleAuthority::IndustryBestPractice,
        "near duplicate",
        "weavatrix-seo",
        "1",
    ),
    def(
        FindingFamily::Content,
        1,
        Severity::Error,
        RuleAuthority::SearchEngineDocumented,
        "missing H1",
        "google-search",
        "1",
    ),
    def(
        FindingFamily::Content,
        2,
        Severity::Warn,
        RuleAuthority::IndustryBestPractice,
        "multiple H1",
        "google-search",
        "1",
    ),
    def(
        FindingFamily::Content,
        3,
        Severity::Warn,
        RuleAuthority::IndustryBestPractice,
        "thin programmatic family",
        "weavatrix-seo",
        "1",
    ),
    def(
        FindingFamily::Content,
        4,
        Severity::Info,
        RuleAuthority::InferredOpportunity,
        "passage gap",
        "gsc-inferred",
        "1",
    ),
    def(
        FindingFamily::Cann,
        1,
        Severity::Warn,
        RuleAuthority::SearchEngineDocumented,
        "query cannibalization",
        "gsc",
        "1",
    ),
    def(
        FindingFamily::Security,
        8,
        Severity::Warn,
        RuleAuthority::ProtocolRequirement,
        "mixed content subresource",
        "mixed-content",
        "1",
    ),
    def(
        FindingFamily::Ai,
        1,
        Severity::Warn,
        RuleAuthority::ExperimentalHeuristic,
        "publisher without @id",
        "ai-search",
        "1",
    ),
    def(
        FindingFamily::Ai,
        2,
        Severity::Warn,
        RuleAuthority::ExperimentalHeuristic,
        "FAQ copy without FAQPage",
        "ai-search",
        "1",
    ),
    def(
        FindingFamily::Ai,
        3,
        Severity::Warn,
        RuleAuthority::ExperimentalHeuristic,
        "FAQ producer without schema",
        "ai-search",
        "1",
    ),
    def(
        FindingFamily::Ai,
        4,
        Severity::Info,
        RuleAuthority::ExperimentalHeuristic,
        "llms.txt absent",
        "llms-txt",
        "1",
    ),
    def(
        FindingFamily::Ai,
        5,
        Severity::Warn,
        RuleAuthority::SearchEngineDocumented,
        "AI agent origin disallow",
        "robots",
        "1",
    ),
    def(
        FindingFamily::Obs,
        1,
        Severity::Info,
        RuleAuthority::SearchEngineDocumented,
        "GSC URL not in crawl",
        "gsc",
        "1",
    ),
    def(
        FindingFamily::Obs,
        2,
        Severity::Info,
        RuleAuthority::SearchEngineDocumented,
        "search demand without bot hits",
        "gsc",
        "1",
    ),
    def(
        FindingFamily::Obs,
        3,
        Severity::Error,
        RuleAuthority::ProtocolRequirement,
        "invalid observation file",
        "input",
        "1",
    ),
    def(
        FindingFamily::Obs,
        4,
        Severity::Info,
        RuleAuthority::InferredOpportunity,
        "CTR gap versus expected",
        "gsc-inferred",
        "1",
    ),
    def(
        FindingFamily::Obs,
        5,
        Severity::Info,
        RuleAuthority::SearchEngineDocumented,
        "striking distance query",
        "gsc",
        "1",
    ),
    def(
        FindingFamily::Obs,
        6,
        Severity::Info,
        RuleAuthority::SearchEngineDocumented,
        "content decay",
        "gsc",
        "1",
    ),
];

const fn def(
    family: FindingFamily,
    number: u16,
    default_severity: Severity,
    authority: RuleAuthority,
    title: &'static str,
    provider_or_standard: &'static str,
    semantics_version: &'static str,
) -> RuleDefinition {
    RuleDefinition {
        family,
        number,
        default_severity,
        authority,
        title,
        provider_or_standard,
        semantics_version,
    }
}

#[cfg(test)]
mod tests {
    use super::{RULES, all, authority, lookup};
    use crate::{FindingFamily, RuleAuthority, Severity};
    use std::collections::BTreeSet;

    #[test]
    fn registered_rules_override_family_defaults() {
        assert_eq!(
            authority(FindingFamily::Meta, 1),
            RuleAuthority::SearchEngineDocumented
        );
        assert_eq!(
            authority(FindingFamily::Meta, 6),
            RuleAuthority::IndustryBestPractice
        );
        assert_eq!(
            authority(FindingFamily::Ai, 4),
            RuleAuthority::ExperimentalHeuristic
        );
        assert_eq!(
            lookup(FindingFamily::Canon, 4).expect("canon-004").title,
            "canonical target unmeasured"
        );
    }

    #[test]
    fn registry_codes_are_unique() {
        let mut seen = BTreeSet::new();
        for rule in all() {
            let code = rule.code();
            assert!(seen.insert(code.clone()), "duplicate {code}");
        }
        assert_eq!(seen.len(), RULES.len());
    }

    #[test]
    fn lookup_matches_default_severity() {
        for rule in all() {
            let found = lookup(rule.family, rule.number).expect("registered");
            assert_eq!(found.default_severity, rule.default_severity);
            assert_eq!(found.authority, rule.authority);
        }
        assert_eq!(
            lookup(FindingFamily::Schema, 4)
                .expect("schema-004")
                .default_severity,
            Severity::Info
        );
        assert_eq!(
            lookup(FindingFamily::Obs, 3)
                .expect("obs-003")
                .default_severity,
            Severity::Error
        );
    }
}

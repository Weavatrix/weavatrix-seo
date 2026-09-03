//! Search-feature eligibility is not schema.org vocabulary validity.
//!
//! Profiles are a versioned knowledge base. Strong eligibility findings are
//! only emitted for [`FeatureStatus::Active`] Google features. Retired
//! rich results stay INFO / historical.

use crate::RuleAuthority;

/// Who defined the feature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaProvider {
    /// schema.org vocabulary.
    SchemaOrg,
    /// A search engine rich-result feature.
    Google,
}

/// Lifecycle of a documented feature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureStatus {
    /// Currently documented and eligible.
    Active,
    /// Documented as deprecated; keep compatibility notes only.
    Deprecated,
    /// Removed from the provider's search appearance.
    Removed,
    /// Experimental.
    Experimental,
    /// Not established.
    Unknown,
}

impl FeatureStatus {
    /// Wire token used in the semantics digest.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Deprecated => "deprecated",
            Self::Removed => "removed",
            Self::Experimental => "experimental",
            Self::Unknown => "unknown",
        }
    }

    /// Whether missing required fields are current eligibility failures.
    #[must_use]
    pub const fn strong_eligibility(self) -> bool {
        matches!(self, Self::Active)
    }
}

/// A required property path or a composite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Requirement {
    /// A property on the typed node, for example `name`.
    Path(&'static str),
    /// Any one of the paths/composites.
    AnyOf(&'static [Requirement]),
    /// All of the paths/composites.
    All(&'static [Requirement]),
}

impl Requirement {
    /// Canonical tree, for example `ANY(price,priceSpecification)`.
    #[must_use]
    pub fn canonical(self) -> String {
        match self {
            Self::Path(name) => name.to_owned(),
            Self::AnyOf(items) => {
                let inner = items
                    .iter()
                    .map(|item| item.canonical())
                    .collect::<Vec<_>>()
                    .join(",");
                format!("ANY({inner})")
            }
            Self::All(items) => {
                let inner = items
                    .iter()
                    .map(|item| item.canonical())
                    .collect::<Vec<_>>()
                    .join(",");
                format!("ALL({inner})")
            }
        }
    }
}

/// One documented search or vocabulary feature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchemaFeatureProfile {
    /// Provider.
    pub provider: SchemaProvider,
    /// Feature id, for example `MerchantListing`.
    pub feature: &'static str,
    /// Lifecycle.
    pub status: FeatureStatus,
    /// Primary `@type` this applies to.
    pub applies_to: &'static str,
    /// Required tree.
    pub required: Requirement,
    /// Recommended property paths. Absence is not a rich-result failure.
    pub recommended: &'static [&'static str],
    /// Why this profile is legitimate.
    pub authority: RuleAuthority,
    /// Docs revision label, not a live fetch.
    pub docs_revision: &'static str,
    /// Inclusive start of the documented contract, `YYYY-MM-DD`.
    pub effective_from: Option<&'static str>,
    /// Inclusive end when the feature left the provider, `YYYY-MM-DD`.
    pub effective_until: Option<&'static str>,
    /// Last human review of the cited documentation.
    pub docs_checked_at: &'static str,
}

/// Last review date for volatile Google / AI-agent knowledge.
pub const KNOWLEDGE_CHECKED_AT: &str = "2026-09-03";

/// Shipped Google rich-result and vocabulary profiles.
#[must_use]
pub fn profiles() -> &'static [SchemaFeatureProfile] {
    PROFILES
}

const PRODUCT_PROOF: &[Requirement] = &[
    Requirement::Path("review"),
    Requirement::Path("aggregateRating"),
    Requirement::Path("offers"),
];
const PRODUCT_REQUIRED: &[Requirement] =
    &[Requirement::Path("name"), Requirement::AnyOf(PRODUCT_PROOF)];
const LOCAL_REQUIRED: &[Requirement] = &[Requirement::Path("name"), Requirement::Path("address")];
const HOWTO_REQUIRED: &[Requirement] = &[Requirement::Path("name"), Requirement::Path("step")];
const OFFER_PRICE: &[Requirement] = &[
    Requirement::Path("price"),
    Requirement::Path("priceSpecification"),
];

const PROFILES: &[SchemaFeatureProfile] = &[
    SchemaFeatureProfile {
        provider: SchemaProvider::Google,
        feature: "FAQ",
        status: FeatureStatus::Removed,
        applies_to: "FAQPage",
        required: Requirement::Path("mainEntity"),
        recommended: &["mainEntity.acceptedAnswer"],
        authority: RuleAuthority::SearchEngineDocumented,
        docs_revision: "google-search/faq-retired-2026-05",
        effective_from: Some("2018-05-08"),
        effective_until: Some("2026-05-07"),
        docs_checked_at: KNOWLEDGE_CHECKED_AT,
    },
    SchemaFeatureProfile {
        provider: SchemaProvider::Google,
        feature: "QAndA",
        status: FeatureStatus::Active,
        applies_to: "QAPage",
        required: Requirement::Path("mainEntity"),
        recommended: &["mainEntity.acceptedAnswer"],
        authority: RuleAuthority::SearchEngineDocumented,
        docs_revision: "google-search/qapage-2026-06",
        effective_from: Some("2019-05-01"),
        effective_until: None,
        docs_checked_at: KNOWLEDGE_CHECKED_AT,
    },
    SchemaFeatureProfile {
        provider: SchemaProvider::Google,
        feature: "MerchantListing",
        status: FeatureStatus::Active,
        applies_to: "Offer",
        required: Requirement::AnyOf(OFFER_PRICE),
        recommended: &["priceCurrency", "availability"],
        authority: RuleAuthority::SearchEngineDocumented,
        docs_revision: "google-search/merchant-listing-2025-12",
        effective_from: None,
        effective_until: None,
        docs_checked_at: KNOWLEDGE_CHECKED_AT,
    },
    SchemaFeatureProfile {
        provider: SchemaProvider::Google,
        feature: "Product",
        status: FeatureStatus::Active,
        applies_to: "Product",
        required: Requirement::All(PRODUCT_REQUIRED),
        recommended: &["image"],
        authority: RuleAuthority::SearchEngineDocumented,
        docs_revision: "google-search/product-snippet-2025-12",
        effective_from: None,
        effective_until: None,
        docs_checked_at: KNOWLEDGE_CHECKED_AT,
    },
    SchemaFeatureProfile {
        provider: SchemaProvider::Google,
        feature: "LocalBusiness",
        status: FeatureStatus::Active,
        applies_to: "LocalBusiness",
        required: Requirement::All(LOCAL_REQUIRED),
        recommended: &["telephone", "url"],
        authority: RuleAuthority::SearchEngineDocumented,
        docs_revision: "google-search/local-business-2025-12",
        effective_from: None,
        effective_until: None,
        docs_checked_at: KNOWLEDGE_CHECKED_AT,
    },
    SchemaFeatureProfile {
        provider: SchemaProvider::Google,
        feature: "Article",
        status: FeatureStatus::Active,
        applies_to: "Article",
        required: Requirement::Path("headline"),
        recommended: &["image", "datePublished", "author"],
        authority: RuleAuthority::SearchEngineDocumented,
        docs_revision: "google-search/article-2026-06",
        effective_from: None,
        effective_until: None,
        docs_checked_at: KNOWLEDGE_CHECKED_AT,
    },
    SchemaFeatureProfile {
        provider: SchemaProvider::Google,
        feature: "HowTo",
        status: FeatureStatus::Removed,
        applies_to: "HowTo",
        required: Requirement::All(HOWTO_REQUIRED),
        recommended: &["totalTime", "image"],
        authority: RuleAuthority::SearchEngineDocumented,
        docs_revision: "google-search/howto-retired-2023-09",
        effective_from: Some("2019-06-01"),
        effective_until: Some("2023-09-08"),
        docs_checked_at: KNOWLEDGE_CHECKED_AT,
    },
    SchemaFeatureProfile {
        provider: SchemaProvider::Google,
        feature: "Breadcrumb",
        status: FeatureStatus::Active,
        applies_to: "BreadcrumbList",
        required: Requirement::Path("itemListElement"),
        recommended: &[],
        authority: RuleAuthority::SearchEngineDocumented,
        docs_revision: "google-search/breadcrumb-2026-06",
        effective_from: None,
        effective_until: None,
        docs_checked_at: KNOWLEDGE_CHECKED_AT,
    },
    SchemaFeatureProfile {
        provider: SchemaProvider::SchemaOrg,
        feature: "Organization",
        status: FeatureStatus::Active,
        applies_to: "Organization",
        required: Requirement::Path("name"),
        recommended: &["url", "sameAs"],
        authority: RuleAuthority::IndustryBestPractice,
        docs_revision: "schema.org/Organization",
        effective_from: None,
        effective_until: None,
        docs_checked_at: KNOWLEDGE_CHECKED_AT,
    },
];

/// Whether `properties` on a typed node satisfy `requirement`.
#[must_use]
pub fn satisfied(requirement: Requirement, properties: &[String]) -> bool {
    match requirement {
        Requirement::Path(name) => properties.iter().any(|item| path_matches(item, name)),
        Requirement::AnyOf(items) => items.iter().any(|item| satisfied(*item, properties)),
        Requirement::All(items) => items.iter().all(|item| satisfied(*item, properties)),
    }
}

/// Missing required paths for display.
#[must_use]
pub fn missing(requirement: Requirement, properties: &[String]) -> Vec<&'static str> {
    match requirement {
        Requirement::Path(name) => {
            if satisfied(requirement, properties) {
                Vec::new()
            } else {
                vec![name]
            }
        }
        Requirement::AnyOf(items) => {
            if satisfied(requirement, properties) {
                Vec::new()
            } else {
                items
                    .iter()
                    .flat_map(|item| missing(*item, properties))
                    .collect()
            }
        }
        Requirement::All(items) => items
            .iter()
            .flat_map(|item| missing(*item, properties))
            .collect(),
    }
}

fn path_matches(observed: &str, required: &str) -> bool {
    observed.eq_ignore_ascii_case(required)
        || required
            .rsplit_once('.')
            .is_some_and(|(parent, _)| observed.eq_ignore_ascii_case(parent))
}

#[cfg(test)]
mod tests {
    use super::{
        FeatureStatus, KNOWLEDGE_CHECKED_AT, Requirement, SchemaProvider, profiles, satisfied,
    };
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct KnowledgeFile {
        checked_at: String,
        max_age_days: u16,
        #[serde(default)]
        features: Vec<KnowledgeFeature>,
    }

    #[derive(Deserialize)]
    struct KnowledgeFeature {
        feature: String,
        applies_to: String,
        status: String,
        required: String,
        docs_revision: String,
    }

    #[test]
    fn faq_is_not_a_current_google_rich_result() {
        let faq = profiles()
            .iter()
            .find(|profile| profile.applies_to == "FAQPage")
            .expect("faq");
        assert_eq!(faq.status, FeatureStatus::Removed);
        assert!(!faq.status.strong_eligibility());
        assert_eq!(faq.effective_until, Some("2026-05-07"));
    }

    #[test]
    fn howto_was_removed_in_2023() {
        let howto = profiles()
            .iter()
            .find(|profile| profile.feature == "HowTo")
            .expect("howto");
        assert_eq!(howto.status, FeatureStatus::Removed);
        assert_eq!(howto.effective_until, Some("2023-09-08"));
    }

    #[test]
    fn product_snippets_need_name_and_a_proof_path() {
        let product = profiles()
            .iter()
            .find(|profile| profile.feature == "Product")
            .expect("product");
        assert_eq!(
            product.required.canonical(),
            "ALL(name,ANY(review,aggregateRating,offers))"
        );
        let only_name = vec!["name".into()];
        assert!(!satisfied(product.required, &only_name));
        let with_offer = vec!["name".into(), "offers".into()];
        assert!(satisfied(product.required, &with_offer));
    }

    #[test]
    fn knowledge_manifest_matches_shipped_google_profiles() {
        let raw = include_str!("../knowledge/google-search.json");
        let file: KnowledgeFile = blazingly_json::from_str(raw).expect("google knowledge");
        assert_eq!(file.checked_at, KNOWLEDGE_CHECKED_AT);
        assert_eq!(file.max_age_days, 90);
        let google: Vec<_> = profiles()
            .iter()
            .filter(|profile| profile.provider == SchemaProvider::Google)
            .collect();
        assert_eq!(file.features.len(), google.len());
        for rust in google {
            let json = file
                .features
                .iter()
                .find(|item| item.feature == rust.feature && item.applies_to == rust.applies_to)
                .unwrap_or_else(|| panic!("{} {}", rust.feature, rust.applies_to));
            assert_eq!(json.status, rust.status.as_str());
            assert_eq!(json.required, rust.required.canonical());
            assert_eq!(json.docs_revision, rust.docs_revision);
            assert_eq!(rust.docs_checked_at, KNOWLEDGE_CHECKED_AT);
        }
    }

    #[test]
    fn google_knowledge_is_not_stale() {
        let raw = include_str!("../knowledge/google-search.json");
        let file: KnowledgeFile = blazingly_json::from_str(raw).expect("google knowledge");
        let age = days_since(&file.checked_at);
        assert!(
            age <= i64::from(file.max_age_days),
            "google-search.json checked_at {} is {age} days old (max {})",
            file.checked_at,
            file.max_age_days
        );
    }

    #[test]
    fn requirement_trees_are_canonical() {
        assert_eq!(Requirement::Path("name").canonical(), "name");
        assert_eq!(
            Requirement::AnyOf(&[Requirement::Path("a"), Requirement::Path("b")]).canonical(),
            "ANY(a,b)"
        );
    }

    fn days_since(iso: &str) -> i64 {
        let mut parts = iso.split('-');
        let year: i32 = parts.next().and_then(|item| item.parse().ok()).unwrap_or(0);
        let month: u32 = parts.next().and_then(|item| item.parse().ok()).unwrap_or(1);
        let day: u32 = parts.next().and_then(|item| item.parse().ok()).unwrap_or(1);
        let then = civil_days(year, month, day);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| i64::try_from(duration.as_secs() / 86_400).unwrap_or(0))
            .unwrap_or(0)
            + 719_163;
        now - then
    }

    fn civil_days(year: i32, month: u32, day: u32) -> i64 {
        let y = i64::from(if month <= 2 { year - 1 } else { year });
        let era = y.div_euclid(400);
        let yoe = y.rem_euclid(400);
        let mp = i64::from(if month > 2 { month - 3 } else { month + 9 });
        let doy = (153 * mp + 2) / 5 + i64::from(day) - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468 + 719_163
    }
}

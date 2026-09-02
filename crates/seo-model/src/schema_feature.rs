//! Search-feature eligibility is not schema.org vocabulary validity.

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
    /// Currently documented.
    Active,
    /// Documented as deprecated.
    Deprecated,
    /// Experimental.
    Experimental,
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
}

/// Shipped Google rich-result and vocabulary profiles.
#[must_use]
pub fn profiles() -> &'static [SchemaFeatureProfile] {
    PROFILES
}

const PROFILES: &[SchemaFeatureProfile] = &[
    SchemaFeatureProfile {
        provider: SchemaProvider::Google,
        feature: "FAQ",
        status: FeatureStatus::Active,
        applies_to: "FAQPage",
        required: Requirement::Path("mainEntity"),
        recommended: &["mainEntity.acceptedAnswer"],
        authority: RuleAuthority::SearchEngineDocumented,
        docs_revision: "google-search/faq-2024",
    },
    SchemaFeatureProfile {
        provider: SchemaProvider::Google,
        feature: "FAQ",
        status: FeatureStatus::Active,
        applies_to: "QAPage",
        required: Requirement::Path("mainEntity"),
        recommended: &["mainEntity.acceptedAnswer"],
        authority: RuleAuthority::SearchEngineDocumented,
        docs_revision: "google-search/faq-2024",
    },
    SchemaFeatureProfile {
        provider: SchemaProvider::Google,
        feature: "MerchantListing",
        status: FeatureStatus::Active,
        applies_to: "Offer",
        required: Requirement::AnyOf(&[
            Requirement::Path("price"),
            Requirement::Path("priceSpecification"),
        ]),
        recommended: &["priceCurrency", "availability"],
        authority: RuleAuthority::SearchEngineDocumented,
        docs_revision: "google-search/merchant-listing-2024",
    },
    SchemaFeatureProfile {
        provider: SchemaProvider::Google,
        feature: "Product",
        status: FeatureStatus::Active,
        applies_to: "Product",
        required: Requirement::Path("name"),
        recommended: &["image", "offers"],
        authority: RuleAuthority::SearchEngineDocumented,
        docs_revision: "google-search/product-2024",
    },
    SchemaFeatureProfile {
        provider: SchemaProvider::Google,
        feature: "LocalBusiness",
        status: FeatureStatus::Active,
        applies_to: "LocalBusiness",
        required: Requirement::All(&[Requirement::Path("name"), Requirement::Path("address")]),
        recommended: &["telephone", "url"],
        authority: RuleAuthority::SearchEngineDocumented,
        docs_revision: "google-search/local-2024",
    },
    SchemaFeatureProfile {
        provider: SchemaProvider::Google,
        feature: "Article",
        status: FeatureStatus::Active,
        applies_to: "Article",
        required: Requirement::Path("headline"),
        recommended: &["image", "datePublished", "author"],
        authority: RuleAuthority::SearchEngineDocumented,
        docs_revision: "google-search/article-2024",
    },
    SchemaFeatureProfile {
        provider: SchemaProvider::Google,
        feature: "HowTo",
        status: FeatureStatus::Active,
        applies_to: "HowTo",
        required: Requirement::All(&[Requirement::Path("name"), Requirement::Path("step")]),
        recommended: &["totalTime", "image"],
        authority: RuleAuthority::SearchEngineDocumented,
        docs_revision: "google-search/howto-2024",
    },
    SchemaFeatureProfile {
        provider: SchemaProvider::Google,
        feature: "Breadcrumb",
        status: FeatureStatus::Active,
        applies_to: "BreadcrumbList",
        required: Requirement::Path("itemListElement"),
        recommended: &[],
        authority: RuleAuthority::SearchEngineDocumented,
        docs_revision: "google-search/breadcrumb-2024",
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
    },
];

/// Whether `properties` on a typed node satisfy `requirement`.
#[must_use]
pub fn satisfied(requirement: Requirement, properties: &[String]) -> bool {
    match requirement {
        Requirement::Path(name) => properties
            .iter()
            .any(|item| item.eq_ignore_ascii_case(name)),
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

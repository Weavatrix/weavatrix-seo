//! Programmatic page-matrix safety verdicts.

#![forbid(unsafe_code)]

mod compile;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use weavatrix_seo_model::{
    ContentHash, Evidence, Finding, FindingFamily, Indexability, Inventory, Locator, Severity,
};

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

impl SafetyVerdict {
    /// Stable label matching the serialized form.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::SafeToGenerate => "SAFE_TO_GENERATE",
            Self::SafeIfRequirementsMet => "SAFE_IF_REQUIREMENTS_MET",
            Self::Consolidate => "CONSOLIDATE",
            Self::NoindexByDefault => "NOINDEX_BY_DEFAULT",
            Self::RejectLowValue => "REJECT_LOW_VALUE",
            Self::Review => "REVIEW",
            Self::Unmeasured => "UNMEASURED",
        }
    }
}

/// One proposed family.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageMatrix {
    /// Family identity.
    pub family: String,
    /// URLs of this family that this crawl actually measured.
    ///
    /// This is not the size of the generated matrix. Estimating that needs the
    /// route generators, which this compiler does not read yet.
    pub measured_urls: u64,
    /// Verdict.
    pub verdict: SafetyVerdict,
}

pub use compile::compile;

/// Default compiler output before route generators are wired.
#[must_use]
pub fn unmeasured(family: impl Into<String>) -> PageMatrix {
    PageMatrix {
        family: family.into(),
        measured_urls: 0,
        verdict: SafetyVerdict::Unmeasured,
    }
}

/// Flags city/service variants whose remaining facts are identical after the city token is removed.
#[must_use]
pub fn thin_city_variants(inventory: &Inventory) -> Vec<Finding> {
    let mut families: BTreeMap<String, BTreeMap<ContentHash, Vec<String>>> = BTreeMap::new();
    for page in inventory.pages.iter().filter(|page| {
        page.status == 200
            && page.indexability == Indexability::Indexable
            && !page.text.trim().is_empty()
    }) {
        let Some((family, city)) = city_family(page.url.path()) else {
            continue;
        };
        let stripped = strip_city(&page.text, &city);
        if stripped.is_empty() {
            continue;
        }
        families
            .entry(family)
            .or_default()
            .entry(ContentHash::of_str(&stripped))
            .or_default()
            .push(page.url.to_string());
    }
    let mut findings = Vec::new();
    for (family, groups) in families {
        for urls in groups.into_values().filter(|urls| urls.len() > 1) {
            let subject = format!("{family}:{}", urls.join(" "));
            findings.push(
                Finding::new(
                    FindingFamily::Prog,
                    2,
                    Severity::Warn,
                    &subject,
                    format!(
                        "{} city variants in {family} share the same facts after the city token is removed",
                        urls.len()
                    ),
                    Locator::Url(urls[0].clone()),
                    Evidence::http(),
                )
                .with_affected(urls)
                .explained(
                    "Programmatic URLs must carry unique facts, not only a swapped city name.",
                    "Add unique local facts or consolidate the matrix.",
                    "Each city URL has distinct remaining content after the city token is stripped.",
                ),
            );
        }
    }
    findings
}

fn city_family(path: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = path.split('/').filter(|part| !part.is_empty()).collect();
    let rest = if parts
        .first()
        .is_some_and(|part| matches!(*part, "en" | "ru" | "he" | "es" | "fr" | "de"))
    {
        parts.get(1..)?
    } else {
        &parts
    };
    match rest {
        ["category" | "services", slug, city] if looks_like_city(city) => {
            Some((format!("category/{slug}"), (*city).to_owned()))
        }
        _ => None,
    }
}

fn looks_like_city(segment: &str) -> bool {
    if matches!(
        segment,
        "prices" | "reviews" | "about" | "new" | "edit" | "index" | "all"
    ) {
        return false;
    }
    segment.contains('-') || segment.len() >= 4
}

fn strip_city(text: &str, city: &str) -> String {
    let mut hay = text.to_ascii_lowercase();
    let slug = city.replace('-', " ");
    for token in [city, slug.as_str()] {
        let needle = token.to_ascii_lowercase();
        while let Some(at) = hay.find(&needle) {
            hay.replace_range(at..at + needle.len(), " ");
        }
    }
    if let Some((name, region)) = city.rsplit_once('-')
        && region.len() == 2
    {
        let name = name.replace('-', " ");
        while let Some(at) = hay.find(&name) {
            hay.replace_range(at..at + name.len(), " ");
        }
    }
    hay.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::{city_family, strip_city};

    #[test]
    fn strips_city_token_from_shared_copy() {
        let family = city_family("/category/electrician/vancouver-wa");
        assert_eq!(
            family
                .as_ref()
                .map(|(key, city)| (key.as_str(), city.as_str())),
            Some(("category/electrician", "vancouver-wa"))
        );
        let left = strip_city(
            "Electrician in Vancouver WA. Licensed. Same facts.",
            "vancouver-wa",
        );
        let right = strip_city("Electrician in Camas WA. Licensed. Same facts.", "camas-wa");
        assert_eq!(left, right);
    }
}

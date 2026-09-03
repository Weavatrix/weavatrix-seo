//! Intent / query-fanout coverage from page copy.

use crate::chunk::chunks;
use weavatrix_seo_model::{IntentCoverage, Inventory};

const FANOUT: &[(&str, &[&str])] = &[
    (
        "local service",
        &[
            "price",
            "how long",
            "warranty",
            "same-day",
            "service area",
            "licensed",
            "insured",
            "permit",
        ],
    ),
    (
        "contractor verification",
        &[
            "license",
            "bonded",
            "insurance",
            "reviews",
            "years of experience",
        ],
    ),
];

/// Maps common service questions onto measured chunks.
#[must_use]
pub fn fanout(inventory: &Inventory) -> Vec<IntentCoverage> {
    let chunks = chunks(inventory);
    let hay: String = chunks
        .iter()
        .map(|chunk| chunk.text.to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join("\n");
    FANOUT
        .iter()
        .map(|(intent, questions)| {
            let mut answered = Vec::new();
            let mut missing = Vec::new();
            for question in *questions {
                if hay.contains(question) {
                    answered.push((*question).to_owned());
                } else {
                    missing.push((*question).to_owned());
                }
            }
            let coverage = if questions.is_empty() {
                "UNMEASURED".into()
            } else {
                format!("{}/{}", answered.len(), questions.len())
            };
            IntentCoverage {
                intent: (*intent).to_owned(),
                questions: questions.iter().map(|item| (*item).to_owned()).collect(),
                answered,
                missing,
                coverage,
                subject: None,
                subject_kind: None,
            }
        })
        .collect()
}

/// Fanout against one URL or family haystack. Additive to the site-wide map.
#[must_use]
pub fn fanout_subject(subject: &str, subject_kind: &str, hay: &str) -> Vec<IntentCoverage> {
    let hay = hay.to_ascii_lowercase();
    FANOUT
        .iter()
        .map(|(intent, questions)| {
            let mut answered = Vec::new();
            let mut missing = Vec::new();
            for question in *questions {
                if hay.contains(question) {
                    answered.push((*question).to_owned());
                } else {
                    missing.push((*question).to_owned());
                }
            }
            let coverage = format!("{}/{}", answered.len(), questions.len());
            IntentCoverage {
                intent: (*intent).to_owned(),
                questions: questions.iter().map(|item| (*item).to_owned()).collect(),
                answered,
                missing,
                coverage,
                subject: Some(subject.to_owned()),
                subject_kind: Some(subject_kind.to_owned()),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{FANOUT, fanout_subject};

    #[test]
    fn local_service_fanout_is_declared() {
        assert!(FANOUT.iter().any(|(name, _)| *name == "local service"));
    }

    #[test]
    fn fanout_subject_tags_the_scope() {
        let rows = fanout_subject(
            "category/electrician",
            "route_family",
            "licensed insured permit same-day",
        );
        assert!(
            rows.iter()
                .any(|row| row.subject.as_deref() == Some("category/electrician")
                    && row.subject_kind.as_deref() == Some("route_family"))
        );
        assert!(
            rows.iter()
                .any(|row| row.answered.contains(&"licensed".into()))
        );
    }
}

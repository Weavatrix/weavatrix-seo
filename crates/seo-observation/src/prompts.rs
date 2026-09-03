//! AI-visibility prompt imports. File only; no vendor API client.

use crate::{Observation, ObservationKind, ObservationSnapshot};
use weavatrix_seo_model::{
    Evidence, EvidenceKind, EvidenceSource, Finding, FindingFamily, Locator, PromptObservation,
};

/// Turns prompt rows into citations plus a prompt inventory.
#[must_use]
pub fn expand(
    prompts: Vec<PromptObservation>,
    provider: &str,
    evidence: &Evidence,
) -> (Vec<Observation>, Vec<PromptObservation>) {
    let mut rows = Vec::new();
    for prompt in &prompts {
        for url in &prompt.cited_urls {
            rows.push(Observation {
                kind: ObservationKind::AiCitation,
                query: Some(prompt.prompt.clone()),
                url: url.clone(),
                provider: provider.to_owned(),
                evidence: evidence.clone(),
                clicks: 0,
                impressions: 0,
                hits: 1,
                position: prompt.brand_position.map(f32::from),
                period: prompt.period.clone(),
                user_agent: None,
                status: None,
                bot_role: None,
                verified_bot: None,
                referer: None,
                volume: 0,
                difficulty: None,
                serp_features: Vec::new(),
                referring_domains: None,
            });
        }
        if prompt.cited_urls.is_empty() {
            rows.push(Observation {
                kind: ObservationKind::AiPrompt,
                query: Some(prompt.prompt.clone()),
                url: format!("prompt:{}", prompt.platform),
                provider: provider.to_owned(),
                evidence: evidence.clone(),
                clicks: 0,
                impressions: 0,
                hits: 0,
                position: prompt.brand_position.map(f32::from),
                period: prompt.period.clone(),
                user_agent: None,
                status: None,
                bot_role: None,
                verified_bot: None,
                referer: None,
                volume: 0,
                difficulty: None,
                serp_features: Vec::new(),
                referring_domains: None,
            });
        }
    }
    (rows, prompts)
}

/// Citations present in a previous prompt window and missing now.
#[must_use]
pub fn citation_drops(snapshot: &ObservationSnapshot) -> Vec<Finding> {
    let previous: Vec<&PromptObservation> = snapshot
        .prompts
        .iter()
        .filter(|item| period_is_previous(item.period.as_deref()))
        .collect();
    if previous.is_empty() {
        return Vec::new();
    }
    let current: Vec<&PromptObservation> = snapshot
        .prompts
        .iter()
        .filter(|item| !period_is_previous(item.period.as_deref()))
        .collect();
    let mut findings = Vec::new();
    for prior in previous {
        for url in &prior.cited_urls {
            let still = current.iter().any(|item| {
                item.prompt == prior.prompt && item.cited_urls.iter().any(|cited| cited == url)
            });
            if still {
                continue;
            }
            findings.push(
                Finding::from_rule(
                    FindingFamily::Obs,
                    12,
                    url,
                    format!(
                        "citation of {url} dropped for prompt `{}` on {}",
                        prior.prompt, prior.platform
                    ),
                    Locator::Url(url.clone()),
                    Evidence {
                        kind: EvidenceKind::Observed,
                        source: EvidenceSource::Provider,
                        confidence: weavatrix_seo_model::Confidence::Medium,
                        snapshot_id: None,
                        revision: None,
                        policy_version: None,
                    },
                )
                .explained(
                    "The previous AI-visibility window cited this URL; the current window does not.",
                    "Diff the producer revision and the answering chunk for this prompt.",
                    "A later prompt import cites the URL again, or the prompt is abandoned.",
                ),
            );
        }
    }
    findings
}

fn period_is_previous(period: Option<&str>) -> bool {
    period.is_some_and(|value| {
        let lower = value.to_ascii_lowercase();
        lower.contains("prev") || lower.contains("prior") || lower.contains("before")
    })
}

#[cfg(test)]
mod tests {
    use super::{citation_drops, expand};
    use crate::ObservationKind;
    use weavatrix_seo_model::{
        Evidence, EvidenceKind, EvidenceSource, InputState, PromptObservation,
    };

    #[test]
    fn cited_urls_become_citation_rows() {
        let prompt = PromptObservation {
            prompt: "best electrician".into(),
            platform: "chatgpt".into(),
            cited_urls: vec!["https://x.test/a".into()],
            ..PromptObservation::default()
        };
        let evidence = Evidence {
            kind: EvidenceKind::Observed,
            source: EvidenceSource::Provider,
            confidence: weavatrix_seo_model::Confidence::High,
            snapshot_id: None,
            revision: None,
            policy_version: None,
        };
        let (rows, prompts) = expand(vec![prompt], "semrush-ai", &evidence);
        assert_eq!(rows[0].kind, ObservationKind::AiCitation);
        assert_eq!(rows[0].url, "https://x.test/a");
        assert_eq!(prompts.len(), 1);
    }

    #[test]
    fn a_lost_citation_is_obs_012() {
        let previous = PromptObservation {
            prompt: "best electrician".into(),
            platform: "chatgpt".into(),
            cited_urls: vec!["https://x.test/a".into()],
            period: Some("previous".into()),
            ..PromptObservation::default()
        };
        let current = PromptObservation {
            prompt: "best electrician".into(),
            platform: "chatgpt".into(),
            cited_urls: vec!["https://other.test/b".into()],
            period: Some("current".into()),
            ..PromptObservation::default()
        };
        let snapshot = crate::ObservationSnapshot {
            rows: Vec::new(),
            connected: true,
            input: InputState::connected("prompt"),
            prompts: vec![previous, current],
        };
        let findings = citation_drops(&snapshot);
        assert!(findings.iter().any(|item| item.code == "WVX-SEO-OBS-012"));
    }
}

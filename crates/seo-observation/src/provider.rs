//! Observation providers. File imports only; no vendor crawlers.

use crate::{Observation, ObservationKind, ObservationSnapshot};
use serde::Deserialize;
use weavatrix_seo_model::{Evidence, EvidenceKind, InputState};

#[derive(Debug, Deserialize)]
struct File {
    #[serde(default)]
    provider: Option<String>,
    /// Default kind for every row in the file.
    #[serde(default)]
    kind: Option<String>,
    /// Site origin used to absolutize combined-log paths.
    #[serde(default)]
    origin: Option<String>,
    /// `combined` for nginx/Apache lines in `lines` / `log`.
    #[serde(default)]
    format: Option<String>,
    #[serde(default)]
    log: Option<String>,
    #[serde(default)]
    lines: Vec<String>,
    #[serde(default)]
    rows: Vec<Row>,
    /// Keyword-tool rows. Always `keyword_volume`, never GSC demand.
    #[serde(default)]
    keywords: Vec<Row>,
    /// SERP snapshot rows.
    #[serde(default)]
    serp: Vec<Row>,
    /// Backlink / referring-domain rows.
    #[serde(default)]
    backlinks: Vec<Row>,
    #[serde(default)]
    prompts: Vec<weavatrix_seo_model::PromptObservation>,
}

#[derive(Debug, Deserialize)]
struct Row {
    #[serde(default)]
    query: Option<String>,
    url: String,
    #[serde(default)]
    clicks: u32,
    #[serde(default)]
    impressions: u32,
    #[serde(default)]
    hits: u32,
    #[serde(default)]
    position: Option<f32>,
    #[serde(default)]
    provider: Option<String>,
    /// Row-level kind. Wins over the file and the provider name.
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    period: Option<String>,
    #[serde(default)]
    user_agent: Option<String>,
    #[serde(default)]
    status: Option<u16>,
    #[serde(default)]
    referer: Option<String>,
    #[serde(default)]
    volume: u32,
    #[serde(default)]
    search_volume: u32,
    #[serde(default)]
    difficulty: Option<u16>,
    #[serde(default)]
    keyword_difficulty: Option<u16>,
    #[serde(default)]
    features: Vec<String>,
    #[serde(default)]
    referring_domains: Option<u32>,
    #[serde(default)]
    source_url: Option<String>,
    #[serde(default)]
    backlinks: u32,
}

/// Loads GSC, Bing, bot-log, analytics, or AI-citation JSON.
///
/// An unknown provider stays labelled and is never faked as search demand.
///
/// # Errors
///
/// Returns IO or JSON errors.
pub fn load_any(path: &str) -> Result<ObservationSnapshot, String> {
    let raw = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    from_any(&raw)
}

/// Parses a provider export in memory.
///
/// # Errors
///
/// Returns JSON errors.
#[allow(clippy::too_many_lines)]
pub fn from_any(raw: &str) -> Result<ObservationSnapshot, String> {
    let file: File = blazingly_json::from_str(weavatrix_seo_model::strip_bom(raw))
        .map_err(|error| error.to_string())?;
    let default_provider = file
        .provider
        .as_deref()
        .unwrap_or("gsc")
        .to_ascii_lowercase();
    let file_kind = file.kind.as_deref().and_then(ObservationKind::parse);
    let mut rows: Vec<Observation> = file
        .rows
        .into_iter()
        .map(|row| {
            let provider = row
                .provider
                .clone()
                .unwrap_or_else(|| default_provider.clone())
                .to_ascii_lowercase();
            let classified = row
                .user_agent
                .as_deref()
                .and_then(crate::logs::classify_agent);
            let mut kind = row
                .kind
                .as_deref()
                .and_then(ObservationKind::parse)
                .or(file_kind)
                .unwrap_or_else(|| ObservationKind::from_provider(&provider));
            if classified.is_some() && kind == ObservationKind::Analytics {
                kind = ObservationKind::BotCrawl;
            }
            observation_from_row(kind, provider, row, classified)
        })
        .collect();
    rows.extend(file.keywords.into_iter().map(|row| {
        let provider = row
            .provider
            .clone()
            .unwrap_or_else(|| default_provider.clone())
            .to_ascii_lowercase();
        let kind = row
            .kind
            .as_deref()
            .and_then(ObservationKind::parse)
            .unwrap_or(ObservationKind::KeywordVolume);
        observation_from_row(kind, provider, row, None)
    }));
    rows.extend(file.serp.into_iter().map(|row| {
        let provider = row
            .provider
            .clone()
            .unwrap_or_else(|| default_provider.clone())
            .to_ascii_lowercase();
        let kind = row
            .kind
            .as_deref()
            .and_then(ObservationKind::parse)
            .unwrap_or(ObservationKind::SerpPosition);
        observation_from_row(kind, provider, row, None)
    }));
    rows.extend(file.backlinks.into_iter().map(|row| {
        let provider = row
            .provider
            .clone()
            .unwrap_or_else(|| default_provider.clone())
            .to_ascii_lowercase();
        let kind = row
            .kind
            .as_deref()
            .and_then(ObservationKind::parse)
            .unwrap_or(ObservationKind::Backlink);
        observation_from_row(kind, provider, row, None)
    }));
    let combined_format = file
        .format
        .as_deref()
        .is_some_and(|format| format.eq_ignore_ascii_case("combined"));
    if combined_format || !file.lines.is_empty() || file.log.is_some() {
        let origin = file.origin.as_deref().unwrap_or("");
        let mut lines = file.lines;
        if let Some(log) = file.log {
            lines.extend(log.lines().map(str::to_owned));
        }
        rows.extend(crate::logs::from_combined(
            origin,
            &lines,
            &default_provider,
        ));
    }
    let prompt_evidence = Evidence {
        kind: EvidenceKind::Observed,
        source: weavatrix_seo_model::EvidenceSource::Provider,
        confidence: weavatrix_seo_model::Confidence::High,
        snapshot_id: None,
        revision: None,
        policy_version: None,
    };
    let (prompt_rows, prompts) =
        crate::prompts::expand(file.prompts, &default_provider, &prompt_evidence);
    rows.extend(prompt_rows);
    let input = if rows.is_empty() && prompts.is_empty() {
        InputState::empty("GSC")
    } else {
        InputState::connected("GSC")
    };
    Ok(ObservationSnapshot {
        rows,
        connected: true,
        input,
        prompts,
    })
}

fn observation_from_row(
    kind: ObservationKind,
    provider: String,
    row: Row,
    classified: Option<crate::logs::ClassifiedBot>,
) -> Observation {
    let source = kind.source(&provider);
    let evidence_kind = if kind.is_external_market() {
        EvidenceKind::External
    } else {
        EvidenceKind::Observed
    };
    let confidence = if kind.is_external_market() {
        weavatrix_seo_model::Confidence::Medium
    } else {
        weavatrix_seo_model::Confidence::High
    };
    let volume = if kind == ObservationKind::KeywordVolume {
        row.volume.max(row.search_volume).max(row.impressions)
    } else {
        0
    };
    let hits = if kind.is_search_demand() {
        0
    } else if kind == ObservationKind::Backlink {
        row.backlinks.max(row.hits).max(row.impressions).max(1)
    } else if kind == ObservationKind::KeywordVolume {
        0
    } else {
        row.hits.max(row.impressions)
    };
    Observation {
        kind,
        query: row.query,
        url: row.url,
        provider,
        evidence: Evidence {
            kind: evidence_kind,
            source,
            confidence,
            snapshot_id: None,
            revision: None,
            policy_version: None,
        },
        clicks: row.clicks,
        impressions: if kind.is_search_demand() {
            row.impressions
        } else {
            0
        },
        hits,
        position: row.position,
        period: row.period,
        user_agent: row.user_agent,
        status: row.status,
        bot_role: classified.map(|bot| bot.role.to_owned()),
        verified_bot: classified.map(|bot| bot.verified),
        referer: row.referer.or(row.source_url),
        volume,
        difficulty: row.difficulty.or(row.keyword_difficulty),
        serp_features: row.features,
        referring_domains: row.referring_domains,
    }
}

#[cfg(test)]
mod tests {
    use super::from_any;
    use crate::ObservationKind;
    use weavatrix_seo_model::EvidenceSource;

    #[test]
    fn bing_is_search_performance_and_keeps_its_provider() {
        let bing =
            from_any(r#"{"provider":"bing","rows":[{"url":"https://x.test/","impressions":9}]}"#)
                .expect("bing");
        assert_eq!(bing.rows[0].provider, "bing");
        assert_eq!(bing.rows[0].kind, ObservationKind::SearchPerformance);
        assert_eq!(bing.rows[0].impressions, 9);
        assert_eq!(bing.rows[0].evidence.source, EvidenceSource::Provider);
    }

    #[test]
    fn log_hits_stay_hits() {
        let logs = from_any(r#"{"provider":"logs","rows":[{"url":"https://x.test/","hits":40}]}"#)
            .expect("logs");
        assert_eq!(logs.rows[0].kind, ObservationKind::BotCrawl);
        assert_eq!(logs.rows[0].hits, 40);
        assert_eq!(
            logs.rows[0].impressions, 0,
            "crawler activity is not a search impression"
        );
        assert_eq!(logs.rows[0].evidence.source, EvidenceSource::Logs);
    }

    #[test]
    fn a_log_row_labelled_as_impressions_is_still_a_hit() {
        let logs =
            from_any(r#"{"provider":"logs","rows":[{"url":"https://x.test/","impressions":40}]}"#)
                .expect("logs");
        assert_eq!(logs.rows[0].hits, 40);
        assert_eq!(logs.rows[0].impressions, 0);
    }

    #[test]
    fn combined_nginx_lines_become_bot_crawl() {
        let snap = from_any(
            r#"{"provider":"nginx","origin":"https://x.test","format":"combined","lines":["1.1.1.1 - - [03/Sep/2026:10:00:00 +0000] \"GET /a HTTP/1.1\" 200 10 \"-\" \"ChatGPT-User/1.0\""]}"#,
        )
        .expect("nginx");
        assert_eq!(snap.rows[0].kind, ObservationKind::BotCrawl);
        assert_eq!(snap.rows[0].bot_role.as_deref(), Some("citation_fetch"));
        assert_eq!(snap.rows[0].url, "https://x.test/a");
    }

    #[test]
    fn prompt_file_expands_citations() {
        let snap = from_any(
            r#"{"provider":"semrush-ai","prompts":[{"prompt":"best electrician","platform":"chatgpt","cited_urls":["https://x.test/a"]}]}"#,
        )
        .expect("prompt");
        assert_eq!(snap.prompts.len(), 1);
        assert_eq!(snap.rows[0].kind, ObservationKind::AiCitation);
        assert_eq!(snap.rows[0].url, "https://x.test/a");
    }

    #[test]
    fn keyword_json_is_external_and_not_search_demand() {
        let snap = from_any(
            r#"{"provider":"semrush","keywords":[{"query":"electrician vancouver","url":"https://x.test/a","volume":2400,"difficulty":47}]}"#,
        )
        .expect("semrush");
        assert_eq!(snap.rows[0].kind, ObservationKind::KeywordVolume);
        assert_eq!(snap.rows[0].volume, 2400);
        assert_eq!(snap.rows[0].impressions, 0);
        assert_eq!(snap.rows[0].hits, 0);
        assert_eq!(snap.rows[0].difficulty, Some(47));
        assert_eq!(
            snap.rows[0].evidence.kind,
            weavatrix_seo_model::EvidenceKind::External
        );
        assert!(!snap.rows[0].kind.is_search_demand());
    }

    #[test]
    fn keyword_volume_on_rows_does_not_become_impressions() {
        let snap = from_any(
            r#"{"provider":"ahrefs","rows":[{"url":"https://x.test/a","impressions":900,"volume":900}]}"#,
        )
        .expect("ahrefs");
        assert_eq!(snap.rows[0].kind, ObservationKind::KeywordVolume);
        assert_eq!(snap.rows[0].volume, 900);
        assert_eq!(snap.rows[0].impressions, 0);
    }
}

//! External keyword / SERP / backlink contracts. File imports only.

use crate::{Observation, ObservationKind};
use weavatrix_seo_model::{Evidence, EvidenceKind, EvidenceSource};

/// One keyword-tool volume row. Volume is not Search Console demand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeywordRecord {
    /// Query text when known.
    pub query: Option<String>,
    /// URL this keyword is attached to, when the export named one.
    pub url: String,
    /// Monthly search volume from the vendor.
    pub volume: u32,
    /// Keyword difficulty 0–100 when supplied.
    pub difficulty: Option<u16>,
}

/// One SERP snapshot row.
#[derive(Debug, Clone, PartialEq)]
pub struct SerpRecord {
    /// Query.
    pub query: Option<String>,
    /// Ranking URL (ours or a competitor).
    pub url: String,
    /// Position when the vendor supplied it.
    pub position: Option<f32>,
    /// Feature labels: `paa`, `ai_overview`, `featured_snippet`, …
    pub features: Vec<String>,
}

/// One backlink / referring-domain row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BacklinkRecord {
    /// Target URL on the audited site.
    pub url: String,
    /// Linking page when the export named it.
    pub source_url: Option<String>,
    /// Backlink count.
    pub backlinks: u32,
    /// Referring domains when supplied.
    pub referring_domains: Option<u32>,
}

/// Keyword-volume import. Implementations must not present volume as GSC demand.
pub trait KeywordProvider {
    /// Keyword rows.
    fn keywords(&self) -> &[KeywordRecord];
}

/// SERP snapshot import.
pub trait SerpProvider {
    /// SERP rows.
    fn serp(&self) -> &[SerpRecord];
}

/// Backlink import.
pub trait BacklinkProvider {
    /// Backlink rows.
    fn backlinks(&self) -> &[BacklinkRecord];
}

/// Combined market import (`DataForSEO` / Semrush / Ahrefs / custom JSON).
pub trait MarketProvider: KeywordProvider + SerpProvider + BacklinkProvider {
    /// Vendor label stored on every observation.
    fn provider_name(&self) -> &str;
}

/// Parsed custom JSON market file. No vendor API client.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct JsonMarket {
    /// Provider name (`semrush`, `ahrefs`, `dataforseo`, `custom`, …).
    pub provider: String,
    /// Keyword rows.
    pub keywords: Vec<KeywordRecord>,
    /// SERP rows.
    pub serp: Vec<SerpRecord>,
    /// Backlink rows.
    pub backlinks: Vec<BacklinkRecord>,
}

impl KeywordProvider for JsonMarket {
    fn keywords(&self) -> &[KeywordRecord] {
        &self.keywords
    }
}

impl SerpProvider for JsonMarket {
    fn serp(&self) -> &[SerpRecord] {
        &self.serp
    }
}

impl BacklinkProvider for JsonMarket {
    fn backlinks(&self) -> &[BacklinkRecord] {
        &self.backlinks
    }
}

impl MarketProvider for JsonMarket {
    fn provider_name(&self) -> &str {
        &self.provider
    }
}

/// Turns a market import into observation rows labelled `EXTERNAL`.
#[must_use]
pub fn observations(market: &impl MarketProvider) -> Vec<Observation> {
    let provider = market.provider_name().to_ascii_lowercase();
    let evidence = Evidence {
        kind: EvidenceKind::External,
        source: EvidenceSource::Provider,
        confidence: weavatrix_seo_model::Confidence::Medium,
        snapshot_id: None,
        revision: None,
        policy_version: None,
    };
    let mut rows = Vec::new();
    for item in market.keywords() {
        rows.push(Observation {
            kind: ObservationKind::KeywordVolume,
            query: item.query.clone(),
            url: item.url.clone(),
            provider: provider.clone(),
            evidence: evidence.clone(),
            clicks: 0,
            impressions: 0,
            hits: 0,
            position: None,
            period: None,
            user_agent: None,
            status: None,
            bot_role: None,
            verified_bot: None,
            referer: None,
            volume: item.volume,
            difficulty: item.difficulty,
            serp_features: Vec::new(),
            referring_domains: None,
        });
    }
    for item in market.serp() {
        let kind = if item.position.is_none() && !item.features.is_empty() {
            ObservationKind::SerpFeature
        } else {
            ObservationKind::SerpPosition
        };
        rows.push(Observation {
            kind,
            query: item.query.clone(),
            url: item.url.clone(),
            provider: provider.clone(),
            evidence: evidence.clone(),
            clicks: 0,
            impressions: 0,
            hits: 0,
            position: item.position,
            period: None,
            user_agent: None,
            status: None,
            bot_role: None,
            verified_bot: None,
            referer: None,
            volume: 0,
            difficulty: None,
            serp_features: item.features.clone(),
            referring_domains: None,
        });
    }
    for item in market.backlinks() {
        rows.push(Observation {
            kind: ObservationKind::Backlink,
            query: item.source_url.clone(),
            url: item.url.clone(),
            provider: provider.clone(),
            evidence: evidence.clone(),
            clicks: 0,
            impressions: 0,
            hits: item.backlinks.max(1),
            position: None,
            period: None,
            user_agent: None,
            status: None,
            bot_role: None,
            verified_bot: None,
            referer: item.source_url.clone(),
            volume: 0,
            difficulty: None,
            serp_features: Vec::new(),
            referring_domains: item.referring_domains,
        });
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::{
        BacklinkRecord, JsonMarket, KeywordRecord, MarketProvider, SerpRecord, observations,
    };
    use crate::ObservationKind;
    use weavatrix_seo_model::EvidenceKind;

    #[test]
    fn keyword_volume_is_external_and_not_demand() {
        let market = JsonMarket {
            provider: "semrush".into(),
            keywords: vec![KeywordRecord {
                query: Some("electrician vancouver".into()),
                url: "https://x.test/a".into(),
                volume: 2400,
                difficulty: Some(47),
            }],
            serp: vec![SerpRecord {
                query: Some("electrician vancouver".into()),
                url: "https://competitor.test/".into(),
                position: Some(1.0),
                features: vec!["paa".into(), "ai_overview".into()],
            }],
            backlinks: vec![BacklinkRecord {
                url: "https://x.test/a".into(),
                source_url: Some("https://news.test/article".into()),
                backlinks: 3,
                referring_domains: Some(12),
            }],
        };
        assert_eq!(market.provider_name(), "semrush");
        let rows = observations(&market);
        assert_eq!(rows[0].kind, ObservationKind::KeywordVolume);
        assert_eq!(rows[0].volume, 2400);
        assert_eq!(rows[0].impressions, 0);
        assert_eq!(rows[0].evidence.kind, EvidenceKind::External);
        assert!(!rows[0].kind.is_search_demand());
        assert_eq!(rows[1].kind, ObservationKind::SerpPosition);
        assert_eq!(rows[1].serp_features, ["paa", "ai_overview"]);
        assert_eq!(rows[2].kind, ObservationKind::Backlink);
        assert_eq!(rows[2].referring_domains, Some(12));
    }
}

//! What a provider row actually measured.

use serde::{Deserialize, Serialize};
use weavatrix_seo_model::EvidenceSource;

/// Observation kinds. Each has its own meaning and its own aggregator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationKind {
    /// Query impressions, clicks, and average position from a search engine.
    SearchPerformance,
    /// Crawler or bot requests from server and CDN logs.
    BotCrawl,
    /// A generative-search answer that cited the URL.
    AiCitation,
    /// An AI-visibility prompt (brand mention / citation tracking). Not demand.
    AiPrompt,
    /// A click or session referred from an AI answer.
    AiReferral,
    /// A measured SERP position for one query.
    SerpPosition,
    /// Keyword-tool search volume. Never Search Console demand.
    KeywordVolume,
    /// Referring-domain / backlink import. External, not demand.
    Backlink,
    /// A SERP feature (PAA, AI Overview, featured snippet) on a query.
    SerpFeature,
    /// Site analytics sessions or pageviews.
    Analytics,
}

impl ObservationKind {
    /// Whether rows of this kind may contribute to search demand.
    #[must_use]
    pub const fn is_search_demand(self) -> bool {
        matches!(self, Self::SearchPerformance)
    }

    /// Third-party keyword / SERP / backlink imports. Evidence is `EXTERNAL`.
    #[must_use]
    pub const fn is_external_market(self) -> bool {
        matches!(
            self,
            Self::KeywordVolume | Self::SerpPosition | Self::Backlink | Self::SerpFeature
        )
    }

    /// Parses a declared kind from an import file.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().replace('-', "_").as_str() {
            "search_performance" | "search" => Some(Self::SearchPerformance),
            "bot_crawl" | "bot" | "crawl" => Some(Self::BotCrawl),
            "ai_citation" | "ai" => Some(Self::AiCitation),
            "ai_prompt" | "prompt" | "ai_visibility" => Some(Self::AiPrompt),
            "ai_referral" | "ai_click" | "referral" => Some(Self::AiReferral),
            "serp_position" | "serp" => Some(Self::SerpPosition),
            "keyword_volume" | "keyword" | "keywords" | "search_volume" => {
                Some(Self::KeywordVolume)
            }
            "backlink" | "backlinks" => Some(Self::Backlink),
            "serp_feature" | "paa" | "ai_overview" => Some(Self::SerpFeature),
            "analytics" => Some(Self::Analytics),
            _ => None,
        }
    }

    /// Kind implied by a known provider name.
    ///
    /// An unrecognised provider stays [`Self::Analytics`]: it is a real
    /// observation, but nothing licenses reading it as search demand. Declare
    /// `kind` in the import to say otherwise.
    #[must_use]
    pub fn from_provider(provider: &str) -> Self {
        match provider {
            "gsc" | "search-console" | "bing" | "yandex" => Self::SearchPerformance,
            "logs" | "cdn" | "bot-logs" | "nginx" | "apache" | "cloudflare" | "fastly"
            | "vercel" => Self::BotCrawl,
            "chatgpt" | "perplexity" | "gemini" | "copilot" | "ai-search" => Self::AiCitation,
            "semrush-ai" | "ai-visibility" | "prompt" => Self::AiPrompt,
            "chatgpt-user" | "claude-user" | "ai-referral" => Self::AiReferral,
            "serp" | "serpapi" | "serpstat" => Self::SerpPosition,
            "semrush" | "ahrefs" | "dataforseo" | "keywords" | "keyword" | "moz" => {
                Self::KeywordVolume
            }
            "backlinks" | "backlink" | "majestic" => Self::Backlink,
            "paa" | "serp-features" => Self::SerpFeature,
            _ => Self::Analytics,
        }
    }

    /// Evidence source for this kind and provider.
    #[must_use]
    pub fn source(self, provider: &str) -> EvidenceSource {
        match (self, provider) {
            (_, "gsc" | "search-console") => EvidenceSource::Gsc,
            (Self::BotCrawl, _) => EvidenceSource::Logs,
            _ => EvidenceSource::Provider,
        }
    }
}

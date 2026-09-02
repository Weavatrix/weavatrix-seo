//! AI crawler tokens are not one role.

use crate::RuleAuthority;

/// What an AI user-agent is documented to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiAgentRole {
    /// Builds or refreshes a search/discovery index.
    SearchDiscovery,
    /// Fetches a URL to cite it.
    CitationFetch,
    /// User-triggered live fetch.
    UserInitiatedFetch,
    /// Training corpus collection.
    Training,
    /// Grounding / extended-use control token.
    GroundingControl,
    /// Web archive.
    Archive,
    /// Unclassified.
    Other,
}

/// One documented AI crawler token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AiAgentDefinition {
    /// robots.txt token, lowercased.
    pub token: &'static str,
    /// Vendor.
    pub provider: &'static str,
    /// Documented roles.
    pub roles: &'static [AiAgentRole],
    /// Effect on search visibility when the origin is `Disallow: /`.
    pub search_visibility_effect: &'static str,
    /// Docs label, not a live fetch.
    pub docs_source: &'static str,
    /// Authority of that documentation.
    pub authority: RuleAuthority,
}

/// Known agents. Unknown tokens are not invented.
#[must_use]
pub fn lookup(token: &str) -> Option<&'static AiAgentDefinition> {
    let token = token.to_ascii_lowercase();
    AGENTS.iter().find(|agent| agent.token == token)
}

/// All known tokens.
#[must_use]
pub fn all() -> &'static [AiAgentDefinition] {
    AGENTS
}

const SEARCH: &[AiAgentRole] = &[AiAgentRole::SearchDiscovery];
const TRAIN: &[AiAgentRole] = &[AiAgentRole::Training];
const CITE: &[AiAgentRole] = &[AiAgentRole::CitationFetch, AiAgentRole::UserInitiatedFetch];
const GROUND: &[AiAgentRole] = &[AiAgentRole::GroundingControl, AiAgentRole::Training];

const AGENTS: &[AiAgentDefinition] = &[
    AiAgentDefinition {
        token: "gptbot",
        provider: "OpenAI",
        roles: TRAIN,
        search_visibility_effect: "none_established",
        docs_source: "openai.com/gptbot",
        authority: RuleAuthority::SearchEngineDocumented,
    },
    AiAgentDefinition {
        token: "oai-searchbot",
        provider: "OpenAI",
        roles: SEARCH,
        search_visibility_effect: "may_limit_chatgpt_search_discovery",
        docs_source: "openai.com/oai-searchbot",
        authority: RuleAuthority::SearchEngineDocumented,
    },
    AiAgentDefinition {
        token: "chatgpt-user",
        provider: "OpenAI",
        roles: CITE,
        search_visibility_effect: "may_limit_live_citation_fetch",
        docs_source: "openai.com/chatgpt-user",
        authority: RuleAuthority::SearchEngineDocumented,
    },
    AiAgentDefinition {
        token: "claudebot",
        provider: "Anthropic",
        roles: TRAIN,
        search_visibility_effect: "none_established",
        docs_source: "anthropic.com/claudebot",
        authority: RuleAuthority::SearchEngineDocumented,
    },
    AiAgentDefinition {
        token: "claude-searchbot",
        provider: "Anthropic",
        roles: SEARCH,
        search_visibility_effect: "may_limit_claude_search_discovery",
        docs_source: "anthropic.com/claude-searchbot",
        authority: RuleAuthority::SearchEngineDocumented,
    },
    AiAgentDefinition {
        token: "claude-user",
        provider: "Anthropic",
        roles: CITE,
        search_visibility_effect: "may_limit_live_citation_fetch",
        docs_source: "anthropic.com/claude-user",
        authority: RuleAuthority::SearchEngineDocumented,
    },
    AiAgentDefinition {
        token: "perplexitybot",
        provider: "Perplexity",
        roles: SEARCH,
        search_visibility_effect: "may_limit_perplexity_discovery",
        docs_source: "perplexity.ai/perplexitybot",
        authority: RuleAuthority::SearchEngineDocumented,
    },
    AiAgentDefinition {
        token: "google-extended",
        provider: "Google",
        roles: GROUND,
        search_visibility_effect: "none_on_google_web_search",
        docs_source: "developers.google.com/search/docs/crawling-indexing/overview-google-crawlers",
        authority: RuleAuthority::SearchEngineDocumented,
    },
    AiAgentDefinition {
        token: "google-agent",
        provider: "Google",
        roles: CITE,
        search_visibility_effect: "may_limit_agentic_fetch",
        docs_source: "developers.google.com/search",
        authority: RuleAuthority::ExperimentalHeuristic,
    },
    AiAgentDefinition {
        token: "applebot-extended",
        provider: "Apple",
        roles: TRAIN,
        search_visibility_effect: "none_established",
        docs_source: "apple.com/applebot",
        authority: RuleAuthority::SearchEngineDocumented,
    },
    AiAgentDefinition {
        token: "bytespider",
        provider: "ByteDance",
        roles: TRAIN,
        search_visibility_effect: "none_established",
        docs_source: "bytespider",
        authority: RuleAuthority::ExperimentalHeuristic,
    },
    AiAgentDefinition {
        token: "ccbot",
        provider: "Common Crawl",
        roles: &[AiAgentRole::Archive],
        search_visibility_effect: "none_established",
        docs_source: "commoncrawl.org/ccbot",
        authority: RuleAuthority::SearchEngineDocumented,
    },
    AiAgentDefinition {
        token: "anthropic-ai",
        provider: "Anthropic",
        roles: TRAIN,
        search_visibility_effect: "none_established",
        docs_source: "anthropic.com",
        authority: RuleAuthority::SearchEngineDocumented,
    },
];

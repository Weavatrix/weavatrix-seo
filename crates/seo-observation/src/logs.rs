//! Server-log intelligence. Combined-log lines and JSON rows share one bot model.

use crate::{Observation, ObservationKind, ObservationSnapshot};
use std::collections::BTreeMap;
use weavatrix_seo_model::{
    AiAgentRole, Evidence, EvidenceKind, EvidenceSource, Finding, FindingFamily, Indexability,
    Inventory, Locator,
};

/// Classifies a User-Agent into a documented bot role.
#[must_use]
pub fn classify_agent(user_agent: &str) -> Option<ClassifiedBot> {
    let lower = user_agent.to_ascii_lowercase();
    if lower.contains("googlebot") && !lower.contains("google-extended") {
        return Some(ClassifiedBot {
            token: "googlebot",
            role: "search_discovery",
            verified: lower.contains("google.com/bot.html"),
        });
    }
    if lower.contains("bingbot") {
        return Some(ClassifiedBot {
            token: "bingbot",
            role: "search_discovery",
            verified: lower.contains("bing.com/bingbot"),
        });
    }
    let mut agents: Vec<_> = weavatrix_seo_model::ai_agents().iter().collect();
    agents.sort_by_key(|agent| std::cmp::Reverse(agent.token.len()));
    for agent in agents {
        if lower.contains(agent.token) {
            let role = primary_role(agent.roles);
            return Some(ClassifiedBot {
                token: agent.token,
                role,
                verified: true,
            });
        }
    }
    if lower.contains("bot") || lower.contains("spider") || lower.contains("crawler") {
        return Some(ClassifiedBot {
            token: "unknown-bot",
            role: "other",
            verified: false,
        });
    }
    None
}

/// One recognised crawler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClassifiedBot {
    /// Token, lowercased.
    pub token: &'static str,
    /// `search_discovery`, `citation_fetch`, `training`, or `other`.
    pub role: &'static str,
    /// True when the UA matches a documented token or Google's bot URL.
    pub verified: bool,
}

fn primary_role(roles: &[AiAgentRole]) -> &'static str {
    if roles.contains(&AiAgentRole::SearchDiscovery) {
        "search_discovery"
    } else if roles.contains(&AiAgentRole::CitationFetch)
        || roles.contains(&AiAgentRole::UserInitiatedFetch)
    {
        "citation_fetch"
    } else if roles.contains(&AiAgentRole::Training) {
        "training"
    } else {
        "other"
    }
}

/// Parses nginx/Apache combined log lines into bot-crawl observations.
#[must_use]
pub fn from_combined(origin: &str, lines: &[String], provider: &str) -> Vec<Observation> {
    let mut rolled: BTreeMap<(String, String, u16), Observation> = BTreeMap::new();
    for line in lines {
        let Some(parsed) = parse_combined_line(line) else {
            continue;
        };
        let Some(bot) = classify_agent(&parsed.user_agent) else {
            continue;
        };
        let url = absolute_url(origin, &parsed.path);
        let key = (url.clone(), bot.role.to_owned(), parsed.status);
        let entry = rolled.entry(key).or_insert_with(|| Observation {
            kind: ObservationKind::BotCrawl,
            query: None,
            url,
            provider: provider.to_owned(),
            evidence: Evidence {
                kind: EvidenceKind::Observed,
                source: EvidenceSource::Logs,
                confidence: weavatrix_seo_model::Confidence::High,
                snapshot_id: None,
                revision: None,
                policy_version: None,
            },
            clicks: 0,
            impressions: 0,
            hits: 0,
            position: None,
            period: None,
            user_agent: Some(parsed.user_agent.clone()),
            status: Some(parsed.status),
            bot_role: Some(bot.role.to_owned()),
            verified_bot: Some(bot.verified),
            referer: parsed.referer.clone(),
            volume: 0,
            difficulty: None,
            serp_features: Vec::new(),
            referring_domains: None,
        });
        entry.hits = entry.hits.saturating_add(1);
    }
    rolled.into_values().collect()
}

struct CombinedHit {
    path: String,
    status: u16,
    user_agent: String,
    referer: Option<String>,
}

fn parse_combined_line(line: &str) -> Option<CombinedHit> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    let request_start = line.find(" \"")?;
    let after_open = request_start + 2;
    let request_end = line[after_open..].find('"')?;
    let request = &line[after_open..after_open + request_end];
    let mut req_parts = request.split_whitespace();
    let _method = req_parts.next()?;
    let path = req_parts.next()?.to_owned();
    let rest = line[after_open + request_end + 1..].trim();
    let status: u16 = rest.split_whitespace().next()?.parse().ok()?;
    let ua = last_quoted(line)?;
    let referer = second_last_quoted(line).filter(|value| value != "-");
    Some(CombinedHit {
        path,
        status,
        user_agent: ua,
        referer,
    })
}

fn last_quoted(line: &str) -> Option<String> {
    let end = line.rfind('"')?;
    let start = line[..end].rfind('"')?;
    Some(line[start + 1..end].to_owned())
}

fn second_last_quoted(line: &str) -> Option<String> {
    let ua_end = line.rfind('"')?;
    let ua_start = line[..ua_end].rfind('"')?;
    let end = line[..ua_start].rfind('"')?;
    let start = line[..end].rfind('"')?;
    Some(line[start + 1..end].to_owned())
}

fn absolute_url(origin: &str, path: &str) -> String {
    if path.starts_with("http://") || path.starts_with("https://") {
        return path.to_owned();
    }
    format!("{}{path}", origin.trim_end_matches('/'))
}

/// Findings from imported bot hits crossed with the crawl inventory.
#[must_use]
pub fn analyze(snapshot: &ObservationSnapshot, inventory: &Inventory) -> Vec<Finding> {
    if !snapshot.has(ObservationKind::BotCrawl) {
        return Vec::new();
    }
    let mut findings = Vec::new();
    findings.extend(error_hits(snapshot));
    findings.extend(noindex_hits(snapshot, inventory));
    findings.extend(waste_hits(snapshot, inventory));
    findings.extend(orphan_hits(snapshot, inventory));
    findings
}

fn error_hits(snapshot: &ObservationSnapshot) -> Vec<Finding> {
    snapshot
        .rows
        .iter()
        .filter(|row| row.kind == ObservationKind::BotCrawl)
        .filter(|row| row.hits > 0)
        .filter(|row| row.status.is_some_and(|status| status >= 400))
        .map(|row| {
            Finding::from_rule(
                FindingFamily::Obs,
                7,
                &row.url,
                format!(
                    "bots hit {} with HTTP {} ({} hits)",
                    row.url,
                    row.status.unwrap_or(0),
                    row.hits
                ),
                Locator::Url(row.url.clone()),
                row.evidence.clone(),
            )
            .explained(
                "Crawler budget spent on an error URL is wasted.",
                "Fix the status or remove internal and sitemap links to this URL.",
                "Bot hits to this URL return 200, or the URL is gone from logs.",
            )
        })
        .collect()
}

fn noindex_hits(snapshot: &ObservationSnapshot, inventory: &Inventory) -> Vec<Finding> {
    let noindex: Vec<String> = inventory
        .pages
        .iter()
        .filter(|page| page.indexability != Indexability::Indexable)
        .map(|page| page.url.to_string())
        .collect();
    snapshot
        .rows
        .iter()
        .filter(|row| row.kind == ObservationKind::BotCrawl && row.hits > 0)
        .filter(|row| {
            noindex
                .iter()
                .any(|url| url.trim_end_matches('/') == row.url.trim_end_matches('/'))
        })
        .map(|row| {
            Finding::from_rule(
                FindingFamily::Obs,
                8,
                &row.url,
                format!(
                    "bots still crawl noindex URL {} ({} hits)",
                    row.url, row.hits
                ),
                Locator::Url(row.url.clone()),
                row.evidence.clone(),
            )
            .explained(
                "A noindex URL that is still crawled spends budget without ranking.",
                "Drop internal links or return 404/410 if the URL should stay out of search.",
                "Bot hits fall, or the URL is indexable on purpose.",
            )
        })
        .collect()
}

fn waste_hits(snapshot: &ObservationSnapshot, inventory: &Inventory) -> Vec<Finding> {
    if !snapshot.has(ObservationKind::SearchPerformance) {
        return Vec::new();
    }
    snapshot
        .rows
        .iter()
        .filter(|row| row.kind == ObservationKind::BotCrawl && row.hits >= 20)
        .filter(|row| {
            let impressions: u32 = snapshot
                .rows
                .iter()
                .filter(|item| item.kind == ObservationKind::SearchPerformance)
                .filter(|item| item.url.trim_end_matches('/') == row.url.trim_end_matches('/'))
                .map(|item| item.impressions)
                .sum();
            impressions == 0
        })
        .filter(|row| {
            inventory
                .measured_urls()
                .iter()
                .any(|url| url.trim_end_matches('/') == row.url.trim_end_matches('/'))
        })
        .map(|row| {
            Finding::from_rule(
                FindingFamily::Obs,
                9,
                &row.url,
                format!(
                    "bots hit {} {} times with no imported search impressions",
                    row.url, row.hits
                ),
                Locator::Url(row.url.clone()),
                row.evidence.clone(),
            )
            .explained(
                "Crawl budget on a URL that search does not measure is the reverse of demand without hits.",
                "noindex, consolidate, or add a query this URL can uniquely answer.",
                "GSC impressions appear, or bot hits fall.",
            )
        })
        .collect()
}

fn orphan_hits(snapshot: &ObservationSnapshot, inventory: &Inventory) -> Vec<Finding> {
    snapshot
        .rows
        .iter()
        .filter(|row| row.kind == ObservationKind::BotCrawl && row.hits > 0)
        .filter(|row| {
            inventory.pages.iter().any(|page| {
                page.url.to_string().trim_end_matches('/') == row.url.trim_end_matches('/')
                    && page.indexability == Indexability::Indexable
                    && !page.linked_from_page
                    && page.url.path() != "/"
            })
        })
        .map(|row| {
            Finding::from_rule(
                FindingFamily::Obs,
                10,
                &row.url,
                format!(
                    "bots crawl orphan indexable URL {} ({} hits)",
                    row.url, row.hits
                ),
                Locator::Url(row.url.clone()),
                row.evidence.clone(),
            )
            .explained(
                "Search bots found a URL the internal graph does not.",
                "Add an internal link from a crawlable template, or noindex if it should stay private.",
                "The URL is internally reachable, or bots stop hitting it.",
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{classify_agent, from_combined, parse_combined_line};

    #[test]
    fn googlebot_is_search_discovery() {
        let bot = classify_agent(
            "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)",
        )
        .expect("googlebot");
        assert_eq!(bot.token, "googlebot");
        assert_eq!(bot.role, "search_discovery");
        assert!(bot.verified);
    }

    #[test]
    fn chatgpt_user_is_citation_fetch() {
        let bot = classify_agent("ChatGPT-User/1.0").expect("chatgpt-user");
        assert_eq!(bot.role, "citation_fetch");
    }

    #[test]
    fn browsers_are_not_bots() {
        assert!(classify_agent("Mozilla/5.0 Chrome/120.0.0.0").is_none());
    }

    #[test]
    fn combined_line_rolls_up_googlebot_404() {
        let line = r#"66.249.66.1 - - [03/Sep/2026:10:00:00 +0000] "GET /gone HTTP/1.1" 404 12 "-" "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)""#;
        let parsed = parse_combined_line(line).expect("parse");
        assert_eq!(parsed.path, "/gone");
        assert_eq!(parsed.status, 404);
        let rows = from_combined("https://x.test", &[line.into()], "nginx");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].url, "https://x.test/gone");
        assert_eq!(rows[0].hits, 1);
        assert_eq!(rows[0].status, Some(404));
        assert_eq!(rows[0].bot_role.as_deref(), Some("search_discovery"));
    }
}

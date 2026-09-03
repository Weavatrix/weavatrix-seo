//! robots.txt parser. Specific groups win over `User-agent: *`.

use weavatrix_seo_model::{AbsoluteUrl, AiAgentPolicy, AiAgentRole};

/// Parsed robots policy for one host.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Robots {
    groups: Vec<RobotsGroup>,
    allows: Vec<String>,
    disallows: Vec<String>,
    /// Sitemap URLs declared by the file.
    pub sitemaps: Vec<String>,
    /// Known AI user-agents whose matching group disallows `/`.
    pub ai_disallow_all: Vec<String>,
}

impl Robots {
    /// Parses a robots.txt body for `product`.
    #[must_use]
    pub fn parse(body: &str, product: &str) -> Self {
        let (groups, sitemaps) = parse_groups(body);
        let product = product.to_ascii_lowercase();
        let (allows, disallows) = merge_rules(&groups, &product);
        let mut ai_disallow_all = Vec::new();
        for group in &groups {
            if !group.disallows.iter().any(|rule| rule == "/") {
                continue;
            }
            for agent in &group.agents {
                if agent != "*" && weavatrix_seo_model::ai_agent(agent).is_some() {
                    ai_disallow_all.push(agent.clone());
                }
            }
        }
        ai_disallow_all.sort();
        ai_disallow_all.dedup();
        Self {
            groups,
            allows,
            disallows,
            sitemaps,
            ai_disallow_all,
        }
    }

    /// Whether `url` is allowed for the product this file was parsed for.
    #[must_use]
    pub fn allows(&self, url: &AbsoluteUrl) -> bool {
        path_allowed(&self.allows, &self.disallows, &url.request_target())
    }

    /// Whether `url` is allowed for a named user-agent token.
    #[must_use]
    pub fn allows_agent(&self, agent: &str, url: &AbsoluteUrl) -> bool {
        Self::agent_allows(&self.groups, agent, &url.request_target())
    }

    /// Specificity of the winning group for `agent`. `0` means only `*` applied.
    #[must_use]
    pub fn specificity(&self, agent: &str) -> usize {
        winning_specificity(&self.groups, &agent.to_ascii_lowercase())
    }

    /// Role-aware robots decisions for every documented AI crawler.
    #[must_use]
    pub fn agent_matrix(&self, origin: &AbsoluteUrl) -> Vec<AiAgentPolicy> {
        weavatrix_seo_model::ai_agents()
            .iter()
            .map(|agent| {
                let allowed = self.allows_agent(agent.token, origin);
                let specific = self.specificity(agent.token);
                let policy_intent = if specific == 0 {
                    "UNDECLARED"
                } else if allowed {
                    "ALLOW"
                } else {
                    "BLOCK"
                };
                AiAgentPolicy {
                    agent: agent.token.to_owned(),
                    provider: agent.provider.to_owned(),
                    role: role_token(agent.roles.first().copied()),
                    allowed,
                    search_impact: agent.search_visibility_effect.to_owned(),
                    policy_intent: policy_intent.to_owned(),
                }
            })
            .collect()
    }

    fn agent_allows(groups: &[RobotsGroup], agent: &str, path: &str) -> bool {
        let agent = agent.to_ascii_lowercase();
        let (allows, disallows) = merge_rules(groups, &agent);
        path_allowed(&allows, &disallows, path)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct RobotsGroup {
    agents: Vec<String>,
    allows: Vec<String>,
    disallows: Vec<String>,
}

fn parse_groups(body: &str) -> (Vec<RobotsGroup>, Vec<String>) {
    let mut groups = Vec::new();
    let mut current = RobotsGroup::default();
    let mut saw_rule = false;
    let mut sitemaps = Vec::new();
    let flush = |groups: &mut Vec<RobotsGroup>, current: &mut RobotsGroup| {
        if !current.agents.is_empty() {
            groups.push(std::mem::take(current));
        }
    };
    for raw in body.lines() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let Some((field, value)) = line.split_once(':') else {
            continue;
        };
        let field = field.trim().to_ascii_lowercase();
        let value = value.trim();
        match field.as_str() {
            "user-agent" => {
                if saw_rule {
                    flush(&mut groups, &mut current);
                    saw_rule = false;
                }
                current.agents.push(value.to_ascii_lowercase());
            }
            "disallow" => {
                // Empty Disallow means allow all for this group.
                if !value.is_empty() {
                    current.disallows.push(value.to_owned());
                }
                saw_rule = true;
            }
            "allow" => {
                if !value.is_empty() {
                    current.allows.push(value.to_owned());
                }
                saw_rule = true;
            }
            "sitemap" => sitemaps.push(value.to_owned()),
            _ => {}
        }
    }
    flush(&mut groups, &mut current);
    (groups, sitemaps)
}

fn role_token(role: Option<AiAgentRole>) -> String {
    match role {
        Some(AiAgentRole::SearchDiscovery) => "search_discovery",
        Some(AiAgentRole::CitationFetch) => "citation_fetch",
        Some(AiAgentRole::UserInitiatedFetch) => "user_fetch",
        Some(AiAgentRole::Training) => "training",
        Some(AiAgentRole::GroundingControl) => "grounding_control",
        Some(AiAgentRole::Archive) => "archive",
        _ => "other",
    }
    .to_owned()
}

fn group_specificity(group: &RobotsGroup, product: &str) -> Option<usize> {
    group
        .agents
        .iter()
        .filter_map(|agent| {
            if agent == "*" {
                Some(0)
            } else if product == agent || product.starts_with(agent) {
                Some(agent.len())
            } else {
                None
            }
        })
        .max()
}

fn winning_specificity(groups: &[RobotsGroup], product: &str) -> usize {
    groups
        .iter()
        .filter_map(|group| group_specificity(group, product))
        .max()
        .unwrap_or(0)
}

fn merge_rules(groups: &[RobotsGroup], product: &str) -> (Vec<String>, Vec<String>) {
    let max = winning_specificity(groups, product);
    let mut allows = Vec::new();
    let mut disallows = Vec::new();
    for group in groups {
        if group_specificity(group, product) != Some(max) {
            continue;
        }
        allows.extend(group.allows.iter().cloned());
        disallows.extend(group.disallows.iter().cloned());
    }
    (allows, disallows)
}

fn path_allowed(allows: &[String], disallows: &[String], path: &str) -> bool {
    let disallow = longest_match(disallows, path);
    let allow = longest_match(allows, path);
    match (allow, disallow) {
        (Some(allow_rule), Some(disallow_rule)) => allow_rule.len() >= disallow_rule.len(),
        (None, Some(_)) => false,
        _ => true,
    }
}

fn longest_match<'a>(rules: &'a [String], path: &str) -> Option<&'a String> {
    rules
        .iter()
        .filter(|rule| path_matches(rule, path))
        .max_by_key(|rule| rule.len())
}

/// Google robots.txt wildcards: `*` any run of bytes, `$` end-anchor.
/// A rule without either remains a prefix match.
fn path_matches(rule: &str, path: &str) -> bool {
    let (pattern, anchored) = match rule.strip_suffix('$') {
        Some(stripped) => (stripped, true),
        None => (rule, false),
    };
    if !pattern.contains('*') {
        return if anchored {
            path == pattern
        } else {
            path.starts_with(pattern)
        };
    }
    wildcard_match(pattern, path, anchored)
}

fn wildcard_match(pattern: &str, path: &str, anchored: bool) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    let mut cursor = path;
    let Some((first, rest)) = parts.split_first() else {
        return true;
    };
    if !cursor.starts_with(first) {
        return false;
    }
    cursor = &cursor[first.len()..];
    for (index, part) in rest.iter().enumerate() {
        let last = index + 1 == rest.len();
        if part.is_empty() {
            if last && anchored {
                return cursor.is_empty();
            }
            continue;
        }
        if last && anchored {
            return cursor.ends_with(part);
        }
        match cursor.find(part) {
            Some(at) => cursor = &cursor[at + part.len()..],
            None => return false,
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::Robots;
    use weavatrix_seo_model::AbsoluteUrl;

    fn url(path: &str) -> AbsoluteUrl {
        AbsoluteUrl::parse(&format!("https://x.test{path}")).unwrap()
    }

    #[test]
    fn longest_match_wins() {
        let robots = Robots::parse(
            "User-agent: *\nDisallow: /private\nAllow: /private/public\nSitemap: https://x.test/sitemap.xml\n",
            "weavatrix-seo",
        );
        assert!(!robots.allows(&url("/private/a")));
        assert!(robots.allows(&url("/private/public")));
        assert_eq!(robots.sitemaps[0], "https://x.test/sitemap.xml");
    }

    #[test]
    fn star_and_end_anchor_follow_google_robots() {
        let robots = Robots::parse(
            "User-agent: *\nDisallow: /*.php$\nAllow: /public/*.php$\nDisallow: /tmp\n",
            "weavatrix-seo",
        );
        assert!(!robots.allows(&url("/page.php")));
        assert!(robots.allows(&url("/page.php?x=1")));
        assert!(robots.allows(&url("/public/a.php")));
        assert!(robots.allows(&url("/page.html")));
    }

    #[test]
    fn specific_group_does_not_inherit_star() {
        let robots = Robots::parse(
            "User-agent: *\nDisallow: /secret\n\nUser-agent: weavatrix-seo\nAllow: /\n",
            "weavatrix-seo",
        );
        assert!(robots.allows(&url("/secret")));
        assert!(robots.specificity("weavatrix-seo") > 0);
    }

    #[test]
    fn star_applies_when_no_specific_group_exists() {
        let robots = Robots::parse("User-agent: *\nDisallow: /secret\n", "weavatrix-seo");
        assert!(!robots.allows(&url("/secret")));
        assert_eq!(robots.specificity("weavatrix-seo"), 0);
    }

    #[test]
    fn equal_specificity_groups_merge() {
        let robots = Robots::parse(
            "User-agent: googlebot\nDisallow: /a\n\nUser-agent: googlebot\nDisallow: /b\n",
            "googlebot",
        );
        assert!(!robots.allows(&url("/a")));
        assert!(!robots.allows(&url("/b")));
        assert!(robots.allows(&url("/c")));
    }

    #[test]
    fn multiple_user_agent_lines_share_one_group() {
        let robots = Robots::parse(
            "User-agent: googlebot\nUser-agent: weavatrix-seo\nDisallow: /lab\n",
            "weavatrix-seo",
        );
        assert!(!robots.allows(&url("/lab")));
    }

    #[test]
    fn empty_disallow_means_allow_all() {
        let robots = Robots::parse("User-agent: *\nDisallow:\n", "weavatrix-seo");
        assert!(robots.allows(&url("/anything")));
    }

    #[test]
    fn query_strings_are_part_of_the_request_target() {
        let robots = Robots::parse("User-agent: *\nDisallow: /search?\n", "weavatrix-seo");
        assert!(!robots.allows(&url("/search?q=1")));
        assert!(robots.allows(&url("/search")));
    }

    #[test]
    fn records_ai_agents_disallowed_at_origin() {
        let robots = Robots::parse(
            "User-agent: *\nAllow: /\n\nUser-agent: GPTBot\nDisallow: /\nUser-agent: ClaudeBot\nDisallow: /\n",
            "weavatrix-seo",
        );
        assert!(robots.allows(&url("/")));
        assert!(robots.ai_disallow_all.iter().any(|agent| agent == "gptbot"));
        assert!(
            robots
                .ai_disallow_all
                .iter()
                .any(|agent| agent == "claudebot")
        );
        assert!(!robots.allows_agent("gptbot", &url("/")));
        assert!(robots.allows_agent("oai-searchbot", &url("/")));
    }
}

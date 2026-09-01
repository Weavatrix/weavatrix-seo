//! robots.txt parser. Honours `User-agent: *` and this product's token.

use weavatrix_seo_model::AbsoluteUrl;

/// Parsed robots policy for one host.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Robots {
    allows: Vec<String>,
    disallows: Vec<String>,
    /// Sitemap URLs declared by the file.
    pub sitemaps: Vec<String>,
    /// Known AI user-agents whose group contains `Disallow: /`.
    pub ai_disallow_all: Vec<String>,
}

impl Robots {
    /// Parses a robots.txt body.
    #[must_use]
    pub fn parse(body: &str, product: &str) -> Self {
        let mut robots = Self::default();
        let mut applies = false;
        let product = product.to_ascii_lowercase();
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
                    let agent = value.to_ascii_lowercase();
                    applies =
                        agent == "*" || product.starts_with(&agent) || agent == "weavatrix-seo";
                }
                "disallow" if applies => {
                    if !value.is_empty() {
                        robots.disallows.push(value.to_owned());
                    }
                }
                "allow" if applies => {
                    if !value.is_empty() {
                        robots.allows.push(value.to_owned());
                    }
                }
                "sitemap" => robots.sitemaps.push(value.to_owned()),
                _ => {}
            }
        }
        robots.ai_disallow_all = ai_disallow_all(body);
        robots
    }

    /// Whether `url` is allowed for this crawler.
    #[must_use]
    pub fn allows(&self, url: &AbsoluteUrl) -> bool {
        let path = url.request_target();
        let disallow = longest_match(&self.disallows, &path);
        let allow = longest_match(&self.allows, &path);
        match (allow, disallow) {
            (Some(allow_rule), Some(disallow_rule)) => allow_rule.len() >= disallow_rule.len(),
            (None, Some(_)) => false,
            _ => true,
        }
    }
}

const AI_AGENTS: &[&str] = &[
    "gptbot",
    "chatgpt-user",
    "oai-searchbot",
    "claudebot",
    "claude-searchbot",
    "claude-user",
    "perplexitybot",
    "google-extended",
    "google-agent",
    "applebot-extended",
    "bytespider",
    "ccbot",
    "anthropic-ai",
];

fn ai_disallow_all(body: &str) -> Vec<String> {
    let mut agents = Vec::new();
    let mut disallows = Vec::new();
    let mut blocked = Vec::new();
    let mut saw_rule = false;
    for raw in body.lines() {
        let line = raw.split('#').next().unwrap_or("").trim();
        let Some((field, value)) = line.split_once(':') else {
            continue;
        };
        let field = field.trim().to_ascii_lowercase();
        let value = value.trim();
        match field.as_str() {
            "user-agent" => {
                if saw_rule {
                    record_ai_group(&agents, &disallows, &mut blocked);
                    agents.clear();
                    disallows.clear();
                    saw_rule = false;
                }
                agents.push(value.to_ascii_lowercase());
            }
            "disallow" => {
                disallows.push(value.to_owned());
                saw_rule = true;
            }
            "allow" => saw_rule = true,
            _ => {}
        }
    }
    record_ai_group(&agents, &disallows, &mut blocked);
    blocked.sort();
    blocked.dedup();
    blocked
}

fn record_ai_group(agents: &[String], disallows: &[String], blocked: &mut Vec<String>) {
    if !disallows.iter().any(|rule| rule == "/") {
        return;
    }
    for agent in agents {
        if AI_AGENTS.contains(&agent.as_str()) && !blocked.contains(agent) {
            blocked.push(agent.clone());
        }
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

    #[test]
    fn longest_match_wins() {
        let robots = Robots::parse(
            "User-agent: *\nDisallow: /private\nAllow: /private/public\nSitemap: https://x.test/sitemap.xml\n",
            "weavatrix-seo",
        );
        let blocked = AbsoluteUrl::parse("https://x.test/private/a").unwrap();
        let allowed = AbsoluteUrl::parse("https://x.test/private/public").unwrap();
        assert!(!robots.allows(&blocked));
        assert!(robots.allows(&allowed));
        assert_eq!(robots.sitemaps[0], "https://x.test/sitemap.xml");
    }

    #[test]
    fn star_and_end_anchor_follow_google_robots() {
        let robots = Robots::parse(
            "User-agent: *\nDisallow: /*.php$\nAllow: /public/*.php$\nDisallow: /tmp\n",
            "weavatrix-seo",
        );
        let blocked = AbsoluteUrl::parse("https://x.test/page.php").unwrap();
        let query = AbsoluteUrl::parse("https://x.test/page.php?x=1").unwrap();
        let allowed = AbsoluteUrl::parse("https://x.test/public/a.php").unwrap();
        let html = AbsoluteUrl::parse("https://x.test/page.html").unwrap();
        assert!(!robots.allows(&blocked));
        assert!(robots.allows(&query));
        assert!(robots.allows(&allowed));
        assert!(robots.allows(&html));
    }

    #[test]
    fn records_ai_agents_disallowed_at_origin() {
        let robots = Robots::parse(
            "User-agent: *\nAllow: /\n\nUser-agent: GPTBot\nDisallow: /\nUser-agent: ClaudeBot\nDisallow: /\n",
            "weavatrix-seo",
        );
        assert!(robots.allows(&AbsoluteUrl::parse("https://x.test/").unwrap()));
        assert!(robots.ai_disallow_all.iter().any(|agent| agent == "gptbot"));
        assert!(
            robots
                .ai_disallow_all
                .iter()
                .any(|agent| agent == "claudebot")
        );
    }
}

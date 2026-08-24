//! robots.txt parser. Honours `User-agent: *` and this product's token.

use weavatrix_seo_model::AbsoluteUrl;

/// Parsed robots policy for one host.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Robots {
    allows: Vec<String>,
    disallows: Vec<String>,
    /// Sitemap URLs declared by the file.
    pub sitemaps: Vec<String>,
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
        robots
    }

    /// Whether `url` is allowed for this crawler.
    #[must_use]
    pub fn allows(&self, url: &AbsoluteUrl) -> bool {
        let path = url.request_target();
        let disallow = longest_prefix(&self.disallows, &path);
        let allow = longest_prefix(&self.allows, &path);
        match (allow, disallow) {
            (Some(allow_rule), Some(disallow_rule)) => allow_rule.len() >= disallow_rule.len(),
            (None, Some(_)) => false,
            _ => true,
        }
    }
}

fn longest_prefix<'a>(rules: &'a [String], path: &str) -> Option<&'a String> {
    rules
        .iter()
        .filter(|rule| path.starts_with(rule.as_str()))
        .max_by_key(|rule| rule.len())
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
}
